use crate::adapters::{
    DiscordService, EventResponse, EventSender, ReactParams, ReplyParams, ResponseAction,
    ThreadParams,
};
use anyhow::Context as _;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Bridge Discord Gateway events to external endpoints
pub struct EventBridge<D, S>
where
    D: DiscordService,
    S: EventSender,
{
    discord_service: Arc<D>,
    event_sender: Arc<S>,
}

impl<D, S> EventBridge<D, S>
where
    D: DiscordService,
    S: EventSender,
{
    /// Create a new EventBridge
    ///
    /// # Arguments
    ///
    /// * `discord_service` - The Discord service for operations
    /// * `event_sender` - The event sender for forwarding events
    pub fn new(discord_service: Arc<D>, event_sender: Arc<S>) -> Self {
        Self {
            discord_service,
            event_sender,
        }
    }

    /// Handle a message event
    ///
    /// Sends event to webhook and returns the response.
    ///
    /// # Arguments
    ///
    /// * `message` - The message event from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (may contain actions)
    pub async fn handle_message(
        &self,
        message: &Message,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            message_id = %message.id,
            author = %message.author.name,
            content = %message.content,
            "Processing message event"
        );

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("message", message)
            .await
            .context("Failed to send message event to HTTP endpoint")
    }

    /// Handle a ready event
    ///
    /// # Arguments
    ///
    /// * `ready` - The ready event from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (may contain actions)
    pub async fn handle_ready(&self, ready: &Ready) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            user = %ready.user.display_name(),
            "Processing ready event"
        );

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("ready", ready)
            .await
            .context("Failed to send ready event to HTTP endpoint")
    }

    /// Execute actions from webhook response
    ///
    /// # Arguments
    ///
    /// * `http` - The HTTP client for Discord API calls
    /// * `message` - The message that triggered the event (for context)
    /// * `event_response` - The response from webhook containing actions
    pub async fn execute_actions(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        event_response: &EventResponse,
    ) -> anyhow::Result<()> {
        for action in &event_response.actions {
            // Execute action (log error and continue with next)
            if let Err(err) = self.execute_action(http, message, action).await {
                error!(?err, ?action, "Failed to execute action, continuing with next");
            }
        }
        Ok(())
    }

    /// Execute a single action
    async fn execute_action(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        action: &ResponseAction,
    ) -> anyhow::Result<()> {
        match action {
            ResponseAction::Reply(params) => self.execute_reply(http, message, params).await,
            ResponseAction::React(params) => self.execute_react(http, message, params).await,
            ResponseAction::Thread(params) => self.execute_thread(http, message, params).await,
        }
    }

    /// Execute Reply action
    ///
    /// # Content Handling
    /// - Content exceeding 2000 characters is truncated with warning log
    ///
    /// # Mention
    /// - `params.mention = true`: Reply with ping (user receives notification)
    /// - `params.mention = false`: Reply without ping (default)
    async fn execute_reply(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        params: &ReplyParams,
    ) -> anyhow::Result<()> {
        let content = truncate_content(&params.content);

        self.discord_service
            .reply_in_channel(http, message.channel_id, message.id, &content, params.mention)
            .await
            .context("Failed to send reply to Discord")?;

        info!(
            message_id = %message.id,
            mention = params.mention,
            content_len = content.chars().count(),
            "Successfully executed reply action"
        );

        Ok(())
    }

    /// Execute React action
    ///
    /// # Emoji Format
    /// - Unicode emoji: "👍", "🎉", etc.
    /// - Custom emoji: "name:id" format (e.g., "customemoji:123456789")
    async fn execute_react(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        params: &ReactParams,
    ) -> anyhow::Result<()> {
        self.discord_service
            .react_to_message(http, message.channel_id, message.id, &params.emoji)
            .await
            .context("Failed to add reaction to Discord")?;

        info!(
            message_id = %message.id,
            emoji = %params.emoji,
            "Successfully executed react action"
        );

        Ok(())
    }

    /// Execute Thread action
    ///
    /// # Thread Name
    /// - `params.name = Some(...)`: Use specified name
    /// - `params.name = None`: Auto-generate from first line of message (max 100 chars)
    ///   - Falls back to "Thread" if message content is empty
    /// - Name is ignored if already in a thread
    ///
    /// # Content Handling
    /// - Content exceeding 2000 characters is truncated with warning log
    ///
    /// # Auto-archive Duration
    /// - Valid values: 60, 1440, 4320, 10080 (minutes)
    /// - Invalid values fall back to 1440 (OneDay) with warning log
    async fn execute_thread(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        params: &ThreadParams,
    ) -> anyhow::Result<()> {
        // Ensure we're in a guild (threads not supported in DM)
        message.guild_id
            .context("Thread action is not supported in DM")?;

        // Check if already in thread (API call, no cache)
        let is_in_thread = self.discord_service
            .is_thread_channel(http, message.channel_id)
            .await
            .context("Failed to check if channel is thread")?;

        // Determine target channel ID
        let target_channel_id = if is_in_thread {
            // Already in thread → use as-is
            info!("Message is already in thread, skipping thread creation");
            message.channel_id
        } else {
            // Normal channel → create new thread
            let thread_name = match &params.name {
                Some(name) => truncate_thread_name(name),
                None => generate_thread_name(message),
            };

            // Convert auto_archive_duration to enum
            use serenity::model::channel::AutoArchiveDuration;
            let auto_archive_duration = match params.auto_archive_duration {
                60 => AutoArchiveDuration::OneHour,
                1440 => AutoArchiveDuration::OneDay,
                4320 => AutoArchiveDuration::ThreeDays,
                10080 => AutoArchiveDuration::OneWeek,
                invalid => {
                    warn!(
                        invalid_value = invalid,
                        "Invalid auto_archive_duration, using default (1440 = OneDay)"
                    );
                    AutoArchiveDuration::OneDay
                }
            };

            let thread = self
                .discord_service
                .create_thread_from_message(
                    http,
                    message,
                    &thread_name,
                    auto_archive_duration,
                )
                .await
                .context("Failed to create thread")?;

            info!(
                thread_id = %thread.id,
                thread_name = %thread_name,
                "Created new thread"
            );
            thread.id
        };

        // Truncate content
        let content = truncate_content(&params.content);

        // Post message
        if params.reply {
            self.discord_service
                .reply_in_channel(http, target_channel_id, message.id, &content, params.mention)
                .await
                .context("Failed to send reply in thread")?;

            info!(
                channel_id = %target_channel_id,
                message_id = %message.id,
                reply = true,
                mention = params.mention,
                "Successfully executed thread action with reply"
            );
        } else {
            self.discord_service
                .send_message_to_channel(http, target_channel_id, &content)
                .await
                .context("Failed to send message to thread")?;

            info!(
                channel_id = %target_channel_id,
                reply = false,
                "Successfully executed thread action"
            );
        }

        Ok(())
    }
}

/// Truncate content to Discord's 2000 character limit
///
/// If content exceeds limit, truncates to 1997 chars and appends "..."
/// Logs warning with original and truncated length.
fn truncate_content(content: &str) -> String {
    const MAX_LEN: usize = 2000;

    let char_count = content.chars().count();

    if char_count > MAX_LEN {
        let truncated: String = content.chars().take(MAX_LEN - 3).collect();
        let result = format!("{}...", truncated);

        warn!(
            original_len = char_count,
            truncated_len = result.chars().count(),
            "Content exceeds 2000 chars, truncated"
        );

        result
    } else {
        content.to_string()
    }
}

/// Generate thread name from message content
///
/// Uses first line of message content (max 100 chars, Discord API limit).
/// Returns "Thread" if content is empty after trimming.
fn generate_thread_name(message: &Message) -> String {
    // Use first line only, trim whitespace
    let content = message
        .content
        .lines()
        .next()
        .unwrap_or("")
        .trim();

    if content.is_empty() {
        return "Thread".to_string();
    }

    truncate_thread_name(content)
}

/// Truncate thread name to Discord's 100 character limit
///
/// If name exceeds limit, truncates to 100 chars.
fn truncate_thread_name(name: &str) -> String {
    const MAX_LEN: usize = 100; // Discord API maximum

    let char_count = name.chars().count();

    if char_count <= MAX_LEN {
        name.to_string()
    } else {
        // Truncate to API limit
        name.chars().take(MAX_LEN).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serenity::model::channel::Message;
    use serenity::model::id::{ChannelId, MessageId};
    use serenity::model::user::User;

    // Helper to create a test message with specific content
    fn create_message(content: &str) -> Message {
        let mut message = Message::default();
        message.id = MessageId::new(1);
        message.channel_id = ChannelId::new(1);
        message.content = content.to_string();
        message.author = User::default();
        message
    }

    // Tests for truncate_content

    #[rstest]
    #[case("", "")]                           // Empty string
    #[case("Hello", "Hello")]                 // Short string
    fn test_truncate_content_no_truncation(#[case] input: &str, #[case] expected: &str) {
        let result = truncate_content(input);
        assert_eq!(result, expected);
        assert_eq!(result.chars().count(), expected.chars().count());
    }

    #[test]
    fn test_truncate_content_exactly_2000_chars() {
        let content = "a".repeat(2000);
        let result = truncate_content(&content);

        assert_eq!(result, content);
        assert_eq!(result.chars().count(), 2000);
    }

    #[test]
    fn test_truncate_content_truncates_long_content() {
        let long_content = "a".repeat(2100);
        let result = truncate_content(&long_content);

        assert_eq!(result.chars().count(), 2000);
        assert!(result.ends_with("..."));
        assert_eq!(&result[..result.len() - 3], &"a".repeat(1997));
    }

    #[test]
    fn test_truncate_content_handles_multibyte_chars() {
        // 2001 characters with emoji (multibyte)
        let content = format!("{}{}", "あ".repeat(1999), "🎉🎉");
        let result = truncate_content(&content);

        assert_eq!(result.chars().count(), 2000);
        assert!(result.ends_with("..."));
    }

    // Tests for truncate_thread_name

    #[rstest]
    #[case("", "")]                           // Empty string
    #[case("Thread", "Thread")]               // Short name
    fn test_truncate_thread_name_no_truncation(#[case] input: &str, #[case] expected: &str) {
        let result = truncate_thread_name(input);
        assert_eq!(result, expected);
        assert_eq!(result.chars().count(), expected.chars().count());
    }

    #[test]
    fn test_truncate_thread_name_exactly_100_chars() {
        let name = "a".repeat(100);
        let result = truncate_thread_name(&name);

        assert_eq!(result, name);
        assert_eq!(result.chars().count(), 100);
    }

    #[test]
    fn test_truncate_thread_name_truncates_long_name() {
        let long_name = "a".repeat(150);
        let result = truncate_thread_name(&long_name);

        assert_eq!(result.chars().count(), 100);
        assert_eq!(result, "a".repeat(100));
    }

    #[test]
    fn test_truncate_thread_name_handles_multibyte_chars() {
        // 120 characters with emoji
        let name = format!("{}{}", "あ".repeat(100), "🎉".repeat(20));
        let result = truncate_thread_name(&name);

        assert_eq!(result.chars().count(), 100);
    }

    // Tests for generate_thread_name

    #[test]
    fn test_generate_thread_name_from_content() {
        let message = create_message("This is a test message");
        let result = generate_thread_name(&message);

        assert_eq!(result, "This is a test message");
    }

    #[test]
    fn test_generate_thread_name_empty_message() {
        let message = create_message("");
        let result = generate_thread_name(&message);

        assert_eq!(result, "Thread");
    }

    #[test]
    fn test_generate_thread_name_whitespace_only() {
        let message = create_message("   \t\n   ");
        let result = generate_thread_name(&message);

        assert_eq!(result, "Thread");
    }

    #[test]
    fn test_generate_thread_name_trims_whitespace() {
        let message = create_message("  Hello World  ");
        let result = generate_thread_name(&message);

        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_generate_thread_name_first_line_only() {
        let message = create_message("First line\nSecond line\nThird line");
        let result = generate_thread_name(&message);

        assert_eq!(result, "First line");
    }

    #[test]
    fn test_generate_thread_name_truncates_long_line() {
        let long_line = "a".repeat(150);
        let message = create_message(&long_line);
        let result = generate_thread_name(&message);

        assert_eq!(result.chars().count(), 100);
        assert_eq!(result, "a".repeat(100));
    }

    #[test]
    fn test_generate_thread_name_first_line_with_trailing_newlines() {
        let message = create_message("First line\n\n\n");
        let result = generate_thread_name(&message);

        assert_eq!(result, "First line");
    }

    #[test]
    fn test_generate_thread_name_multiline_with_whitespace() {
        let message = create_message("  First line  \nSecond line");
        let result = generate_thread_name(&message);

        assert_eq!(result, "First line");
    }
}
