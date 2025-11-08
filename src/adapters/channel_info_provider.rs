use serenity::async_trait;
use serenity::model::id::{ChannelId, GuildId};

/// Interface for retrieving channel information
#[async_trait]
pub trait ChannelInfoProvider: Send + Sync {
    /// Check if a channel is a thread
    ///
    /// # Arguments
    ///
    /// * `guild_id` - Optional guild ID for direct cache access (performance optimization)
    ///   - `Some(guild_id)`: Direct guild cache lookup (O(1) - fast)
    ///   - `None`: Search all guilds in cache (O(n) where n = number of guilds)
    /// * `channel_id` - The channel ID to check
    ///
    /// # Returns
    ///
    /// `true` if the channel is a thread (Public/Private/News), `false` otherwise
    ///
    /// # Implementation Note
    ///
    /// Implementations that use cache (like SerenityChannelInfoProvider) should hold
    /// Arc<Cache> and Arc<Http> internally. These are automatically updated/maintained
    /// by Serenity's event loop and never change during Client lifetime.
    async fn is_thread(
        &self,
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error>;
}
