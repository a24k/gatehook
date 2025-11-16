use serenity::model::channel::{Reaction, ReactionType};
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, GuildId, MessageId, UserId};
use serenity::model::user::User;

/// Helper to create mock Reaction instances for testing
pub struct MockReactionBuilder {
    channel_id: ChannelId,
    emoji: ReactionType,
    guild_id: Option<GuildId>,
    member: Option<Member>,
    message_id: MessageId,
    user_id: Option<UserId>,
}

impl MockReactionBuilder {
    pub fn new(message_id: u64, channel_id: u64) -> Self {
        Self {
            channel_id: ChannelId::new(channel_id),
            emoji: ReactionType::Unicode("👍".to_string()),
            guild_id: None,
            member: None,
            message_id: MessageId::new(message_id),
            user_id: None,
        }
    }

    pub fn emoji(mut self, emoji: &str) -> Self {
        self.emoji = ReactionType::Unicode(emoji.to_string());
        self
    }

    pub fn user_id(mut self, user_id: u64) -> Self {
        self.user_id = Some(UserId::new(user_id));
        self
    }

    pub fn guild(mut self, guild_id: u64, user_id: u64) -> Self {
        self.guild_id = Some(GuildId::new(guild_id));

        // Create member with user info
        let mut user = User::default();
        user.id = UserId::new(user_id);
        user.bot = false;

        let mut member = Member::default();
        member.user = user;

        self.member = Some(member);
        self.user_id = Some(UserId::new(user_id));
        self
    }

    pub fn build(self) -> Reaction {
        // Use serde_json to construct the non-exhaustive Reaction struct
        let json = serde_json::json!({
            "type": 0, // Normal reaction
            "channel_id": self.channel_id.to_string(),
            "emoji": {
                "name": match &self.emoji {
                    ReactionType::Unicode(s) => s.clone(),
                    _ => "👍".to_string(),
                },
                "id": null
            },
            "guild_id": self.guild_id.map(|id| id.to_string()),
            "member": self.member.as_ref().map(|m| {
                serde_json::json!({
                    "user": {
                        "id": m.user.id.to_string(),
                        "username": "test_user",
                        "discriminator": "0",
                        "global_name": null,
                        "avatar": null,
                        "bot": m.user.bot,
                        "public_flags": 0,
                        "flags": 0
                    },
                    "nick": null,
                    "avatar": null,
                    "roles": [],
                    "joined_at": "2024-01-01T00:00:00.000000+00:00",
                    "deaf": false,
                    "mute": false,
                    "flags": 0
                })
            }),
            "message_id": self.message_id.to_string(),
            "user_id": self.user_id.map(|id| id.to_string()),
            "count_details": {
                "burst": 0,
                "normal": 1
            },
            "burst_colours": [],
            "me_burst": false,
            "me": false,
            "burst": false,
            "message_author_id": self.user_id.map(|id| id.to_string())
        });

        serde_json::from_value(json).expect("Failed to deserialize mock Reaction")
    }
}
