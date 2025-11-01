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
    /// * `webhook_url` - The URL of the webhook endpoint
    /// * `insecure_mode` - If true, accept invalid TLS certificates
    pub fn new(webhook_url: String, insecure_mode: bool) -> anyhow::Result<Self> {
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(insecure_mode)
            .build()
            .context("Building HTTP Client")?;

        let endpoint = Url::parse(&webhook_url)
            .context("Parsing webhook URL")?;

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
            .post(self.endpoint.as_str())
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
        let sender = HttpEventSender::new("https://example.com/webhook".to_string(), false);
        assert!(sender.is_ok());
    }

    #[test]
    fn test_http_event_sender_creation_insecure() {
        let sender = HttpEventSender::new("https://example.com/webhook".to_string(), true);
        assert!(sender.is_ok());
    }

    #[test]
    fn test_endpoint_getter() {
        let url_str = "https://example.com/webhook";
        let sender = HttpEventSender::new(url_str.to_string(), false).unwrap();
        assert_eq!(sender.endpoint().as_str(), url_str);
    }
}
