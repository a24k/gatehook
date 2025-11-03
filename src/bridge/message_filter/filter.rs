use serenity::model::id::UserId;

use super::filterable_message::FilterableMessage;
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
    use rstest::rstest;

    // Helper function to create message for each sender type
    fn create_message(sender_type: &str, self_id: u64) -> MockMessage {
        match sender_type {
            "self" => MockMessage::new(self_id),
            "webhook" => MockMessage::new(456).bot().webhook(789),
            "system" => MockMessage::new(456).system(),
            "bot" => MockMessage::new(456).bot(),
            "user" => MockMessage::new(456),
            _ => panic!("Unknown sender type: {}", sender_type),
        }
    }

    #[rstest]
    // self messages
    #[case("self", "all", true)]
    #[case("self", "user,bot,webhook,system", false)]
    // webhook messages
    #[case("webhook", "webhook", true)]
    #[case("webhook", "user,bot", false)]
    // system messages
    #[case("system", "system", true)]
    #[case("system", "user,bot", false)]
    // bot messages
    #[case("bot", "bot", true)]
    #[case("bot", "user", false)]
    // user messages
    #[case("user", "user", true)]
    #[case("user", "bot", false)]
    fn test_sender_type_filtering(
        #[case] sender_type: &str,
        #[case] policy_str: &str,
        #[case] should_allow: bool,
    ) {
        let policy = MessageFilterPolicy::from_policy(policy_str);
        let filter = policy.for_user(UserId::new(123));
        let message = create_message(sender_type, 123);

        assert_eq!(
            filter.should_process(&message),
            should_allow,
            "Sender type '{}' with policy '{}' should {}",
            sender_type,
            policy_str,
            if should_allow { "allow" } else { "block" }
        );
    }

    #[rstest]
    // self takes priority over webhook
    #[case(MockMessage::new(123).bot().webhook(789), "webhook", "self over webhook")]
    // webhook takes priority over bot
    #[case(MockMessage::new(456).bot().webhook(789), "bot", "webhook over bot")]
    // webhook takes priority over system
    #[case(MockMessage::new(456).system().webhook(789), "system", "webhook over system")]
    // system takes priority over bot
    #[case(MockMessage::new(456).bot().system(), "bot", "system over bot")]
    fn test_sender_type_priority(
        #[case] message: MockMessage,
        #[case] lower_priority_policy: &str,
        #[case] description: &str,
    ) {
        let policy = MessageFilterPolicy::from_policy(lower_priority_policy);
        let filter = policy.for_user(UserId::new(123));

        assert!(
            !filter.should_process(&message),
            "Priority test failed: {}",
            description
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
