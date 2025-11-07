
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

### Sender Type Classification

Messages are classified into mutually exclusive sender categories. Each message falls into exactly one category:

1. **self** - Bot's own messages
2. **webhook** - Webhook messages (excluding self)
3. **system** - Discord system messages (excluding self and webhooks)
4. **bot** - Other bot messages (excluding self and webhooks)
5. **user** - Human user messages (default/fallback)

**Note**: Discord webhooks have `author.bot = true`, but are classified as `webhook` rather than `bot` to allow separate filtering policies.

This classification ensures:
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

## Webhook Response Actions

Your HTTP endpoint can respond with actions for gatehook to execute on Discord. This enables bidirectional communication - your webhook receives events and can instruct gatehook to reply, react, or perform other Discord operations.

### Response Format

Return a JSON object with an `actions` array:

```json
{
  "actions": [
    {
      "type": "reply",
      "content": "Hello! I received your message.",
      "mention": false
    }
  ]
}
```

### Available Actions

#### `reply` - Reply to Message

Reply to the message that triggered the event.

**Available in:** `message` handler only

```json
{
  "type": "reply",
  "content": "Your reply text here",
  "mention": false
}
```

**Parameters:**
- `content` (string, required): Reply text (max 2000 Unicode codepoints, auto-truncated if exceeded)
- `mention` (boolean, optional, default: `false`): Whether to ping/notify the original message author

**Examples:**

```json
// Simple reply without notification
{
  "type": "reply",
  "content": "Got it! Processing your request...",
  "mention": false
}

// Reply with notification
{
  "type": "reply",
  "content": "⚠️ Important: Your request requires attention!",
  "mention": true
}

// Long content (automatically truncated to 2000 chars)
{
  "type": "reply",
  "content": "Very long text... (will be truncated to 1997 chars + '...')",
  "mention": false
}
```

#### `react` - Add Reaction

Add a reaction emoji to the message that triggered the event.

**Available in:** `message` handler only

```json
{
  "type": "react",
  "emoji": "👍"
}
```

**Parameters:**
- `emoji` (string, required): Emoji to react with
  - **Unicode emoji**: Use the emoji directly (e.g., `"👍"`, `"❤️"`, `"🎉"`)
  - **Custom emoji**: Use format `"name:id"` (e.g., `"customemoji:123456789012345678"`)

**Examples:**

```json
// Unicode emoji
{
  "type": "react",
  "emoji": "✅"
}

// Custom server emoji
{
  "type": "react",
  "emoji": "mycustomemoji:987654321098765432"
}
```

#### `thread` - Create Thread

Create a thread from the message that triggered the event (or send a reply if already in a thread).

**Available in:** `message` handler in guild channels only (not DMs)

```json
{
  "type": "thread",
  "name": "Discussion Topic",
  "auto_archive_duration": 1440,
  "reply": {
    "content": "Let's discuss this here!",
    "mention": false
  }
}
```

**Parameters:**
- `name` (string, optional): Thread name (max 100 Unicode codepoints, auto-truncated if exceeded)
  - If omitted, auto-generated from first line of message
  - If message is empty, defaults to "Thread"
- `auto_archive_duration` (integer, optional, default: `1440`): Minutes until thread auto-archives
  - Valid values: `60` (1 hour), `1440` (1 day), `4320` (3 days), `10080` (1 week)
  - Invalid values default to `1440` with warning log
- `reply` (object, optional): Optional reply to send in the thread
  - `content` (string, required): Reply text (max 2000 chars)
  - `mention` (boolean, optional, default: `false`): Whether to mention the author

**Behavior:**
- **Normal channel**: Creates a new thread from the message
- **Already in thread**: Skips thread creation, sends reply to existing thread
- **DM channel**: Fails with error (threads not supported in DMs)

**Examples:**

```json
// Create thread with auto-generated name
{
  "type": "thread",
  "auto_archive_duration": 1440
}

// Create thread with custom name and reply
{
  "type": "thread",
  "name": "Bug Report #1234",
  "auto_archive_duration": 10080,
  "reply": {
    "content": "Thanks for reporting! Let's track this here.",
    "mention": true
  }
}

// Create thread with minimal config
{
  "type": "thread",
  "name": "Quick Discussion"
}
```

### Multiple Actions

Execute multiple actions in sequence:

```json
{
  "actions": [
    {
      "type": "react",
      "emoji": "👀"
    },
    {
      "type": "reply",
      "content": "Processing your request...",
      "mention": false
    },
    {
      "type": "thread",
      "name": "Request Processing",
      "reply": {
        "content": "Tracking progress here!",
        "mention": false
      }
    }
  ]
}
```

**Note:** Actions are executed sequentially in the order specified. If one action fails, remaining actions continue.

### Error Handling

- **Non-2xx HTTP Status**: Actions are still executed if present in response body
- **Invalid JSON**: Logged as warning, no actions executed
- **Action Execution Failure**: Logged as error, remaining actions continue
- **Content Too Long**: Automatically truncated to 2000 chars with "..." suffix

### Empty or No Response

If your endpoint returns an empty response or no `actions` field, no actions are executed:

```json
{}
// or
{"actions": []}
// Both are valid and result in no action
```

## Gateway Intents Support

[Discord Developer Portal - List of Intents](https://discord.com/developers/docs/events/gateway#list-of-intents)

**Legend:** 🔒 Privileged intent • 🎯 Sender filtering available

### Messages

- **GUILD_MESSAGES** 🎯
  - [x] `MESSAGE_CREATE` via `MESSAGE_GUILD`
  - [ ] `MESSAGE_UPDATE`
  - [ ] `MESSAGE_DELETE`
  - [ ] `MESSAGE_DELETE_BULK`
- **DIRECT_MESSAGES** 🎯
  - [x] `MESSAGE_CREATE` via `MESSAGE_DIRECT`
  - [ ] `MESSAGE_UPDATE`
  - [ ] `MESSAGE_DELETE`
  - [ ] `CHANNEL_PINS_UPDATE`
- **MESSAGE_CONTENT** *(Auto-enabled with MESSAGE_*)*
  - [x] Automatically enabled
- **GUILD_MESSAGE_REACTIONS**
  - [ ] `MESSAGE_REACTION_ADD`
  - [ ] `MESSAGE_REACTION_REMOVE`
  - [ ] `MESSAGE_REACTION_REMOVE_ALL`
  - [ ] `MESSAGE_REACTION_REMOVE_EMOJI`
- **DIRECT_MESSAGE_REACTIONS**
  - [ ] `MESSAGE_REACTION_ADD`
  - [ ] `MESSAGE_REACTION_REMOVE`
  - [ ] `MESSAGE_REACTION_REMOVE_ALL`
  - [ ] `MESSAGE_REACTION_REMOVE_EMOJI`
- **GUILD_MESSAGE_TYPING**
  - [ ] `TYPING_START`
- **DIRECT_MESSAGE_TYPING**
  - [ ] `TYPING_START`

### Guilds & Channels

- **GUILDS**
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

## Development

See [CLAUDE.md](CLAUDE.md) for development guidelines and architecture details.

## References

- [Discord Developer Portal - Overview of Events](https://discord.com/developers/docs/events/overview)
- [serenity](https://github.com/serenity-rs/serenity)
