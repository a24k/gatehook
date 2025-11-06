use crate::adapters::{
    DiscordService, EventResponse, EventSender, ReactParams, ReplyParams, ResponseAction,
    ThreadParams,
};
use anyhow::Context as _;
use serenity::client::Context as SerenityContext;
use serenity::model::channel::{ChannelType, Message};
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

    /// Execute actions from webhook response (production version with cache access)
    ///
    /// # Arguments
    ///
    /// * `ctx` - The serenity Context (provides cache and HTTP client)
    /// * `message` - The message that triggered the event (for context)
    /// * `event_response` - The response from webhook containing actions
    pub async fn execute_actions(
        &self,
        ctx: &SerenityContext,
        message: &Message,
        event_response: &EventResponse,
    ) -> anyhow::Result<()> {
        for action in &event_response.actions {
            // Execute action (log error and continue with next)
            if let Err(err) = self.execute_action(ctx, message, action).await {
                error!(?err, ?action, "Failed to execute action, continuing with next");
            }
        }
        Ok(())
    }

    /// Execute actions for testing (accepts Http directly, no cache access)
    ///
    /// This version is used by integration tests which cannot easily construct
    /// a full serenity Context. It skips cache optimization and uses API calls directly.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub async fn execute_actions_for_test(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        event_response: &EventResponse,
    ) -> anyhow::Result<()> {
        for action in &event_response.actions {
            // Execute action (log error and continue with next)
            if let Err(err) = self.execute_action_for_test(http, message, action).await {
                error!(?err, ?action, "Failed to execute action, continuing with next");
            }
        }
        Ok(())
    }

    /// Execute a single action (production version with cache access)
    async fn execute_action(
        &self,
        ctx: &SerenityContext,
        message: &Message,
        action: &ResponseAction,
    ) -> anyhow::Result<()> {
        match action {
            ResponseAction::Reply(params) => self.execute_reply(ctx, message, params).await,
            ResponseAction::React(params) => self.execute_react(ctx, message, params).await,
            ResponseAction::Thread(params) => self.execute_thread(ctx, message, params).await,
        }
    }

    /// Execute a single action (test version without cache)
    #[allow(dead_code)]
    async fn execute_action_for_test(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        action: &ResponseAction,
    ) -> anyhow::Result<()> {
        match action {
            ResponseAction::Reply(params) => self.execute_reply_for_test(http, message, params).await,
            ResponseAction::React(params) => self.execute_react_for_test(http, message, params).await,
            ResponseAction::Thread(params) => self.execute_thread_for_test(http, message, params).await,
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
        ctx: &SerenityContext,
        message: &Message,
        params: &ReplyParams,
    ) -> anyhow::Result<()> {
        let content = truncate_content(&params.content);

        self.discord_service
            .reply_to_message(&ctx.http, message.channel_id, message.id, &content, params.mention)
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

    /// Execute Reply action (test version)
    #[allow(dead_code)]
    async fn execute_reply_for_test(
        &self,
        http: &serenity::http::Http,
        message: &Message,
        params: &ReplyParams,
    ) -> anyhow::Result<()> {
        let content = truncate_content(&params.content);

        self.discord_service
            .reply_to_message(http, message.channel_id, message.id, &content, params.mention)
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
        ctx: &SerenityContext,
        message: &Message,
        params: &ReactParams,
    ) -> anyhow::Result<()> {
        self.discord_service
            .react_to_message(&ctx.http, message.channel_id, message.id, &params.emoji)
            .await
            .context("Failed to add reaction to Discord")?;

        info!(
            message_id = %message.id,
            emoji = %params.emoji,
            "Successfully executed react action"
        );

        Ok(())
    }

    /// Execute React action (test version)
    #[allow(dead_code)]
    async fn execute_react_for_test(
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

    /// Execute Thread action (production version with cache-first channel detection)
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
    ///
    /// # Channel Detection
    /// - Checks cache first for channel type (fast, no API call)
    /// - Falls back to API call if cache miss (rare)
    async fn execute_thread(
        &self,
        ctx: &SerenityContext,
        message: &Message,
        params: &ThreadParams,
    ) -> anyhow::Result<()> {
        // Ensure we're in a guild (threads not supported in DM)
        let guild_id = message.guild_id
            .context("Thread action is not supported in DM")?;

        // Check if already in thread (cache-first approach)
        // Extract channel kind from cache (if available) before any await points
        let channel_kind_from_cache = ctx.cache.guild(guild_id)
            .and_then(|guild| guild.channels.get(&message.channel_id).map(|ch| ch.kind));

        let is_in_thread = if let Some(kind) = channel_kind_from_cache {
            // Cache hit - fast path (no await needed)
            matches!(
                kind,
                ChannelType::PublicThread | ChannelType::PrivateThread | ChannelType::NewsThread
            )
        } else {
            // Cache miss - fallback to API call
            if ctx.cache.guild(guild_id).is_none() {
                debug!(guild_id = %guild_id, "Guild not in cache, using API call");
            } else {
                debug!(channel_id = %message.channel_id, "Channel not in guild cache, using API call");
            }

            self.discord_service
                .is_thread_channel(&ctx.http, message.channel_id)
                .await
                .context("Failed to check if channel is thread")?
        };

        // Determine target channel ID
        let target_channel_id = if is_in_thread {
            // Already in thread → use as-is
            if params.name.is_some() {
                debug!("Already in thread, ignoring 'name' parameter");
            }
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
                    &ctx.http,
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
                .reply_in_channel(&ctx.http, target_channel_id, message.id, &content, params.mention)
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
                .send_message_to_channel(&ctx.http, target_channel_id, &content)
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

    /// Execute Thread action (test version using API calls only)
    #[allow(dead_code)]
    async fn execute_thread_for_test(
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
            if params.name.is_some() {
                debug!("Already in thread, ignoring 'name' parameter");
            }
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
