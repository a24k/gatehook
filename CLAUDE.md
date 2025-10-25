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
├── main.rs         # Entry point, Discord event handler
├── lib.rs          # Library exports
├── params.rs       # Configuration (env vars)
└── webhook.rs      # WebhookClient for HTTP forwarding

tests/
└── webhook_integration_test.rs
```

## Key Modules

### `params.rs`
- `Params` struct: Configuration loaded from environment variables
- Required: `DISCORD_TOKEN`, `WEBHOOK_URL`
- Optional: `INSECURE_MODE`, `RUST_LOG`

### `webhook.rs`
- `WebhookClient`: Encapsulates HTTP client and webhook URL
- `send()`: Low-level webhook sending
- `send_with_logging()`: Convenience method with structured logging

### `main.rs`
- `Handler`: Implements serenity's `EventHandler` trait
- Currently handles: `ready`, `message`, `reaction_add` events
- Uses `WebhookClient` for forwarding events

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
- Unit tests in module files (`#[cfg(test)]`)
- Integration tests in `tests/` directory
- Mock external services when possible

### Future Considerations
- Add mock HTTP server for webhook tests (e.g., wiremock)
- Test Discord event handling with mock serenity context

## Architecture Principles

### Current (Minimal)
- Simple module separation
- `WebhookClient` abstracts HTTP concerns
- Tests verify basic functionality

### Future Growth Path
When complexity increases:
1. Add trait abstraction for `WebhookClient` (for mocking)
2. Separate handlers into `handlers/` module
3. Implement retry logic and error handling strategies
4. Consider middleware pattern for event processing

## Common Tasks

### Adding New Event Handler
1. Update `GatewayIntents` in `main.rs` if needed
2. Implement handler method in `EventHandler` trait
3. Use `WebhookClient::send_with_logging()` to forward
4. Add tests

### Adding Configuration
1. Add field to `Params` struct in `params.rs`
2. Use `#[serde(default)]` for optional values
3. Update README.md environment variables table

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
