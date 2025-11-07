// Mock implementations for adapter layer testing

pub mod mock_channel_info;
pub mod mock_discord;
pub mod mock_event_sender;

pub use mock_channel_info::MockChannelInfoProvider;
pub use mock_discord::MockDiscordService;
pub use mock_event_sender::MockEventSender;
