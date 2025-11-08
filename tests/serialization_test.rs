use serenity::model::prelude::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_channel_type_serialization() {
    // Test various ChannelType values to see how they serialize
    let types = vec![
        (ChannelType::Text, 0),
        (ChannelType::Voice, 2),
        (ChannelType::Category, 4),
        (ChannelType::News, 5),
        (ChannelType::PublicThread, 11),
        (ChannelType::PrivateThread, 12),
        (ChannelType::NewsThread, 10),
        (ChannelType::Stage, 13),
        (ChannelType::Forum, 15),
    ];

    println!("\n=== ChannelType Serialization ===");
    for (channel_type, expected_value) in types {
        let json = serde_json::to_string(&channel_type).unwrap();
        println!("{:?} serializes to: {} (expected: {})", channel_type, json, expected_value);
        assert_eq!(json, expected_value.to_string());
    }
}

#[test]
fn test_guild_channel_field_names() {
    // Test by deserializing a Discord API JSON and re-serializing
    // This will show us what field names serenity uses
    let discord_api_json = json!({
        "id": "41771983423143937",
        "guild_id": "41771983423143937",
        "name": "general",
        "type": 0,  // Discord API uses "type"
        "position": 6,
        "permission_overwrites": [],
        "nsfw": false,
    });

    println!("\n=== GuildChannel Field Names ===");
    println!("Input (Discord API format):\n{}", serde_json::to_string_pretty(&discord_api_json).unwrap());

    // Deserialize from Discord API format
    let channel: GuildChannel = serde_json::from_value(discord_api_json).unwrap();

    // Serialize back to JSON
    let serialized = serde_json::to_string_pretty(&channel).unwrap();
    println!("\nSerenity serialized format:\n{}", serialized);

    // Check if the serialized JSON contains "type" or "kind"
    let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    if value.get("type").is_some() {
        println!("\n✓ Uses 'type' field (matches Discord API)");
        println!("  Value: {}", value["type"]);
    } else if value.get("kind").is_some() {
        println!("\n✗ Uses 'kind' field (different from Discord API)");
        println!("  Value: {}", value["kind"]);
    } else {
        println!("\n⚠ Neither 'type' nor 'kind' found!");
    }
}

#[test]
fn test_message_field_structure() {
    // Test Message structure to verify field names
    println!("\n=== Message Field Names ===");

    // Create a minimal message-like JSON
    let message_json = json!({
        "id": "123456789012345678",
        "channel_id": "987654321098765432",
        "author": {
            "id": "234567890123456789",
            "username": "testuser",
            "discriminator": "0",
            "avatar": null,
            "bot": false,
        },
        "content": "Hello!",
        "timestamp": "2024-01-15T12:34:56.789000+00:00",
        "edited_timestamp": null,
        "tts": false,
        "mention_everyone": false,
        "mentions": [],
        "mention_roles": [],
        "attachments": [],
        "embeds": [],
        "pinned": false,
        "type": 0,
    });

    // Deserialize
    let message: Message = serde_json::from_value(message_json).unwrap();

    // Serialize back
    let serialized = serde_json::to_string_pretty(&message).unwrap();
    println!("Message serialized:\n{}", serialized);

    // Parse to check field names
    let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    // Check key fields exist with correct names
    let mut field_checks = HashMap::new();
    field_checks.insert("id", value.get("id").is_some());
    field_checks.insert("channel_id", value.get("channel_id").is_some());
    field_checks.insert("author", value.get("author").is_some());
    field_checks.insert("content", value.get("content").is_some());
    field_checks.insert("timestamp", value.get("timestamp").is_some());

    println!("\n=== Field Name Verification ===");
    for (field, exists) in field_checks {
        let status = if exists { "✓" } else { "✗" };
        println!("{} {} field", status, field);
    }
}

#[test]
fn test_ready_field_structure() {
    // Note: Ready is difficult to construct manually due to complex types
    // We'll just document the expected field names
    println!("\n=== Ready Event Field Names ===");
    println!("Expected fields in Ready event:");
    println!("  - v: Gateway version (integer)");
    println!("  - user: Current user object");
    println!("  - guilds: Array of unavailable guild objects");
    println!("  - session_id: Session ID string");
    println!("  - resume_gateway_url: Resume gateway URL");
    println!("  - shard: Optional shard information [shard_id, num_shards]");
    println!("  - application: Partial application object");

    // These match Discord API field names directly
    println!("\nNote: serenity preserves Discord API field names for Ready event");
}
