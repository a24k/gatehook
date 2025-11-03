use serenity::model::id::UserId;

use super::message_like::FilterableMessage;
use super::policy::MessageFilterPolicy;

/// Active message filter with bot's user ID
///
/// This is created after the bot connects to Discord and knows its user ID.
/// Can only be created via `MessageFilterPolicy::for_user()`.
#[derive(Debug, Clone)]
pub struct MessageFilter {
    current_user_id: UserId,
    policy: MessageFilterPolicy,
}

impl MessageFilter {
    /// Create a new MessageFilter (private constructor)
    ///
    /// This is intentionally not public. Use `MessageFilterPolicy::for_user()` instead.
    pub(super) fn new(current_user_id: UserId, policy: MessageFilterPolicy) -> Self {
        Self {
            current_user_id,
            policy,
        }
    }

    /// Check if a message should be processed based on this filter
    ///
    /// # Sender Type Classification
    ///
    /// Messages are classified into mutually exclusive categories:
    /// 1. self - Bot's own messages
    /// 2. webhook - Webhook messages (excluding self)
    /// 3. system - System messages (excluding self and webhooks)
    /// 4. bot - Other bot messages (excluding self and webhooks)
    /// 5. user - Human user messages (default/fallback)
    ///
    /// This ensures every message falls into exactly one category.
    pub fn should_process<M: FilterableMessage>(&self, message: &M) -> bool {
        // Sender type classification

        // 1. self
        if message.author_id() == self.current_user_id {
            return self.policy.allow_self;
        }

        // 2. webhook (excluding self)
        if message.webhook_id().is_some() {
            return self.policy.allow_webhook;
        }

        // 3. system (excluding self and webhooks)
        if message.is_system() {
            return self.policy.allow_system;
        }

        // 4. bot (excluding self and webhooks)
        // Note: Discord webhooks have author.bot = true, but are classified as 'webhook' above
        if message.is_bot() {
            return self.policy.allow_bot;
        }

        // 5. user (default)
        self.policy.allow_user
    }
}
