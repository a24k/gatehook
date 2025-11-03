use super::discord_service::DiscordService;
use serenity::async_trait;
use serenity::model::id::{ChannelId, MessageId};

/// Implementation for Discord operations via Serenity
pub struct SerenityDiscordService;

#[async_trait]
impl DiscordService for SerenityDiscordService {
    async fn reply_to_message(
        &self,
        http: &serenity::http::Http,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
        mention: bool,
    ) -> Result<(), serenity::Error> {
        use serenity::builder::{CreateAllowedMentions, CreateMessage};

        let builder = CreateMessage::new()
            .content(content)
            .reference_message((channel_id, message_id))
            .allowed_mentions(CreateAllowedMentions::new().replied_user(mention));

        channel_id.send_message(http, builder).await?;
        Ok(())
    }
}
