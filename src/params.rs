use anyhow::Context as _;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Params {
    #[serde(default)]
    pub insecure_mode: bool,
    pub discord_token: String,
    pub webhook_url: String,
}

impl Params {
    pub fn new() -> anyhow::Result<Params> {
        envy::from_env::<Params>().context("Failed to load configuration")
    }
}
