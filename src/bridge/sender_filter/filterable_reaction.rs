use serenity::model::id::UserId;

/// Trait for types that can be filtered as reactions
///
/// This abstraction allows testing without depending on serenity's non-exhaustive Reaction type.
pub trait FilterableReaction {
    /// Get the user ID who added the reaction
    fn user_id(&self) -> Option<UserId>;

    /// Check if the user is a bot
    fn is_bot(&self) -> bool;
}

// Implement for serenity's Reaction type
impl FilterableReaction for serenity::model::channel::Reaction {
    fn user_id(&self) -> Option<UserId> {
        self.user_id
    }

    fn is_bot(&self) -> bool {
        self.member.as_ref()
            .and_then(|m| Some(m.user.bot))
            .unwrap_or(false)
    }
}
