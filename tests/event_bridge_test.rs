// Unit tests for EventBridge business logic
// These tests verify that events are correctly processed and forwarded

use gatehook::logic::event_bridge::EventBridge;
use gatehook::services::discord::test_helpers::MockDiscordService;
use gatehook::services::event_sender::test_helpers::MockEventSender;
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

    // Create a "Ping!" message
    let message = create_test_message("Ping!", 123, 456);

    // Execute
    let result = bridge.handle_message(&message).await;

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

    // Create a normal message (not "Ping!")
    let message = create_test_message("Hello, world!", 789, 456);

    // Execute
    let result = bridge.handle_message(&message).await;

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
