use serde::Serialize;
use serenity::model::id::{ChannelId, GuildId, MessageId};

/// Payload for MESSAGE_DELETE_BULK event
///
/// This payload is sent to the webhook endpoint when multiple messages are deleted at once.
/// Bulk deletion typically occurs when moderators use Discord's bulk delete feature.
/// Note that the Discord API only provides IDs, not the message content.
///
/// JSON structure:
/// ```json
/// {
///   "message_delete_bulk": {
///     "ids": ["123...", "456...", "789..."],
///     "channel_id": "111...",
///     "guild_id": "222..." // optional
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct MessageDeleteBulkPayload {
    pub message_delete_bulk: MessageDeleteBulk,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageDeleteBulk {
    /// IDs of the deleted messages
    pub ids: Vec<MessageId>,
    /// ID of the channel where messages were deleted
    pub channel_id: ChannelId,
    /// ID of the guild (bulk delete only occurs in guilds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
}

impl MessageDeleteBulkPayload {
    /// Create a new MessageDeleteBulkPayload
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel where messages were deleted
    /// * `message_ids` - The IDs of deleted messages
    /// * `guild_id` - The guild ID (bulk delete is typically guild-only)
    pub fn new(
        channel_id: ChannelId,
        message_ids: Vec<MessageId>,
        guild_id: Option<GuildId>,
    ) -> Self {
        Self {
            message_delete_bulk: MessageDeleteBulk {
                ids: message_ids,
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
    fn test_message_delete_bulk_payload_new() {
        let channel_id = ChannelId::new(999);
        let message_ids = vec![
            MessageId::new(111),
            MessageId::new(222),
            MessageId::new(333),
        ];
        let guild_id = Some(GuildId::new(777));

        let payload = MessageDeleteBulkPayload::new(channel_id, message_ids.clone(), guild_id);

        assert_eq!(payload.message_delete_bulk.ids, message_ids);
        assert_eq!(payload.message_delete_bulk.channel_id, channel_id);
        assert_eq!(payload.message_delete_bulk.guild_id, guild_id);
    }

    #[test]
    fn test_message_delete_bulk_payload_serialize_with_guild() {
        let payload = MessageDeleteBulkPayload::new(
            ChannelId::new(999),
            vec![MessageId::new(111), MessageId::new(222), MessageId::new(333)],
            Some(GuildId::new(777)),
        );

        let json = serde_json::to_value(&payload).unwrap();

        assert_eq!(json["message_delete_bulk"]["ids"].as_array().unwrap().len(), 3);
        assert_eq!(json["message_delete_bulk"]["ids"][0], "111");
        assert_eq!(json["message_delete_bulk"]["ids"][1], "222");
        assert_eq!(json["message_delete_bulk"]["ids"][2], "333");
        assert_eq!(json["message_delete_bulk"]["channel_id"], "999");
        assert_eq!(json["message_delete_bulk"]["guild_id"], "777");
    }

    #[test]
    fn test_message_delete_bulk_payload_serialize_without_guild() {
        let payload = MessageDeleteBulkPayload::new(
            ChannelId::new(999),
            vec![MessageId::new(111), MessageId::new(222)],
            None,
        );

        let json = serde_json::to_value(&payload).unwrap();

        assert_eq!(json["message_delete_bulk"]["ids"].as_array().unwrap().len(), 2);
        assert_eq!(json["message_delete_bulk"]["ids"][0], "111");
        assert_eq!(json["message_delete_bulk"]["ids"][1], "222");
        assert_eq!(json["message_delete_bulk"]["channel_id"], "999");
        assert_eq!(json["message_delete_bulk"].get("guild_id"), None); // Should be omitted
    }

    #[test]
    fn test_message_delete_bulk_payload_empty_ids() {
        let payload = MessageDeleteBulkPayload::new(ChannelId::new(999), vec![], None);

        let json = serde_json::to_value(&payload).unwrap();

        assert_eq!(json["message_delete_bulk"]["ids"].as_array().unwrap().len(), 0);
    }
}
