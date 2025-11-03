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

        let mut builder = CreateMessage::new()
            .content(content)
            .reference_message((channel_id, message_id));

        // Configure mention settings
        if mention {
            // Enable mention notification (ping the user)
            builder = builder.allowed_mentions(CreateAllowedMentions::new().replied_user(true));
        } else {
            // Disable mention notification (no ping)
            builder = builder.allowed_mentions(CreateAllowedMentions::new().replied_user(false));
        }

        channel_id.send_message(http, builder).await?;
        Ok(())
    }
}
