use super::event_response::EventResponse;
use super::event_sender_trait::EventSender;
use anyhow::Context as _;
use serde::Serialize;
use serenity::async_trait;
use tracing::{error, info, warn};
use url::Url;

/// Implementation for sending events via HTTP
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
}

#[async_trait]
impl EventSender for HttpEventSender {
    async fn send<T: Serialize + Send + Sync>(
        &self,
        handler: &str,
        payload: &T,
    ) -> anyhow::Result<Option<EventResponse>> {
        let response = self
            .client
            .post(self.endpoint.clone())
            .query(&[("handler", handler)])
            .json(payload)
            .send()
            .await?;

        let status = response.status();

        // Try to parse the body regardless of status code
        match response.json::<EventResponse>().await {
            Ok(event_response) => {
                let action_count = event_response.actions.len();
                if status.is_success() {
                    info!(
                        %status,
                        %handler,
                        actions = action_count,
                        "HTTP endpoint returned success status, response body parsed"
                    );
                } else {
                    warn!(
                        %status,
                        %handler,
                        actions = action_count,
                        "HTTP endpoint returned non-success status, response body parsed"
                    );
                }
                Ok(Some(event_response))
            }
            Err(err) => {
                if status.is_success() {
                    error!(
                        ?err,
                        %status,
                        %handler,
                        "HTTP endpoint returned success status, response body could not be parsed"
                    );
                } else {
                    error!(
                        ?err,
                        %status,
                        %handler,
                        "HTTP endpoint returned non-success status, response body could not be parsed"
                    );
                }
                Ok(None)
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
