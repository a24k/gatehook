use super::event_sender_trait::EventSender;
use anyhow::Context as _;
use serde::Serialize;
use serenity::async_trait;
use tracing::{error, info};
use url::Url;

/// HTTP経由でイベントを送信する実装
pub struct HttpEventSender {
    client: reqwest::Client,
    endpoint: Url,
}

impl HttpEventSender {
    /// Create a new HttpEventSender
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The HTTP endpoint URL
    /// * `insecure_mode` - If true, accept invalid TLS certificates
    pub fn new(endpoint: Url, insecure_mode: bool) -> anyhow::Result<Self> {
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(insecure_mode)
            .build()
            .context("Building HTTP Client")?;

        Ok(Self {
            client,
            endpoint,
        })
    }

    /// Get the endpoint URL (for testing)
    #[cfg(test)]
    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    /// Send a payload to the webhook endpoint (low-level)
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler name (e.g., "message", "reaction")
    /// * `payload` - The payload to send (will be serialized as JSON)
    async fn send_request<T: Serialize>(
        &self,
        handler: &str,
        payload: &T,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(self.endpoint.clone())
            .query(&[("handler", handler)])
            .json(payload)
            .send()
            .await
    }
}

#[async_trait]
impl EventSender for HttpEventSender {
    async fn send<T: Serialize + Send + Sync>(
        &self,
        handler: &str,
        payload: &T,
    ) -> anyhow::Result<()> {
        match self.send_request(handler, payload).await {
            Ok(response) => {
                info!(
                    status = %response.status(),
                    handler = %handler,
                    "Successfully sent event to webhook"
                );
                Ok(())
            }
            Err(err) => {
                error!(
                    error = ?err,
                    handler = %handler,
                    endpoint = %self.endpoint,
                    "Failed to send event to webhook"
                );
                Err(err.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_event_sender_creation() {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let sender = HttpEventSender::new(url, false);
        assert!(sender.is_ok());
    }

    #[test]
    fn test_http_event_sender_creation_insecure() {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let sender = HttpEventSender::new(url, true);
        assert!(sender.is_ok());
    }

    #[test]
    fn test_endpoint_getter() {
        let url_str = "https://example.com/webhook";
        let url = Url::parse(url_str).unwrap();
        let sender = HttpEventSender::new(url, false).unwrap();
        assert_eq!(sender.endpoint().as_str(), url_str);
    }
}
