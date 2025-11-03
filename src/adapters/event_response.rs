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

    #[test]
    fn test_parse_empty_response() {
        let json = r#"{}"#;
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 0);
    }

    #[test]
    fn test_parse_empty_actions() {
        let json = r#"{"actions": []}"#;
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 0);
    }

    #[test]
    fn test_parse_reply_without_mention() {
        let json = r#"{"actions":[{"type":"reply","content":"Hello"}]}"#;
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 1);

        match &response.actions[0] {
            Action::Reply { content, mention } => {
                assert_eq!(content, "Hello");
                assert!(!mention);
            }
        }
    }

    #[test]
    fn test_parse_reply_with_mention() {
        let json = r#"{"actions":[{"type":"reply","content":"Hi there","mention":true}]}"#;
        let response: EventResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 1);

        match &response.actions[0] {
            Action::Reply { content, mention } => {
                assert_eq!(content, "Hi there");
                assert!(mention);
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
