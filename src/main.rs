use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    webhook_url: String,
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
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true) // TODO: to be optional
            .build()
            .unwrap();
        let res = client
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
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    dbg!(&token);

    // Webhook URL from the environment
    let webhook_url = env::var("WEBHOOK_URL").expect("Expected a webhook url in the environment");
    dbg!(&webhook_url);

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { webhook_url })
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
