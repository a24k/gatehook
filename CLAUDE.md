# Claude Development Guide

This document provides essential information for AI assistants working on the gatehook project.

## Project Overview

**gatehook** bridges Discord Gateway (WebSocket) events to HTTP webhooks.

- **Language**: Rust (edition 2024)
- **Architecture**: Binary + Library
- **Main Dependencies**: serenity (Discord), reqwest (HTTP), tokio (async runtime)

## Project Structure

```
src/
├── main.rs                 # Entry point, Discord event handler
├── lib.rs                  # Library exports
├── params.rs               # Configuration (env vars)
├── adapters/               # External service adapters
│   ├── discord_service.rs                  # Discord operations trait
│   ├── serenity_discord_service.rs         # Serenity implementation
│   ├── channel_info_provider.rs            # Channel information retrieval trait
│   ├── serenity_channel_info_provider.rs   # Serenity implementation (cache-first)
│   ├── event_sender_trait.rs               # Event sending trait
│   ├── http_event_sender.rs                # HTTP implementation
│   ├── event_response.rs                   # Webhook response types (EventResponse, ResponseAction)
│   └── mod.rs
└── bridge/                 # Business logic layer
    ├── event_bridge.rs     # Event processing logic + action execution
    ├── message_payload.rs  # MessagePayload wrapper with GuildChannel metadata
    ├── ready_payload.rs    # ReadyPayload wrapper for ready events
    ├── discord_text.rs     # Discord text utilities (truncation, thread name generation)
    ├── reaction_payload.rs # ReactionPayload wrapper with GuildChannel metadata
    ├── action_target.rs    # ActionTarget abstraction for executing webhook actions
    ├── sender_filter/      # Event filtering by sender type (MESSAGE, REACTION_ADD)
    │   ├── mod.rs              # Public API re-exports
    │   ├── policy.rs           # SenderFilterPolicy (startup parsing)
    │   ├── message_filter.rs   # MessageFilter (runtime filtering for MESSAGE events)
    │   ├── reaction_filter.rs  # ReactionFilter (runtime filtering for REACTION_ADD events)
    │   ├── filterable_message.rs # FilterableMessage trait
    │   ├── filterable_reaction.rs # FilterableReaction trait
    │   └── tests.rs            # Shared test helpers (MockMessage, MockReaction)
    └── mod.rs

tests/
├── adapters/               # Mock implementations
│   ├── mock_discord_service.rs
│   ├── mock_event_sender.rs
│   ├── mock_channel_info.rs
│   └── mod.rs
└── event_bridge_test.rs    # Integration tests (includes action execution tests)
```

## Architecture

The project follows a **layered architecture** with clear separation of concerns:

### Adapters Layer (`src/adapters/`)
External service abstractions and implementations:

- **`DiscordService` trait**: Abstracts Discord write operations
  - Methods: `react_to_message`, `create_thread_from_message`, `send_message_to_channel`, `reply_in_channel`, `get_message`
  - `SerenityDiscordService`: Production implementation using serenity
    - Handles Discord API type conversions (e.g., u16 → AutoArchiveDuration)
  - `MockDiscordService` (tests): Records calls for verification

- **`ChannelInfoProvider` trait**: Abstracts Discord read operations (channel metadata)
  - Separation of concerns: Read operations vs write operations (DiscordService)
  - Methods: `is_thread_channel`
  - `SerenityChannelInfoProvider`: Production implementation with **cache-first optimization**
    - Searches cache via `cache.guilds().iter()` to find channel metadata
    - Falls back to Discord API (`http.get_channel()`) only on cache miss
    - Minimizes API rate limit impact when processing all messages
  - `MockChannelInfoProvider` (tests): Configurable responses via `set_is_thread()`

- **`EventSender` trait**: Abstracts event forwarding
  - Returns `Option<EventResponse>` containing webhook's response actions
  - `HttpEventSender`: Sends events to HTTP endpoints, parses JSON responses
  - `MockEventSender` (tests): Records sent events, can return pre-configured responses

- **`EventResponse` and `ResponseAction` types**: Webhook response structure
  - `EventResponse`: Container for action list from webhook
  - `ResponseAction` enum: Represents Discord operations
    - `Reply { content, mention }`: Reply to message with optional mention
    - `React { emoji }`: Add reaction (Unicode or custom emoji "name:id")
    - `Thread { name, content, auto_archive_duration }`: Create thread or send message to existing thread
  - Deserialized from webhook's JSON response using `#[serde(tag = "type")]`

### Bridge Layer (`src/bridge/`)
Business logic that orchestrates adapters:

- **`EventBridge`**: Coordinates Discord and HTTP operations
  - Generic over `DiscordService`, `EventSender`, and `ChannelInfoProvider` traits
  - Implements ping-pong logic and event forwarding
  - **Webhook payload enrichment**: Wraps Message with channel metadata via `MessagePayload`
    - `build_message_payload()`: Retrieves GuildChannel from cache
    - Cache-first approach: Extracts channel data without holding locks across await points
    - Sends enriched payload to webhook (Message + optional GuildChannel)
  - **Action execution**: Processes webhook response actions
    - `execute_actions()`: Iterates through actions, logs errors, continues on failure
    - `execute_reply()`: Handles reply action with 2000 char truncation
    - `execute_react()`: Handles reaction action (Unicode/custom emoji parsing)
    - `execute_thread()`: Creates threads with auto-naming, or sends message to existing thread
      - Auto-generates thread name from message if not specified
      - Detects if already in thread via `ChannelInfoProvider` (skips creation, sends message instead)
  - Fully testable with mocks

- **`MessagePayload`**: Wrapper for webhook payloads with channel metadata
  - Combines serenity's `Message` with optional `GuildChannel`
  - `with_channel()`: Constructor for guild messages with channel info
  - `new()`: Constructor for DMs or cache misses (no channel field)
  - JSON structure: `{ "message": {...}, "channel": {...} }`
  - `message` field contains all Discord Message fields
  - `channel` field omitted from JSON when None via `#[serde(skip_serializing_if)]`

- **`ReadyPayload`**: Wrapper for ready event webhook payloads
  - Wraps serenity's `Ready` event
  - `new()`: Constructor that takes ready event reference
  - JSON structure: `{ "ready": {...} }`
  - Contains bot connection information (user, guilds, session_id, etc.)

- **`ReactionPayload`**: Wrapper for webhook payloads with channel metadata
  - Combines serenity's `Reaction` with optional `GuildChannel`
  - `with_channel()`: Constructor for guild reactions with channel info
  - `new()`: Constructor for DMs or cache misses (no channel field)
  - JSON structure: `{ "reaction": {...}, "channel": {...} }`
  - `reaction` field contains all Discord Reaction fields
  - `channel` field omitted from JSON when None via `#[serde(skip_serializing_if)]`

- **`ActionTarget`**: Abstraction for webhook response action execution
  - Represents minimal information needed to execute Discord actions (message_id, channel_id, guild_id)
  - Enables different event types (Message, Reaction, etc.) to be used as action targets
  - `From<&Message>` and `From<&Reaction>` implementations for easy conversion
  - Provides guild_id for performance optimization (O(1) cache lookups) and future guild-specific actions

- **`sender_filter` module**: Filters events based on sender type (2-phase initialization)
  - **`SenderFilterPolicy`**: Parsed at startup from environment variables via serde
    - Shared policy for both MESSAGE and REACTION_ADD events
    - Creates `MessageFilter` via `for_user()` method
    - Creates `ReactionFilter` via `for_reaction()` method
  - **`MessageFilter`**: Created in `ready` event with bot's user_id for runtime filtering
    - Sender type classification (mutually exclusive): self, webhook, system, bot, user
  - **`ReactionFilter`**: Created in `ready` event with bot's user_id for runtime filtering
    - Sender type classification (mutually exclusive): self, bot, user
    - Excludes webhook/system types (MESSAGE-only concepts)
  - **`FilterableMessage`**: Trait abstraction for testing without serenity's Message type
  - **`FilterableReaction`**: Trait abstraction for testing without serenity's Reaction type
  - **`MockMessage`** and **`MockReaction`**: Shared test helpers for unit tests
  - Tests colocated with modules using rstest for parameterized testing

### Application Layer (`src/main.rs`)
Entry point that wires everything together:

- `Handler`: Thin adapter implementing serenity's `EventHandler`
- Passes `ctx.http` from Context to bridge (follows serenity's design)
- Stores `MessageFilter` and `ReactionFilter` instances in `OnceLock` for Direct/Guild contexts
- 2-phase initialization: Policy parsed at startup, Filters created in `ready` event
- Dynamically builds `GatewayIntents` based on enabled events
- Currently handles: `ready`, `message`, `message_delete`, `message_delete_bulk`, `message_update`, `reaction_add` events
- Applies `MessageFilter` based on message context (Direct/Guild)
- Applies `ReactionFilter` based on reaction context (Direct/Guild)
- **Webhook action flow**: `handle_message`/`handle_reaction_add` → webhook response → `execute_actions`

## Key Modules

### `params.rs`
- `Params` struct: Configuration loaded from environment variables using serde
- Required: `DISCORD_TOKEN`, `HTTP_ENDPOINT`
- Optional: `INSECURE_MODE`, `RUST_LOG`
- Event configuration (all optional):
  - MESSAGE events: `MESSAGE_DIRECT`, `MESSAGE_GUILD` (parsed into `Option<SenderFilterPolicy>`)
  - MESSAGE_DELETE events: `MESSAGE_DELETE_DIRECT`, `MESSAGE_DELETE_GUILD`, `MESSAGE_DELETE_BULK_GUILD`
  - MESSAGE_UPDATE events: `MESSAGE_UPDATE_DIRECT`, `MESSAGE_UPDATE_GUILD`
  - REACTION_ADD events: `REACTION_ADD_DIRECT`, `REACTION_ADD_GUILD` (parsed into `Option<SenderFilterPolicy>`)
  - Context-independent: `READY`
- Custom serde deserializer: `deserialize_sender_filter_policy`
- Helper methods: `has_direct_message_events()`, `has_guild_message_events()`, `has_direct_reaction_add_events()`, `has_guild_reaction_add_events()`, etc.

### `adapters/http_event_sender.rs`
- `HttpEventSender`: Sends events to HTTP endpoints and parses responses
- Uses `url::Url` type for early URL validation
- Configurable TLS certificate validation (insecure mode for testing)
- **Response handling**: Parses `EventResponse` from JSON, handles non-2xx status codes gracefully

### `adapters/event_response.rs`
- `EventResponse`: Webhook response container with `actions: Vec<ResponseAction>`
- `ResponseAction` enum: Tagged union of Discord operations
  - `Reply { content, mention }`: Reply to message with optional mention
  - `React { emoji }`: Add reaction (Unicode or custom emoji "name:id")
  - `Thread { name, content, auto_archive_duration }`: Create thread or send message to existing thread
    - auto_archive_duration: 60, 1440, 4320, 10080 (minutes)
- Uses serde with `#[serde(tag = "type")]` for type-safe deserialization
- Comprehensive tests with rstest for all action types and edge cases

### `adapters/channel_info_provider.rs`
- `ChannelInfoProvider` trait: Abstracts channel metadata retrieval operations
- Separates read operations from write operations (DiscordService)
- Method signature: `async fn is_thread_channel(&self, cache: &Cache, http: &Http, channel_id: ChannelId) -> Result<bool, Error>`
- Accepts both cache and http for cache-first optimization pattern
- Enables testing without Discord API access via mock implementations

### `adapters/serenity_channel_info_provider.rs`
- Production implementation of `ChannelInfoProvider`
- **Cache-first optimization**: Searches all guilds in cache before API call
- Implementation details:
  - Iterates `cache.guilds()` to find channel across all cached guilds
  - Extracts channel data without holding locks (avoids Send trait issues)
  - Falls back to `http.get_channel()` only on cache miss
  - Logs cache hits and misses for observability
- Thread detection: Checks for `PublicThread`, `PrivateThread`, `NewsThread`

### `bridge/message_payload.rs`
- `MessagePayload<'a>`: Wrapper struct for webhook payloads
- Fields:
  - `message: &'a Message` - Discord Message wrapped in "message" key
  - `channel: Option<GuildChannel>` - Optional channel metadata, omitted when None
- JSON structure: `{ "message": {...}, "channel": {...} }`
- Constructors:
  - `new(message)` - For DMs or cache misses (no channel info)
  - `with_channel(message, channel)` - For guild messages with channel metadata
- Serde attributes:
  - `#[serde(skip_serializing_if = "Option::is_none")]` on channel: Clean JSON output

### `bridge/ready_payload.rs`
- `ReadyPayload<'a>`: Wrapper struct for ready event webhook payloads
- Fields:
  - `ready: &'a Ready` - Discord Ready event wrapped in "ready" key
- JSON structure: `{ "ready": {...} }`
- Constructor:
  - `new(ready)` - Wraps ready event for webhook delivery
- Contains bot connection info: user, guilds, session_id, shard info, etc.

### `bridge/event_bridge.rs`
- `EventBridge`: Core business logic
- Generic design enables testing without external dependencies
- Receives `http` from Context (not stored as state)
- **Action execution**:
  - Sequential processing of actions (preserves order)
  - Error isolation (one failure doesn't stop others)
  - `execute_reply()`: Reply with content truncation (2000 chars)
  - `execute_react()`: Add reactions (Unicode/custom emoji)
  - `execute_thread()`: Create threads or send message to existing thread
    - Auto-generates thread name from message if not specified
    - Detects if already in thread (skips creation, sends message instead)
    - Handles error 160004 (thread already exists): Retrieves message, finds existing thread, posts to it

### `bridge/sender_filter/`
Modular event filtering by sender type with 2-phase initialization:

**`policy.rs` - SenderFilterPolicy**
- Parsed at startup from environment variables using `from_policy("user,bot")`
- Special values: `"all"` (everything), `""` (everything except self)
- Implements `Default` trait (safe default: allow all except self)
- Shared policy for both MESSAGE and REACTION_ADD events
- Creates `MessageFilter` instances via `for_user(current_user_id)` method
- Creates `ReactionFilter` instances via `for_reaction(current_user_id)` method
- Tests: policy parsing, Default trait, for_user() and for_reaction() methods (using rstest)

**`message_filter.rs` - MessageFilter**
- Created in `ready` event with bot's `user_id` embedded
- Runtime filtering for MESSAGE events via `should_process(&message)` → bool
- Classification categories (mutually exclusive, priority order):
  1. `self` - Bot's own messages
  2. `webhook` - Webhook messages (excluding self)
  3. `system` - System messages (excluding self and webhooks)
  4. `bot` - Other bot messages (excluding self and webhooks)
  5. `user` - Human users (default/fallback)
- Tests: sender type filtering, priority rules (using rstest)

**`reaction_filter.rs` - ReactionFilter**
- Created in `ready` event with bot's `user_id` embedded
- Runtime filtering for REACTION_ADD events via `should_process(&reaction)` → bool
- Classification categories (mutually exclusive, priority order):
  1. `self` - Bot's own reactions
  2. `bot` - Other bot reactions (excluding self)
  3. `user` - Human users (default/fallback)
- Note: Webhook and system types excluded (MESSAGE-only concepts)
- Tests: sender type filtering, priority rules (using rstest)

**`filterable_message.rs` - FilterableMessage**
- Trait abstraction for message properties needed for filtering
- Implemented by serenity's `Message` type
- Enables testing without constructing serenity's non-exhaustive Message struct

**`filterable_reaction.rs` - FilterableReaction**
- Trait abstraction for reaction properties needed for filtering
- Implemented by serenity's `Reaction` type
- Enables testing without constructing serenity's non-exhaustive Reaction struct
- Methods: `user_id()`, `is_bot()`

**`tests.rs` - Test Helpers**
- `MockMessage`: Shared test helper with builder pattern
- `MockReaction`: Shared test helper with builder pattern
- Used by tests in policy.rs, message_filter.rs, and reaction_filter.rs

### `bridge/discord_text.rs`
Discord text processing utilities for API length limitations:

- `truncate_content(content: &str) -> String`: Truncates to 2000 chars (Discord message limit)
  - Adds "..." suffix when truncated
  - Counts Unicode characters (not bytes) for multibyte safety
  - Logs warning with original and truncated lengths

- `generate_thread_name(message: &Message) -> String`: Auto-generates thread name from message
  - Uses first line of message content (trimmed)
  - Falls back to "Thread" if content is empty
  - Truncates to 100 chars maximum

- `truncate_thread_name(name: &str) -> String`: Truncates to 100 chars (Discord thread name limit)
  - Counts Unicode characters (not bytes)
  - No suffix added (preserves user input)

- Comprehensive tests: 18 unit tests covering edge cases, Unicode handling, boundary conditions

## Development Workflow

### Before Committing

**Always run these checks in order:**

```bash
cargo check                  # Verify compilation
cargo clippy --all-targets   # Lint checks (lib, bin, tests, examples)
cargo test                   # Run all tests
```

All checks must pass before committing.

**Note**: `--all-targets` ensures clippy checks all code including tests and examples, not just the library and binary.

### Code Style

- Follow Rust standard conventions
- Use `tracing` for logging (not `println!`)
- Keep functions focused and testable
- Add tests for new functionality

## Testing Strategy

### Current Approach
- **Trait-based mocking**: `DiscordService` and `EventSender` traits enable clean mocks
- **Unit tests**: In `src/adapters/` modules (`#[cfg(test)]`)
- **Integration tests**: In `tests/` directory, testing `EventBridge` with mocks
- **Mock implementations**: Located in `tests/adapters/` for reusability

### Test Organization
```
tests/
├── adapters/
│   ├── mock_discord_service.rs # MockDiscordService with RecordedReply/RecordedReaction/RecordedThread
│   ├── mock_event_sender.rs    # MockEventSender with SentEvent
│   ├── mock_channel_info.rs    # MockChannelInfoProvider with configurable responses
│   └── mod.rs                  # Public exports
└── event_bridge_test.rs        # EventBridge logic tests (Reply/React/Thread actions)

src/adapters/event_response.rs  # Contains #[cfg(test)] mod tests (18 tests)
src/bridge/discord_text.rs      # Contains #[cfg(test)] mod tests (18 tests)
src/bridge/sender_filter/
├── policy.rs                # Contains #[cfg(test)] mod tests
├── message_filter.rs        # Contains #[cfg(test)] mod tests
├── reaction_filter.rs       # Contains #[cfg(test)] mod tests
└── tests.rs                 # Shared MockMessage and MockReaction helpers
```

### Testing Best Practices
- Mock implementations record calls for verification
- Tests use dummy `Http` instances (not needed by mocks)
- Each test creates fresh mock instances (no shared state)
- Business logic fully testable without external services
- Unit tests colocated with modules they test
- **Use rstest for parameterized tests**: When multiple test cases share identical logic but differ only in inputs/outputs, use rstest to reduce duplication (e.g., `message_filter` tests, `event_response` tests)

## Architecture Principles

### Design Patterns
- **Hexagonal Architecture**: Clear separation between adapters and business logic
- **Dependency Injection**: Traits injected via generics
- **Zero-Cost Abstractions**: Trait objects only where needed (Arc<dyn Trait>)

### Key Decisions
1. **Traits over concrete types**: Enables testing and future flexibility
2. **Context-based Http passing**: Follows serenity's design philosophy
3. **Unit structs for stateless services**: `SerenityDiscordService` has no fields
4. **Type-safe URLs**: `url::Url` provides early validation
5. **Environment variable-based event control**: Handlers only registered when configured
6. **2-phase filter initialization**: Policy at startup, Filter at ready event
7. **Serde custom deserializers**: Parse filters at config load time
8. **Sender type classification**: Mutually exclusive categories eliminate ambiguity
9. **Context-aware filtering**: Separate policies for Direct Messages vs Guild messages
10. **Dynamic Gateway Intents**: Only request permissions for enabled events
11. **Trait abstraction for testing**: FilterableMessage enables testing without serenity Message
12. **Colocated tests**: Unit tests in same file as implementation using `#[cfg(test)]`
13. **Adapter layer type conversion**: Discord API-specific types (e.g., AutoArchiveDuration) converted in adapter layer, not business logic
14. **Utility module extraction**: Discord API constraints (text limits) separated into dedicated module (discord_text)
15. **Separation of read/write operations**: ChannelInfoProvider (reads) separate from DiscordService (writes) for clear API boundaries
16. **Cache-first optimization**: Minimize Discord API calls by checking cache before HTTP requests (rate limit management)
17. **GUILDS intent auto-enabled**: When MESSAGE_GUILD is enabled, GUILDS intent automatically added to populate cache with channel metadata
18. **Enriched webhook payloads**: Send complete GuildChannel metadata to webhooks instead of just boolean flags (more flexible for webhook consumers)
19. **Lock-free cache extraction**: Extract data from cache before await points to avoid Send trait issues with Arc<RwLock<>>
20. **Optional payload fields**: Use `#[serde(skip_serializing_if)]` to keep webhook payloads clean (omit None values)
21. **Shared SenderFilterPolicy**: Single policy abstraction for all sender-based filtering (MESSAGE, REACTION_ADD) reduces code duplication
22. **Event-specific filter types**: MessageFilter and ReactionFilter tailor sender classification to event context (e.g., reactions exclude webhook/system types)
23. **ActionTarget abstraction**: Unified target for webhook response actions enables different event types (Message, Reaction) to support same action types
24. **Trait abstraction for event types**: FilterableReaction trait mirrors FilterableMessage pattern for consistent testing approach

### Future Growth Path
When complexity increases:
1. ✅ ~~Add trait abstraction~~ (Done)
2. Add retry logic and circuit breakers for HTTP calls
3. Implement more sophisticated error handling strategies
4. Consider event middleware/pipeline pattern for transformations

## Common Tasks

### Adding New Event Handler
1. Add environment variable field to `Params` struct in `params.rs`
   - For context-specific events: `<event>_direct` and `<event>_guild` fields
   - For context-independent events: single `<event>` field
   - Use `#[serde(default)]` to make them optional
2. Update helper methods in `Params` if needed (e.g., `has_direct_*_events()`)
3. Update `build_gateway_intents()` in `main.rs` to include necessary intents
4. Add event handler method to `EventBridge` in `src/bridge/event_bridge.rs`
5. Implement handler in `Handler` trait in `main.rs`:
   - Check if event is enabled via `params.<event>_direct` / `params.<event>_guild`
   - Apply `MessageFilter` or `ReactionFilter` if applicable (for events with sender filtering)
   - Call bridge method
   - Execute webhook response actions if supported for this event type
6. Update `.env.example` with new environment variable
7. Update README.md "Available Events" table
8. Add tests in `tests/event_bridge_test.rs` using mocks

### Adding New Discord Operation

**For write operations (creating, updating, deleting):**
1. Add method to `DiscordService` trait in `src/adapters/discord_service.rs`
2. Implement in `SerenityDiscordService` (accepts `http` parameter)
3. Implement in `MockDiscordService` (record call for verification)
4. Use from `EventBridge` by passing `http` from Context

**For read operations (querying metadata, checking state):**
1. Add method to `ChannelInfoProvider` trait in `src/adapters/channel_info_provider.rs`
2. Implement in `SerenityChannelInfoProvider` (accepts both `cache` and `http`)
   - Use cache-first approach: check cache, fallback to API on miss
   - Extract data from cache before await points (avoid Send trait issues)
3. Implement in `MockChannelInfoProvider` (return configurable responses)
4. Use from `EventBridge` by passing both `cache` and `http` from Context

### Adding Configuration
1. Add field to `Params` struct in `params.rs`
2. Use `#[serde(default)]` for optional values
3. Update README.md environment variables table
4. Update `.env.example` with new variable

### Refactoring Guidelines
- Keep changes incremental
- Maintain backward compatibility when possible
- Add tests before refactoring (if missing)
- Run `cargo clippy` and `cargo test` after changes

## Version Management

- Follow Semantic Versioning (SemVer)
- Update version in `Cargo.toml` for releases
- `Cargo.lock` updates automatically on build/test

## Additional Resources

- [Discord Developer Docs](https://discord.com/developers/docs)
- [Serenity Documentation](https://docs.rs/serenity/)
- [Tokio Documentation](https://docs.rs/tokio/)
