use gatehook::adapters::DiscordService;
use serenity::async_trait;
use serenity::model::id::{ChannelId, MessageId};
use std::sync::{Arc, Mutex};

pub struct MockDiscordService {
    pub replies: Arc<Mutex<Vec<RecordedReply>>>,
}

#[derive(Debug, Clone)]
pub struct RecordedReply {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub content: String,
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
        }
    }

    pub fn get_replies(&self) -> Vec<RecordedReply> {
        self.replies.lock().unwrap().clone()
    }
}

#[async_trait]
impl DiscordService for MockDiscordService {
    async fn reply_to_message(
        &self,
        _http: &serenity::http::Http,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> Result<(), serenity::Error> {
        self.replies.lock().unwrap().push(RecordedReply {
            channel_id,
            message_id,
            content: content.to_string(),
        });
        Ok(())
    }
}
