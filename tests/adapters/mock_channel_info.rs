use gatehook::adapters::ChannelInfoProvider;
use serenity::async_trait;
use serenity::model::id::{ChannelId, GuildId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock implementation of ChannelInfoProvider for testing
pub struct MockChannelInfoProvider {
    responses: Arc<Mutex<HashMap<ChannelId, bool>>>,
}

impl MockChannelInfoProvider {
    /// Create a new MockChannelInfoProvider
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set the response for a specific channel ID
    pub fn set_is_thread(&self, channel_id: ChannelId, is_thread: bool) {
        self.responses.lock().unwrap().insert(channel_id, is_thread);
    }
}

impl Default for MockChannelInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChannelInfoProvider for MockChannelInfoProvider {
    async fn is_thread(
        &self,
        _cache: &serenity::cache::Cache,
        _http: &serenity::http::Http,
        _guild_id: Option<GuildId>,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error> {
        // Return configured response, default to false if not set
        Ok(self
            .responses
            .lock()
            .unwrap()
            .get(&channel_id)
            .copied()
            .unwrap_or(false))
    }
}
