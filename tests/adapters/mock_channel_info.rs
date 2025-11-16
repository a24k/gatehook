use gatehook::adapters::ChannelInfoProvider;
use serenity::async_trait;
use serenity::model::channel::GuildChannel;
use serenity::model::id::{ChannelId, GuildId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock implementation of ChannelInfoProvider for testing
pub struct MockChannelInfoProvider {
    is_thread_responses: Arc<Mutex<HashMap<ChannelId, bool>>>,
    is_thread_errors: Arc<Mutex<HashMap<ChannelId, String>>>,
    channel_responses: Arc<Mutex<HashMap<ChannelId, GuildChannel>>>,
}

impl MockChannelInfoProvider {
    /// Create a new MockChannelInfoProvider
    pub fn new() -> Self {
        Self {
            is_thread_responses: Arc::new(Mutex::new(HashMap::new())),
            is_thread_errors: Arc::new(Mutex::new(HashMap::new())),
            channel_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set the is_thread response for a specific channel ID
    pub fn set_is_thread(&self, channel_id: ChannelId, is_thread: bool) {
        self.is_thread_responses
            .lock()
            .unwrap()
            .insert(channel_id, is_thread);
    }

    /// Set is_thread to return an error for a specific channel ID
    pub fn set_is_thread_error(&self, channel_id: ChannelId, error_message: String) {
        self.is_thread_errors
            .lock()
            .unwrap()
            .insert(channel_id, error_message);
    }

    /// Set the channel response for a specific channel ID
    pub fn set_channel(&self, channel_id: ChannelId, channel: GuildChannel) {
        self.channel_responses
            .lock()
            .unwrap()
            .insert(channel_id, channel);
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
        _guild_id: Option<GuildId>,
        channel_id: ChannelId,
    ) -> Result<bool, serenity::Error> {
        // Check for configured error first
        if self.is_thread_errors.lock().unwrap().contains_key(&channel_id) {
            return Err(serenity::Error::Other("Mock error"));
        }

        // Return configured response, default to false if not set
        Ok(self
            .is_thread_responses
            .lock()
            .unwrap()
            .get(&channel_id)
            .copied()
            .unwrap_or(false))
    }

    async fn get_channel(
        &self,
        _guild_id: Option<GuildId>,
        channel_id: ChannelId,
    ) -> Result<Option<GuildChannel>, serenity::Error> {
        // Return configured channel, None if not set
        Ok(self
            .channel_responses
            .lock()
            .unwrap()
            .get(&channel_id)
            .cloned())
    }
}
