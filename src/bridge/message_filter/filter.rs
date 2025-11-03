use serenity::model::id::UserId;

use super::message_like::FilterableMessage;
use super::policy::MessageFilterPolicy;

/// Active message filter with bot's user ID
///
/// This is created after the bot connects to Discord and knows its user ID.
/// Can only be created via `MessageFilterPolicy::for_user()`.
#[derive(Debug, Clone)]
pub struct MessageFilter {
    user_id: UserId,
    policy: MessageFilterPolicy,
}

impl MessageFilter {
    /// Create a new MessageFilter (private constructor)
    ///
    /// This is intentionally not public. Use `MessageFilterPolicy::for_user()` instead.
    pub(super) fn new(user_id: UserId, policy: MessageFilterPolicy) -> Self {
        Self {
            user_id,
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
        if message.author_id() == self.user_id {
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::MockMessage;
    use crate::bridge::message_filter::policy::MessageFilterPolicy;

    #[test]
    fn test_should_process_self_message() {
        let policy = MessageFilterPolicy::from_policy("all");
        let filter = policy.for_user(UserId::new(123));

        let self_message = MockMessage::new(123);
        assert!(
            filter.should_process(&self_message),
            "Policy 'all' should allow self messages"
        );

        let policy_no_self = MessageFilterPolicy::from_policy("user,bot,webhook,system");
        let filter_no_self = policy_no_self.for_user(UserId::new(123));
        assert!(
            !filter_no_self.should_process(&self_message),
            "Policy without 'self' should block self messages"
        );
    }

    #[test]
    fn test_should_process_webhook_message() {
        let policy = MessageFilterPolicy::from_policy("webhook");
        let filter = policy.for_user(UserId::new(123));

        // Webhook messages have webhook_id and typically bot=true
        let webhook_message = MockMessage::new(456).bot().webhook(789);
        assert!(
            filter.should_process(&webhook_message),
            "Policy 'webhook' should allow webhook messages"
        );

        let policy_no_webhook = MessageFilterPolicy::from_policy("user,bot");
        let filter_no_webhook = policy_no_webhook.for_user(UserId::new(123));
        assert!(
            !filter_no_webhook.should_process(&webhook_message),
            "Policy without 'webhook' should block webhook messages"
        );
    }

    #[test]
    fn test_should_process_system_message() {
        let policy = MessageFilterPolicy::from_policy("system");
        let filter = policy.for_user(UserId::new(123));

        let system_message = MockMessage::new(456).system();
        assert!(
            filter.should_process(&system_message),
            "Policy 'system' should allow system messages"
        );

        let policy_no_system = MessageFilterPolicy::from_policy("user,bot");
        let filter_no_system = policy_no_system.for_user(UserId::new(123));
        assert!(
            !filter_no_system.should_process(&system_message),
            "Policy without 'system' should block system messages"
        );
    }

    #[test]
    fn test_should_process_bot_message() {
        let policy = MessageFilterPolicy::from_policy("bot");
        let filter = policy.for_user(UserId::new(123));

        // Bot message (not webhook, not system)
        let bot_message = MockMessage::new(456).bot();
        assert!(
            filter.should_process(&bot_message),
            "Policy 'bot' should allow bot messages"
        );

        let policy_no_bot = MessageFilterPolicy::from_policy("user");
        let filter_no_bot = policy_no_bot.for_user(UserId::new(123));
        assert!(
            !filter_no_bot.should_process(&bot_message),
            "Policy without 'bot' should block bot messages"
        );
    }

    #[test]
    fn test_should_process_user_message() {
        let policy = MessageFilterPolicy::from_policy("user");
        let filter = policy.for_user(UserId::new(123));

        // Regular user message (not bot, not system, not webhook)
        let user_message = MockMessage::new(456);
        assert!(
            filter.should_process(&user_message),
            "Policy 'user' should allow user messages"
        );

        let policy_no_user = MessageFilterPolicy::from_policy("bot");
        let filter_no_user = policy_no_user.for_user(UserId::new(123));
        assert!(
            !filter_no_user.should_process(&user_message),
            "Policy without 'user' should block user messages"
        );
    }

    #[test]
    fn test_sender_type_priority_self_over_webhook() {
        // If bot itself sends a message via webhook, it should be classified as 'self'
        let policy = MessageFilterPolicy::from_policy("webhook");
        let filter = policy.for_user(UserId::new(123));

        let self_webhook_message = MockMessage::new(123).bot().webhook(789);
        assert!(
            !filter.should_process(&self_webhook_message),
            "Self messages take priority over webhook classification"
        );
    }

    #[test]
    fn test_sender_type_priority_webhook_over_bot() {
        // Webhooks have bot=true, but should be classified as webhook, not bot
        let policy = MessageFilterPolicy::from_policy("bot");
        let filter = policy.for_user(UserId::new(123));

        let webhook_message = MockMessage::new(456).bot().webhook(789);
        assert!(
            !filter.should_process(&webhook_message),
            "Webhook messages should not be classified as bot messages"
        );
    }

    #[test]
    fn test_sender_type_priority_webhook_over_system() {
        // If a webhook is also marked as system (unlikely), webhook takes priority
        let policy = MessageFilterPolicy::from_policy("system");
        let filter = policy.for_user(UserId::new(123));

        let webhook_system_message = MockMessage::new(456).system().webhook(789);
        assert!(
            !filter.should_process(&webhook_system_message),
            "Webhook classification takes priority over system"
        );
    }

    #[test]
    fn test_sender_type_priority_system_over_bot() {
        // System messages that are also bots should be classified as system
        let policy = MessageFilterPolicy::from_policy("bot");
        let filter = policy.for_user(UserId::new(123));

        let system_bot_message = MockMessage::new(456).bot().system();
        assert!(
            !filter.should_process(&system_bot_message),
            "System classification takes priority over bot"
        );
    }

    #[test]
    fn test_default_policy_blocks_self_allows_others() {
        let policy = MessageFilterPolicy::default();
        let filter = policy.for_user(UserId::new(123));

        assert!(
            !filter.should_process(&MockMessage::new(123)),
            "Default blocks self"
        );
        assert!(
            filter.should_process(&MockMessage::new(456)),
            "Default allows users"
        );
        assert!(
            filter.should_process(&MockMessage::new(456).bot()),
            "Default allows bots"
        );
        assert!(
            filter.should_process(&MockMessage::new(456).bot().webhook(789)),
            "Default allows webhooks"
        );
        assert!(
            filter.should_process(&MockMessage::new(456).system()),
            "Default allows system"
        );
    }

    #[test]
    fn test_empty_policy_same_as_default() {
        let empty_policy = MessageFilterPolicy::from_policy("");
        let default_policy = MessageFilterPolicy::default();
        let filter_empty = empty_policy.for_user(UserId::new(123));
        let filter_default = default_policy.for_user(UserId::new(123));

        let test_messages = vec![
            MockMessage::new(123),                    // self
            MockMessage::new(456),                    // user
            MockMessage::new(456).bot(),              // bot
            MockMessage::new(456).bot().webhook(789), // webhook
            MockMessage::new(456).system(),           // system
        ];

        for msg in &test_messages {
            assert_eq!(
                filter_empty.should_process(msg),
                filter_default.should_process(msg),
                "Empty policy should behave same as default"
            );
        }
    }
}
