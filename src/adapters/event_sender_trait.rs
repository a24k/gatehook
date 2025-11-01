use serde::Serialize;
use serenity::async_trait;

/// イベントを外部エンドポイントに送信するインターフェース
#[async_trait]
pub trait EventSender: Send + Sync {
    /// Send an event to the endpoint
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler name (e.g., "message", "reaction_add")
    /// * `payload` - The payload to send (will be serialized as JSON)
    async fn send<T: Serialize + Send + Sync>(
        &self,
        handler: &str,
        payload: &T,
    ) -> anyhow::Result<()>;
}
