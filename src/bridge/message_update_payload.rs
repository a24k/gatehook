use serde::Serialize;
use serenity::model::event::MessageUpdateEvent;

/// Payload for MESSAGE_UPDATE event
///
/// This payload is sent to the webhook endpoint when a message is updated.
/// Note that Discord only provides the fields that were changed, along with
/// always-present fields like id and channel_id.
///
/// JSON structure:
/// ```json
/// {
///   "message_update": {
///     "id": "123...",
///     "channel_id": "456...",
///     "guild_id": "789...", // optional
///     "content": "Updated content", // only if content was changed
///     "edited_timestamp": "2024-01-15T12:35:00.000Z",
///     // ... other updated fields
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct MessageUpdatePayload {
    pub message_update: MessageUpdateEvent,
}

impl MessageUpdatePayload {
    /// Create a new MessageUpdatePayload
    ///
    /// # Arguments
    ///
    /// * `event` - The MessageUpdateEvent from Discord
    pub fn new(event: MessageUpdateEvent) -> Self {
        Self {
            message_update: event,
        }
    }
}

// Note: Tests omitted because MessageUpdateEvent is a non-exhaustive struct
// and cannot be constructed in tests. The wrapper is simple enough that
// testing would not provide significant value.
