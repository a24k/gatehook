use serde::Deserialize;

/// Response from webhook endpoint
///
/// The response returned from the webhook endpoint after sending a Discord event.
/// Contains a list of actions for the bot to execute.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct EventResponse {
    /// List of actions to execute
    ///
    /// If empty or the field is missing, no actions will be performed.
    #[serde(default)]
    pub actions: Vec<ResponseAction>,
}

/// Action executable from webhook response
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseAction {
    /// Reply to a message
    ///
    /// # Context Requirements
    /// This action requires message context (`message` handler only).
    Reply {
        /// Reply content
        ///
        /// Content exceeding 2000 characters (Unicode code points) will be automatically truncated.
        content: String,
        /// Whether to ping/mention the user
        ///
        /// - `true`: Notification will be sent to the replied user (`reply_ping`)
        /// - `false`: Reply without notification (`reply`, default)
        #[serde(default)]
        mention: bool,
    },
    /// Add a reaction to a message
    ///
    /// # Context Requirements
    /// This action requires message context (`message` handler only).
    React {
        /// Emoji to react with
        ///
        /// Can be:
        /// - Unicode emoji (e.g., "👍", "🎉")
        /// - Custom emoji in format "name:id" (e.g., "customemoji:123456789")
        emoji: String,
    },
    /// Create a thread or post to existing thread
    ///
    /// # Context Requirements
    /// MESSAGE_GUILD only (not supported in DM).
    ///
    /// # Behavior
    /// - **Normal channel**: Create new thread → Post message
    ///   - `name: Some(...)`: Use specified name
    ///   - `name: None`: Auto-generate from message content (max 100 chars)
    /// - **Already in thread**: Post to current thread (name is ignored)
    /// - **DM**: Error (not supported)
    Thread {
        /// Thread name (auto-generated if omitted)
        ///
        /// - 1-100 characters (Discord API limit)
        /// - If omitted: Generated from first line of message (max 100 chars)
        /// - Empty message fallback: "Thread"
        /// - Ignored if already in thread
        #[serde(default)]
        name: Option<String>,
        /// Message content (2000 char limit, auto-truncated)
        content: String,
        /// Whether to post as a reply
        ///
        /// - `true`: Post as reply to the original message
        /// - `false`: Post as normal message (default)
        #[serde(default)]
        reply: bool,
        /// Whether to mention the user (only effective when reply=true)
        ///
        /// - `true`: Mention the replied user
        /// - `false`: No mention (default)
        #[serde(default)]
        mention: bool,
        /// Auto-archive duration in minutes
        ///
        /// Valid values: 60, 1440, 4320, 10080
        /// Default: 1440 (24 hours)
        #[serde(default = "default_auto_archive")]
        auto_archive_duration: u16,
    },
}

/// Default auto-archive duration (24 hours)
fn default_auto_archive() -> u16 {
    1440
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::empty_object(r#"{}"#, 0)]
    #[case::empty_array(r#"{"actions": []}"#, 0)]
    fn test_parse_empty_response(#[case] json: &str, #[case] expected_len: usize) {
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), expected_len);
    }

    #[rstest]
    #[case::without_mention(
        r#"{"actions":[{"type":"reply","content":"Hello"}]}"#,
        "Hello",
        false
    )]
    #[case::with_mention(
        r#"{"actions":[{"type":"reply","content":"Hi there","mention":true}]}"#,
        "Hi there",
        true
    )]
    #[case::explicit_false_mention(
        r#"{"actions":[{"type":"reply","content":"Test","mention":false}]}"#,
        "Test",
        false
    )]
    fn test_parse_single_reply_action(
        #[case] json: &str,
        #[case] expected_content: &str,
        #[case] expected_mention: bool,
    ) {
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 1);

        match &response.actions[0] {
            ResponseAction::Reply { content, mention } => {
                assert_eq!(content, expected_content);
                assert_eq!(*mention, expected_mention);
            }
            _ => panic!("Expected Reply action"),
        }
    }

    #[test]
    fn test_parse_multiple_actions() {
        let json = r#"{
            "actions": [
                {"type":"reply","content":"First reply"},
                {"type":"reply","content":"Second reply","mention":true}
            ]
        }"#;
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 2);

        match &response.actions[0] {
            ResponseAction::Reply { content, mention } => {
                assert_eq!(content, "First reply");
                assert!(!mention);
            }
            _ => panic!("Expected Reply action"),
        }

        match &response.actions[1] {
            ResponseAction::Reply { content, mention } => {
                assert_eq!(content, "Second reply");
                assert!(mention);
            }
            _ => panic!("Expected Reply action"),
        }
    }

    #[rstest]
    #[case::unicode_emoji(r#"{"actions":[{"type":"react","emoji":"👍"}]}"#, "👍")]
    #[case::custom_emoji(
        r#"{"actions":[{"type":"react","emoji":"customemoji:123456789"}]}"#,
        "customemoji:123456789"
    )]
    fn test_parse_react_action(#[case] json: &str, #[case] expected_emoji: &str) {
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 1);

        match &response.actions[0] {
            ResponseAction::React { emoji } => {
                assert_eq!(emoji, expected_emoji);
            }
            _ => panic!("Expected React action"),
        }
    }

    #[rstest]
    #[case::with_name(
        r#"{"actions":[{"type":"thread","name":"Discussion","content":"Let's talk"}]}"#,
        Some("Discussion"),
        "Let's talk",
        false,
        false,
        1440
    )]
    #[case::without_name(
        r#"{"actions":[{"type":"thread","content":"Message"}]}"#,
        None,
        "Message",
        false,
        false,
        1440
    )]
    #[case::with_reply(
        r#"{"actions":[{"type":"thread","name":"Support","content":"Help needed","reply":true,"mention":true}]}"#,
        Some("Support"),
        "Help needed",
        true,
        true,
        1440
    )]
    #[case::custom_auto_archive(
        r#"{"actions":[{"type":"thread","content":"Test","auto_archive_duration":60}]}"#,
        None,
        "Test",
        false,
        false,
        60
    )]
    fn test_parse_thread_action(
        #[case] json: &str,
        #[case] expected_name: Option<&str>,
        #[case] expected_content: &str,
        #[case] expected_reply: bool,
        #[case] expected_mention: bool,
        #[case] expected_auto_archive: u16,
    ) {
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 1);

        match &response.actions[0] {
            ResponseAction::Thread {
                name,
                content,
                reply,
                mention,
                auto_archive_duration,
            } => {
                assert_eq!(name.as_deref(), expected_name);
                assert_eq!(content, expected_content);
                assert_eq!(*reply, expected_reply);
                assert_eq!(*mention, expected_mention);
                assert_eq!(*auto_archive_duration, expected_auto_archive);
            }
            _ => panic!("Expected Thread action"),
        }
    }

    #[test]
    fn test_parse_mixed_actions() {
        let json = r#"{
            "actions": [
                {"type":"reply","content":"Reply message"},
                {"type":"react","emoji":"👍"},
                {"type":"thread","name":"Discussion","content":"Thread message"}
            ]
        }"#;
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 3);

        match &response.actions[0] {
            ResponseAction::Reply { .. } => {}
            _ => panic!("Expected Reply action"),
        }

        match &response.actions[1] {
            ResponseAction::React { .. } => {}
            _ => panic!("Expected React action"),
        }

        match &response.actions[2] {
            ResponseAction::Thread { .. } => {}
            _ => panic!("Expected Thread action"),
        }
    }

}
