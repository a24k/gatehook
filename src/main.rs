use std::env;

use anyhow::Context as _;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

fn fetch_env(key: &str) -> anyhow::Result<String> {
    env::var(key).with_context(|| format!("Fetching environment variable: {}", key))
}

struct Handler {
    webhook_url: String,
    httpclient: reqwest::Client,
}

impl Handler {
    fn new() -> anyhow::Result<Handler> {
        let insecure = fetch_env("INSECURE").is_ok();
        dbg!(&insecure);

        let webhook_url = fetch_env("WEBHOOK_URL")?;
        dbg!(&webhook_url);

        let httpclient = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(insecure)
            .build()
            .context("Building HTTP Client")?;

        Ok(Handler {
            webhook_url,
            httpclient,
        })
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.display_name());
        // dbg!(&ready);
        println!(
            "Install URL: https://discord.com/oauth2/authorize?client_id={}&scope=bot",
            ready.application.id
        );
        println!("Webhook URL: {}", self.webhook_url);
    }

    async fn message(&self, ctx: Context, message: Message) {
        dbg!(&message);

        if message.content == "Ping!" {
            if let Err(why) = message.reply(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }

        // simple web get request
        let res = self
            .httpclient
            .post(&self.webhook_url)
            .query(&[("handler", "message")])
            .json(&message)
            .send()
            .await;
        dbg!(&res);
    }

    async fn reaction_add(&self, _: Context, reaction: Reaction) {
        dbg!(&reaction);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Login with a bot token from the environment
    let token = fetch_env("DISCORD_TOKEN")?;
    dbg!(&token);

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::new()?)
        .await
        .context("Creating Discord Client")?;

    // Start listening for events by starting a single shard
    client.start().await.context("Starting Discord Client")?;

    Ok(())
}
