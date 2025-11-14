use serde::Serialize;
use serenity::model::id::{ChannelId, GuildId, MessageId};

/// Payload for MESSAGE_DELETE event
///
/// This payload is sent to the webhook endpoint when a message is deleted.
/// Note that the Discord API only provides IDs, not the message content.
///
/// JSON structure:
/// ```json
/// {
///   "message_delete": {
///     "id": "123...",
///     "channel_id": "456...",
///     "guild_id": "789..." // optional
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct MessageDeletePayload {
    pub message_delete: MessageDelete,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageDelete {
    /// ID of the deleted message
    pub id: MessageId,
    /// ID of the channel where the message was deleted
    pub channel_id: ChannelId,
    /// ID of the guild (None for DMs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
}

impl MessageDeletePayload {
    /// Create a new MessageDeletePayload
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel where the message was deleted
    /// * `message_id` - The ID of the deleted message
    /// * `guild_id` - The guild ID (None for DMs)
    pub fn new(
        channel_id: ChannelId,
        message_id: MessageId,
        guild_id: Option<GuildId>,
    ) -> Self {
        Self {
            message_delete: MessageDelete {
                id: message_id,
                channel_id,
                guild_id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_delete_payload_new() {
        let channel_id = ChannelId::new(999);
        let message_id = MessageId::new(888);
        let guild_id = Some(GuildId::new(777));

        let payload = MessageDeletePayload::new(channel_id, message_id, guild_id);

        assert_eq!(payload.message_delete.id, message_id);
        assert_eq!(payload.message_delete.channel_id, channel_id);
        assert_eq!(payload.message_delete.guild_id, guild_id);
    }

    #[test]
    fn test_message_delete_payload_serialize_with_guild() {
        let payload = MessageDeletePayload::new(
            ChannelId::new(999),
            MessageId::new(888),
            Some(GuildId::new(777)),
        );

        let json = serde_json::to_value(&payload).unwrap();

        assert_eq!(json["message_delete"]["id"], "888");
        assert_eq!(json["message_delete"]["channel_id"], "999");
        assert_eq!(json["message_delete"]["guild_id"], "777");
    }

    #[test]
    fn test_message_delete_payload_serialize_without_guild() {
        let payload = MessageDeletePayload::new(ChannelId::new(999), MessageId::new(888), None);

        let json = serde_json::to_value(&payload).unwrap();

        assert_eq!(json["message_delete"]["id"], "888");
        assert_eq!(json["message_delete"]["channel_id"], "999");
        assert_eq!(json["message_delete"].get("guild_id"), None); // Should be omitted
    }
}
