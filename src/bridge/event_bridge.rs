use crate::adapters::{DiscordService, EventSender};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use std::sync::Arc;
use tracing::{debug, error};

/// Discord Gateway イベントを外部エンドポイントに橋渡しする
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
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `message` - The message event from Discord
    pub async fn handle_message(
        &self,
        http: &serenity::http::Http,
        message: &Message,
    ) -> anyhow::Result<()> {
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
                .reply_to_message(http, message.channel_id, message.id, "Pong!")
                .await
        {
            error!(error = ?err, "Failed to send message reply");
        }

        // Forward event to webhook endpoint
        self.event_sender.send("message", message).await?;

        Ok(())
    }

    /// Handle a ready event
    ///
    /// # Arguments
    ///
    /// * `ready` - The ready event from Discord
    pub async fn handle_ready(&self, ready: &Ready) -> anyhow::Result<()> {
        debug!(
            user = %ready.user.display_name(),
            "Processing ready event"
        );

        // Forward event to webhook endpoint
        self.event_sender.send("ready", ready).await?;

        Ok(())
    }
}
