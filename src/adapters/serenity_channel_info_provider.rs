use super::channel_info_provider::ChannelInfoProvider;
use serenity::async_trait;
use serenity::model::channel::{Channel, ChannelType};
use serenity::model::id::ChannelId;
use tracing::debug;

/// Implementation for channel information retrieval via Serenity
///
/// Uses cache-first approach with API fallback for optimal performance.
pub struct SerenityChannelInfoProvider;

#[async_trait]
impl ChannelInfoProvider for SerenityChannelInfoProvider {
    async fn is_thread_channel(
        &self,
        cache: &serenity::cache::Cache,
        http: &serenity::http::Http,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error> {
        // Try cache first (fast path)
        // Extract channel kind from cache without holding the lock across await points
        // Note: We need to iterate through all guilds to find the channel
        let cached_result: Option<bool> = {
            // Search all cached guilds for this channel
            cache
                .guilds()
                .iter()
                .find_map(|guild_id| {
                    cache.guild(*guild_id).and_then(|guild_ref| {
                        guild_ref
                            .channels
                            .get(&channel_id)
                            .map(|channel| {
                                let is_thread = matches!(
                                    channel.kind,
                                    ChannelType::PublicThread
                                        | ChannelType::PrivateThread
                                        | ChannelType::NewsThread
                                );
                                debug!(
                                    channel_id = %channel_id,
                                    is_thread = is_thread,
                                    "Channel type resolved from cache"
                                );
                                is_thread
                            })
                    })
                })
        }; // Cache references are dropped here

        // Return cached result if available
        if let Some(is_thread) = cached_result {
            return Ok(is_thread);
        }

        // Cache miss - fallback to API (slow path)
        debug!(
            channel_id = %channel_id,
            "Cache miss, fetching channel info from API"
        );

        let channel = http.get_channel(channel_id).await?;
        let is_thread = matches!(
            channel,
            Channel::Guild(ref c) if matches!(
                c.kind,
                ChannelType::PublicThread | ChannelType::PrivateThread | ChannelType::NewsThread
            )
        );

        Ok(is_thread)
    }
}
