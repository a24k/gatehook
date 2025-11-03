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
