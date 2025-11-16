use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, GuildId, MessageId};

/// Target for webhook response actions.
///
/// Represents the minimal information needed to execute Discord actions
/// (reply, react, thread) on a message. This abstraction allows different
/// event types (Message, Reaction, etc.) to be used as action targets.
///
/// The `guild_id` field enables:
/// - Performance optimization (O(1) cache lookups)
/// - Future guild-specific actions (roles, permissions, etc.)
/// - Clear DM vs Guild context distinction
#[derive(Debug, Clone, Copy)]
pub struct ActionTarget {
    pub message_id: MessageId,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
}

#[cfg(test)]
impl ActionTarget {
    /// Create a new ActionTarget with the given message and channel IDs.
    pub fn new(message_id: MessageId, channel_id: ChannelId) -> Self {
        Self {
            message_id,
            channel_id,
            guild_id: None,
        }
    }
}

/// Convert a Message reference into an ActionTarget.
impl From<&Message> for ActionTarget {
    fn from(message: &Message) -> Self {
        Self {
            message_id: message.id,
            channel_id: message.channel_id,
            guild_id: message.guild_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::model::id::{ChannelId, MessageId};

    #[test]
    fn test_action_target_new() {
        let message_id = MessageId::new(123456789);
        let channel_id = ChannelId::new(987654321);

        let target = ActionTarget::new(message_id, channel_id);

        assert_eq!(target.message_id, message_id);
        assert_eq!(target.channel_id, channel_id);
    }
}
