
# gatehook

Convert events on Discord's Gateway (WebSocket API) to simple Webhook events.

## Supported Gateway Intents

[Discord Developer Portal - List of Intents](https://discord.com/developers/docs/events/gateway#list-of-intents)

- [ ] GUILDS
  - `GUILD_CREATE` `GUILD_UPDATE` `GUILD_DELETE`
  - `GUILD_ROLE_CREATE` `GUILD_ROLE_UPDATE` `GUILD_ROLE_DELETE`
  - `CHANNEL_CREATE` `CHANNEL_UPDATE` `CHANNEL_DELETE`
  - `CHANNEL_PINS_UPDATE`
  - `THREAD_CREATE` `THREAD_UPDATE` `THREAD_DELETE`
  - `THREAD_LIST_SYNC`
  - `THREAD_MEMBER_UPDATE` `THREAD_MEMBERS_UPDATE`
  - `STAGE_INSTANCE_CREATE` `STAGE_INSTANCE_UPDATE` `STAGE_INSTANCE_DELETE`
- [ ] GUILD_MEMBERS
  - `GUILD_MEMBER_ADD` `GUILD_MEMBER_UPDATE` `GUILD_MEMBER_REMOVE`
  - `THREAD_MEMBERS_UPDATE`
- [ ] GUILD_MODERATION
  - `GUILD_AUDIT_LOG_ENTRY_CREATE`
  - `GUILD_BAN_ADD` `GUILD_BAN_REMOVE`
- [ ] GUILD_EXPRESSIONS
  - `GUILD_EMOJIS_UPDATE` `GUILD_STICKERS_UPDATE`
  - `GUILD_SOUNDBOARD_SOUND_CREATE` `GUILD_SOUNDBOARD_SOUND_UPDATE` `GUILD_SOUNDBOARD_SOUND_DELETE` `GUILD_SOUNDBOARD_SOUNDS_UPDATE`
- [ ] GUILD_INTEGRATIONS
  - `GUILD_INTEGRATIONS_UPDATE`
  - `INTEGRATION_CREATE` `INTEGRATION_UPDATE` `INTEGRATION_DELETE`
- [ ] GUILD_WEBHOOKS
  - `WEBHOOKS_UPDATE`
- [ ] GUILD_INVITES
  - `INVITE_CREATE` `INVITE_DELETE`
- [ ] GUILD_VOICE_STATES
  - `VOICE_CHANNEL_EFFECT_SEND`
  - `VOICE_STATE_UPDATE`
- [ ] GUILD_PRESENCES
  - `PRESENCE_UPDATE`
- [ ] GUILD_MESSAGES
  - `MESSAGE_CREATE` `MESSAGE_UPDATE` `MESSAGE_DELETE` `MESSAGE_DELETE_BULK`
- [ ] GUILD_MESSAGE_REACTIONS
  - `MESSAGE_REACTION_ADD` `MESSAGE_REACTION_REMOVE` `MESSAGE_REACTION_REMOVE_ALL` `MESSAGE_REACTION_REMOVE_EMOJI`
- [ ] GUILD_MESSAGE_TYPING
  - `TYPING_START`
- [ ] DIRECT_MESSAGES
  - `MESSAGE_CREATE` `MESSAGE_UPDATE` `MESSAGE_DELETE`
  - `CHANNEL_PINS_UPDATE`
- [ ] DIRECT_MESSAGE_REACTIONS
  - `MESSAGE_REACTION_ADD` `MESSAGE_REACTION_REMOVE` `MESSAGE_REACTION_REMOVE_ALL` `MESSAGE_REACTION_REMOVE_EMOJI`
- [ ] DIRECT_MESSAGE_TYPING
  - `TYPING_START`
- [ ] MESSAGE_CONTENT
- [ ] GUILD_SCHEDULED_EVENTS
  - `GUILD_SCHEDULED_EVENT_CREATE` `GUILD_SCHEDULED_EVENT_UPDATE` `GUILD_SCHEDULED_EVENT_DELETE`
  - `GUILD_SCHEDULED_EVENT_USER_ADD` `GUILD_SCHEDULED_EVENT_USER_REMOVE`
- [ ] AUTO_MODERATION_CONFIGURATION
  - `AUTO_MODERATION_RULE_CREATE` `AUTO_MODERATION_RULE_UPDATE` `AUTO_MODERATION_RULE_DELETE`
- [ ] AUTO_MODERATION_EXECUTION
  - `AUTO_MODERATION_ACTION_EXECUTION`
- [ ] GUILD_MESSAGE_POLLS
  - `MESSAGE_POLL_VOTE_ADD` `MESSAGE_POLL_VOTE_REMOVE`
- [ ] DIRECT_MESSAGE_POLLS
  - `MESSAGE_POLL_VOTE_ADD` `MESSAGE_POLL_VOTE_REMOVE`

## References

- [Discord Developer Portal - Overview of Events](https://discord.com/developers/docs/events/overview)
- [serenity](https://github.com/serenity-rs/serenity)
