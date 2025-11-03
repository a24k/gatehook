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
│   ├── discord_service.rs           # Discord operations trait
│   ├── serenity_discord_service.rs  # Serenity implementation
│   ├── event_sender_trait.rs        # Event sending trait
│   ├── http_event_sender.rs         # HTTP implementation
│   └── mod.rs
└── bridge/                 # Business logic layer
    ├── event_bridge.rs     # Event processing logic
    ├── message_filter/     # Message filtering by sender type
    │   ├── mod.rs              # Public API re-exports
    │   ├── policy.rs           # MessageFilterPolicy (startup parsing)
    │   ├── filter.rs           # MessageFilter (runtime filtering)
    │   ├── filterable_message.rs # FilterableMessage trait
    │   └── tests.rs            # Shared test helpers (MockMessage)
    └── mod.rs

tests/
├── adapters/               # Mock implementations
│   ├── mock_discord.rs
│   ├── mock_event_sender.rs
│   └── mod.rs
└── event_bridge_test.rs    # Integration tests
```

## Architecture

The project follows a **layered architecture** with clear separation of concerns:

### Adapters Layer (`src/adapters/`)
External service abstractions and implementations:

- **`DiscordService` trait**: Abstracts Discord operations
  - `SerenityDiscordService`: Production implementation using serenity
  - `MockDiscordService` (tests): Records calls for verification

- **`EventSender` trait**: Abstracts event forwarding
  - `HttpEventSender`: Sends events to HTTP endpoints
  - `MockEventSender` (tests): Records sent events

### Bridge Layer (`src/bridge/`)
Business logic that orchestrates adapters:

- **`EventBridge`**: Coordinates Discord and HTTP operations
  - Generic over `DiscordService` and `EventSender` traits
  - Implements ping-pong logic and event forwarding
  - Fully testable with mocks

- **`MessageFilter` module**: Filters messages based on sender type (2-phase initialization)
  - **`MessageFilterPolicy`**: Parsed at startup from environment variables via serde
  - **`MessageFilter`**: Created in `ready` event with bot's user_id for runtime filtering
  - **`FilterableMessage`**: Trait abstraction for testing without serenity's Message type
  - **`MockMessage`**: Shared test helper for unit tests
  - Sender type classification (mutually exclusive categories): self, webhook, system, bot, user
  - Tests colocated with modules using rstest for parameterized testing

### Application Layer (`src/main.rs`)
Entry point that wires everything together:

- `Handler`: Thin adapter implementing serenity's `EventHandler`
- Passes `ctx.http` from Context to bridge (follows serenity's design)
- Stores `MessageFilter` instances in `OnceLock` for Direct/Guild contexts
- 2-phase initialization: Policy parsed at startup, Filter created in `ready` event
- Dynamically builds `GatewayIntents` based on enabled events
- Currently handles: `ready`, `message` events
- Applies `MessageFilter` based on message context (Direct/Guild)

## Key Modules

### `params.rs`
- `Params` struct: Configuration loaded from environment variables using serde
- Required: `DISCORD_TOKEN`, `HTTP_ENDPOINT`
- Optional: `INSECURE_MODE`, `RUST_LOG`
- Event configuration: `MESSAGE_DIRECT`, `MESSAGE_GUILD`, `READY` (all optional)
  - Parsed into `Option<MessageFilterPolicy>` using custom serde deserializer
- Helper methods: `has_direct_message_events()`, `has_guild_message_events()`

### `adapters/http_event_sender.rs`
- `HttpEventSender`: Sends events to HTTP endpoints
- Uses `url::Url` type for early URL validation
- Configurable TLS certificate validation (insecure mode for testing)

### `bridge/event_bridge.rs`
- `EventBridge`: Core business logic
- Generic design enables testing without external dependencies
- Receives `http` from Context (not stored as state)

### `bridge/message_filter/`
Modular message filtering with 2-phase initialization:

**`policy.rs` - MessageFilterPolicy**
- Parsed at startup from environment variables using `from_policy("user,bot")`
- Special values: `"all"` (everything), `""` (everything except self)
- Implements `Default` trait (safe default: allow all except self)
- Creates `MessageFilter` instances via `for_user(current_user_id)` method
- Tests: policy parsing, Default trait, for_user() method (using rstest)

**`filter.rs` - MessageFilter**
- Created in `ready` event with bot's `user_id` embedded
- Runtime filtering via `should_process(&message)` → bool
- Classification categories (mutually exclusive, priority order):
  1. `self` - Bot's own messages
  2. `webhook` - Webhook messages (excluding self)
  3. `system` - System messages (excluding self and webhooks)
  4. `bot` - Other bot messages (excluding self and webhooks)
  5. `user` - Human users (default/fallback)
- Tests: sender type filtering, priority rules (using rstest)

**`filterable_message.rs` - FilterableMessage**
- Trait abstraction for message properties needed for filtering
- Implemented by serenity's `Message` type
- Enables testing without constructing serenity's non-exhaustive Message struct

**`tests.rs` - Test Helpers**
- `MockMessage`: Shared test helper with builder pattern
- Used by tests in both `policy.rs` and `filter.rs`

## Development Workflow

### Before Committing

**Always run these checks in order:**

```bash
cargo check         # Verify compilation
cargo clippy        # Lint checks
cargo test          # Run all tests
```

All checks must pass before committing.

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
│   ├── mock_discord.rs      # MockDiscordService with RecordedReply
│   ├── mock_event_sender.rs # MockEventSender with SentEvent
│   └── mod.rs               # Public exports
└── event_bridge_test.rs     # EventBridge logic tests

src/bridge/message_filter/
├── policy.rs                # Contains #[cfg(test)] mod tests
├── filter.rs                # Contains #[cfg(test)] mod tests
└── tests.rs                 # Shared MockMessage helper
```

### Testing Best Practices
- Mock implementations record calls for verification
- Tests use dummy `Http` instances (not needed by mocks)
- Each test creates fresh mock instances (no shared state)
- Business logic fully testable without external services
- Unit tests colocated with modules they test
- Parameterized tests using rstest for reducing duplication

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
   - Apply `MessageFilter` if applicable
   - Call bridge method, passing `ctx.http`
6. Update `.env.example` with new environment variable
7. Update README.md "Available Events" table
8. Add tests in `tests/event_bridge_test.rs` using mocks

### Adding New Discord Operation
1. Add method to `DiscordService` trait in `src/adapters/discord_service.rs`
2. Implement in `SerenityDiscordService` (accepts `http` parameter)
3. Implement in `MockDiscordService` (record call for verification)
4. Use from `EventBridge` by passing `http` from Context

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
