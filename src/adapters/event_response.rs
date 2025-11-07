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

/// Parameters for Reply action
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ReplyParams {
    /// Reply content (any length accepted, truncated at execution if needed)
    pub content: String,
    /// Whether to ping/mention the user (default: false)
    #[serde(default)]
    pub mention: bool,
}

/// Parameters for React action
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ReactParams {
    /// Emoji to react with
    ///
    /// Can be:
    /// - Unicode emoji (e.g., "👍", "🎉")
    /// - Custom emoji in format "name:id" (e.g., "customemoji:123456789")
    pub emoji: String,
}

/// Parameters for Thread action
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ThreadParams {
    /// Thread name (auto-generated from message content if omitted)
    #[serde(default)]
    pub name: Option<String>,
    /// Message content (any length accepted, truncated at execution if needed)
    pub content: String,
    /// Auto-archive duration in minutes (default: 1440)
    ///
    /// Valid values: 60, 1440, 4320, 10080
    #[serde(default = "default_auto_archive")]
    pub auto_archive_duration: u16,
}

/// Action to execute in response to a Discord event
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseAction {
    /// Reply to a message (requires message context)
    Reply(ReplyParams),
    /// Add a reaction to a message (requires message context)
    React(ReactParams),
    /// Create thread or post to existing thread (MESSAGE_GUILD only)
    Thread(ThreadParams),
}

/// Default auto-archive duration (1440 minutes = 24 hours)
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
            ResponseAction::Reply(params) => {
                assert_eq!(params.content, expected_content);
                assert_eq!(params.mention, expected_mention);
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
            ResponseAction::Reply(params) => {
                assert_eq!(params.content, "First reply");
                assert!(!params.mention);
            }
            _ => panic!("Expected Reply action"),
        }

        match &response.actions[1] {
            ResponseAction::Reply(params) => {
                assert_eq!(params.content, "Second reply");
                assert!(params.mention);
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
            ResponseAction::React(params) => {
                assert_eq!(params.emoji, expected_emoji);
            }
            _ => panic!("Expected React action"),
        }
    }

    #[rstest]
    #[case::with_name(
        r#"{"actions":[{"type":"thread","name":"Discussion","content":"Let's talk"}]}"#,
        Some("Discussion"),
        "Let's talk",
        1440
    )]
    #[case::without_name(
        r#"{"actions":[{"type":"thread","content":"Message"}]}"#,
        None,
        "Message",
        1440
    )]
    #[case::custom_auto_archive(
        r#"{"actions":[{"type":"thread","content":"Test","auto_archive_duration":60}]}"#,
        None,
        "Test",
        60
    )]
    fn test_parse_thread_action(
        #[case] json: &str,
        #[case] expected_name: Option<&str>,
        #[case] expected_content: &str,
        #[case] expected_auto_archive: u16,
    ) {
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 1);

        match &response.actions[0] {
            ResponseAction::Thread(params) => {
                assert_eq!(params.name.as_deref(), expected_name);
                assert_eq!(params.content, expected_content);
                assert_eq!(params.auto_archive_duration, expected_auto_archive);
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

    #[test]
    fn test_parse_thread_invalid_auto_archive_duration() {
        // Invalid duration values are accepted as-is (validated at execution time)
        let json = r#"{"actions":[{"type":"thread","content":"Test","auto_archive_duration":100}]}"#;
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 1);

        match &response.actions[0] {
            ResponseAction::Thread(params) => {
                assert_eq!(params.auto_archive_duration, 100);
            }
            _ => panic!("Expected Thread action"),
        }
    }

    #[rstest]
    #[case::one_hour(60)]
    #[case::one_day(1440)]
    #[case::three_days(4320)]
    #[case::one_week(10080)]
    fn test_parse_thread_valid_auto_archive_durations(#[case] duration_minutes: u16) {
        let json = format!(
            r#"{{"actions":[{{"type":"thread","content":"Test","auto_archive_duration":{}}}]}}"#,
            duration_minutes
        );
        let response: EventResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.actions.len(), 1);

        match &response.actions[0] {
            ResponseAction::Thread(params) => {
                assert_eq!(params.auto_archive_duration, duration_minutes);
            }
            _ => panic!("Expected Thread action"),
        }
    }

}
