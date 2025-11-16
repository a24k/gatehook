// Unit tests for EventBridge business logic
// These tests verify that events are correctly processed and forwarded

mod adapters;

use adapters::{MockChannelInfoProvider, MockDiscordService, MockEventSender, MockReactionBuilder};
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
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_test_message("Test message", 111, 222);

    // Create EventResponse with reply action
    let event_response = EventResponse {
        actions: vec![ResponseAction::Reply(ReplyParams {
            content: expected_content.to_string(),
            mention,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&message, &event_response).await;

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
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

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
    let result = bridge.execute_actions(&message, &event_response).await;

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
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

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
    let result = bridge.execute_actions(&message, &event_response).await;

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
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_test_message("Hello", 999, 1000);

    // Execute handle_message (which should return the EventResponse)
    let result = bridge.handle_message(&message).await;

    // Verify
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.is_some());
    let response = response.unwrap();
    assert_eq!(response.actions.len(), 1);

    // Event was sent to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(
        sent_events[0].handler, "message",
        "Event handler should be 'message'"
    );
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
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_test_message("Test message", 111, 222);

    // Create EventResponse with react action
    let event_response = EventResponse {
        actions: vec![ResponseAction::React(ReactParams {
            emoji: emoji.to_string(),
        })],
    };

    // Execute
    let result = bridge.execute_actions(&message, &event_response).await;

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
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    channel_info.set_is_thread(ChannelId::new(222), false); // Not in thread
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_guild_message("Original message", 111, 222, 333);

    // Create EventResponse with thread action
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some("Discussion".to_string()),
            content: "Let's discuss".to_string(),
            auto_archive_duration: 1440,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&message, &event_response).await;

    // Verify
    assert!(result.is_ok(), "execute_actions should succeed");

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1, "Should create one thread");
    assert_eq!(threads[0].channel_id, ChannelId::new(222));
    assert_eq!(threads[0].message_id, MessageId::new(111));
    assert_eq!(threads[0].name, "Discussion");
    assert_eq!(threads[0].auto_archive_duration, 1440);

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
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    channel_info.set_is_thread(ChannelId::new(222), false);
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_guild_message("This is the original message content", 111, 222, 333);

    // Thread action without name (should auto-generate)
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: None,
            content: "Response".to_string(),
            auto_archive_duration: 1440,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&message, &event_response).await;

    // Verify
    assert!(result.is_ok());

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1);
    // After ActionTarget refactoring, name defaults to "Thread" when not specified
    // (since ActionTarget doesn't have message content)
    assert_eq!(threads[0].name, "Thread");
}

#[tokio::test]
async fn test_execute_actions_thread_long_name() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    channel_info.set_is_thread(ChannelId::new(222), false);
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_guild_message("Original message", 111, 222, 333);

    // Thread action with name exceeding 100 chars (should be truncated)
    let long_name = "a".repeat(150); // 150 characters
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some(long_name),
            content: "Response".to_string(),
            auto_archive_duration: 1440,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&message, &event_response).await;

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
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    channel_info.set_is_thread(ChannelId::new(222), true); // Already in thread
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_guild_message("Thread message", 111, 222, 333);

    // Thread action (should skip thread creation)
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some("Ignored".to_string()),
            content: "Reply in thread".to_string(),
            auto_archive_duration: 1440,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&message, &event_response).await;

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
async fn test_execute_actions_thread_create_with_custom_duration() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    channel_info.set_is_thread(ChannelId::new(222), false); // Creating NEW thread
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_guild_message("Original", 111, 222, 333);

    // Thread action with custom auto_archive_duration
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some("Support".to_string()),
            content: "Help needed".to_string(),
            auto_archive_duration: 60,
        })],
    };

    // Execute
    let result = bridge.execute_actions(&message, &event_response).await;

    // Verify
    assert!(result.is_ok());

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].name, "Support");
    assert_eq!(threads[0].auto_archive_duration, 60);

    let messages = discord_service.get_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Help needed");
    assert_eq!(messages[0].channel_id, ChannelId::new(222));
    assert_eq!(messages[0].reply_to, None);
}


#[tokio::test]
async fn test_execute_actions_thread_in_dm_fails() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    // Configure channel_info to return an error for DM channel (simulating API behavior)
    channel_info.set_is_thread_error(ChannelId::new(222), "DM channels don't support threads".to_string());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let message = create_test_message("DM message", 111, 222); // No guild_id

    // Thread action
    let event_response = EventResponse {
        actions: vec![ResponseAction::Thread(ThreadParams {
            name: Some("Thread".to_string()),
            content: "Content".to_string(),
            auto_archive_duration: 1440,
        })],
    };

    // Execute (should complete but log error)
    let result = bridge.execute_actions(&message, &event_response).await;

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
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    channel_info.set_is_thread(ChannelId::new(222), false);
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

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
                auto_archive_duration: 1440,
            }),
        ],
    };

    // Execute
    let result = bridge.execute_actions(&message, &event_response).await;

    // Verify
    assert!(result.is_ok());

    let replies = discord_service.get_replies();
    assert_eq!(replies.len(), 1);

    let reactions = discord_service.get_reactions();
    assert_eq!(reactions.len(), 1);

    let threads = discord_service.get_threads();
    assert_eq!(threads.len(), 1);
}

#[tokio::test]
async fn test_handle_message_with_channel_info() {
    use serenity::model::channel::{ChannelType, GuildChannel};

    // Setup: MockChannelInfoProvider with pre-configured channel
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());

    // Create a guild channel with specific properties
    let mut test_channel = GuildChannel::default();
    test_channel.id = ChannelId::new(1000);
    test_channel.name = "test-channel".to_string();
    test_channel.kind = ChannelType::Text;
    test_channel.guild_id = GuildId::new(5000);

    // Configure mock to return this channel
    channel_info.set_channel(ChannelId::new(1000), test_channel.clone());

    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let message = create_guild_message("Hello", 999, 1000, 5000);

    // Execute handle_message
    let result = bridge.handle_message(&message).await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook with channel information
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(sent_events[0].handler, "message");

    // Verify that the payload contains channel information
    let payload_json = &sent_events[0].payload;
    assert!(
        payload_json.contains("test-channel"),
        "Payload should contain channel name"
    );
    assert!(
        payload_json.contains("\"channel\""),
        "Payload should contain channel field"
    );
}

#[tokio::test]
async fn test_handle_message_without_channel_info() {
    // Setup: MockChannelInfoProvider without pre-configured channel (simulates cache miss + API failure)
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());

    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let message = create_guild_message("Hello", 999, 1000, 5000);

    // Execute handle_message
    let result = bridge.handle_message(&message).await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(sent_events[0].handler, "message");

    // Verify that the payload does NOT contain channel information
    // (MockChannelInfoProvider returns None if channel not set)
    let payload_json = &sent_events[0].payload;
    // The JSON should not have a "channel" field when it's None
    // (due to #[serde(skip_serializing_if = "Option::is_none")])
    let json_value: serde_json::Value = serde_json::from_str(payload_json).unwrap();
    assert!(
        json_value.get("channel").is_none(),
        "Payload should not contain channel field when channel info is unavailable"
    );
}

#[tokio::test]
async fn test_handle_message_delete() {
    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let channel_id = ChannelId::new(999);
    let message_id = MessageId::new(888);
    let guild_id = Some(GuildId::new(777));

    // Execute handle_message_delete
    let result = bridge
        .handle_message_delete(channel_id, message_id, guild_id)
        .await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(sent_events[0].handler, "message_delete");

    // Verify payload structure
    let payload_json = &sent_events[0].payload;
    let json_value: serde_json::Value = serde_json::from_str(payload_json).unwrap();

    assert_eq!(json_value["message_delete"]["id"], "888");
    assert_eq!(json_value["message_delete"]["channel_id"], "999");
    assert_eq!(json_value["message_delete"]["guild_id"], "777");
}

#[tokio::test]
async fn test_handle_message_delete_without_guild() {
    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let channel_id = ChannelId::new(999);
    let message_id = MessageId::new(888);

    // Execute handle_message_delete (DM scenario)
    let result = bridge
        .handle_message_delete(channel_id, message_id, None)
        .await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1);
    assert_eq!(sent_events[0].handler, "message_delete");

    // Verify guild_id is omitted
    let payload_json = &sent_events[0].payload;
    let json_value: serde_json::Value = serde_json::from_str(payload_json).unwrap();

    assert_eq!(json_value["message_delete"]["id"], "888");
    assert_eq!(json_value["message_delete"]["channel_id"], "999");
    assert!(
        json_value["message_delete"].get("guild_id").is_none(),
        "guild_id should be omitted for DMs"
    );
}

#[tokio::test]
async fn test_handle_message_delete_bulk() {
    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let channel_id = ChannelId::new(999);
    let message_ids = vec![
        MessageId::new(111),
        MessageId::new(222),
        MessageId::new(333),
    ];
    let guild_id = Some(GuildId::new(777));

    // Execute handle_message_delete_bulk
    let result = bridge
        .handle_message_delete_bulk(channel_id, message_ids.clone(), guild_id)
        .await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(sent_events[0].handler, "message_delete_bulk");

    // Verify payload structure
    let payload_json = &sent_events[0].payload;
    let json_value: serde_json::Value = serde_json::from_str(payload_json).unwrap();

    let ids = json_value["message_delete_bulk"]["ids"]
        .as_array()
        .expect("ids should be an array");
    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0], "111");
    assert_eq!(ids[1], "222");
    assert_eq!(ids[2], "333");
    assert_eq!(json_value["message_delete_bulk"]["channel_id"], "999");
    assert_eq!(json_value["message_delete_bulk"]["guild_id"], "777");
}

#[tokio::test]
async fn test_handle_message_delete_bulk_empty() {
    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let channel_id = ChannelId::new(999);
    let message_ids: Vec<MessageId> = vec![];
    let guild_id = Some(GuildId::new(777));

    // Execute handle_message_delete_bulk with empty list
    let result = bridge
        .handle_message_delete_bulk(channel_id, message_ids, guild_id)
        .await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1);

    // Verify empty ids array
    let payload_json = &sent_events[0].payload;
    let json_value: serde_json::Value = serde_json::from_str(payload_json).unwrap();

    let ids = json_value["message_delete_bulk"]["ids"]
        .as_array()
        .expect("ids should be an array");
    assert_eq!(ids.len(), 0, "Should have empty ids array");
}

// ========================================
// REACTION_ADD Event Tests
// ========================================

#[tokio::test]
async fn test_handle_reaction_add_with_channel_info() {
    use serenity::model::channel::{ChannelType, GuildChannel};

    // Setup: MockChannelInfoProvider with pre-configured channel
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());

    // Create a guild channel with specific properties
    let mut test_channel = GuildChannel::default();
    test_channel.id = ChannelId::new(2000);
    test_channel.name = "reaction-channel".to_string();
    test_channel.kind = ChannelType::Text;
    test_channel.guild_id = GuildId::new(6000);

    // Configure mock to return this channel
    channel_info.set_channel(ChannelId::new(2000), test_channel.clone());

    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let reaction = MockReactionBuilder::new(2222, 2000)
        .emoji("👍")
        .guild(6000, 1111)
        .build();

    // Execute handle_reaction_add
    let result = bridge.handle_reaction_add(&reaction).await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook with channel information
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(sent_events[0].handler, "reaction_add");

    // Verify that the payload contains channel information
    let payload_json = &sent_events[0].payload;
    assert!(
        payload_json.contains("reaction-channel"),
        "Payload should contain channel name"
    );
    assert!(
        payload_json.contains("\"channel\""),
        "Payload should contain channel field"
    );
}

#[tokio::test]
async fn test_handle_reaction_add_without_channel_info() {
    // Setup: MockChannelInfoProvider without pre-configured channel (simulates cache miss + API failure)
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());

    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let reaction = MockReactionBuilder::new(2222, 2000)
        .emoji("👍")
        .guild(6000, 1111)
        .build();

    // Execute handle_reaction_add
    let result = bridge.handle_reaction_add(&reaction).await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(sent_events[0].handler, "reaction_add");

    // Verify that the payload does NOT contain channel information
    // (MockChannelInfoProvider returns None if channel not set)
    let payload_json = &sent_events[0].payload;
    // The JSON should not have a "channel" field when it's None
    // (due to #[serde(skip_serializing_if = "Option::is_none")])
    let json_value: serde_json::Value = serde_json::from_str(payload_json).unwrap();
    assert!(
        json_value.get("channel").is_none(),
        "Payload should not contain channel field when channel info is unavailable"
    );
}

#[tokio::test]
async fn test_handle_reaction_add_dm() {
    // Setup for DM reaction (no guild_id)
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());

    let bridge = EventBridge::new(discord_service, event_sender.clone(), channel_info);

    let reaction = MockReactionBuilder::new(4444, 5000)
        .emoji("❤️")
        .user_id(3333)
        .build();

    // Execute handle_reaction_add
    let result = bridge.handle_reaction_add(&reaction).await;

    // Verify
    assert!(result.is_ok());

    // Check that the event was sent to webhook
    let sent_events = event_sender.get_sent_events();
    assert_eq!(sent_events.len(), 1, "Should send one event to webhook");
    assert_eq!(sent_events[0].handler, "reaction_add");

    // Verify payload structure (DM reaction should not have channel field)
    let payload_json = &sent_events[0].payload;
    let json_value: serde_json::Value = serde_json::from_str(payload_json).unwrap();

    assert!(
        json_value.get("reaction").is_some(),
        "Payload should contain reaction field"
    );
    assert!(
        json_value.get("channel").is_none(),
        "DM reaction should not have channel field"
    );
    assert!(
        json_value["reaction"]["guild_id"].is_null(),
        "DM reaction should have null guild_id"
    );
}

#[tokio::test]
async fn test_execute_actions_from_reaction() {
    use gatehook::adapters::{EventResponse, ResponseAction};

    // Setup
    let discord_service = Arc::new(MockDiscordService::new());
    let event_sender = Arc::new(MockEventSender::new());
    let channel_info = Arc::new(MockChannelInfoProvider::new());
    let bridge = EventBridge::new(discord_service.clone(), event_sender.clone(), channel_info);

    let reaction = MockReactionBuilder::new(8888, 9999)
        .emoji("🎉")
        .guild(1234, 7777)
        .build();

    // Create EventResponse with reply and react actions
    let event_response = EventResponse {
        actions: vec![
            ResponseAction::Reply(ReplyParams {
                content: "Thanks for the reaction!".to_string(),
                mention: false,
            }),
            ResponseAction::React(ReactParams {
                emoji: "✅".to_string(),
            }),
        ],
    };

    // Execute actions from reaction event
    let result = bridge.execute_actions(&reaction, &event_response).await;

    // Verify
    assert!(result.is_ok(), "execute_actions should succeed");

    // Verify reply was sent to the original message
    let replies = discord_service.get_replies();
    assert_eq!(replies.len(), 1, "Should send one reply");
    assert_eq!(replies[0].content, "Thanks for the reaction!");
    assert_eq!(replies[0].message_id, MessageId::new(8888)); // reaction.message_id
    assert_eq!(replies[0].channel_id, ChannelId::new(9999)); // reaction.channel_id

    // Verify reaction was added to the original message
    let reactions = discord_service.get_reactions();
    assert_eq!(reactions.len(), 1, "Should add one reaction");
    assert_eq!(reactions[0].emoji, "✅");
    assert_eq!(reactions[0].message_id, MessageId::new(8888));
    assert_eq!(reactions[0].channel_id, ChannelId::new(9999));
}
