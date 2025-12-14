use crate::adapters::{
    ChannelInfoProvider, DiscordService, EventResponse, EventSender, ReactParams, ReplyParams,
    ResponseAction, ThreadParams,
};
use crate::bridge::action_target::ActionTarget;
use crate::bridge::discord_text::{truncate_content, truncate_thread_name};
use crate::bridge::message_delete_bulk_payload::MessageDeleteBulkPayload;
use crate::bridge::message_delete_payload::MessageDeletePayload;
use crate::bridge::message_payload::MessagePayload;
use crate::bridge::message_update_payload::MessageUpdatePayload;
use crate::bridge::reaction_payload::ReactionPayload;
use crate::bridge::ready_payload::ReadyPayload;
use crate::bridge::resumed_payload::ResumedPayload;
use anyhow::Context as _;
use serenity::model::channel::{Message, Reaction};
use serenity::model::event::{MessageUpdateEvent, ResumedEvent};
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Bridge Discord Gateway events to external endpoints
pub struct EventBridge<D, S, C>
where
    D: DiscordService,
    S: EventSender,
    C: ChannelInfoProvider,
{
    discord_service: Arc<D>,
    event_sender: Arc<S>,
    channel_info: Arc<C>,
    max_actions: usize,
}

impl<D, S, C> EventBridge<D, S, C>
where
    D: DiscordService,
    S: EventSender,
    C: ChannelInfoProvider,
{
    /// Create a new EventBridge
    ///
    /// # Arguments
    ///
    /// * `discord_service` - The Discord service for operations
    /// * `event_sender` - The event sender for forwarding events
    /// * `channel_info` - The channel info provider for retrieving channel information
    /// * `max_actions` - Maximum number of actions to execute per event (for DoS protection)
    pub fn new(discord_service: Arc<D>, event_sender: Arc<S>, channel_info: Arc<C>, max_actions: usize) -> Self {
        Self {
            discord_service,
            event_sender,
            channel_info,
            max_actions,
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

        // Build payload with channel information (cache-first with API fallback)
        let payload = self.build_message_payload(message).await;

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("message", &payload)
            .await
            .context("Failed to send message event to HTTP endpoint")
    }

    /// Build MessagePayload with channel information
    ///
    /// Attempts to retrieve GuildChannel from ChannelInfoProvider (cache-first with API fallback).
    /// If not available (DM or error), creates payload without channel info.
    async fn build_message_payload<'a>(
        &self,
        message: &'a Message,
    ) -> MessagePayload<'a> {
        // Try to get channel from provider (cache-first, API fallback)
        let channel = match self.channel_info
            .get_channel(message.guild_id, message.channel_id)
            .await
        {
            Ok(Some(ch)) => Some(ch),
            Ok(None) => {
                debug!(
                    channel_id = %message.channel_id,
                    "Channel not found (likely DM)"
                );
                None
            }
            Err(err) => {
                debug!(
                    channel_id = %message.channel_id,
                    ?err,
                    "Failed to retrieve channel information"
                );
                None
            }
        };

        match channel {
            Some(ch) => {
                debug!(
                    channel_id = %message.channel_id,
                    channel_name = %ch.name,
                    channel_kind = ?ch.kind,
                    "Channel information retrieved"
                );
                MessagePayload::with_channel(message, ch)
            }
            None => MessagePayload::new(message),
        }
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

        // Build payload with ready event
        let payload = ReadyPayload::new(ready);

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("ready", &payload)
            .await
            .context("Failed to send ready event to HTTP endpoint")
    }

    /// Handle a resumed event
    ///
    /// Sends event to webhook and returns the response.
    ///
    /// # Arguments
    ///
    /// * `resumed` - The resumed event from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (may contain actions)
    pub async fn handle_resumed(
        &self,
        resumed: &ResumedEvent,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!("Processing resumed event");

        // Build payload with resumed event
        let payload = ResumedPayload::new(resumed);

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("resumed", &payload)
            .await
            .context("Failed to send resumed event to HTTP endpoint")
    }

    /// Handle a reaction add event
    ///
    /// # Arguments
    ///
    /// * `reaction` - The reaction event from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (may contain actions)
    pub async fn handle_reaction_add(
        &self,
        reaction: &Reaction,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            user_id = ?reaction.user_id,
            message_id = %reaction.message_id,
            channel_id = %reaction.channel_id,
            "Processing reaction add event"
        );

        // Build payload with optional channel metadata
        let payload = self.build_reaction_payload(reaction).await;

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("reaction_add", &payload)
            .await
            .context("Failed to send reaction add event to HTTP endpoint")
    }

    /// Build reaction payload with optional channel info from cache
    async fn build_reaction_payload<'a>(&self, reaction: &'a Reaction) -> ReactionPayload<'a> {
        // Try to get channel info from cache if this is a guild reaction
        match reaction.guild_id {
            Some(guild_id) => {
                match self
                    .channel_info
                    .get_channel(Some(guild_id), reaction.channel_id)
                    .await
                {
                    Ok(Some(channel)) => ReactionPayload::with_channel(reaction, channel),
                    _ => ReactionPayload::new(reaction),
                }
            }
            None => ReactionPayload::new(reaction),
        }
    }

    /// Handle a reaction remove event
    ///
    /// # Arguments
    ///
    /// * `reaction` - The reaction event from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (may contain actions)
    pub async fn handle_reaction_remove(
        &self,
        reaction: &Reaction,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            user_id = ?reaction.user_id,
            message_id = %reaction.message_id,
            channel_id = %reaction.channel_id,
            "Processing reaction remove event"
        );

        // Build payload with optional channel metadata
        let payload = self.build_reaction_payload(reaction).await;

        // Forward event to webhook endpoint and return response
        self.event_sender
            .send("reaction_remove", &payload)
            .await
            .context("Failed to send reaction remove event to HTTP endpoint")
    }

    /// Execute actions from webhook response
    ///
    /// # Arguments
    ///
    /// * `target` - The action target (message, reaction, etc.)
    /// * `event_response` - The response from webhook containing actions
    ///
    /// # Security
    ///
    /// Limits the number of actions to `max_actions` to prevent DoS attacks.
    /// Logs action type only (not content) to prevent sensitive information exposure.
    pub async fn execute_actions(
        &self,
        target: impl Into<ActionTarget>,
        event_response: &EventResponse,
    ) -> anyhow::Result<()> {
        let target = target.into();

        let total_actions = event_response.actions.len();

        // Limit actions for DoS protection
        let actions_to_execute = if total_actions > self.max_actions {
            tracing::warn!(
                total_actions,
                max_actions = self.max_actions,
                "Too many actions in webhook response, truncating to max_actions"
            );
            &event_response.actions[..self.max_actions]
        } else {
            &event_response.actions[..]
        };

        for action in actions_to_execute {
            // Execute action (log error and continue with next)
            // Note: Only log action type, not content, to prevent sensitive information exposure
            if let Err(err) = self.execute_action(&target, action).await {
                error!(
                    ?err,
                    action_type = ?std::mem::discriminant(action),
                    "Failed to execute action, continuing with next"
                );
            }
        }
        Ok(())
    }

    /// Execute a single action
    async fn execute_action(
        &self,
        target: &ActionTarget,
        action: &ResponseAction,
    ) -> anyhow::Result<()> {
        match action {
            ResponseAction::Reply(params) => self.execute_reply(target, params).await,
            ResponseAction::React(params) => self.execute_react(target, params).await,
            ResponseAction::Thread(params) => self.execute_thread(target, params).await,
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
        target: &ActionTarget,
        params: &ReplyParams,
    ) -> anyhow::Result<()> {
        let content = truncate_content(&params.content);

        self.discord_service
            .reply_in_channel(target.channel_id, target.message_id, &content, params.mention)
            .await
            .context("Failed to send reply to Discord")?;

        info!(
            message_id = %target.message_id,
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
        target: &ActionTarget,
        params: &ReactParams,
    ) -> anyhow::Result<()> {
        self.discord_service
            .react_to_message(target.channel_id, target.message_id, &params.emoji)
            .await
            .context("Failed to add reaction to Discord")?;

        info!(
            message_id = %target.message_id,
            emoji = %params.emoji,
            "Successfully executed react action"
        );

        Ok(())
    }

    /// Execute Thread action
    ///
    /// # Thread Name
    /// - `params.name = Some(...)`: Use specified name
    /// - `params.name = None`: Defaults to "Thread"
    /// - Name is ignored if already in a thread
    ///
    /// # Content Handling
    /// - Content exceeding 2000 characters is truncated with warning log
    ///
    /// # Auto-archive Duration
    /// - Valid values: 60, 1440, 4320, 10080 (minutes)
    /// - Invalid values fall back to 1440 (OneDay) with warning log
    ///
    /// # Error Handling
    /// - If thread creation fails with error code 160004 (thread already exists),
    ///   retrieves the existing thread and posts to it
    async fn execute_thread(
        &self,
        target: &ActionTarget,
        params: &ThreadParams,
    ) -> anyhow::Result<()> {
        // Check if already in thread (cache-first with API fallback)
        // Note: This will fail for DM channels (threads not supported)
        let is_in_thread = self.channel_info
            .is_thread(target.guild_id, target.channel_id)
            .await
            .context("Failed to check if channel is thread (threads not supported in DM)")?;

        // Determine target channel ID
        let target_channel_id = if is_in_thread {
            // Already in thread → use as-is
            info!("Message is already in thread, skipping thread creation");
            target.channel_id
        } else {
            // Normal channel → create new thread (with fallback to existing thread)
            self.create_or_get_thread(target, params).await?
        };

        // Truncate content
        let content = truncate_content(&params.content);

        // Post message to thread
        self.discord_service
            .send_message_to_channel(target_channel_id, &content)
            .await
            .context("Failed to send message to thread")?;

        info!(
            channel_id = %target_channel_id,
            is_in_thread = is_in_thread,
            "Successfully executed thread action"
        );

        Ok(())
    }

    /// Create a new thread or retrieve existing thread if error 160004 occurs
    async fn create_or_get_thread(
        &self,
        target: &ActionTarget,
        params: &ThreadParams,
    ) -> anyhow::Result<serenity::model::id::ChannelId> {
        let thread_name = match &params.name {
            Some(name) => truncate_thread_name(name),
            None => "Thread".to_string(),
        };

        // Try to create thread
        match self
            .discord_service
            .create_thread_from_message(
                target.channel_id,
                target.message_id,
                &thread_name,
                params.auto_archive_duration,
            )
            .await
        {
            Ok(thread) => {
                info!(
                    thread_id = %thread.id,
                    thread_name = %thread_name,
                    "Created new thread"
                );
                Ok(thread.id)
            }
            Err(serenity::Error::Http(ref http_error))
                if Self::is_thread_already_exists_error(http_error) =>
            {
                // Thread already exists (error 160004) - retrieve it
                info!("Thread already exists for this message, retrieving existing thread");
                self.get_existing_thread_id(target).await
            }
            Err(e) => Err(e).context("Failed to create thread"),
        }
    }

    /// Retrieve existing thread ID from message
    async fn get_existing_thread_id(
        &self,
        target: &ActionTarget,
    ) -> anyhow::Result<serenity::model::id::ChannelId> {
        let message = self
            .discord_service
            .get_message(target.channel_id, target.message_id)
            .await
            .context("Failed to get message to find existing thread")?;

        let thread_id = message
            .thread
            .as_ref()
            .map(|t| t.id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Thread creation failed with 160004 but message has no thread field"
                )
            })?;

        info!(
            thread_id = %thread_id,
            "Found existing thread from message"
        );
        Ok(thread_id)
    }

    /// Check if HTTP error is "thread already exists" (error code 160004)
    fn is_thread_already_exists_error(http_error: &serenity::http::HttpError) -> bool {
        use serenity::http::HttpError;

        matches!(
            http_error,
            HttpError::UnsuccessfulRequest(error_response) if error_response.error.code == 160004
        )
    }

    /// Handle a message_delete event
    ///
    /// Sends event to webhook and returns the response.
    /// Note: Actions are not supported for delete events.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel where the message was deleted
    /// * `message_id` - The ID of the deleted message
    /// * `guild_id` - The guild ID (None for DMs)
    ///
    /// # Returns
    ///
    /// Response from webhook (actions are not supported for delete events)
    pub async fn handle_message_delete(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        guild_id: Option<GuildId>,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            message_id = %message_id,
            channel_id = %channel_id,
            ?guild_id,
            "Processing message_delete event"
        );

        let payload = MessageDeletePayload::new(channel_id, message_id, guild_id);

        self.event_sender
            .send("message_delete", &payload)
            .await
            .context("Failed to send message_delete event to HTTP endpoint")
    }

    /// Handle a message_delete_bulk event
    ///
    /// Sends event to webhook and returns the response.
    /// Note: Actions are not supported for delete events.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The channel where messages were deleted
    /// * `message_ids` - The IDs of deleted messages
    /// * `guild_id` - The guild ID (None for DMs, but bulk delete is typically guild-only)
    ///
    /// # Returns
    ///
    /// Response from webhook (actions are not supported for delete events)
    pub async fn handle_message_delete_bulk(
        &self,
        channel_id: ChannelId,
        message_ids: Vec<MessageId>,
        guild_id: Option<GuildId>,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            message_count = message_ids.len(),
            channel_id = %channel_id,
            ?guild_id,
            "Processing message_delete_bulk event"
        );

        let payload = MessageDeleteBulkPayload::new(channel_id, message_ids, guild_id);

        self.event_sender
            .send("message_delete_bulk", &payload)
            .await
            .context("Failed to send message_delete_bulk event to HTTP endpoint")
    }

    /// Handle a message_update event
    ///
    /// Sends event to webhook and returns the response.
    /// Note: Discord only provides changed fields in MessageUpdateEvent.
    /// Note: Actions are not supported for update events.
    ///
    /// # Arguments
    ///
    /// * `event` - The MessageUpdateEvent from Discord
    ///
    /// # Returns
    ///
    /// Response from webhook (actions are not supported for update events)
    pub async fn handle_message_update(
        &self,
        event: MessageUpdateEvent,
    ) -> anyhow::Result<Option<EventResponse>> {
        debug!(
            message_id = %event.id,
            channel_id = %event.channel_id,
            ?event.guild_id,
            "Processing message_update event"
        );

        let payload = MessageUpdatePayload::new(event);

        self.event_sender
            .send("message_update", &payload)
            .await
            .context("Failed to send message_update event to HTTP endpoint")
    }
}
