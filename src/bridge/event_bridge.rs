use crate::adapters::{
    DiscordService, EventResponse, EventSender, ReactParams, ReplyParams, ResponseAction,
    ThreadParams,
};
use crate::bridge::discord_text::{generate_thread_name, truncate_content, truncate_thread_name};
use anyhow::Context as _;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use std::sync::Arc;
use tracing::{debug, error, info};

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
    /// * `http` - The HTTP client for Discord API calls
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
            ResponseAction::Reply(params) => self.execute_reply(http, message, params).await,
            ResponseAction::React(params) => self.execute_react(http, message, params).await,
            ResponseAction::Thread(params) => self.execute_thread(http, message, params).await,
        }
    }

    /// Execute Reply action
    ///
    /// # Content Handling
    /// - Content exceeding 2000 characters is truncated with warning log
    ///
    /// # Mention
    /// - `params.mention = true`: Reply with ping (user receives notification)
    /// - `params.mention = false`: Reply without ping (default)
    async fn execute_reply(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        params: &ReplyParams,
    ) -> anyhow::Result<()> {
        let content = truncate_content(&params.content);

        self.discord_service
            .reply_in_channel(http, message.channel_id, message.id, &content, params.mention)
            .await
            .context("Failed to send reply to Discord")?;

        info!(
            message_id = %message.id,
            mention = params.mention,
            content_len = content.chars().count(),
            "Successfully executed reply action"
        );

        Ok(())
    }

    /// Execute React action
    ///
    /// # Emoji Format
    /// - Unicode emoji: "👍", "🎉", etc.
    /// - Custom emoji: "name:id" format (e.g., "customemoji:123456789")
    async fn execute_react(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        params: &ReactParams,
    ) -> anyhow::Result<()> {
        self.discord_service
            .react_to_message(http, message.channel_id, message.id, &params.emoji)
            .await
            .context("Failed to add reaction to Discord")?;

        info!(
            message_id = %message.id,
            emoji = %params.emoji,
            "Successfully executed react action"
        );

        Ok(())
    }

    /// Execute Thread action
    ///
    /// # Thread Name
    /// - `params.name = Some(...)`: Use specified name
    /// - `params.name = None`: Auto-generate from first line of message (max 100 chars)
    ///   - Falls back to "Thread" if message content is empty
    /// - Name is ignored if already in a thread
    ///
    /// # Content Handling
    /// - Content exceeding 2000 characters is truncated with warning log
    ///
    /// # Auto-archive Duration
    /// - Valid values: 60, 1440, 4320, 10080 (minutes)
    /// - Invalid values fall back to 1440 (OneDay) with warning log
    async fn execute_thread(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        params: &ThreadParams,
    ) -> anyhow::Result<()> {
        // Ensure we're in a guild (threads not supported in DM)
        message.guild_id
            .context("Thread action is not supported in DM")?;

        // Check if already in thread (API call, no cache)
        let is_in_thread = self.discord_service
            .is_thread_channel(http, message.channel_id)
            .await
            .context("Failed to check if channel is thread")?;

        // Determine target channel ID
        let target_channel_id = if is_in_thread {
            // Already in thread → use as-is
            info!("Message is already in thread, skipping thread creation");
            message.channel_id
        } else {
            // Normal channel → create new thread
            let thread_name = match &params.name {
                Some(name) => truncate_thread_name(name),
                None => generate_thread_name(message),
            };

            let thread = self
                .discord_service
                .create_thread_from_message(
                    http,
                    message,
                    &thread_name,
                    params.auto_archive_duration,
                )
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
        let content = truncate_content(&params.content);

        // Post message to thread
        self.discord_service
            .send_message_to_channel(http, target_channel_id, &content)
            .await
            .context("Failed to send message to thread")?;

        info!(
            channel_id = %target_channel_id,
            is_in_thread = is_in_thread,
            "Successfully executed thread action"
        );

        Ok(())
    }
}
