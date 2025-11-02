use serenity::model::channel::Message;
use serenity::model::id::UserId;

/// Message filter policy parsed from environment variable
///
/// This represents the filtering rules without being tied to a specific bot user ID.
/// Can be created at startup before connecting to Discord.
#[derive(Debug, Clone)]
pub struct MessageFilterPolicy {
    allow_self: bool,
    allow_webhook: bool,
    allow_system: bool,
    allow_bot: bool,
    allow_user: bool,
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

        // Empty string = everything except self (safe default)
        if policy.is_empty() {
            return Self::default_allow();
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

    /// Default allow (empty string): everything except self
    pub fn default_allow() -> Self {
        Self {
            allow_self: false,
            allow_webhook: true,
            allow_system: true,
            allow_bot: true,
            allow_user: true,
        }
    }

    /// Create an active filter with the bot's user ID
    pub fn with_user_id(&self, current_user_id: UserId) -> MessageFilter {
        MessageFilter {
            current_user_id,
            policy: self.clone(),
        }
    }
}

/// Active message filter with bot's user ID
///
/// This is created after the bot connects to Discord and knows its user ID.
#[derive(Debug, Clone)]
pub struct MessageFilter {
    current_user_id: UserId,
    policy: MessageFilterPolicy,
}

impl MessageFilter {
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
    pub fn should_process(&self, message: &Message) -> bool {
        // Sender type classification

        // 1. self
        if message.author.id == self.current_user_id {
            return self.policy.allow_self;
        }

        // 2. webhook (excluding self)
        if message.webhook_id.is_some() {
            return self.policy.allow_webhook;
        }

        // 3. system (excluding self and webhooks)
        if message.author.system {
            return self.policy.allow_system;
        }

        // 4. bot (excluding self and webhooks)
        // Note: Discord webhooks have author.bot = true, but are classified as 'webhook' above
        if message.author.bot {
            return self.policy.allow_bot;
        }

        // 5. user (default)
        self.policy.allow_user
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
}
