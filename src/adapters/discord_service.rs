use serenity::async_trait;
use serenity::model::channel::{AutoArchiveDuration, GuildChannel, Message};
use serenity::model::id::{ChannelId, MessageId};

/// Interface for Discord operations
#[async_trait]
pub trait DiscordService: Send + Sync {
    /// Add a reaction to a message
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `channel_id` - The channel where the message was sent
    /// * `message_id` - The message to react to
    /// * `emoji` - The emoji to react with (Unicode or custom emoji format)
    async fn react_to_message(
        &self,
        http: &serenity::http::Http,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &str,
    ) -> Result<(), serenity::Error>;

    /// Create a thread from a message
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `message` - The message to create a thread from
    /// * `name` - The thread name
    /// * `auto_archive_duration` - Auto-archive duration
    async fn create_thread_from_message(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        name: &str,
        auto_archive_duration: AutoArchiveDuration,
    ) -> Result<GuildChannel, serenity::Error>;

    /// Send a message to a channel
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `channel_id` - The channel to send the message to
    /// * `content` - The message content
    async fn send_message_to_channel(
        &self,
        http: &serenity::http::Http,
        channel_id: ChannelId,
        content: &str,
    ) -> Result<Message, serenity::Error>;

    /// Reply to a message in a specific channel
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `channel_id` - The channel to send the reply in
    /// * `message_id` - The message to reply to
    /// * `content` - The reply content
    /// * `mention` - Whether to mention the user
    async fn reply_in_channel(
        &self,
        http: &serenity::http::Http,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
        mention: bool,
    ) -> Result<Message, serenity::Error>;

    /// Check if a channel is a thread
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client from Context
    /// * `channel_id` - The channel ID to check
    async fn is_thread_channel(
        &self,
        http: &serenity::http::Http,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error>;
}
