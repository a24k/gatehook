use serenity::model::channel::Message;
use serenity::model::id::UserId;

/// Trait for filterable message objects
///
/// This trait abstracts the necessary properties of a message for filtering,
/// allowing us to test the filtering logic without depending on serenity's Message type.
pub trait FilterableMessage {
    fn author_id(&self) -> UserId;
    fn is_bot(&self) -> bool;
    fn is_system(&self) -> bool;
    fn webhook_id(&self) -> Option<u64>;
}

impl FilterableMessage for Message {
    fn author_id(&self) -> UserId {
        self.author.id
    }

    fn is_bot(&self) -> bool {
        self.author.bot
    }

    fn is_system(&self) -> bool {
        self.author.system
    }

    fn webhook_id(&self) -> Option<u64> {
        self.webhook_id.map(|id| id.get())
    }
}
