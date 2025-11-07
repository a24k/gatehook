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
    async fn is_thread(
        &self,
        cache: &serenity::cache::Cache,
        http: &serenity::http::Http,
        guild_id: Option<serenity::model::id::GuildId>,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error> {
        // Try cache first (fast path)
        // Extract channel kind from cache without holding the lock across await points
        let cached_result: Option<bool> = if let Some(gid) = guild_id {
            // Direct guild access (O(1) - fast)
            cache.guild(gid).and_then(|guild_ref| {
                guild_ref.channels.get(&channel_id).map(|channel| {
                    let is_thread = matches!(
                        channel.kind,
                        ChannelType::PublicThread
                            | ChannelType::PrivateThread
                            | ChannelType::NewsThread
                    );
                    debug!(
                        guild_id = %gid,
                        channel_id = %channel_id,
                        is_thread = is_thread,
                        "Channel type resolved from cache (direct guild access)"
                    );
                    is_thread
                })
            })
        } else {
            // Search all guilds (O(n) - slower fallback)
            cache.guilds().iter().find_map(|guild_id| {
                cache.guild(*guild_id).and_then(|guild_ref| {
                    guild_ref.channels.get(&channel_id).map(|channel| {
                        let is_thread = matches!(
                            channel.kind,
                            ChannelType::PublicThread
                                | ChannelType::PrivateThread
                                | ChannelType::NewsThread
                        );
                        debug!(
                            guild_id = %guild_id,
                            channel_id = %channel_id,
                            is_thread = is_thread,
                            "Channel type resolved from cache (guild search)"
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
