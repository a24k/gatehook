//! Serialization tests to verify Discord API compatibility
//!
//! These tests ensure that serenity's JSON serialization matches Discord's API specification,
//! which is critical for webhook consumers reading our event payloads.

use rstest::rstest;
use serde_json::json;
use serenity::model::prelude::*;

/// Verify that ChannelType serializes to integers matching Discord API
///
/// Discord API uses integer values for channel types, not string names.
/// This ensures README examples and documentation are accurate.
#[rstest]
#[case(ChannelType::Text, 0)]
#[case(ChannelType::Voice, 2)]
#[case(ChannelType::Category, 4)]
#[case(ChannelType::News, 5)]
#[case(ChannelType::NewsThread, 10)]
#[case(ChannelType::PublicThread, 11)]
#[case(ChannelType::PrivateThread, 12)]
#[case(ChannelType::Stage, 13)]
#[case(ChannelType::Forum, 15)]
fn test_channel_type_serializes_as_integer(
    #[case] channel_type: ChannelType,
    #[case] expected_value: u8,
) {
    let json = serde_json::to_string(&channel_type).unwrap();
    assert_eq!(json, expected_value.to_string());
}

/// Verify that GuildChannel uses "type" field name (not "kind")
///
/// The Rust field is named `kind` (since `type` is a keyword), but it must
/// serialize to "type" to match Discord's API and our README documentation.
#[test]
fn test_guild_channel_uses_type_field_name() {
    let discord_api_json = json!({
        "id": "41771983423143937",
        "guild_id": "41771983423143937",
        "name": "general",
        "type": 0,
        "position": 6,
        "permission_overwrites": [],
        "nsfw": false,
    });

    // Round-trip: Discord API JSON -> GuildChannel -> JSON
    let channel: GuildChannel = serde_json::from_value(discord_api_json).unwrap();
    let serialized = serde_json::to_string(&channel).unwrap();
    let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    // Must use "type" field name (Discord API standard)
    assert!(
        value.get("type").is_some(),
        "GuildChannel must serialize with 'type' field"
    );
    assert_eq!(value["type"], 0);

    // Must NOT use "kind" field name
    assert!(
        value.get("kind").is_none(),
        "GuildChannel must not use 'kind' field in JSON"
    );
}

/// Verify that Message has expected field names
///
/// Ensures our README examples match serenity's actual serialization.
#[rstest]
#[case("id")]
#[case("channel_id")]
#[case("author")]
#[case("content")]
#[case("timestamp")]
fn test_message_has_expected_fields(#[case] field_name: &str) {
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

    let message: Message = serde_json::from_value(message_json).unwrap();
    let serialized = serde_json::to_string(&message).unwrap();
    let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert!(
        value.get(field_name).is_some(),
        "Message must have '{}' field",
        field_name
    );
}
