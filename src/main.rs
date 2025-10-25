mod params;
pub mod webhook;

use anyhow::Context as _;
use tracing::{debug, error, info};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    webhook_client: webhook::WebhookClient,
}

impl Handler {
    fn new(params: &params::Params) -> anyhow::Result<Handler> {
        let webhook_client = webhook::WebhookClient::new(
            params.webhook_url.clone(),
            params.insecure_mode,
        )?;

        Ok(Handler { webhook_client })
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
    }

    async fn message(&self, ctx: Context, message: Message) {
        debug!(
            message_id = %message.id,
            author = %message.author.name,
            content = %message.content,
            "Received message"
        );

        if message.content == "Ping!"
            && let Err(why) = message.reply(&ctx.http, "Pong!").await
        {
            error!(error = ?why, "Failed to send message reply");
        }

        // Send message to webhook endpoint
        self.webhook_client
            .send_with_logging("message", &message, &message.id.to_string())
            .await;
    }

    async fn reaction_add(&self, _: Context, reaction: Reaction) {
        debug!(
            message_id = %reaction.message_id,
            user_id = ?reaction.user_id,
            emoji = ?reaction.emoji,
            "Received reaction"
        );
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
