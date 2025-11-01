// Mock implementations for adapter layer testing

pub mod mock_discord;
pub mod mock_event_sender;

pub use mock_discord::MockDiscordService;
pub use mock_event_sender::MockEventSender;
