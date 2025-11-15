
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
| Message Update | `MESSAGE_UPDATE_DIRECT` | `MESSAGE_UPDATE_GUILD` | Message edited/updated |
| Message Delete | `MESSAGE_DELETE_DIRECT` | `MESSAGE_DELETE_GUILD` | Single message deleted |
| Message Delete Bulk | - | `MESSAGE_DELETE_BULK_GUILD` | Multiple messages deleted at once (guild only) |
| Ready | - | `READY` | Bot connected to Discord |

*More events coming soon: REACTION_ADD, REACTION_REMOVE, etc.*

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

# Example 6: Log message deletions (no filtering available for delete events)
MESSAGE_DELETE_GUILD="all"
MESSAGE_DELETE_BULK_GUILD="all"

# Example 7: Monitor DM deletions
MESSAGE_DELETE_DIRECT="all"
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

Events are forwarded to your HTTP endpoint as JSON POST requests with the event type specified as a query parameter:

```
POST {HTTP_ENDPOINT}?handler=message
```

### Message Event Payload

The request body contains the message data wrapped in a `message` key, with optional channel metadata:

```json
{
  "message": {
    "id": "123456789012345678",
    "content": "Hello!",
    "author": {
      "id": "234567890123456789",
      "username": "user123",
      "discriminator": "0",
      "avatar": "...",
      "bot": false
    },
    "timestamp": "2024-01-15T12:34:56.789Z",
    "channel_id": "987654321098765432",
    "guild_id": "876543210987654321",
    // ... many other Discord Message fields ...
  },
  "channel": {
    "id": "987654321098765432",
    "name": "general",
    "type": 0,
    "parent_id": null,
    "topic": "General discussion",
    "position": 0,
    // ... other GuildChannel fields ...
  }
}
```

**Note:** The `type` field is an integer representing the channel type. See [Discord's Channel Type documentation](https://discord.com/developers/docs/resources/channel#channel-object-channel-types) for the complete mapping.

**The `channel` field:**
- **Present:** For guild (server) messages when `MESSAGE_GUILD` is enabled
- **Absent:** For direct messages, or if channel information is not available in cache

**Detecting threads:**
Check the `channel.type` field:
- `11` - Public thread (`PUBLIC_THREAD`)
- `12` - Private thread (`PRIVATE_THREAD`)
- `10` - News/Announcement thread (`ANNOUNCEMENT_THREAD`)
- `0` - Regular text channel (`GUILD_TEXT`)
- `2` - Voice channel (`GUILD_VOICE`)
- Other types: `4` (Category), `5` (News), `13` (Stage), `15` (Forum), etc.

**Available channel fields** (from Discord's [GuildChannel](https://discord.com/developers/docs/resources/channel#channel-object)):
- `id` - Channel ID
- `name` - Channel name
- `type` - Channel type (integer, see above)
- `position` - Sorting position
- `parent_id` - Parent category ID (null for top-level channels)
- `topic` - Channel topic/description (if set)
- `nsfw` - Whether channel is NSFW
- `thread_metadata` - Thread-specific metadata (if channel is a thread)
- And more...

**Example: Checking if message is in a thread**
```python
# Python example
def is_in_thread(payload):
    channel = payload.get("channel")
    if not channel:
        return False  # DM or no channel info

    channel_type = channel.get("type")
    # 10 = ANNOUNCEMENT_THREAD, 11 = PUBLIC_THREAD, 12 = PRIVATE_THREAD
    return channel_type in [10, 11, 12]
```

### Message Update Event Payload

When a message is edited or updated (if `MESSAGE_UPDATE_DIRECT` or `MESSAGE_UPDATE_GUILD` is enabled):

```
POST {HTTP_ENDPOINT}?handler=message_update
```

The request body contains the updated message data:

```json
{
  "message_update": {
    "id": "1234567890123456789",
    "channel_id": "9876543210987654321",
    "guild_id": "1111111111111111111",
    "content": "Updated content here",
    "edited_timestamp": "2024-01-15T12:35:00.789Z",
    "attachments": [],
    "embeds": []
  }
}
```

**Note:** The `guild_id` field is null for direct messages.

**Important limitations:**
- Discord only provides **changed fields** in the update event, along with always-present fields (`id`, `channel_id`, `guild_id`)
- If only content was edited, fields like `author` may not be included
- Message filtering (by sender type) is not available for update events
- Webhook response actions are not supported for update events
- To get complete message data, you need to cache messages when they're created

**Common updated fields:**
- `content` - Message text (if edited)
- `edited_timestamp` - Timestamp of the edit
- `embeds` - Embed objects (if added/removed/changed)
- `attachments` - Attachment objects (if added/removed)
- `pinned` - Whether message is pinned (if changed)

**Use cases:**
- Content moderation (track edited messages)
- Audit logging (who edited what and when)
- Spam detection (rapid edit patterns)

### Message Delete Event Payload

When a single message is deleted (if `MESSAGE_DELETE_DIRECT` or `MESSAGE_DELETE_GUILD` is enabled):

```
POST {HTTP_ENDPOINT}?handler=message_delete
```

The request body contains only IDs (no message content available):

```json
{
  "message_delete": {
    "id": "1234567890123456789",
    "channel_id": "9876543210987654321",
    "guild_id": "1111111111111111111"
  }
}
```

**Note:** The `guild_id` field is omitted for direct messages.

**Important limitations:**
- Discord only provides message IDs, not the content, author, or timestamp
- Deleted message content can only be obtained if you cached messages before deletion
- Message filtering (by sender type) is not available for delete events
- Webhook response actions are not supported for delete events

### Message Delete Bulk Event Payload

When multiple messages are deleted at once (if `MESSAGE_DELETE_BULK_GUILD` is enabled):

```
POST {HTTP_ENDPOINT}?handler=message_delete_bulk
```

The request body contains multiple message IDs:

```json
{
  "message_delete_bulk": {
    "ids": [
      "1234567890123456789",
      "2345678901234567890",
      "3456789012345678901"
    ],
    "channel_id": "9876543210987654321",
    "guild_id": "1111111111111111111"
  }
}
```

**Use cases:**
- Moderation logging (track when moderators bulk-delete messages)
- Compliance and audit trails
- Anti-spam detection (large bulk deletes may indicate spam cleanup)

**Note:** Bulk delete only occurs in guilds (not DMs) when using Discord's bulk delete API. The same limitations as single delete apply - no content available.

### Ready Event Payload

When the bot connects to Discord (if `READY` is enabled), a ready event is sent:

```
POST {HTTP_ENDPOINT}?handler=ready
```

The request body contains the ready data wrapped in a `ready` key:

```json
{
  "ready": {
    "v": 10,
    "user": {
      "id": "123456789012345678",
      "username": "MyBot",
      "discriminator": "0",
      "avatar": "...",
      "bot": true
    },
    "guilds": [
      {
        "id": "987654321098765432",
        "unavailable": false
      }
    ],
    "session_id": "...",
    "resume_gateway_url": "...",
    "shard": [0, 1],
    "application": {
      "id": "123456789012345678",
      "flags": 0
    }
    // ... other Discord Ready fields ...
  }
}
```

**Ready event fields** (from Discord's [Ready](https://discord.com/developers/docs/topics/gateway-events#ready) event):
- `v` - Gateway version
- `user` - Bot user information
- `guilds` - Guilds the bot is in (may be unavailable during initial connection)
- `session_id` - Session ID for resuming
- `shard` - Shard information (if sharding is used)
- `application` - Application information

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

Create a thread from the message that triggered the event (or send a message if already in a thread).

**Available in:** `message` handler in guild channels only (not DMs)

```json
{
  "type": "thread",
  "name": "Discussion Topic",
  "content": "Let's discuss this here!",
  "auto_archive_duration": 1440
}
```

**Parameters:**
- `name` (string, optional): Thread name (max 100 Unicode codepoints, auto-truncated if exceeded)
  - If omitted, auto-generated from first line of message
  - If message is empty, defaults to "Thread"
- `content` (string, required): Message content to send in the thread (max 2000 Unicode codepoints, auto-truncated if exceeded)
  - Use `<@user_id>` in content to mention users (e.g., `"<@123456789> Hello!"`)
- `auto_archive_duration` (integer, optional, default: `1440`): Minutes until thread auto-archives
  - Valid values: `60` (1 hour), `1440` (1 day), `4320` (3 days), `10080` (1 week)
  - Invalid values default to `1440` with warning log

**Behavior:**
- **Normal channel**: Creates a new thread from the message
- **Already in thread**: Skips thread creation, sends message to existing thread
- **DM channel**: Fails with error (threads not supported in DMs)

**Examples:**

```json
// Create thread with auto-generated name
{
  "type": "thread",
  "content": "Starting a discussion about this topic"
}

// Create thread with custom name and mention user
{
  "type": "thread",
  "name": "Bug Report #1234",
  "content": "<@123456789> Thanks for reporting! Let's track this here.",
  "auto_archive_duration": 10080
}

// Create thread with minimal config
{
  "type": "thread",
  "name": "Quick Discussion",
  "content": "Let's talk about this"
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
      "content": "Tracking progress here!"
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

## Development

See [CLAUDE.md](CLAUDE.md) for development guidelines and architecture details.

## References

- [Discord Developer Portal - Overview of Events](https://discord.com/developers/docs/events/overview)
- [serenity](https://github.com/serenity-rs/serenity)
