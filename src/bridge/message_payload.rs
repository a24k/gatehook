use serde::Serialize;
use serenity::model::channel::Message;

/// Payload for message events sent to webhook
///
/// Wraps the original Discord Message with additional metadata
#[derive(Serialize)]
pub struct MessagePayload<'a> {
    /// The original Discord message
    #[serde(flatten)]
    pub message: &'a Message,

    /// Whether the message was sent in a thread channel
    ///
    /// - `true`: Message is in a thread (Public/Private/News)
    /// - `false`: Message is in a regular channel or DM
    /// - `None`: Thread detection was not performed (e.g., DM without guild_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_thread: Option<bool>,
}

impl<'a> MessagePayload<'a> {
    /// Create a new MessagePayload without thread detection
    pub fn new(message: &'a Message) -> Self {
        Self {
            message,
            is_thread: None,
        }
    }

    /// Create a new MessagePayload with thread detection result
    pub fn with_thread_info(message: &'a Message, is_thread: bool) -> Self {
        Self {
            message,
            is_thread: Some(is_thread),
        }
    }
}
