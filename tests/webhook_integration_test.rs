// Integration tests for webhook functionality
// These tests verify the WebhookClient behavior without requiring a real server

use gatehook::webhook::WebhookClient;

#[test]
fn test_webhook_client_new_normal_mode() {
    let result = WebhookClient::new("https://example.com/webhook".to_string(), false);
    assert!(result.is_ok());
}

#[test]
fn test_webhook_client_new_insecure_mode() {
    let result = WebhookClient::new("https://example.com/webhook".to_string(), true);
    assert!(result.is_ok());
}

#[test]
fn test_webhook_url_getter() {
    let url = "https://example.com/webhook".to_string();
    let client = WebhookClient::new(url.clone(), false).unwrap();
    assert_eq!(client.webhook_url(), url);
}

// Note: Testing the actual send() method would require a mock HTTP server
// For now, we test the creation and basic functionality
// Future: Add integration tests with a mock server using tools like wiremock
