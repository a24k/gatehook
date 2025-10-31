use serenity::async_trait;
use serenity::model::id::{ChannelId, MessageId};
use std::sync::Arc;

/// Discord操作のインターフェース
#[async_trait]
pub trait DiscordService: Send + Sync {
    /// Reply to a message in a channel
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel where the message was sent
    /// * `message_id` - The message to reply to
    /// * `content` - The content of the reply
    async fn reply_to_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> Result<(), serenity::Error>;
}

/// Serenity経由でDiscord操作を行う実装
pub struct SerenityDiscordService {
    http: Arc<serenity::http::Http>,
}

impl SerenityDiscordService {
    /// Create a new SerenityDiscordService
    ///
    /// # Arguments
    ///
    /// * `http` - The serenity HTTP client
    pub fn new(http: Arc<serenity::http::Http>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl DiscordService for SerenityDiscordService {
    async fn reply_to_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> Result<(), serenity::Error> {
        use serenity::builder::CreateMessage;

        let builder = CreateMessage::new()
            .content(content)
            .reference_message((channel_id, message_id));

        channel_id.send_message(&self.http, builder).await?;
        Ok(())
    }
}

/// Test用のモック実装とヘルパー
pub mod test_helpers {
    use super::*;
    use std::sync::{Arc, Mutex};

    pub struct MockDiscordService {
        pub replies: Arc<Mutex<Vec<Reply>>>,
    }

    #[derive(Debug, Clone)]
    pub struct Reply {
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

        pub fn get_replies(&self) -> Vec<Reply> {
            self.replies.lock().unwrap().clone()
        }

        pub fn clear(&self) {
            self.replies.lock().unwrap().clear()
        }
    }

    #[async_trait]
    impl DiscordService for MockDiscordService {
        async fn reply_to_message(
            &self,
            channel_id: ChannelId,
            message_id: MessageId,
            content: &str,
        ) -> Result<(), serenity::Error> {
            self.replies.lock().unwrap().push(Reply {
                channel_id,
                message_id,
                content: content.to_string(),
            });
            Ok(())
        }
    }
}
