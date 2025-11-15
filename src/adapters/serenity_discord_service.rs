use super::discord_service::DiscordService;
use serenity::async_trait;
use serenity::model::channel::{AutoArchiveDuration, GuildChannel, Message};
use serenity::model::id::{ChannelId, MessageId};
use std::sync::Arc;

/// Implementation for Discord operations via Serenity
///
/// Holds a reference to the HTTP client that is maintained by Serenity's event loop.
pub struct SerenityDiscordService {
    http: Arc<serenity::http::Http>,
}

impl SerenityDiscordService {
    /// Create a new SerenityDiscordService with an HTTP client reference
    pub fn new(http: Arc<serenity::http::Http>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl DiscordService for SerenityDiscordService {
    async fn react_to_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &str,
    ) -> Result<(), serenity::Error> {
        use serenity::model::channel::ReactionType;

        // Parse emoji (Unicode or custom emoji format "name:id")
        let reaction_type = if let Some((name, id)) = emoji.split_once(':') {
            // Custom emoji format "name:id"
            ReactionType::Custom {
                animated: false,
                id: id.parse().map_err(|_| {
                    serenity::Error::Other("Invalid custom emoji ID")
                })?,
                name: Some(name.to_string()),
            }
        } else {
            // Unicode emoji
            ReactionType::Unicode(emoji.to_string())
        };

        self.http.create_reaction(channel_id, message_id, &reaction_type)
            .await?;
        Ok(())
    }

    async fn create_thread_from_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        name: &str,
        auto_archive_duration: u16,
    ) -> Result<GuildChannel, serenity::Error> {
        use serenity::builder::CreateThread;
        use tracing::warn;

        // Convert auto_archive_duration to enum
        let auto_archive_duration = match auto_archive_duration {
            60 => AutoArchiveDuration::OneHour,
            1440 => AutoArchiveDuration::OneDay,
            4320 => AutoArchiveDuration::ThreeDays,
            10080 => AutoArchiveDuration::OneWeek,
            invalid => {
                warn!(
                    invalid_value = invalid,
                    "Invalid auto_archive_duration, using default (1440 = OneDay)"
                );
                AutoArchiveDuration::OneDay
            }
        };

        let builder = CreateThread::new(name.to_string())
            .auto_archive_duration(auto_archive_duration);

        channel_id
            .create_thread_from_message(&self.http, message_id, builder)
            .await
    }

    async fn send_message_to_channel(
        &self,
        channel_id: ChannelId,
        content: &str,
    ) -> Result<Message, serenity::Error> {
        use serenity::builder::CreateMessage;

        let builder = CreateMessage::new().content(content);
        channel_id.send_message(&self.http, builder).await
    }

    async fn reply_in_channel(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
        mention: bool,
    ) -> Result<Message, serenity::Error> {
        use serenity::builder::{CreateAllowedMentions, CreateMessage};

        let builder = CreateMessage::new()
            .content(content)
            .reference_message((channel_id, message_id))
            .allowed_mentions(CreateAllowedMentions::new().replied_user(mention));

        channel_id.send_message(&self.http, builder).await
    }
}
