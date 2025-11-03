use serenity::model::id::UserId;

use super::message_like::FilterableMessage;

/// Mock message implementation for unit testing
#[derive(Debug)]
pub(super) struct MockMessage {
    author_id: UserId,
    is_bot: bool,
    is_system: bool,
    webhook_id: Option<u64>,
}

impl MockMessage {
    pub(super) fn new(author_id: u64) -> Self {
        Self {
            author_id: UserId::new(author_id),
            is_bot: false,
            is_system: false,
            webhook_id: None,
        }
    }

    pub(super) fn bot(mut self) -> Self {
        self.is_bot = true;
        self
    }

    pub(super) fn system(mut self) -> Self {
        self.is_system = true;
        self
    }

    pub(super) fn webhook(mut self, webhook_id: u64) -> Self {
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
