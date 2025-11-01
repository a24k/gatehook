use serenity::model::channel::Message;
use serenity::model::id::UserId;

/// Message filter based on sender type classification
///
/// Filters messages by classifying them into mutually exclusive sender categories.
#[derive(Debug, Clone)]
pub struct MessageFilter {
    allow_self: bool,
    allow_webhook: bool,
    allow_system: bool,
    allow_bot: bool,
    allow_user: bool,
}

impl MessageFilter {
    /// Create a filter from a policy string
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
    pub fn should_process(&self, message: &Message, current_user_id: UserId) -> bool {
        // Sender type classification

        // 1. self
        if message.author.id == current_user_id {
            return self.allow_self;
        }

        // 2. webhook (excluding self)
        if message.webhook_id.is_some() {
            return self.allow_webhook;
        }

        // 3. system (excluding self and webhooks)
        if message.author.system {
            return self.allow_system;
        }

        // 4. bot (excluding self and webhooks)
        // Note: Discord webhooks have author.bot = true, but are classified as 'webhook' above
        if message.author.bot {
            return self.allow_bot;
        }

        // 5. user (default)
        self.allow_user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_all() {
        let filter = MessageFilter::from_policy("all");
        assert!(filter.allow_self);
        assert!(filter.allow_webhook);
        assert!(filter.allow_system);
        assert!(filter.allow_bot);
        assert!(filter.allow_user);
    }

    #[test]
    fn test_policy_empty() {
        let filter = MessageFilter::from_policy("");
        assert!(!filter.allow_self); // Self is excluded
        assert!(filter.allow_webhook);
        assert!(filter.allow_system);
        assert!(filter.allow_bot);
        assert!(filter.allow_user);
    }

    #[test]
    fn test_policy_user_only() {
        let filter = MessageFilter::from_policy("user");
        assert!(!filter.allow_self);
        assert!(!filter.allow_webhook);
        assert!(!filter.allow_system);
        assert!(!filter.allow_bot);
        assert!(filter.allow_user);
    }

    #[test]
    fn test_policy_user_and_bot() {
        let filter = MessageFilter::from_policy("user,bot");
        assert!(!filter.allow_self);
        assert!(!filter.allow_webhook);
        assert!(!filter.allow_system);
        assert!(filter.allow_bot);
        assert!(filter.allow_user);
    }

    #[test]
    fn test_policy_self_only() {
        let filter = MessageFilter::from_policy("self");
        assert!(filter.allow_self);
        assert!(!filter.allow_webhook);
        assert!(!filter.allow_system);
        assert!(!filter.allow_bot);
        assert!(!filter.allow_user);
    }

    #[test]
    fn test_policy_webhook_only() {
        let filter = MessageFilter::from_policy("webhook");
        assert!(!filter.allow_self);
        assert!(filter.allow_webhook);
        assert!(!filter.allow_system);
        assert!(!filter.allow_bot);
        assert!(!filter.allow_user);
    }

    #[test]
    fn test_policy_system_only() {
        let filter = MessageFilter::from_policy("system");
        assert!(!filter.allow_self);
        assert!(!filter.allow_webhook);
        assert!(filter.allow_system);
        assert!(!filter.allow_bot);
        assert!(!filter.allow_user);
    }

    #[test]
    fn test_policy_bot_only() {
        let filter = MessageFilter::from_policy("bot");
        assert!(!filter.allow_self);
        assert!(!filter.allow_webhook);
        assert!(!filter.allow_system);
        assert!(filter.allow_bot);
        assert!(!filter.allow_user);
    }

    #[test]
    fn test_policy_complex_combination() {
        let filter = MessageFilter::from_policy("user,bot,webhook");
        assert!(!filter.allow_self);
        assert!(filter.allow_webhook);
        assert!(!filter.allow_system);
        assert!(filter.allow_bot);
        assert!(filter.allow_user);
    }

    #[test]
    fn test_policy_whitespace_handling() {
        let filter = MessageFilter::from_policy("user , bot , webhook");
        assert!(!filter.allow_self);
        assert!(filter.allow_webhook);
        assert!(!filter.allow_system);
        assert!(filter.allow_bot);
        assert!(filter.allow_user);
    }
}
