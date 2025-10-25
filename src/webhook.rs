use anyhow::Context as _;
use serde::Serialize;
use tracing::{error, info};

/// Client for sending events to webhook endpoints
pub struct WebhookClient {
    client: reqwest::Client,
    webhook_url: String,
}

impl WebhookClient {
    /// Create a new WebhookClient
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

        Ok(Self {
            client,
            webhook_url,
        })
    }

    /// Get the webhook URL
    pub fn webhook_url(&self) -> &str {
        &self.webhook_url
    }

    /// Send a payload to the webhook endpoint
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler name (e.g., "message", "reaction")
    /// * `payload` - The payload to send (will be serialized as JSON)
    ///
    /// # Returns
    ///
    /// Returns the HTTP response from the webhook endpoint
    pub async fn send<T: Serialize>(
        &self,
        handler: &str,
        payload: &T,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(&self.webhook_url)
            .query(&[("handler", handler)])
            .json(payload)
            .send()
            .await
    }

    /// Send a payload to the webhook endpoint with logging
    ///
    /// This is a convenience method that wraps `send()` and logs the result.
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler name (e.g., "message", "reaction")
    /// * `payload` - The payload to send
    /// * `event_id` - An identifier for the event (for logging purposes)
    pub async fn send_with_logging<T: Serialize>(
        &self,
        handler: &str,
        payload: &T,
        event_id: &str,
    ) {
        match self.send(handler, payload).await {
            Ok(response) => {
                info!(
                    status = %response.status(),
                    event_id = %event_id,
                    handler = %handler,
                    "Successfully sent event to webhook"
                );
            }
            Err(err) => {
                error!(
                    error = ?err,
                    event_id = %event_id,
                    handler = %handler,
                    webhook_url = %self.webhook_url,
                    "Failed to send event to webhook"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_client_creation() {
        let client = WebhookClient::new("https://example.com/webhook".to_string(), false);
        assert!(client.is_ok());
    }

    #[test]
    fn test_webhook_client_creation_insecure() {
        let client = WebhookClient::new("https://example.com/webhook".to_string(), true);
        assert!(client.is_ok());
    }
}
