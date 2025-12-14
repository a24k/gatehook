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
    max_response_body_size: usize,
}

impl HttpEventSender {
    /// Create a new HttpEventSender
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The HTTP endpoint URL
    /// * `insecure_mode` - If true, accept invalid TLS certificates
    /// * `timeout_secs` - Request timeout in seconds
    /// * `connect_timeout_secs` - Connection timeout in seconds
    /// * `max_response_body_size` - Maximum response body size in bytes (for DoS protection)
    pub fn new(
        endpoint: Url,
        insecure_mode: bool,
        timeout_secs: u64,
        connect_timeout_secs: u64,
        max_response_body_size: usize,
    ) -> anyhow::Result<Self> {
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(insecure_mode)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .connect_timeout(std::time::Duration::from_secs(connect_timeout_secs))
            .build()
            .context("Building HTTP Client")?;

        Ok(Self {
            client,
            endpoint,
            max_response_body_size,
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
        let mut response = self
            .client
            .post(self.endpoint.clone())
            .query(&[("handler", handler)])
            .json(payload)
            .send()
            .await?;

        let status = response.status();

        // Read response body with streaming (DoS protection)
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            // Check size before adding chunk
            if body.len() + chunk.len() > self.max_response_body_size {
                warn!(
                    %handler,
                    %status,
                    current_size = body.len(),
                    chunk_size = chunk.len(),
                    max_size = self.max_response_body_size,
                    "Response body exceeds limit during streaming, rejecting"
                );
                return Ok(None);
            }
            body.extend_from_slice(&chunk);
        }

        // Try to parse the body regardless of status code
        match serde_json::from_slice::<EventResponse>(&body) {
            Ok(event_response) => {
                let action_count = event_response.actions.len();
                if status.is_success() {
                    info!(
                        %handler,
                        %status,
                        actions = action_count,
                        "HTTP endpoint returned success status, response body parsed"
                    );
                } else {
                    warn!(
                        %handler,
                        %status,
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
                        %handler,
                        %status,
                        "HTTP endpoint returned success status, response body could not be parsed"
                    );
                } else {
                    error!(
                        ?err,
                        %handler,
                        %status,
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
    use rstest::rstest;

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_http_event_sender_creation(#[case] insecure_mode: bool) {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let sender = HttpEventSender::new(url, insecure_mode, 300, 10, 131_072);
        assert!(sender.is_ok());
    }

    #[test]
    fn test_endpoint_getter() {
        let url_str = "https://example.com/webhook";
        let url = Url::parse(url_str).unwrap();
        let sender = HttpEventSender::new(url, false, 300, 10, 131_072).unwrap();
        assert_eq!(sender.endpoint().as_str(), url_str);
    }
}
