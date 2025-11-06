use gatehook::adapters::DiscordService;
use serenity::async_trait;
use serenity::model::channel::{GuildChannel, Message};
use serenity::model::id::{ChannelId, GuildId, MessageId};
use std::sync::{Arc, Mutex};

pub struct MockDiscordService {
    pub replies: Arc<Mutex<Vec<RecordedReply>>>,
    pub reactions: Arc<Mutex<Vec<RecordedReaction>>>,
    pub threads: Arc<Mutex<Vec<RecordedThread>>>,
    pub messages: Arc<Mutex<Vec<RecordedMessage>>>,
    pub is_thread: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub struct RecordedReply {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub content: String,
    pub mention: bool,
}

#[derive(Debug, Clone)]
pub struct RecordedReaction {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub emoji: String,
}

#[derive(Debug, Clone)]
pub struct RecordedThread {
    pub message_id: MessageId,
    pub name: String,
    pub auto_archive_duration: u16,
}

#[derive(Debug, Clone)]
pub struct RecordedMessage {
    pub channel_id: ChannelId,
    pub content: String,
    pub reply_to: Option<MessageId>,
    pub mention: bool,
}

impl Default for MockDiscordService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockDiscordService {
    pub fn new() -> Self {
        Self {
            replies: Arc::new(Mutex::new(Vec::new())),
            reactions: Arc::new(Mutex::new(Vec::new())),
            threads: Arc::new(Mutex::new(Vec::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
            is_thread: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_is_thread(&self, value: bool) {
        *self.is_thread.lock().unwrap() = value;
    }

    pub fn get_replies(&self) -> Vec<RecordedReply> {
        self.replies.lock().unwrap().clone()
    }

    pub fn get_reactions(&self) -> Vec<RecordedReaction> {
        self.reactions.lock().unwrap().clone()
    }

    pub fn get_threads(&self) -> Vec<RecordedThread> {
        self.threads.lock().unwrap().clone()
    }

    pub fn get_messages(&self) -> Vec<RecordedMessage> {
        self.messages.lock().unwrap().clone()
    }
}

#[async_trait]
impl DiscordService for MockDiscordService {
    async fn react_to_message(
        &self,
        _http: &serenity::http::Http,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &str,
    ) -> Result<(), serenity::Error> {
        self.reactions.lock().unwrap().push(RecordedReaction {
            channel_id,
            message_id,
            emoji: emoji.to_string(),
        });
        Ok(())
    }

    async fn create_thread_from_message(
        &self,
        _http: &serenity::http::Http,
        message: &Message,
        name: &str,
        auto_archive_duration: u16,
    ) -> Result<GuildChannel, serenity::Error> {
        self.threads.lock().unwrap().push(RecordedThread {
            message_id: message.id,
            name: name.to_string(),
            auto_archive_duration,
        });

        // Return a dummy GuildChannel
        // Note: In real tests, we use the recorded data to verify behavior
        Ok(create_dummy_guild_channel(message.channel_id))
    }

    async fn send_message_to_channel(
        &self,
        _http: &serenity::http::Http,
        channel_id: ChannelId,
        content: &str,
    ) -> Result<Message, serenity::Error> {
        self.messages.lock().unwrap().push(RecordedMessage {
            channel_id,
            content: content.to_string(),
            reply_to: None,
            mention: false,
        });

        // Return a dummy Message
        Ok(create_dummy_message(channel_id, content))
    }

    async fn reply_in_channel(
        &self,
        _http: &serenity::http::Http,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
        mention: bool,
    ) -> Result<Message, serenity::Error> {
        // Record in both replies and messages for backward compatibility
        self.replies.lock().unwrap().push(RecordedReply {
            channel_id,
            message_id,
            content: content.to_string(),
            mention,
        });

        self.messages.lock().unwrap().push(RecordedMessage {
            channel_id,
            content: content.to_string(),
            reply_to: Some(message_id),
            mention,
        });

        // Return a dummy Message
        Ok(create_dummy_message(channel_id, content))
    }

    async fn is_thread_channel(
        &self,
        _http: &serenity::http::Http,
        _channel_id: ChannelId,
    ) -> Result<bool, serenity::Error> {
        Ok(*self.is_thread.lock().unwrap())
    }
}

// Helper function to create dummy GuildChannel for testing
fn create_dummy_guild_channel(channel_id: ChannelId) -> GuildChannel {
    // Use default and override specific fields
    let mut channel = GuildChannel::default();
    channel.id = channel_id;
    channel.guild_id = GuildId::new(1);
    channel.name = "test-thread".to_string();
    channel
}

// Helper function to create dummy Message for testing
fn create_dummy_message(channel_id: ChannelId, content: &str) -> Message {
    // Use default and override specific fields
    let mut message = Message::default();
    message.id = MessageId::new(1);
    message.channel_id = channel_id;
    message.content = content.to_string();
    message
}
