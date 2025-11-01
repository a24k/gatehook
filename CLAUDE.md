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

### Application Layer (`src/main.rs`)
Entry point that wires everything together:

- `Handler`: Thin adapter implementing serenity's `EventHandler`
- Passes `ctx.http` from Context to bridge (follows serenity's design)
- Currently handles: `ready`, `message`, `reaction_add` events

## Key Modules

### `params.rs`
- `Params` struct: Configuration loaded from environment variables
- Required: `DISCORD_TOKEN`, `HTTP_ENDPOINT`
- Optional: `INSECURE_MODE`, `RUST_LOG`

### `adapters/http_event_sender.rs`
- `HttpEventSender`: Sends events to HTTP endpoints
- Uses `url::Url` type for early URL validation
- Configurable TLS certificate validation (insecure mode for testing)

### `bridge/event_bridge.rs`
- `EventBridge`: Core business logic
- Generic design enables testing without external dependencies
- Receives `http` from Context (not stored as state)

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

### Future Growth Path
When complexity increases:
1. ✅ ~~Add trait abstraction~~ (Done)
2. Add retry logic and circuit breakers for HTTP calls
3. Implement more sophisticated error handling strategies
4. Consider event middleware/pipeline pattern for transformations

## Common Tasks

### Adding New Event Handler
1. Update `GatewayIntents` in `main.rs` if needed
2. Add method to `EventBridge` in `src/bridge/event_bridge.rs`
3. Call bridge method from `Handler` in `main.rs`, passing `ctx.http`
4. Use `event_sender.send()` to forward events
5. Add tests in `tests/event_bridge_test.rs` using mocks

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
