use std::env;

use anyhow::Context as _;

#[derive(Debug)]
pub struct Params {
    pub insecure_mode: bool,
    pub discord_token: String,
    pub webhook_url: String,
}

impl Params {
    pub fn new() -> anyhow::Result<Params> {
        let insecure_mode = Self::fetch("INSECURE_MODE").is_ok();

        let discord_token = Self::fetch("DISCORD_TOKEN")?;

        let webhook_url = Self::fetch("WEBHOOK_URL")?;

        Ok(Params {
            insecure_mode,
            discord_token,
            webhook_url,
        })
    }

    fn fetch(key: &str) -> anyhow::Result<String> {
        env::var(key).with_context(|| format!("Fetching environment variable: {}", key))
    }
}
