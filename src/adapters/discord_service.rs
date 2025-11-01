use serenity::async_trait;
use serenity::model::id::{ChannelId, MessageId};

/// Discord操作のインターフェース
#[async_trait]
pub trait DiscordService: Send + Sync {
    /// Reply to a message in a channel
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel where the message was sent
    /// * `message_id` - The message to reply to
    /// * `content` - The content of the reply
    async fn reply_to_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> Result<(), serenity::Error>;
}
