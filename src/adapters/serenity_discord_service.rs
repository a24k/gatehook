use super::discord_service::DiscordService;
use serenity::async_trait;
use serenity::model::id::{ChannelId, MessageId};
use std::sync::Arc;

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
