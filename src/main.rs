mod adapters;
mod bridge;
mod params;

use anyhow::Context as _;
use adapters::{HttpEventSender, SerenityChannelInfoProvider, SerenityDiscordService};
use bridge::event_bridge::EventBridge;
use bridge::sender_filter::{MessageFilter, ReactionFilter};
use std::sync::Arc;
use tracing::{error, info};

use serenity::async_trait;
use serenity::model::channel::{Message, Reaction};
use serenity::model::event::{MessageUpdateEvent, ResumedEvent};
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::prelude::*;

struct Handler {
    bridge: std::sync::OnceLock<EventBridge<SerenityDiscordService, HttpEventSender, SerenityChannelInfoProvider>>,
    params: Arc<params::Params>,
    // Active filters initialized in ready event
    message_direct_filter: std::sync::OnceLock<MessageFilter>,
    message_guild_filter: std::sync::OnceLock<MessageFilter>,
    reaction_add_direct_filter: std::sync::OnceLock<ReactionFilter>,
    reaction_add_guild_filter: std::sync::OnceLock<ReactionFilter>,
    reaction_remove_direct_filter: std::sync::OnceLock<ReactionFilter>,
    reaction_remove_guild_filter: std::sync::OnceLock<ReactionFilter>,
}

impl Handler {
    fn new(params: &params::Params) -> anyhow::Result<Handler> {
        Ok(Handler {
            bridge: std::sync::OnceLock::new(),
            params: Arc::new(params.clone()),
            message_direct_filter: std::sync::OnceLock::new(),
            message_guild_filter: std::sync::OnceLock::new(),
            reaction_add_direct_filter: std::sync::OnceLock::new(),
            reaction_add_guild_filter: std::sync::OnceLock::new(),
            reaction_remove_direct_filter: std::sync::OnceLock::new(),
            reaction_remove_guild_filter: std::sync::OnceLock::new(),
        })
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let current_user_id = ready.user.id;

        // Initialize EventBridge with cache and http from Context
        // Both are kept alive and maintained by Serenity's event loop
        let discord_service = Arc::new(SerenityDiscordService::new(ctx.http.clone()));
        let channel_info = Arc::new(SerenityChannelInfoProvider::new(
            ctx.cache.clone(),
            ctx.http.clone()
        ));

        let endpoint = url::Url::parse(&self.params.http_endpoint)
            .expect("HTTP_ENDPOINT already validated");
        let event_sender = Arc::new(
            HttpEventSender::new(
                endpoint,
                self.params.insecure_mode,
                self.params.http_timeout,
                self.params.http_connect_timeout,
                self.params.max_response_body_size,
            )
            .expect("HttpEventSender already validated")
        );

        let bridge = EventBridge::new(discord_service, event_sender, channel_info, self.params.max_actions);
        let _ = self.bridge.set(bridge);

        // Initialize active filters with current user ID
        if let Some(policy) = &self.params.message_direct {
            let _ = self
                .message_direct_filter
                .set(policy.for_message(current_user_id));
        }
        if let Some(policy) = &self.params.message_guild {
            let _ = self
                .message_guild_filter
                .set(policy.for_message(current_user_id));
        }
        if let Some(policy) = &self.params.reaction_add_direct {
            let _ = self
                .reaction_add_direct_filter
                .set(policy.for_reaction(current_user_id));
        }
        if let Some(policy) = &self.params.reaction_add_guild {
            let _ = self
                .reaction_add_guild_filter
                .set(policy.for_reaction(current_user_id));
        }
        if let Some(policy) = &self.params.reaction_remove_direct {
            let _ = self
                .reaction_remove_direct_filter
                .set(policy.for_reaction(current_user_id));
        }
        if let Some(policy) = &self.params.reaction_remove_guild {
            let _ = self
                .reaction_remove_guild_filter
                .set(policy.for_reaction(current_user_id));
        }

        info!(
            display_name = %ready.user.display_name(),
            user_id = %ready.user.id,
            "Bot is connected"
        );
        info!(
            install_url = %format!("https://discord.com/oauth2/authorize?client_id={}&scope=bot", ready.application.id),
            "Bot install URL available"
        );

        // Check if READY event is enabled
        if self.params.ready.is_none() {
            return;
        }

        // Get bridge (should be initialized above)
        let Some(bridge) = self.bridge.get() else {
            error!("Bridge not initialized - this should not happen");
            return;
        };

        // Handle event (send to webhook + execute actions if needed)
        match bridge.handle_ready(&ready).await {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                // Currently ready event doesn't have associated message context,
                // so we log and skip action execution
                tracing::warn!(
                    action_count = event_response.actions.len(),
                    "Ready event received actions from webhook, but action execution is not supported for ready events"
                );
            }
            Ok(_) => {
                // No response or empty actions - success
            }
            Err(err) => {
                error!(?err, "Failed to handle ready event");
            }
        }
    }

    async fn resume(&self, _ctx: Context, resumed: ResumedEvent) {
        info!("Session resumed successfully");

        // Check if RESUMED event is enabled
        if self.params.resumed.is_none() {
            return;
        }

        // Get bridge (should be initialized in ready event)
        let Some(bridge) = self.bridge.get() else {
            error!("Bridge not initialized - this should not happen");
            return;
        };

        // Handle event (send to webhook + execute actions if needed)
        match bridge.handle_resumed(&resumed).await {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                // Currently resumed event doesn't have associated message context,
                // so we log and skip action execution
                tracing::warn!(
                    action_count = event_response.actions.len(),
                    "Resumed event received actions from webhook, but action execution is not supported for resumed events"
                );
            }
            Ok(_) => {
                // No response or empty actions - success
            }
            Err(err) => {
                error!(?err, "Failed to handle resumed event");
            }
        }
    }

    async fn message(&self, _ctx: Context, message: Message) {
        let is_direct = message.guild_id.is_none();

        // Get the appropriate active filter
        let filter = if is_direct {
            self.message_direct_filter.get()
        } else {
            self.message_guild_filter.get()
        };

        // If filter is not initialized (not ready yet) or not configured, don't process
        let Some(filter) = filter else {
            return;
        };

        // Apply message filter
        if !filter.should_process(&message) {
            return;
        }

        // Get bridge (should be initialized by ready event)
        let Some(bridge) = self.bridge.get() else {
            error!("Bridge not initialized - this should not happen");
            return;
        };

        // Handle event (send to webhook + execute actions)
        match bridge.handle_message(&message).await {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                // Execute actions if webhook responded with any
                if let Err(err) = bridge
                    .execute_actions(&message, &event_response)
                    .await
                {
                    error!(?err, "Failed to execute actions from webhook response");
                }
            }
            Ok(_) => {
                // No response or empty actions - success
            }
            Err(err) => {
                error!(?err, "Failed to handle message event");
            }
        }
    }

    async fn message_delete(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        // Check if event is enabled for this context
        match guild_id {
            None if self.params.message_delete_direct.is_none() => return,
            Some(_) if self.params.message_delete_guild.is_none() => return,
            _ => {}
        }

        // Get bridge
        let Some(bridge) = self.bridge.get() else {
            error!("Bridge not initialized - this should not happen");
            return;
        };

        // Handle event
        match bridge
            .handle_message_delete(channel_id, deleted_message_id, guild_id)
            .await
        {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                tracing::warn!(
                    action_count = event_response.actions.len(),
                    "MessageDelete event received actions from webhook, \
                     but action execution is not supported for delete events"
                );
            }
            Ok(_) => {
                // Success
            }
            Err(err) => {
                error!(?err, "Failed to handle message_delete event");
            }
        }
    }

    async fn message_delete_bulk(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        multiple_deleted_messages_ids: Vec<MessageId>,
        guild_id: Option<GuildId>,
    ) {
        // Check if event is enabled
        if self.params.message_delete_bulk_guild.is_none() {
            return;
        }

        // Get bridge
        let Some(bridge) = self.bridge.get() else {
            error!("Bridge not initialized - this should not happen");
            return;
        };

        // Handle event
        match bridge
            .handle_message_delete_bulk(channel_id, multiple_deleted_messages_ids, guild_id)
            .await
        {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                tracing::warn!(
                    action_count = event_response.actions.len(),
                    "MessageDeleteBulk event received actions from webhook, \
                     but action execution is not supported for delete events"
                );
            }
            Ok(_) => {
                // Success
            }
            Err(err) => {
                error!(?err, "Failed to handle message_delete_bulk event");
            }
        }
    }

    async fn message_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Message>,
        _new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        // Check if event is enabled for this context
        match event.guild_id {
            None if self.params.message_update_direct.is_none() => return,
            Some(_) if self.params.message_update_guild.is_none() => return,
            _ => {}
        }

        // Get bridge
        let Some(bridge) = self.bridge.get() else {
            error!("Bridge not initialized - this should not happen");
            return;
        };

        // Handle event
        match bridge.handle_message_update(event).await {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                tracing::warn!(
                    action_count = event_response.actions.len(),
                    "MessageUpdate event received actions from webhook, \
                     but action execution is not supported for update events"
                );
            }
            Ok(_) => {
                // Success
            }
            Err(err) => {
                error!(?err, "Failed to handle message_update event");
            }
        }
    }

    async fn reaction_add(&self, _ctx: Context, reaction: Reaction) {
        // Determine filter based on context (DM vs Guild)
        let filter = match reaction.guild_id {
            None => self.reaction_add_direct_filter.get(),
            Some(_) => self.reaction_add_guild_filter.get(),
        };

        // Check if event is enabled and filter passes
        let Some(filter) = filter else {
            return; // Event not enabled for this context
        };
        if !filter.should_process(&reaction) {
            return; // Filtered out
        }

        // Get bridge (should be initialized by ready event)
        let Some(bridge) = self.bridge.get() else {
            error!("Bridge not initialized - this should not happen");
            return;
        };

        // Handle event (send to webhook + execute actions)
        match bridge.handle_reaction_add(&reaction).await {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                // Execute actions if webhook responded with any
                if let Err(err) = bridge
                    .execute_actions(&reaction, &event_response)
                    .await
                {
                    error!(?err, "Failed to execute actions from webhook response");
                }
            }
            Ok(_) => {
                // No response or empty actions - success
            }
            Err(err) => {
                error!(?err, "Failed to handle reaction_add event");
            }
        }
    }

    async fn reaction_remove(&self, _ctx: Context, reaction: Reaction) {
        // Determine filter based on context (DM vs Guild)
        let filter = match reaction.guild_id {
            None => self.reaction_remove_direct_filter.get(),
            Some(_) => self.reaction_remove_guild_filter.get(),
        };

        // Check if event is enabled and filter passes
        let Some(filter) = filter else {
            return; // Event not enabled for this context
        };
        if !filter.should_process(&reaction) {
            return; // Filtered out
        }

        // Get bridge (should be initialized by ready event)
        let Some(bridge) = self.bridge.get() else {
            error!("Bridge not initialized - this should not happen");
            return;
        };

        // Handle event (send to webhook + execute actions)
        match bridge.handle_reaction_remove(&reaction).await {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                // Execute actions if webhook responded with any
                if let Err(err) = bridge
                    .execute_actions(&reaction, &event_response)
                    .await
                {
                    error!(?err, "Failed to execute actions from webhook response");
                }
            }
            Ok(_) => {
                // No response or empty actions - success
            }
            Err(err) => {
                error!(?err, "Failed to handle reaction_remove event");
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file if it exists
    let _ = dotenvy::dotenv();

    // Initialize tracing subscriber for structured logging
    // Default: gatehook=info, serenity=warn (suppress serenity's normal operation logs)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gatehook=info,serenity=warn".into()),
        )
        .init();

    // Display startup banner with version information
    info!(
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        description = env!("CARGO_PKG_DESCRIPTION"),
        "Starting application"
    );

    let params = params::Params::new()?;
    info!(?params, "Application parameters loaded");

    // Build gateway intents based on enabled events
    let intents = build_gateway_intents(&params);
    info!(?intents, "Gateway intents configured");

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&params.discord_token, intents)
        .event_handler(Handler::new(&params)?)
        .await
        .context("Creating Discord Client")?;

    // Start listening for events by starting a single shard
    client
        .start_autosharded()
        .await
        .context("Running Discord Client")
}

/// Build GatewayIntents based on enabled events in parameters
fn build_gateway_intents(params: &params::Params) -> GatewayIntents {
    let mut intents = GatewayIntents::empty();

    // Direct Message events (MESSAGE, MESSAGE_DELETE, MESSAGE_UPDATE, REACTION_ADD, REACTION_REMOVE)
    if params.has_direct_message_events()
        || params.has_message_delete_events()
        || params.has_message_update_events()
        || params.has_direct_reaction_add_events()
        || params.has_direct_reaction_remove_events()
    {
        intents |= GatewayIntents::DIRECT_MESSAGES;
    }

    // MESSAGE_CONTENT is needed for MESSAGE and MESSAGE_UPDATE events, not DELETE or REACTION_ADD/REMOVE
    if params.has_direct_message_events() || params.has_message_update_events() {
        intents |= GatewayIntents::MESSAGE_CONTENT;
    }

    // Direct Message Reactions
    if params.has_direct_reaction_add_events() || params.has_direct_reaction_remove_events() {
        intents |= GatewayIntents::DIRECT_MESSAGE_REACTIONS;
    }

    // Guild Message events (MESSAGE, MESSAGE_DELETE, MESSAGE_DELETE_BULK, MESSAGE_UPDATE, REACTION_ADD, REACTION_REMOVE)
    if params.has_guild_message_events()
        || params.has_message_delete_events()
        || params.has_message_delete_bulk_events()
        || params.has_message_update_events()
        || params.has_guild_reaction_add_events()
        || params.has_guild_reaction_remove_events()
    {
        intents |= GatewayIntents::GUILD_MESSAGES;
        // GUILDS intent is required for cache access (guild/channel data)
        intents |= GatewayIntents::GUILDS;
    }

    // MESSAGE_CONTENT is needed for MESSAGE and MESSAGE_UPDATE events, not DELETE or REACTION_ADD/REMOVE
    if params.has_guild_message_events() || params.has_message_update_events() {
        intents |= GatewayIntents::MESSAGE_CONTENT;
    }

    // Guild Message Reactions
    if params.has_guild_reaction_add_events() || params.has_guild_reaction_remove_events() {
        intents |= GatewayIntents::GUILD_MESSAGE_REACTIONS;
    }

    intents
}
