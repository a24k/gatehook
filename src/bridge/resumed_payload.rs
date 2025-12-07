use serde::Serialize;
use serenity::model::event::ResumedEvent;

/// Payload for resumed events sent to webhook
///
/// Contains the Discord Resumed event wrapped in a `resumed` key.
/// This event is sent when a client has successfully resumed a previously
/// disconnected session.
///
/// JSON structure:
/// ```json
/// {
///   "resumed": { /* Discord Resumed event fields */ }
/// }
/// ```
#[derive(Serialize)]
pub struct ResumedPayload<'a> {
    /// The Discord resumed event
    pub resumed: &'a ResumedEvent,
}

impl<'a> ResumedPayload<'a> {
    /// Create a new ResumedPayload
    pub fn new(resumed: &'a ResumedEvent) -> Self {
        Self { resumed }
    }
}
