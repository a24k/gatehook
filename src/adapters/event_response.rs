use serde::Deserialize;

/// Webhookエンドポイントからの応答
///
/// Discordイベントを送信した結果として、Webhookエンドポイントから返される応答。
/// Botが実行すべきアクションのリストを含む。
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct EventResponse {
    /// 実行するアクションのリスト
    ///
    /// 空の場合、またはフィールドが存在しない場合は何もしない。
    #[serde(default)]
    pub actions: Vec<Action>,
}

/// Discord上で実行可能なアクション
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// メッセージに返信する
    ///
    /// # コンテキスト要件
    /// このアクションはメッセージコンテキストが必要です（`message`ハンドラのみ）。
    Reply {
        /// 返信内容
        ///
        /// 2000文字（Unicode code points）を超える場合は自動的に切り詰められます。
        content: String,
        /// メンション通知するか
        ///
        /// - `true`: 返信先ユーザーに通知が送られる（`reply_ping`）
        /// - `false`: 通知なしで返信（`reply`、デフォルト）
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
            Action::Reply { content, mention } => {
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
            Action::Reply { content, mention } => {
                assert_eq!(content, "First reply");
                assert!(!mention);
            }
        }

        match &response.actions[1] {
            Action::Reply { content, mention } => {
                assert_eq!(content, "Second reply");
                assert!(mention);
            }
        }
    }

}
