use crate::adapters::event_response::EventResponse;
use serde::Serialize;
use serenity::async_trait;

/// Interface for sending events to external endpoints
#[async_trait]
pub trait EventSender: Send + Sync {
    /// Send an event and receive response
    ///
    /// # Arguments
    ///
    /// * `handler` - Handler name (e.g., "message", "ready")
    /// * `payload` - Payload to send (will be serialized to JSON)
    ///
    /// # Returns
    ///
    /// * `Ok(Some(EventResponse))` - Response was successfully parsed
    /// * `Ok(None)` - No response or response could not be parsed
    /// * `Err(_)` - Failed to send the request
    async fn send<T: Serialize + Send + Sync>(
        &self,
        handler: &str,
        payload: &T,
    ) -> anyhow::Result<Option<EventResponse>>;
}
