use serenity::model::id::UserId;

use super::filter::MessageFilter;

/// Message filter policy parsed from environment variable
///
/// This represents the filtering rules without being tied to a specific bot user ID.
/// Can be created at startup before connecting to Discord.
///
/// The default policy allows everything except self (safe default for bots).
#[derive(Debug, Clone)]
pub struct MessageFilterPolicy {
    pub(super) allow_self: bool,
    pub(super) allow_webhook: bool,
    pub(super) allow_system: bool,
    pub(super) allow_bot: bool,
    pub(super) allow_user: bool,
}

impl Default for MessageFilterPolicy {
    /// Default policy: allow all except self
    ///
    /// This is a safe default for bots to avoid processing their own messages.
    fn default() -> Self {
        Self {
            allow_self: false,
            allow_webhook: true,
            allow_system: true,
            allow_bot: true,
            allow_user: true,
        }
    }
}

impl MessageFilterPolicy {
    /// Create a policy from a policy string
    ///
    /// # Policy Syntax
    ///
    /// - `"all"` - Allow all messages including self
    /// - `""` (empty) - Allow all except self (default: user,bot,webhook,system)
    /// - `"user"` - Allow only human users
    /// - `"user,bot"` - Allow humans and other bots
    /// - etc.
    ///
    /// # Available Subjects
    ///
    /// - `self` - Bot's own messages
    /// - `webhook` - Messages from webhooks
    /// - `system` - Discord system messages
    /// - `bot` - Messages from other bots
    /// - `user` - Messages from human users
    pub fn from_policy(policy: &str) -> Self {
        let policy = policy.trim();

        // Empty string = use default (everything except self)
        if policy.is_empty() {
            return Self::default();
        }

        // "all" = everything including self
        if policy == "all" {
            return Self::all();
        }

        // Parse comma-separated list
        let allowed: Vec<&str> = policy.split(',').map(|s| s.trim()).collect();

        Self {
            allow_self: allowed.contains(&"self"),
            allow_webhook: allowed.contains(&"webhook"),
            allow_system: allowed.contains(&"system"),
            allow_bot: allowed.contains(&"bot"),
            allow_user: allowed.contains(&"user"),
        }
    }

    /// Allow all messages including self
    pub fn all() -> Self {
        Self {
            allow_self: true,
            allow_webhook: true,
            allow_system: true,
            allow_bot: true,
            allow_user: true,
        }
    }

    /// Create a MessageFilter for a specific user ID
    pub fn for_user(&self, current_user_id: UserId) -> MessageFilter {
        MessageFilter::new(current_user_id, self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("all", true, true, true, true, true)]
    #[case("", false, true, true, true, true)]
    #[case("user", false, false, false, false, true)]
    #[case("bot", false, false, false, true, false)]
    #[case("webhook", false, true, false, false, false)]
    #[case("system", false, false, true, false, false)]
    #[case("self", true, false, false, false, false)]
    #[case("user,bot", false, false, false, true, true)]
    #[case("user,bot,webhook", false, true, false, true, true)]
    #[case("user , bot , webhook", false, true, false, true, true)]
    fn test_policy_parsing(
        #[case] policy_str: &str,
        #[case] expect_self: bool,
        #[case] expect_webhook: bool,
        #[case] expect_system: bool,
        #[case] expect_bot: bool,
        #[case] expect_user: bool,
    ) {
        let policy = MessageFilterPolicy::from_policy(policy_str);
        assert_eq!(
            policy.allow_self, expect_self,
            "allow_self mismatch for policy: '{}'",
            policy_str
        );
        assert_eq!(
            policy.allow_webhook, expect_webhook,
            "allow_webhook mismatch for policy: '{}'",
            policy_str
        );
        assert_eq!(
            policy.allow_system, expect_system,
            "allow_system mismatch for policy: '{}'",
            policy_str
        );
        assert_eq!(
            policy.allow_bot, expect_bot,
            "allow_bot mismatch for policy: '{}'",
            policy_str
        );
        assert_eq!(
            policy.allow_user, expect_user,
            "allow_user mismatch for policy: '{}'",
            policy_str
        );
    }

    #[test]
    fn test_default_policy() {
        let policy = MessageFilterPolicy::default();
        assert!(!policy.allow_self, "Default should block self");
        assert!(policy.allow_webhook, "Default should allow webhook");
        assert!(policy.allow_system, "Default should allow system");
        assert!(policy.allow_bot, "Default should allow bot");
        assert!(policy.allow_user, "Default should allow user");
    }

    #[test]
    fn test_for_user_creates_filter() {
        use super::super::tests::MockMessage;

        let policy = MessageFilterPolicy::from_policy("user,bot");
        let user_id = UserId::new(12345);

        let filter = policy.for_user(user_id);

        // Verify filter is created with correct user_id (tested via should_process)
        let self_message = MockMessage::new(12345);
        assert!(
            !filter.should_process(&self_message),
            "Should block self messages"
        );

        let user_message = MockMessage::new(67890);
        assert!(
            filter.should_process(&user_message),
            "Should allow user messages"
        );
    }
}
