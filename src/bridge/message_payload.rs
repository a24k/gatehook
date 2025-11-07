use serde::Serialize;
use serenity::model::channel::{GuildChannel, Message};

/// Payload for message events sent to webhook
///
/// Wraps the original Discord Message with additional channel metadata from cache
#[derive(Serialize)]
pub struct MessagePayload<'a> {
    /// The original Discord message
    #[serde(flatten)]
    pub message: &'a Message,

    /// Guild channel information (if available from cache)
    ///
    /// Contains full channel details including:
    /// - `kind`: Channel type (Text, Voice, PublicThread, PrivateThread, etc.)
    /// - `name`: Channel name
    /// - `parent_id`: Parent channel or category ID
    /// - `topic`: Channel topic/description
    /// - `thread_metadata`: Thread-specific metadata (if applicable)
    /// - And many other fields
    ///
    /// This field is `None` for:
    /// - Direct messages (no guild_id)
    /// - Cache misses (channel not yet cached)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<GuildChannel>,
}

impl<'a> MessagePayload<'a> {
    /// Create a new MessagePayload without channel information
    pub fn new(message: &'a Message) -> Self {
        Self {
            message,
            channel: None,
        }
    }

    /// Create a new MessagePayload with channel information from cache
    pub fn with_channel(message: &'a Message, channel: GuildChannel) -> Self {
        Self {
            message,
            channel: Some(channel),
        }
    }
}
