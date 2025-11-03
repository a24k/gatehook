// Unit tests for EventBridge business logic
// These tests verify that events are correctly processed and forwarded

mod adapters;

use adapters::{MockDiscordService, MockEventSender};
use gatehook::bridge::event_bridge::EventBridge;
use rstest::rstest;
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, MessageId};
use serenity::model::user::User;
use std::sync::Arc;

// Helper function to create a test message
fn create_test_message(content: &str, message_id: u64, channel_id: u64) -> Message {
    let mut message = Message::default();
    message.content = content.to_string();
    message.id = MessageId::new(message_id);
    message.channel_id = ChannelId::new(channel_id);
    message.author = User::default();
    message
}

#[tokio::test]
async fn test_handle_message_ping_pong() {
    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    // Create a dummy HTTP client (token doesn't matter for mock)
    let http = serenity::http::Http::new("dummy_token");

    // Create a "Ping!" message
    let message = create_test_message("Ping!", 123, 456);

    // Execute
    let result = bridge.handle_message(&http, &message).await;

    // Verify
    assert!(result.is_ok(), "handle_message should succeed");

    // Check that Discord reply was sent
    let replies = discord_service.get_replies();
    assert_eq!(replies.len(), 1, "Should send one reply");
    assert_eq!(replies[0].content, "Pong!", "Reply content should be 'Pong!'");
    assert_eq!(
        replies[0].message_id,
        MessageId::new(123),
        "Should reply to correct message"
    );
    assert_eq!(
        replies[0].channel_id,
        ChannelId::new(456),
        "Should reply in correct channel"
    );

    // Check that event was forwarded to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(
        sent_events[0].handler, "message",
        "Event handler should be 'message'"
    );
}

#[tokio::test]
async fn test_handle_message_normal() {
    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    // Create a dummy HTTP client (token doesn't matter for mock)
    let http = serenity::http::Http::new("dummy_token");

    // Create a normal message (not "Ping!")
    let message = create_test_message("Hello, world!", 789, 456);

    // Execute
    let result = bridge.handle_message(&http, &message).await;

    // Verify
    assert!(result.is_ok(), "handle_message should succeed");

    // Check that NO Discord reply was sent
    let replies = discord_service.get_replies();
    assert_eq!(replies.len(), 0, "Should not send any reply");

    // Check that event was forwarded to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(
        sent_events[0].handler, "message",
        "Event handler should be 'message'"
    );
}

// Note: test_handle_ready is skipped because Ready doesn't implement Default
// and creating a valid Ready instance requires extensive setup.
// The ready event forwarding is tested through integration testing instead.

#[rstest]
#[case::without_mention("Reply from webhook", false)]
#[case::with_mention("Reply with mention", true)]
#[tokio::test]
async fn test_execute_actions_reply(
    #[case] expected_content: &str,
    #[case] mention: bool,
) {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_test_message("Test message", 111, 222);

    // Create EventResponse with reply action
    let event_response = EventResponse {
        actions: vec![ResponseAction::Reply {
            content: expected_content.to_string(),
            mention,
        }],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok(), "execute_actions should succeed");

    let replies = discord_service.get_replies();
    assert_eq!(replies.len(), 1, "Should send one reply");
    assert_eq!(replies[0].content, expected_content);
    assert_eq!(replies[0].message_id, MessageId::new(111));
    assert_eq!(replies[0].channel_id, ChannelId::new(222));
    assert_eq!(replies[0].mention, mention);
}

#[tokio::test]
async fn test_execute_actions_multiple_replies() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_test_message("Test", 555, 666);

    // Multiple actions
    let event_response = EventResponse {
        actions: vec![
            ResponseAction::Reply {
                content: "First reply".to_string(),
                mention: false,
            },
            ResponseAction::Reply {
                content: "Second reply".to_string(),
                mention: true,
            },
        ],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok());
    let replies = discord_service.get_replies();
    assert_eq!(replies.len(), 2, "Should send two replies");
    assert_eq!(replies[0].content, "First reply");
    assert!(!replies[0].mention);
    assert_eq!(replies[1].content, "Second reply");
    assert!(replies[1].mention);
}

#[tokio::test]
async fn test_execute_actions_long_content_truncated() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_test_message("Test", 777, 888);

    // Create content over 2000 chars
    let long_content = "a".repeat(2100);

    let event_response = EventResponse {
        actions: vec![ResponseAction::Reply {
            content: long_content,
            mention: false,
        }],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok());
    let replies = discord_service.get_replies();
    assert_eq!(replies.len(), 1);
    // Should be truncated to 2000 chars (1997 + "...")
    assert_eq!(replies[0].content.chars().count(), 2000);
    assert!(replies[0].content.ends_with("..."));
}

#[tokio::test]
async fn test_handle_message_with_webhook_response() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup: MockEventSender with pre-configured response
    let discord_service = Arc::new(MockDiscordService::new());
    let event_response = EventResponse {
        actions: vec![ResponseAction::Reply {
            content: "Webhook responded!".to_string(),
            mention: false,
        }],
    };
    let event_sender = Arc::new(MockEventSender::with_response(event_response));
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_test_message("Hello", 999, 1000);

    // Execute handle_message (which should return the EventResponse)
    let result = bridge.handle_message(&http, &message).await;

    // Verify
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.is_some());
    let response = response.unwrap();
    assert_eq!(response.actions.len(), 1);

    // Event was sent
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1);
}
