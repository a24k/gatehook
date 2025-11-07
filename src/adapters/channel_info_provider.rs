use serenity::async_trait;
use serenity::model::id::ChannelId;

/// Interface for retrieving channel information
#[async_trait]
pub trait ChannelInfoProvider: Send + Sync {
    /// Check if a channel is a thread
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache from Context (for cache-first optimization)
    /// * `http` - The HTTP client from Context (fallback for cache miss)
    /// * `channel_id` - The channel ID to check
    ///
    /// # Returns
    ///
    /// `true` if the channel is a thread (Public/Private/News), `false` otherwise
    async fn is_thread_channel(
        &self,
        cache: &serenity::cache::Cache,
        http: &serenity::http::Http,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error>;
}
