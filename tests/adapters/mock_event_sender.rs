use gatehook::adapters::event_sender::EventSender;
use serde::Serialize;
use serde_json;
use serenity::async_trait;
use std::sync::{Arc, Mutex};

pub struct MockEventSender {
    pub sent_events: Arc<Mutex<Vec<SentEvent>>>,
}

#[derive(Debug, Clone)]
pub struct SentEvent {
    pub handler: String,
    #[allow(dead_code)]
    pub payload: String,
}

impl Default for MockEventSender {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEventSender {
    pub fn new() -> Self {
        Self {
            sent_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_sent_events(&self) -> Vec<SentEvent> {
        self.sent_events.lock().unwrap().clone()
    }
}

#[async_trait]
impl EventSender for MockEventSender {
    async fn send<T: Serialize + Send + Sync>(
        &self,
        handler: &str,
        payload: &T,
    ) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(payload)?;
        self.sent_events.lock().unwrap().push(SentEvent {
            handler: handler.to_string(),
            payload: payload_json,
        });
        Ok(())
    }
}
