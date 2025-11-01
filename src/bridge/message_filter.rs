use serenity::model::channel::Message;
use serenity::model::id::UserId;

/// Message filter based on sender classification
///
/// Filters messages based on MECE (Mutually Exclusive, Collectively Exhaustive)
/// classification of message senders.
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
    /// # MECE Classification
    ///
    /// Messages are classified in priority order:
    /// 1. self - Bot's own messages
    /// 2. webhook - Webhook messages
    /// 3. system - System messages
    /// 4. bot - Other bot messages
    /// 5. user - Human user messages (default)
    ///
    /// This ensures every message falls into exactly one category.
    pub fn should_process(&self, message: &Message, current_user_id: UserId) -> bool {
        // Priority-based MECE classification

        // 1. self
        if message.author.id == current_user_id {
            return self.allow_self;
        }

        // 2. webhook
        if message.webhook_id.is_some() {
            return self.allow_webhook;
        }

        // 3. system
        if message.author.system {
            return self.allow_system;
        }

        // 4. bot
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
    use serenity::model::prelude::*;

    fn create_test_message(
        author_id: u64,
        is_bot: bool,
        is_system: bool,
        webhook_id: Option<u64>,
    ) -> Message {
        // Create a minimal Message for testing
        // Note: This is a simplified mock; actual Message construction is complex
        let mut msg = Message {
            id: MessageId::new(1),
            channel_id: ChannelId::new(1),
            author: User {
                id: UserId::new(author_id),
                bot: is_bot,
                system: is_system,
                ..Default::default()
            },
            content: String::new(),
            timestamp: Default::default(),
            edited_timestamp: None,
            tts: false,
            mention_everyone: false,
            mentions: vec![],
            mention_roles: vec![],
            mention_channels: vec![],
            attachments: vec![],
            embeds: vec![],
            reactions: vec![],
            nonce: None,
            pinned: false,
            webhook_id: webhook_id.map(WebhookId::new),
            kind: MessageType::Regular,
            activity: None,
            application: None,
            application_id: None,
            message_reference: None,
            flags: None,
            referenced_message: None,
            interaction: None,
            components: vec![],
            sticker_items: vec![],
            position: None,
            role_subscription_data: None,
            guild_id: None,
            member: None,
            poll: None,
            call: None,
        };
        msg
    }

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
    fn test_mece_classification_self() {
        let filter = MessageFilter::from_policy("self");
        let current_user_id = UserId::new(123);
        let msg = create_test_message(123, false, false, None);

        assert!(filter.should_process(&msg, current_user_id));
    }

    #[test]
    fn test_mece_classification_webhook() {
        let filter = MessageFilter::from_policy("webhook");
        let current_user_id = UserId::new(123);
        let msg = create_test_message(456, false, false, Some(789));

        assert!(filter.should_process(&msg, current_user_id));
    }

    #[test]
    fn test_mece_classification_bot() {
        let filter = MessageFilter::from_policy("bot");
        let current_user_id = UserId::new(123);
        let msg = create_test_message(456, true, false, None);

        assert!(filter.should_process(&msg, current_user_id));
    }

    #[test]
    fn test_mece_classification_user() {
        let filter = MessageFilter::from_policy("user");
        let current_user_id = UserId::new(123);
        let msg = create_test_message(456, false, false, None);

        assert!(filter.should_process(&msg, current_user_id));
    }

    #[test]
    fn test_filter_excludes_self_by_default() {
        let filter = MessageFilter::from_policy("");
        let current_user_id = UserId::new(123);
        let msg = create_test_message(123, false, false, None);

        assert!(!filter.should_process(&msg, current_user_id));
    }
}
