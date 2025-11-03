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
        }

        match &response.actions[1] {
            ResponseAction::Reply { content, mention } => {
                assert_eq!(content, "Second reply");
                assert!(mention);
            }
        }
    }

}
