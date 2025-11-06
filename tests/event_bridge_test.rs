// Unit tests for EventBridge business logic
// These tests verify that events are correctly processed and forwarded

mod adapters;

use adapters::{MockDiscordService, MockEventSender};
use gatehook::adapters::{ReactParams, ReplyParams, ThreadParams};
use gatehook::bridge::event_bridge::EventBridge;
use rstest::rstest;
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, GuildId, MessageId};
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

// Helper function to create a guild test message
fn create_guild_message(content: &str, message_id: u64, channel_id: u64, guild_id: u64) -> Message {
    let mut message = create_test_message(content, message_id, channel_id);
    message.guild_id = Some(GuildId::new(guild_id));
    message
}

#[tokio::test]
async fn test_handle_message_forwards_to_webhook() {
    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    // Create a message
    let message = create_test_message("Hello, world!", 789, 456);

    // Execute
    let result = bridge.handle_message(&message).await;

    // Verify
    assert!(result.is_ok(), "handle_message should succeed");

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
        actions: vec![ResponseAction::Reply(ReplyParams {
            content: expected_content.to_string(),
            mention,
        })],
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
            ResponseAction::Reply(ReplyParams {
                content: "First reply".to_string(),
                mention: false,
            }),
            ResponseAction::Reply(ReplyParams {
                content: "Second reply".to_string(),
                mention: true,
            }),
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
        actions: vec![ResponseAction::Reply(ReplyParams {
            content: long_content,
            mention: false,
        })],
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
        actions: vec![ResponseAction::Reply(ReplyParams {
            content: "Webhook responded!".to_string(),
            mention: false,
        })],
    };
    let event_sender = Arc::new(MockEventSender::with_response(event_response));
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let message = create_test_message("Hello", 999, 1000);

    // Execute handle_message (which should return the EventResponse)
    let result = bridge.handle_message(&message).await;

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

#[rstest]
#[case::unicode_emoji("👍")]
#[case::custom_emoji("customemoji:123456789")]
#[tokio::test]
async fn test_execute_actions_react(#[case] emoji: &str) {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_test_message("Test message", 111, 222);

    // Create EventResponse with react action
    let event_response = EventResponse {
        actions: vec![ResponseAction::React(ReactParams {
            emoji: emoji.to_string(),
        })],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok(), "execute_actions should succeed");

    let reactions = discord_service.get_reactions();
    assert_eq!(reactions.len(), 1, "Should add one reaction");
    assert_eq!(reactions[0].emoji, emoji);
    assert_eq!(reactions[0].message_id, MessageId::new(111));
    assert_eq!(reactions[0].channel_id, ChannelId::new(222));
}

#[tokio::test]
async fn test_execute_actions_thread_create_new() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    discord_service.set_is_thread(false); // Not in thread
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_guild_message("Original message", 111, 222, 333);

    // Create EventResponse with thread action
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some("Discussion".to_string()),
            content: "Let's discuss".to_string(),
            reply: false,
            mention: false,
            auto_archive_duration: 1440,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok(), "execute_actions should succeed");

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1, "Should create one thread");
    assert_eq!(threads[0].name, "Discussion");
    assert_eq!(threads[0].message_id, MessageId::new(111));
    // auto_archive_duration is converted to enum at execution time
    assert_eq!(threads[0].auto_archive_duration, serenity::model::channel::AutoArchiveDuration::OneDay);

    let messages = discord_service.get_messages();
    assert_eq!(messages.len(), 1, "Should send one message");
    assert_eq!(messages[0].content, "Let's discuss");
    assert_eq!(messages[0].channel_id, ChannelId::new(222));
    assert_eq!(messages[0].reply_to, None);
}

#[tokio::test]
async fn test_execute_actions_thread_auto_name() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    discord_service.set_is_thread(false);
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_guild_message("This is the original message content", 111, 222, 333);

    // Thread action without name (should auto-generate)
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: None,
            content: "Response".to_string(),
            reply: false,
            mention: false,
            auto_archive_duration: 1440,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok());

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].name, "This is the original message content");
}

#[tokio::test]
async fn test_execute_actions_thread_long_name() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    discord_service.set_is_thread(false);
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_guild_message("Original message", 111, 222, 333);

    // Thread action with name exceeding 100 chars (should be truncated)
    let long_name = "a".repeat(150); // 150 characters
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some(long_name),
            content: "Response".to_string(),
            reply: false,
            mention: false,
            auto_archive_duration: 1440,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok());

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1);
    // Name should be truncated to 100 characters
    assert_eq!(threads[0].name.chars().count(), 100);
    assert_eq!(threads[0].name, "a".repeat(100));
}

#[tokio::test]
async fn test_execute_actions_thread_already_in_thread() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    discord_service.set_is_thread(true); // Already in thread
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_guild_message("Thread message", 111, 222, 333);

    // Thread action (should skip thread creation)
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some("Ignored".to_string()),
            content: "Reply in thread".to_string(),
            reply: false,
            mention: false,
            auto_archive_duration: 1440,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok());

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 0, "Should NOT create new thread");

    let messages = discord_service.get_messages();
    assert_eq!(messages.len(), 1, "Should send message to existing thread");
    assert_eq!(messages[0].content, "Reply in thread");
    assert_eq!(messages[0].channel_id, ChannelId::new(222));
}

#[tokio::test]
async fn test_execute_actions_thread_with_reply() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    discord_service.set_is_thread(false);
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_guild_message("Original", 111, 222, 333);

    // Thread action with reply
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some("Support".to_string()),
            content: "Help needed".to_string(),
            reply: true,
            mention: true,
            auto_archive_duration: 60,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok());

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1);

    let messages = discord_service.get_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Help needed");
    assert_eq!(messages[0].channel_id, ChannelId::new(222));
    assert_eq!(messages[0].reply_to, Some(MessageId::new(111)));
    assert!(messages[0].mention);
}

#[tokio::test]
async fn test_execute_actions_thread_in_dm_fails() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_test_message("DM message", 111, 222); // No guild_id

    // Thread action
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some("Thread".to_string()),
            content: "Content".to_string(),
            reply: false,
            mention: false,
            auto_archive_duration: 1440,
        })],
    };

    // Execute (should complete but log error)
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // execute_actions continues on error, so result is Ok
    assert!(result.is_ok());

    // But no thread should be created
    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 0, "Should NOT create thread in DM");
}

#[tokio::test]
async fn test_execute_actions_mixed_types() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    discord_service.set_is_thread(false);
    let event_sender = Arc::new(MockEventSender::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone());

    let http = serenity::http::Http::new("dummy_token");
    let message = create_guild_message("Test", 111, 222, 333);

    // Multiple different action types
    let event_response = EventResponse {
        actions: vec![
            ResponseAction::Reply(ReplyParams {
                content: "Reply message".to_string(),
                mention: false,
            }),
            ResponseAction::React(ReactParams {
                emoji: "👍".to_string(),
            }),
            ResponseAction::Thread(ThreadParams {
                name: Some("Discussion".to_string()),
                content: "Thread content".to_string(),
                reply: false,
                mention: false,
                auto_archive_duration: 1440,
            }),
        ],
    };

    // Execute
    let result = bridge.execute_actions(&http, &message, &event_response).await;

    // Verify
    assert!(result.is_ok());

    let replies = discord_service.get_replies();
    assert_eq!(replies.len(), 1);

    let reactions = discord_service.get_reactions();
    assert_eq!(reactions.len(), 1);

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1);
}
