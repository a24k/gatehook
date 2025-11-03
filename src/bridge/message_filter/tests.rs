use rstest::rstest;
use serenity::model::id::UserId;

use super::message_like::FilterableMessage;
use super::policy::MessageFilterPolicy;

/// Mock message implementation for unit testing
#[derive(Debug)]
struct MockMessage {
    author_id: UserId,
    is_bot: bool,
    is_system: bool,
    webhook_id: Option<u64>,
}

impl MockMessage {
    fn new(author_id: u64) -> Self {
        Self {
            author_id: UserId::new(author_id),
            is_bot: false,
            is_system: false,
            webhook_id: None,
        }
    }

    fn bot(mut self) -> Self {
        self.is_bot = true;
        self
    }

    fn system(mut self) -> Self {
        self.is_system = true;
        self
    }

    fn webhook(mut self, webhook_id: u64) -> Self {
        self.webhook_id = Some(webhook_id);
        self
    }
}

impl FilterableMessage for MockMessage {
    fn author_id(&self) -> UserId {
        self.author_id
    }

    fn is_bot(&self) -> bool {
        self.is_bot
    }

    fn is_system(&self) -> bool {
        self.is_system
    }

    fn webhook_id(&self) -> Option<u64> {
        self.webhook_id
    }
}

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

// Tests for Default trait
#[test]
fn test_default_policy() {
    let policy = MessageFilterPolicy::default();
    assert!(!policy.allow_self, "Default should block self");
    assert!(policy.allow_webhook, "Default should allow webhook");
    assert!(policy.allow_system, "Default should allow system");
    assert!(policy.allow_bot, "Default should allow bot");
    assert!(policy.allow_user, "Default should allow user");
}

// Tests for for_user method
#[test]
fn test_for_user_creates_filter() {
    let policy = MessageFilterPolicy::from_policy("user,bot");
    let user_id = UserId::new(12345);

    let filter = policy.for_user(user_id);

    // Verify filter is created with correct user_id (tested via should_process)
    let self_message = MockMessage::new(12345);
    assert!(!filter.should_process(&self_message), "Should block self messages");

    let user_message = MockMessage::new(67890);
    assert!(filter.should_process(&user_message), "Should allow user messages");
}

// Tests for MessageFilter::should_process - sender type classification

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

    assert!(!filter.should_process(&MockMessage::new(123)), "Default blocks self");
    assert!(filter.should_process(&MockMessage::new(456)), "Default allows users");
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
        MockMessage::new(123),                              // self
        MockMessage::new(456),                              // user
        MockMessage::new(456).bot(),                        // bot
        MockMessage::new(456).bot().webhook(789),           // webhook
        MockMessage::new(456).system(),                     // system
    ];

    for msg in &test_messages {
        assert_eq!(
            filter_empty.should_process(msg),
            filter_default.should_process(msg),
            "Empty policy should behave same as default"
        );
    }
}
