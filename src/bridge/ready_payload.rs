use serde::Serialize;
use serenity::model::gateway::Ready;

/// Payload for ready events sent to webhook
///
/// Contains the Discord Ready event wrapped in a `ready` key.
///
/// JSON structure:
/// ```json
/// {
///   "ready": { /* Discord Ready fields */ }
/// }
/// ```
#[derive(Serialize)]
pub struct ReadyPayload<'a> {
    /// The Discord ready event
    pub ready: &'a Ready,
}

impl<'a> ReadyPayload<'a> {
    /// Create a new ReadyPayload
    pub fn new(ready: &'a Ready) -> Self {
        Self { ready }
    }
}
