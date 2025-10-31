mod adapters;
mod bridge;
mod params;

use anyhow::Context as _;
use adapters::discord::SerenityDiscordService;
use adapters::event_sender::HttpEventSender;
use bridge::event_bridge::EventBridge;
use std::sync::Arc;
use tracing::{error, info};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    bridge: EventBridge<SerenityDiscordService, HttpEventSender>,
}

impl Handler {
    fn new(
        params: &params::Params,
        http: Arc<serenity::http::Http>,
    ) -> anyhow::Result<Handler> {
        let discord_service = Arc::new(SerenityDiscordService::new(http));
        let event_sender = Arc::new(HttpEventSender::new(
            params.webhook_url.clone(),
            params.insecure_mode,
        )?);

        let bridge = EventBridge::new(discord_service, event_sender);

        Ok(Handler { bridge })
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!(
            display_name = %ready.user.display_name(),
            "Bot is connected"
        );
        info!(
            install_url = %format!("https://discord.com/oauth2/authorize?client_id={}&scope=bot", ready.application.id),
            "Bot install URL available"
        );

        if let Err(e) = self.bridge.handle_ready(&ready).await {
            error!(error = ?e, "Failed to handle ready event");
        }
    }

    async fn message(&self, _: Context, message: Message) {
        if let Err(e) = self.bridge.handle_message(&message).await {
            error!(error = ?e, "Failed to handle message event");
        }
    }

    async fn reaction_add(&self, _: Context, reaction: Reaction) {
        if let Err(e) = self.bridge.handle_reaction_add(&reaction).await {
            error!(error = ?e, "Failed to handle reaction_add event");
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

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;

    // Create HTTP client for Discord API (needed for Handler)
    let http = Arc::new(serenity::http::Http::new(&params.discord_token));

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&params.discord_token, intents)
        .event_handler(Handler::new(&params, http)?)
        .await
        .context("Creating Discord Client")?;

    // Start listening for events by starting a single shard
    client
        .start_autosharded()
        .await
        .context("Running Discord Client")
}
