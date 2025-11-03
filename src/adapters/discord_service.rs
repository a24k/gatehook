use serenity::async_trait;
use serenity::model::id::{ChannelId, MessageId};

/// Interface for Discord operations
#[async_trait]
pub trait DiscordService: Send + Sync {
    /// Reply to a message in a channel
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `channel_id` - The channel where the message was sent
    /// * `message_id` - The message to reply to
    /// * `content` - The content of the reply
    /// * `mention` - Whether to ping the user (true) or not (false)
    async fn reply_to_message(
        &self,
        http: &serenity::http::Http,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
        mention: bool,
    ) -> Result<(), serenity::Error>;
}
