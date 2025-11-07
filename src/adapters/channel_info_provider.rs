use serenity::async_trait;
use serenity::model::id::{ChannelId, GuildId};

/// Interface for retrieving channel information
#[async_trait]
pub trait ChannelInfoProvider: Send + Sync {
    /// Check if a channel is a thread
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache from Context (for cache-first optimization)
    /// * `http` - The HTTP client from Context (fallback for cache miss)
    /// * `guild_id` - Optional guild ID for direct cache access (performance optimization)
    ///   - `Some(guild_id)`: Direct guild cache lookup (O(1) - fast)
    ///   - `None`: Search all guilds in cache (O(n) where n = number of guilds)
    /// * `channel_id` - The channel ID to check
    ///
    /// # Returns
    ///
    /// `true` if the channel is a thread (Public/Private/News), `false` otherwise
    async fn is_thread(
        &self,
        cache: &serenity::cache::Cache,
        http: &serenity::http::Http,
        guild_id: Option<GuildId>,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error>;
}
