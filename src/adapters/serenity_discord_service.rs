use super::discord_service::DiscordService;
use serenity::async_trait;
use serenity::model::id::{ChannelId, MessageId};

/// Serenity経由でDiscord操作を行う実装
pub struct SerenityDiscordService;

#[async_trait]
impl DiscordService for SerenityDiscordService {
    async fn reply_to_message(
        &self,
        http: &serenity::http::Http,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> Result<(), serenity::Error> {
        use serenity::builder::CreateMessage;

        let builder = CreateMessage::new()
            .content(content)
            .reference_message((channel_id, message_id));

        channel_id.send_message(http, builder).await?;
        Ok(())
    }
}
