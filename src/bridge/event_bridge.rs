use crate::adapters::{DiscordService, EventResponse, EventSender, ResponseAction};
use anyhow::Context as _;
use serenity::model::channel::Message;
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
    /// Also executes existing business logic (Ping! response).
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `message` - The message event from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (may contain actions)
    pub async fn handle_message(
        &self,
        http: &serenity::http::Http,
        message: &Message,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            message_id = %message.id,
            author = %message.author.name,
            content = %message.content,
            "Processing message event"
        );

        // Business logic: reply to "Ping!" messages
        if message.content == "Ping!"
            && let Err(err) = self
                .discord_service
                .reply_to_message(http, message.channel_id, message.id, "Pong!", false)
                .await
        {
            error!(error = ?err, "Failed to send message reply");
        }

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
        // 2000文字制限チェック + 切り詰め
        let content = if content.chars().count() > 2000 {
            let truncated: String = content.chars().take(1997).collect();
            let truncated = format!("{}...", truncated);

            warn!(
                original_len = content.chars().count(),
                truncated_len = truncated.chars().count(),
                "Reply content exceeds 2000 chars, truncated"
            );

            truncated
        } else {
            content.to_string()
        };

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
}
