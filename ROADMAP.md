# Roadmap

This document tracks planned features and Discord Gateway event support.

## Gateway Intents Support

[Discord Developer Portal - List of Intents](https://discord.com/developers/docs/events/gateway#list-of-intents)

**Legend:** 🔒 Privileged intent • 🎯 Sender filtering available

### Messages

- **GUILD_MESSAGES** 🎯
  - [x] `MESSAGE_CREATE` via `MESSAGE_GUILD`
  - [x] `MESSAGE_UPDATE` via `MESSAGE_UPDATE_GUILD`
  - [x] `MESSAGE_DELETE` via `MESSAGE_DELETE_GUILD`
  - [x] `MESSAGE_DELETE_BULK` via `MESSAGE_DELETE_BULK_GUILD`
- **DIRECT_MESSAGES** 🎯
  - [x] `MESSAGE_CREATE` via `MESSAGE_DIRECT`
  - [x] `MESSAGE_UPDATE` via `MESSAGE_UPDATE_DIRECT`
  - [x] `MESSAGE_DELETE` via `MESSAGE_DELETE_DIRECT`
  - [ ] `CHANNEL_PINS_UPDATE`
- **MESSAGE_CONTENT** 🔒 *(Auto-enabled with MESSAGE_CREATE or MESSAGE_UPDATE)*
  - Automatically enabled when MESSAGE_DIRECT, MESSAGE_GUILD, MESSAGE_UPDATE_DIRECT, or MESSAGE_UPDATE_GUILD is configured
  - Not required for MESSAGE_DELETE events (only IDs available, no content)
- **GUILD_MESSAGE_REACTIONS** 🎯
  - [x] `MESSAGE_REACTION_ADD` via `REACTION_ADD_GUILD`
  - [x] `MESSAGE_REACTION_REMOVE` via `REACTION_REMOVE_GUILD`
  - [ ] `MESSAGE_REACTION_REMOVE_ALL`
  - [ ] `MESSAGE_REACTION_REMOVE_EMOJI`
- **DIRECT_MESSAGE_REACTIONS** 🎯
  - [x] `MESSAGE_REACTION_ADD` via `REACTION_ADD_DIRECT`
  - [x] `MESSAGE_REACTION_REMOVE` via `REACTION_REMOVE_DIRECT`
  - [ ] `MESSAGE_REACTION_REMOVE_ALL`
  - [ ] `MESSAGE_REACTION_REMOVE_EMOJI`
- **GUILD_MESSAGE_TYPING**
  - [ ] `TYPING_START`
- **DIRECT_MESSAGE_TYPING**
  - [ ] `TYPING_START`

### Guilds & Channels

- **GUILDS** *(Auto-enabled with MESSAGE_GUILD)*
  - Automatically enabled for cache access (guild/channel metadata)
  - [ ] `GUILD_CREATE` `GUILD_UPDATE` `GUILD_DELETE`
  - [ ] `GUILD_ROLE_CREATE` `GUILD_ROLE_UPDATE` `GUILD_ROLE_DELETE`
  - [ ] `CHANNEL_CREATE` `CHANNEL_UPDATE` `CHANNEL_DELETE`
  - [ ] `CHANNEL_PINS_UPDATE`
  - [ ] `THREAD_CREATE` `THREAD_UPDATE` `THREAD_DELETE`
  - [ ] `THREAD_LIST_SYNC`
  - [ ] `THREAD_MEMBER_UPDATE` `THREAD_MEMBERS_UPDATE`
  - [ ] `STAGE_INSTANCE_CREATE` `STAGE_INSTANCE_UPDATE` `STAGE_INSTANCE_DELETE`

### Members & Moderation

- **GUILD_MEMBERS** 🔒
  - [ ] `GUILD_MEMBER_ADD`
  - [ ] `GUILD_MEMBER_UPDATE`
  - [ ] `GUILD_MEMBER_REMOVE`
  - [ ] `THREAD_MEMBERS_UPDATE`
- **GUILD_MODERATION**
  - [ ] `GUILD_AUDIT_LOG_ENTRY_CREATE`
  - [ ] `GUILD_BAN_ADD`
  - [ ] `GUILD_BAN_REMOVE`
- **GUILD_PRESENCES** 🔒
  - [ ] `PRESENCE_UPDATE`

### Other Features

- **GUILD_EXPRESSIONS**
  - [ ] `GUILD_EMOJIS_UPDATE`
  - [ ] `GUILD_STICKERS_UPDATE`
  - [ ] Soundboard events
- **GUILD_INTEGRATIONS**
  - [ ] `GUILD_INTEGRATIONS_UPDATE`
  - [ ] `INTEGRATION_CREATE` `INTEGRATION_UPDATE` `INTEGRATION_DELETE`
- **GUILD_WEBHOOKS**
  - [ ] `WEBHOOKS_UPDATE`
- **GUILD_INVITES**
  - [ ] `INVITE_CREATE`
  - [ ] `INVITE_DELETE`
- **GUILD_VOICE_STATES**
  - [ ] `VOICE_CHANNEL_EFFECT_SEND`
  - [ ] `VOICE_STATE_UPDATE`
- **GUILD_SCHEDULED_EVENTS**
  - [ ] `GUILD_SCHEDULED_EVENT_CREATE` `GUILD_SCHEDULED_EVENT_UPDATE` `GUILD_SCHEDULED_EVENT_DELETE`
  - [ ] `GUILD_SCHEDULED_EVENT_USER_ADD` `GUILD_SCHEDULED_EVENT_USER_REMOVE`
- **AUTO_MODERATION_CONFIGURATION**
  - [ ] `AUTO_MODERATION_RULE_CREATE` `AUTO_MODERATION_RULE_UPDATE` `AUTO_MODERATION_RULE_DELETE`
- **AUTO_MODERATION_EXECUTION**
  - [ ] `AUTO_MODERATION_ACTION_EXECUTION`
- **GUILD_MESSAGE_POLLS / DIRECT_MESSAGE_POLLS**
  - [ ] `MESSAGE_POLL_VOTE_ADD`
  - [ ] `MESSAGE_POLL_VOTE_REMOVE`

## Planned Features

### High Priority
- [ ] `MESSAGE_REACTION_REMOVE_ALL` event support
- [ ] `MESSAGE_REACTION_REMOVE_EMOJI` event support
- [ ] Retry logic and circuit breakers for HTTP calls
- [ ] More sophisticated error handling strategies

### Medium Priority
- [ ] `CHANNEL_PINS_UPDATE` event support
- [ ] `TYPING_START` event support
- [ ] Event middleware/pipeline pattern for transformations

### Low Priority
- Guild management events (GUILD_CREATE, GUILD_UPDATE, etc.)
- Member events (GUILD_MEMBER_ADD, GUILD_MEMBER_UPDATE, etc.)
- Voice and presence events

## Contributing

Feature requests and contributions are welcome! Please open an issue to discuss planned features before implementing.
