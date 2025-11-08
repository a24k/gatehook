use serenity::async_trait;
use serenity::model::channel::{GuildChannel, Message};
use serenity::model::id::{ChannelId, MessageId};

/// Interface for Discord operations
///
/// Implementations that use HTTP (like SerenityDiscordService) should hold
/// Arc<Http> internally, which is maintained by Serenity's event loop.
#[async_trait]
pub trait DiscordService: Send + Sync {
    /// Add a reaction to a message
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel where the message was sent
    /// * `message_id` - The message to react to
    /// * `emoji` - The emoji to react with (Unicode or custom emoji format)
    async fn react_to_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &str,
    ) -> Result<(), serenity::Error>;

    /// Create a thread from a message
    ///
    /// # Arguments
    ///
    /// * `message` - The message to create a thread from
    /// * `name` - The thread name
    /// * `auto_archive_duration` - Auto-archive duration in minutes (60, 1440, 4320, 10080)
    async fn create_thread_from_message(
        &self,
        message: &Message,
        name: &str,
        auto_archive_duration: u16,
    ) -> Result<GuildChannel, serenity::Error>;

    /// Send a message to a channel
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel to send the message to
    /// * `content` - The message content
    async fn send_message_to_channel(
        &self,
        channel_id: ChannelId,
        content: &str,
    ) -> Result<Message, serenity::Error>;

    /// Reply to a message in a specific channel
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel to send the reply in
    /// * `message_id` - The message to reply to
    /// * `content` - The reply content
    /// * `mention` - Whether to mention the user
    async fn reply_in_channel(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
        mention: bool,
    ) -> Result<Message, serenity::Error>;
}
