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
    ├── message_filter.rs   # Message filtering by sender type
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

- **`MessageFilter`**: Filters messages based on sender type
  - MECE (Mutually Exclusive, Collectively Exhaustive) classification
  - Policy-based filtering from environment variable strings
  - Supports: self, webhook, system, bot, user

### Application Layer (`src/main.rs`)
Entry point that wires everything together:

- `Handler`: Thin adapter implementing serenity's `EventHandler`
- Passes `ctx.http` from Context to bridge (follows serenity's design)
- Stores `current_user_id` for message filtering
- Dynamically builds `GatewayIntents` based on enabled events
- Currently handles: `ready`, `message` events
- Applies `MessageFilter` based on message context (Direct/Guild)

## Key Modules

### `params.rs`
- `Params` struct: Configuration loaded from environment variables
- Required: `DISCORD_TOKEN`, `HTTP_ENDPOINT`
- Optional: `INSECURE_MODE`, `RUST_LOG`
- Event configuration: `MESSAGE_DIRECT`, `MESSAGE_GUILD`, `READY` (all optional)
- Helper methods: `has_direct_message_events()`, `has_guild_message_events()`

### `adapters/http_event_sender.rs`
- `HttpEventSender`: Sends events to HTTP endpoints
- Uses `url::Url` type for early URL validation
- Configurable TLS certificate validation (insecure mode for testing)

### `bridge/event_bridge.rs`
- `EventBridge`: Core business logic
- Generic design enables testing without external dependencies
- Receives `http` from Context (not stored as state)

### `bridge/message_filter.rs`
- `MessageFilter`: Implements MECE message filtering
- Policy parsing: `from_policy("user,bot")` → filter configuration
- Special values: `"all"` (everything), `""` (everything except self)
- Filter application: `should_process(&message, current_user_id)` → bool
- Classification priority: self → webhook → system → bot → user

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
```

### Testing Best Practices
- Mock implementations record calls for verification
- Tests use dummy `Http` instances (not needed by mocks)
- Each test creates fresh mock instances (no shared state)
- Business logic fully testable without external services

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
6. **MECE message classification**: Eliminates ambiguity in filtering decisions
7. **Context-aware filtering**: Separate policies for Direct Messages vs Guild messages
8. **Dynamic Gateway Intents**: Only request permissions for enabled events

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
