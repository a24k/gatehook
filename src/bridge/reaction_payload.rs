use serde::Serialize;
use serenity::model::channel::{GuildChannel, Reaction};

/// Wrapper for reaction event payload sent to webhook
///
/// Wraps serenity's Reaction with optional channel metadata for richer context.
///
/// # JSON Structure
///
/// ```json
/// {
///   "reaction": { ... },        // Discord Reaction object
///   "channel": { ... }          // Optional GuildChannel (omitted for DMs)
/// }
/// ```
#[derive(Serialize)]
pub struct ReactionPayload<'a> {
    reaction: &'a Reaction,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<GuildChannel>,
}

impl<'a> ReactionPayload<'a> {
    /// Create payload without channel info (for DMs or cache misses)
    pub fn new(reaction: &'a Reaction) -> Self {
        Self {
            reaction,
            channel: None,
        }
    }

    /// Create payload with channel info (for guild reactions)
    pub fn with_channel(reaction: &'a Reaction, channel: GuildChannel) -> Self {
        Self {
            reaction,
            channel: Some(channel),
        }
    }
}
