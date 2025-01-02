mod params;

use anyhow::Context as _;

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
    let params = params::Params::new()?;
    dbg!(&params);

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
