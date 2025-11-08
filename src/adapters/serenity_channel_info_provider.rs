use super::channel_info_provider::ChannelInfoProvider;
use serenity::async_trait;
use serenity::model::channel::{Channel, ChannelType};
use serenity::model::id::ChannelId;
use std::sync::Arc;
use tracing::debug;

/// Implementation for channel information retrieval via Serenity
///
/// Uses cache-first approach with API fallback for optimal performance.
/// Holds references to cache and http that are maintained by Serenity's event loop.
pub struct SerenityChannelInfoProvider {
    cache: Arc<serenity::cache::Cache>,
    http: Arc<serenity::http::Http>,
}

impl SerenityChannelInfoProvider {
    /// Create a new SerenityChannelInfoProvider with cache and http references
    pub fn new(cache: Arc<serenity::cache::Cache>, http: Arc<serenity::http::Http>) -> Self {
        Self { cache, http }
    }
}

#[async_trait]
impl ChannelInfoProvider for SerenityChannelInfoProvider {
    async fn is_thread(
        &self,
        guild_id: Option<serenity::model::id::GuildId>,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error> {
        // Try cache first (fast path)
        // Extract channel kind from cache without holding the lock across await points
        let cached_result: Option<bool> = if let Some(gid) = guild_id {
            // Direct guild access (O(1) - fast)
            self.cache.guild(gid).and_then(|guild_ref| {
                // Check regular channels first, then threads
                guild_ref.channels
                    .get(&channel_id)
                    .cloned()
                    .or_else(|| {
                        guild_ref.threads.iter()
                            .find(|ch| ch.id == channel_id)
                            .cloned()
                    })
                    .map(|channel| {
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
            self.cache.guilds().iter().find_map(|guild_id| {
                self.cache.guild(*guild_id).and_then(|guild_ref| {
                    // Check regular channels first, then threads
                    guild_ref.channels
                        .get(&channel_id)
                        .cloned()
                        .or_else(|| {
                            guild_ref.threads.iter()
                                .find(|ch| ch.id == channel_id)
                                .cloned()
                        })
                        .map(|channel| {
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

        let channel = self.http.get_channel(channel_id).await?;
        let is_thread = matches!(
            channel,
            Channel::Guild(ref c) if matches!(
                c.kind,
                ChannelType::PublicThread | ChannelType::PrivateThread | ChannelType::NewsThread
            )
        );

        Ok(is_thread)
    }

    async fn get_channel(
        &self,
        guild_id: Option<serenity::model::id::GuildId>,
        channel_id: ChannelId,
    ) -> Result<Option<serenity::model::channel::GuildChannel>, serenity::Error> {
        // Try cache first (fast path)
        // Extract channel from cache without holding the lock across await points
        let cached_result: Option<serenity::model::channel::GuildChannel> = if let Some(gid) =
            guild_id
        {
            // Direct guild access (O(1) - fast)
            self.cache.guild(gid).and_then(|guild_ref| {
                // Check regular channels first, then threads
                guild_ref
                    .channels
                    .get(&channel_id)
                    .cloned()
                    .or_else(|| {
                        guild_ref
                            .threads
                            .iter()
                            .find(|ch| ch.id == channel_id)
                            .cloned()
                    })
                    .inspect(|channel| {
                        debug!(
                            guild_id = %gid,
                            channel_id = %channel_id,
                            channel_name = %channel.name,
                            "Channel retrieved from cache (direct guild access)"
                        );
                    })
            })
        } else {
            // Search all guilds (O(n) - slower fallback)
            self.cache.guilds().iter().find_map(|guild_id| {
                self.cache.guild(*guild_id).and_then(|guild_ref| {
                    // Check regular channels first, then threads
                    guild_ref
                        .channels
                        .get(&channel_id)
                        .cloned()
                        .or_else(|| {
                            guild_ref
                                .threads
                                .iter()
                                .find(|ch| ch.id == channel_id)
                                .cloned()
                        })
                        .inspect(|channel| {
                            debug!(
                                guild_id = %guild_id,
                                channel_id = %channel_id,
                                channel_name = %channel.name,
                                "Channel retrieved from cache (guild search)"
                            );
                        })
                })
            })
        }; // Cache references are dropped here

        // Return cached result if available
        if let Some(channel) = cached_result {
            return Ok(Some(channel));
        }

        // Cache miss - fallback to API (slow path)
        debug!(
            channel_id = %channel_id,
            "Cache miss, fetching channel from API"
        );

        let channel = self.http.get_channel(channel_id).await?;
        match channel {
            Channel::Guild(guild_channel) => Ok(Some(guild_channel)),
            _ => Ok(None), // DM channel
        }
    }
}
