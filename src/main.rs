mod params;

use anyhow::Context as _;
use tracing::{debug, error, info};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    webhook_url: String,
    httpclient: reqwest::Client,
}

impl Handler {
    fn new(params: &params::Params) -> anyhow::Result<Handler> {
        let httpclient = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(params.insecure_mode)
            .build()
            .context("Building HTTP Client")?;

        Ok(Handler {
            webhook_url: params.webhook_url.clone(),
            httpclient,
        })
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
            client_id = %ready.application.id,
            "Install URL: https://discord.com/oauth2/authorize?client_id={}&scope=bot",
            ready.application.id
        );
        info!(webhook_url = %self.webhook_url, "Webhook configured");
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
        let res = self
            .httpclient
            .post(&self.webhook_url)
            .query(&[("handler", "message")])
            .json(&message)
            .send()
            .await;

        match res {
            Ok(response) => {
                info!(
                    status = %response.status(),
                    message_id = %message.id,
                    "Successfully sent message to webhook"
                );
            }
            Err(err) => {
                error!(
                    error = ?err,
                    message_id = %message.id,
                    webhook_url = %self.webhook_url,
                    "Failed to send message to webhook"
                );
            }
        }
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

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
