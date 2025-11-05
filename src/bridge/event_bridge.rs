use crate::adapters::{DiscordService, EventResponse, EventSender, ResponseAction};
use anyhow::Context as _;
use serenity::model::channel::{AutoArchiveDuration, Message};
use serenity::model::gateway::Ready;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Bridge Discord Gateway events to external endpoints
pub struct EventBridge<D, S>
where
    D: DiscordService,
    S: EventSender,
{
    discord_service: Arc<D>,
    event_sender: Arc<S>,
}

impl<D, S> EventBridge<D, S>
where
    D: DiscordService,
    S: EventSender,
{
    /// Create a new EventBridge
    ///
    /// # Arguments
    ///
    /// * `discord_service` - The Discord service for operations
    /// * `event_sender` - The event sender for forwarding events
    pub fn new(discord_service: Arc<D>, event_sender: Arc<S>) -> Self {
        Self {
            discord_service,
            event_sender,
        }
    }

    /// Handle a message event
    ///
    /// Sends event to webhook and returns the response.
    ///
    /// # Arguments
    ///
    /// * `message` - The message event from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (may contain actions)
    pub async fn handle_message(
        &self,
        message: &Message,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            message_id = %message.id,
            author = %message.author.name,
            content = %message.content,
            "Processing message event"
        );

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("message", message)
            .await
            .context("Failed to send message event to HTTP endpoint")
    }

    /// Handle a ready event
    ///
    /// # Arguments
    ///
    /// * `ready` - The ready event from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (may contain actions)
    pub async fn handle_ready(&self, ready: &Ready) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            user = %ready.user.display_name(),
            "Processing ready event"
        );

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("ready", ready)
            .await
            .context("Failed to send ready event to HTTP endpoint")
    }

    /// Execute actions from webhook response
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `message` - The message that triggered the event (for context)
    /// * `event_response` - The response from webhook containing actions
    pub async fn execute_actions(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        event_response: &EventResponse,
    ) -> anyhow::Result<()> {
        for action in &event_response.actions {
            // Execute action (log error and continue with next)
            if let Err(err) = self.execute_action(http, message, action).await {
                error!(?err, ?action, "Failed to execute action, continuing with next");
            }
        }
        Ok(())
    }

    /// Execute a single action
    async fn execute_action(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        action: &ResponseAction,
    ) -> anyhow::Result<()> {
        match action {
            ResponseAction::Reply { content, mention } => {
                self.execute_reply(http, message, content, *mention).await
            }
            ResponseAction::React { emoji } => {
                self.execute_react(http, message, emoji).await
            }
            ResponseAction::Thread {
                name,
                content,
                reply,
                mention,
                auto_archive_duration,
            } => {
                self.execute_thread(
                    http,
                    message,
                    name.as_deref(),
                    content,
                    *reply,
                    *mention,
                    *auto_archive_duration,
                )
                .await
            }
        }
    }

    /// Execute Reply action
    async fn execute_reply(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        content: &str,
        mention: bool,
    ) -> anyhow::Result<()> {
        let content = truncate_content(content);

        self.discord_service
            .reply_to_message(http, message.channel_id, message.id, &content, mention)
            .await
            .context("Failed to send reply to Discord")?;

        info!(
            message_id = %message.id,
            mention = mention,
            content_len = content.chars().count(),
            "Successfully executed reply action"
        );

        Ok(())
    }

    /// Execute React action
    async fn execute_react(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        emoji: &str,
    ) -> anyhow::Result<()> {
        self.discord_service
            .react_to_message(http, message.channel_id, message.id, emoji)
            .await
            .context("Failed to add reaction to Discord")?;

        info!(
            message_id = %message.id,
            emoji = emoji,
            "Successfully executed react action"
        );

        Ok(())
    }

    /// Execute Thread action
    #[allow(clippy::too_many_arguments)]
    async fn execute_thread(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        name: Option<&str>,
        content: &str,
        reply: bool,
        mention: bool,
        auto_archive_duration: AutoArchiveDuration,
    ) -> anyhow::Result<()> {
        // Check if DM (guild_id is None)
        if message.guild_id.is_none() {
            anyhow::bail!("Thread action is not supported in DM");
        }

        // Check if already in thread
        let is_in_thread = self
            .discord_service
            .is_thread_channel(http, message.channel_id)
            .await
            .context("Failed to check if channel is thread")?;

        // Determine target channel ID
        let target_channel_id = if is_in_thread {
            // Already in thread → use as-is
            if name.is_some() {
                debug!("Already in thread, ignoring 'name' parameter");
            }
            info!("Message is already in thread, skipping thread creation");
            message.channel_id
        } else {
            // Normal channel → create new thread
            let thread_name = name
                .map(|n| n.to_string())
                .unwrap_or_else(|| generate_thread_name(message));

            let thread = self
                .discord_service
                .create_thread_from_message(http, message, &thread_name, auto_archive_duration)
                .await
                .context("Failed to create thread")?;

            info!(
                thread_id = %thread.id,
                thread_name = %thread_name,
                "Created new thread"
            );
            thread.id
        };

        // Truncate content
        let content = truncate_content(content);

        // Post message
        if reply {
            self.discord_service
                .reply_in_channel(http, target_channel_id, message.id, &content, mention)
                .await
                .context("Failed to send reply in thread")?;

            info!(
                channel_id = %target_channel_id,
                message_id = %message.id,
                reply = true,
                mention = mention,
                "Successfully executed thread action with reply"
            );
        } else {
            self.discord_service
                .send_message_to_channel(http, target_channel_id, &content)
                .await
                .context("Failed to send message to thread")?;

            info!(
                channel_id = %target_channel_id,
                reply = false,
                "Successfully executed thread action"
            );
        }

        Ok(())
    }
}

/// Truncate content to Discord's 2000 character limit
fn truncate_content(content: &str) -> String {
    const MAX_LEN: usize = 2000;

    let char_count = content.chars().count();

    if char_count > MAX_LEN {
        let truncated: String = content.chars().take(MAX_LEN - 3).collect();
        let result = format!("{}...", truncated);

        warn!(
            original_len = char_count,
            truncated_len = result.chars().count(),
            "Content exceeds 2000 chars, truncated"
        );

        result
    } else {
        content.to_string()
    }
}

/// Generate thread name from message content
fn generate_thread_name(message: &Message) -> String {
    const MAX_LEN: usize = 100; // Discord API maximum

    // Use first line only, trim whitespace
    let content = message
        .content
        .lines()
        .next()
        .unwrap_or("")
        .trim();

    if content.is_empty() {
        return "Thread".to_string();
    }

    let char_count = content.chars().count();

    if char_count <= MAX_LEN {
        content.to_string()
    } else {
        // Truncate to API limit
        content.chars().take(MAX_LEN).collect()
    }
}
