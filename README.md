
# gatehook

Bridge Discord Gateway (WebSocket) events to HTTP webhooks.

## Features

- 🎯 **Selective Event Handling** - Enable only the events you need via environment variables
- 🔍 **Fine-grained Message Filtering** - Filter messages by sender type (self, bot, user, webhook, system)
- 📨 **Context-aware Configuration** - Separate filters for Direct Messages and Guild (server) messages
- ⚡ **Dynamic Gateway Intents** - Automatically requests only the permissions needed for enabled events
- 🔐 **Secure by Default** - Bot's own messages filtered out by default to prevent loops

## Quick Start

```bash
# 1. Set required environment variables
export DISCORD_TOKEN="your_discord_bot_token"
export HTTP_ENDPOINT="https://your-webhook-endpoint.com/webhook"

# 2. Enable desired events
export MESSAGE_DIRECT="user,bot,webhook,system"  # DM: everything except self
export MESSAGE_GUILD="user"                      # Guild: only human users

# 3. Run
cargo run --release
```

## Configuration

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DISCORD_TOKEN` | Discord bot token from Discord Developer Portal | `MTA1234...` |
| `HTTP_ENDPOINT` | HTTP endpoint URL to forward Discord events | `https://example.com/webhook` |

### Optional Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `INSECURE_MODE` | Accept invalid TLS certificates (testing only) | `false` | `true` |
| `RUST_LOG` | Logging level (see [Logging](#logging)) | `gatehook=info,serenity=warn` | `debug` |

### Event Handler Configuration

Events are configured via environment variables in the format: `<EVENT_NAME>_<CONTEXT>=<allowed_subjects>`

**Contexts:**
- `DIRECT` - Direct messages (1-on-1 and group DMs)
- `GUILD` - Guild (server) messages
- *(none)* - Context-independent events (e.g., `READY`)

**Allowed Subjects (comma-separated):**
- `self` - Bot's own messages
- `webhook` - Messages from webhooks
- `system` - Discord system messages
- `bot` - Messages from other bots
- `user` - Messages from human users
- `all` - All of the above
- *(empty string)* - Everything except self (default: `user,bot,webhook,system`)

**If an environment variable is not set, that event handler will not be registered.**

#### Available Events

| Event | Direct Variable | Guild Variable | Description |
|-------|----------------|----------------|-------------|
| Message | `MESSAGE_DIRECT` | `MESSAGE_GUILD` | New message created |
| Ready | - | `READY` | Bot connected to Discord |

*More events coming soon: MESSAGE_UPDATE, MESSAGE_DELETE, REACTION_ADD, etc.*

#### Configuration Examples

```bash
# Example 1: Log all human messages in guilds, all DMs
MESSAGE_GUILD="user"
MESSAGE_DIRECT="user,bot,webhook,system"

# Example 2: Debug mode - include bot's own messages
MESSAGE_GUILD="all"
MESSAGE_DIRECT="all"

# Example 3: Only humans and other bots, no webhooks
MESSAGE_GUILD="user,bot"
MESSAGE_DIRECT="user,bot"

# Example 4: Everything except self (safe default)
MESSAGE_GUILD=""
MESSAGE_DIRECT=""

# Example 5: Enable READY event forwarding
READY="all"
```

### Filter Behavior (MECE Classification)

Messages are classified in priority order. Each message falls into exactly one category:

1. **self** - Bot's own messages (checked first by author ID)
2. **webhook** - Webhook messages (has webhook_id)
3. **system** - Discord system messages (author.system = true)
4. **bot** - Other bot messages (author.bot = true)
5. **user** - Human user messages (default/fallback)

This MECE (Mutually Exclusive, Collectively Exhaustive) design ensures:
- Every message is classified exactly once
- No ambiguity in filtering decisions
- Predictable behavior

## Logging

gatehook uses the [tracing](https://github.com/tokio-rs/tracing) crate for structured logging.

### Log Levels

- `error` - Only error messages
- `warn` - Warnings and errors
- `info` - Informational messages (default)
- `debug` - Detailed debug information including message contents
- `trace` - Very verbose logs for debugging

### Examples

```bash
# Default logging
./gatehook

# Debug logging for gatehook, suppress serenity
RUST_LOG=gatehook=debug,serenity=warn ./gatehook

# Trace everything
RUST_LOG=trace ./gatehook
```

## Event Forwarding Format

Events are forwarded to your HTTP endpoint as JSON POST requests:

```json
{
  "event_type": "message",
  "payload": {
    // Discord event data (serenity Message struct serialized)
  }
}
```

## Supported Gateway Intents

[Discord Developer Portal - List of Intents](https://discord.com/developers/docs/events/gateway#list-of-intents)

**Legend:**
- ✅ Fully supported with filtering
- 🚧 Partially supported
- ⬜ Planned
- 🔒 Requires privileged intent

- ⬜ GUILDS
  - `GUILD_CREATE` `GUILD_UPDATE` `GUILD_DELETE`
  - `GUILD_ROLE_CREATE` `GUILD_ROLE_UPDATE` `GUILD_ROLE_DELETE`
  - `CHANNEL_CREATE` `CHANNEL_UPDATE` `CHANNEL_DELETE`
  - `CHANNEL_PINS_UPDATE`
  - `THREAD_CREATE` `THREAD_UPDATE` `THREAD_DELETE`
  - `THREAD_LIST_SYNC`
  - `THREAD_MEMBER_UPDATE` `THREAD_MEMBERS_UPDATE`
  - `STAGE_INSTANCE_CREATE` `STAGE_INSTANCE_UPDATE` `STAGE_INSTANCE_DELETE`
- ⬜ 🔒 GUILD_MEMBERS
  - `GUILD_MEMBER_ADD` `GUILD_MEMBER_UPDATE` `GUILD_MEMBER_REMOVE`
  - `THREAD_MEMBERS_UPDATE`
- ⬜ GUILD_MODERATION
  - `GUILD_AUDIT_LOG_ENTRY_CREATE`
  - `GUILD_BAN_ADD` `GUILD_BAN_REMOVE`
- ⬜ GUILD_EXPRESSIONS
  - `GUILD_EMOJIS_UPDATE` `GUILD_STICKERS_UPDATE`
  - `GUILD_SOUNDBOARD_SOUND_CREATE` `GUILD_SOUNDBOARD_SOUND_UPDATE` `GUILD_SOUNDBOARD_SOUND_DELETE` `GUILD_SOUNDBOARD_SOUNDS_UPDATE`
- ⬜ GUILD_INTEGRATIONS
  - `GUILD_INTEGRATIONS_UPDATE`
  - `INTEGRATION_CREATE` `INTEGRATION_UPDATE` `INTEGRATION_DELETE`
- ⬜ GUILD_WEBHOOKS
  - `WEBHOOKS_UPDATE`
- ⬜ GUILD_INVITES
  - `INVITE_CREATE` `INVITE_DELETE`
- ⬜ GUILD_VOICE_STATES
  - `VOICE_CHANNEL_EFFECT_SEND`
  - `VOICE_STATE_UPDATE`
- ⬜ 🔒 GUILD_PRESENCES
  - `PRESENCE_UPDATE`
- 🚧 GUILD_MESSAGES
  - ✅ `MESSAGE_CREATE` (via `MESSAGE_GUILD`)
  - ⬜ `MESSAGE_UPDATE` `MESSAGE_DELETE` `MESSAGE_DELETE_BULK`
- ⬜ GUILD_MESSAGE_REACTIONS
  - `MESSAGE_REACTION_ADD` `MESSAGE_REACTION_REMOVE` `MESSAGE_REACTION_REMOVE_ALL` `MESSAGE_REACTION_REMOVE_EMOJI`
- ⬜ GUILD_MESSAGE_TYPING
  - `TYPING_START`
- 🚧 DIRECT_MESSAGES
  - ✅ `MESSAGE_CREATE` (via `MESSAGE_DIRECT`)
  - ⬜ `MESSAGE_UPDATE` `MESSAGE_DELETE`
  - `CHANNEL_PINS_UPDATE`
- ⬜ DIRECT_MESSAGE_REACTIONS
  - `MESSAGE_REACTION_ADD` `MESSAGE_REACTION_REMOVE` `MESSAGE_REACTION_REMOVE_ALL` `MESSAGE_REACTION_REMOVE_EMOJI`
- ⬜ DIRECT_MESSAGE_TYPING
  - `TYPING_START`
- ✅ MESSAGE_CONTENT *(automatically enabled when MESSAGE_* events are configured)*
- ⬜ GUILD_SCHEDULED_EVENTS
  - `GUILD_SCHEDULED_EVENT_CREATE` `GUILD_SCHEDULED_EVENT_UPDATE` `GUILD_SCHEDULED_EVENT_DELETE`
  - `GUILD_SCHEDULED_EVENT_USER_ADD` `GUILD_SCHEDULED_EVENT_USER_REMOVE`
- ⬜ AUTO_MODERATION_CONFIGURATION
  - `AUTO_MODERATION_RULE_CREATE` `AUTO_MODERATION_RULE_UPDATE` `AUTO_MODERATION_RULE_DELETE`
- ⬜ AUTO_MODERATION_EXECUTION
  - `AUTO_MODERATION_ACTION_EXECUTION`
- ⬜ GUILD_MESSAGE_POLLS
  - `MESSAGE_POLL_VOTE_ADD` `MESSAGE_POLL_VOTE_REMOVE`
- ⬜ DIRECT_MESSAGE_POLLS
  - `MESSAGE_POLL_VOTE_ADD` `MESSAGE_POLL_VOTE_REMOVE`

## Development

See [CLAUDE.md](CLAUDE.md) for development guidelines and architecture details.

## References

- [Discord Developer Portal - Overview of Events](https://discord.com/developers/docs/events/overview)
- [serenity](https://github.com/serenity-rs/serenity)
