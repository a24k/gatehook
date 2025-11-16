use serenity::model::id::UserId;

use super::filterable_reaction::FilterableReaction;
use super::policy::SenderFilterPolicy;

/// Active reaction filter with bot's user ID
///
/// This is created after the bot connects to Discord and knows its user ID.
/// Can only be created via `SenderFilterPolicy::for_reaction()`.
#[derive(Debug, Clone)]
pub struct ReactionFilter {
    user_id: UserId,
    policy: SenderFilterPolicy,
}

impl ReactionFilter {
    /// Create a new ReactionFilter (private constructor)
    ///
    /// This is intentionally not public. Use `SenderFilterPolicy::for_reaction()` instead.
    pub(super) fn new(user_id: UserId, policy: SenderFilterPolicy) -> Self {
        Self { user_id, policy }
    }

    /// Check if a reaction should be processed based on this filter
    ///
    /// # Sender Type Classification
    ///
    /// Reactions are classified into mutually exclusive categories:
    /// 1. self - Bot's own reactions
    /// 2. bot - Other bot reactions (excluding self)
    /// 3. user - Human user reactions (default/fallback)
    ///
    /// Note: Reactions don't have webhook or system types (MESSAGE-only concepts).
    pub fn should_process<R: FilterableReaction>(&self, reaction: &R) -> bool {
        // Sender type classification

        // 1. self
        if reaction.user_id() == Some(self.user_id) {
            return self.policy.allow_self;
        }

        // 2. bot (excluding self)
        if reaction.is_bot() {
            return self.policy.allow_bot;
        }

        // 3. user (default/fallback)
        self.policy.allow_user
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::sender_filter::policy::SenderFilterPolicy;
    use rstest::rstest;

    // Mock reaction for testing
    struct MockReaction {
        user_id: Option<UserId>,
        is_bot: bool,
    }

    impl MockReaction {
        fn new(user_id: u64) -> Self {
            Self {
                user_id: Some(UserId::new(user_id)),
                is_bot: false,
            }
        }

        fn bot(mut self) -> Self {
            self.is_bot = true;
            self
        }
    }

    impl FilterableReaction for MockReaction {
        fn user_id(&self) -> Option<UserId> {
            self.user_id
        }

        fn is_bot(&self) -> bool {
            self.is_bot
        }
    }

    // Helper to create a basic Reaction for testing
    fn create_test_reaction(user_id: u64, is_bot: bool) -> MockReaction {
        if is_bot {
            MockReaction::new(user_id).bot()
        } else {
            MockReaction::new(user_id)
        }
    }

    #[rstest]
    // self reactions
    #[case("self", "all", true)]
    #[case("self", "user,bot", false)]
    // bot reactions
    #[case("bot", "bot", true)]
    #[case("bot", "user", false)]
    // user reactions
    #[case("user", "user", true)]
    #[case("user", "bot", false)]
    fn test_sender_type_filtering(
        #[case] sender_type: &str,
        #[case] policy_str: &str,
        #[case] should_allow: bool,
    ) {
        let policy = SenderFilterPolicy::from_policy(policy_str);
        let filter = ReactionFilter::new(UserId::new(123), policy);

        let reaction = match sender_type {
            "self" => create_test_reaction(123, false),
            "bot" => create_test_reaction(456, true),
            "user" => create_test_reaction(456, false),
            _ => panic!("Unknown sender type: {}", sender_type),
        };

        assert_eq!(
            filter.should_process(&reaction),
            should_allow,
            "Filter mismatch for sender_type='{}', policy='{}': expected {}, got {}",
            sender_type,
            policy_str,
            should_allow,
            filter.should_process(&reaction)
        );
    }

    #[test]
    fn test_default_policy_blocks_self_allows_others() {
        let policy = SenderFilterPolicy::default();
        let filter = ReactionFilter::new(UserId::new(123), policy);

        assert!(
            !filter.should_process(&create_test_reaction(123, false)),
            "Default blocks self"
        );
        assert!(
            filter.should_process(&create_test_reaction(456, false)),
            "Default allows users"
        );
        assert!(
            filter.should_process(&create_test_reaction(789, true)),
            "Default allows bots"
        );
    }

    #[rstest]
    #[case("self", create_test_reaction(123, false), "user", "self has highest priority")]
    #[case("bot", create_test_reaction(456, true), "user", "bot has priority over user")]
    fn test_sender_type_priority(
        #[case] _expected_type: &str,
        #[case] reaction: MockReaction,
        #[case] lower_priority_policy: &str,
        #[case] description: &str,
    ) {
        let policy = SenderFilterPolicy::from_policy(lower_priority_policy);
        let filter = ReactionFilter::new(UserId::new(123), policy);

        assert!(
            !filter.should_process(&reaction),
            "Priority test failed: {}",
            description
        );
    }
}
