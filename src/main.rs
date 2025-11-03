mod adapters;
mod bridge;
mod params;

use anyhow::Context as _;
use adapters::{HttpEventSender, SerenityDiscordService};
use bridge::event_bridge::EventBridge;
use bridge::message_filter::MessageFilter;
use std::sync::Arc;
use tracing::{error, info};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    bridge: EventBridge<SerenityDiscordService, HttpEventSender>,
    params: Arc<params::Params>,
    // Active filters initialized in ready event
    message_direct_filter: std::sync::OnceLock<MessageFilter>,
    message_guild_filter: std::sync::OnceLock<MessageFilter>,
}

impl Handler {
    fn new(params: &params::Params) -> anyhow::Result<Handler> {
        let discord_service = Arc::new(SerenityDiscordService);

        let endpoint = url::Url::parse(&params.http_endpoint)
            .context("Parsing HTTP_ENDPOINT URL")?;
        let event_sender = Arc::new(HttpEventSender::new(
            endpoint,
            params.insecure_mode,
        )?);

        let bridge = EventBridge::new(discord_service, event_sender);

        Ok(Handler {
            bridge,
            params: Arc::new(params.clone()),
            message_direct_filter: std::sync::OnceLock::new(),
            message_guild_filter: std::sync::OnceLock::new(),
        })
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        let current_user_id = ready.user.id;

        // Initialize active filters with current user ID
        if let Some(policy) = &self.params.message_direct {
            let _ = self
                .message_direct_filter
                .set(policy.for_user(current_user_id));
        }
        if let Some(policy) = &self.params.message_guild {
            let _ = self
                .message_guild_filter
                .set(policy.for_user(current_user_id));
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

        if let Err(e) = self.bridge.handle_ready(&ready).await {
            error!(error = ?e, "Failed to handle ready event");
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
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

        // Handle event (send to webhook + execute actions)
        match self.bridge.handle_message(&ctx.http, &message).await {
            Ok(Some(event_response)) if !event_response.actions.is_empty() => {
                // Execute actions if webhook responded with any
                if let Err(err) = self
                    .bridge
                    .execute_actions(&ctx.http, &message, &event_response)
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

    // Direct Message events
    if params.has_direct_message_events() {
        intents |= GatewayIntents::DIRECT_MESSAGES;
        intents |= GatewayIntents::MESSAGE_CONTENT;
    }

    // Guild Message events
    if params.has_guild_message_events() {
        intents |= GatewayIntents::GUILD_MESSAGES;
        intents |= GatewayIntents::MESSAGE_CONTENT;
    }

    intents
}
