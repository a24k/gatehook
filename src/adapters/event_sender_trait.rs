use crate::adapters::event_response::EventResponse;
use serde::Serialize;
use serenity::async_trait;

/// イベントを外部エンドポイントに送信するインターフェース
#[async_trait]
pub trait EventSender: Send + Sync {
    /// イベントを送信し、応答を取得
    ///
    /// # Arguments
    ///
    /// * `handler` - ハンドラ名 (e.g., "message", "ready")
    /// * `payload` - 送信するペイロード（JSONにシリアライズされる）
    ///
    /// # Returns
    ///
    /// * `Ok(Some(EventResponse))` - 応答が正常にパースできた場合
    /// * `Ok(None)` - 応答がない、またはパースできない場合
    /// * `Err(_)` - 送信に失敗した場合
    async fn send<T: Serialize + Send + Sync>(
        &self,
        handler: &str,
        payload: &T,
    ) -> anyhow::Result<Option<EventResponse>>;
}
