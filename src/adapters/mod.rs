// Trait definitions
pub mod discord_service;
pub mod event_sender_trait;

// Type definitions
pub mod event_response;

// Implementations
pub mod http_event_sender;
pub mod serenity_discord_service;

// Re-exports for convenience
pub use discord_service::DiscordService;
pub use event_response::{EventResponse, ReactParams, ReplyParams, ResponseAction, ThreadParams};
pub use event_sender_trait::EventSender;
pub use http_event_sender::HttpEventSender;
pub use serenity_discord_service::SerenityDiscordService;
