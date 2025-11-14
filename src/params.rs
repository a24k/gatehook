use anyhow::Context as _;
use serde::Deserialize;
use crate::bridge::message_filter::MessageFilterPolicy;

/// Deserialize environment variable string into MessageFilterPolicy
fn deserialize_message_filter_policy<'de, D>(
    deserializer: D,
) -> Result<Option<MessageFilterPolicy>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.map(|policy| MessageFilterPolicy::from_policy(&policy)))
}

#[derive(Deserialize, Clone)]
pub struct Params {
    #[serde(default)]
    pub insecure_mode: bool,
    pub discord_token: String,
    pub http_endpoint: String,

    // ========================================
    // Event Configuration
    // ========================================
    // Direct Message Events
    #[serde(default, deserialize_with = "deserialize_message_filter_policy")]
    pub message_direct: Option<MessageFilterPolicy>,

    // Guild Events
    #[serde(default, deserialize_with = "deserialize_message_filter_policy")]
    pub message_guild: Option<MessageFilterPolicy>,

    // Message Delete Events
    #[serde(default)]
    pub message_delete_direct: Option<bool>,
    #[serde(default)]
    pub message_delete_guild: Option<bool>,
    #[serde(default)]
    pub message_delete_bulk_guild: Option<bool>,

    // Context-Independent Events
    #[serde(default)]
    pub ready: Option<String>,
}

/// Mask sensitive strings by showing only first and last few characters
fn mask_token(s: &str) -> String {
    const VISIBLE_CHARS: usize = 4;

    if s.len() <= VISIBLE_CHARS * 2 {
        // If string is too short, mask everything except first char
        if s.is_empty() {
            return "<empty>".to_string();
        }
        return format!("{}***", &s[..1]);
    }

    format!(
        "{}***{}",
        &s[..VISIBLE_CHARS],
        &s[s.len() - VISIBLE_CHARS..]
    )
}

impl std::fmt::Debug for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Params")
            .field("insecure_mode", &self.insecure_mode)
            .field("discord_token", &mask_token(&self.discord_token))
            .field("http_endpoint", &self.http_endpoint)
            .field("message_direct", &self.message_direct)
            .field("message_guild", &self.message_guild)
            .field("message_delete_direct", &self.message_delete_direct)
            .field("message_delete_guild", &self.message_delete_guild)
            .field("message_delete_bulk_guild", &self.message_delete_bulk_guild)
            .field("ready", &self.ready)
            .finish()
    }
}

impl Params {
    pub fn new() -> anyhow::Result<Params> {
        envy::from_env::<Params>().context("Failed to load configuration")
    }

    /// Check if Direct Message events are enabled
    pub fn has_direct_message_events(&self) -> bool {
        self.message_direct.is_some()
    }

    /// Check if Guild Message events are enabled
    pub fn has_guild_message_events(&self) -> bool {
        self.message_guild.is_some()
    }

    /// Check if any MESSAGE_DELETE events are enabled
    pub fn has_message_delete_events(&self) -> bool {
        self.message_delete_direct.is_some() || self.message_delete_guild.is_some()
    }

    /// Check if MESSAGE_DELETE_BULK event is enabled
    pub fn has_message_delete_bulk_events(&self) -> bool {
        self.message_delete_bulk_guild.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::long_string("MTExMjIyMzMzNDQ0NTU1NjY2Nzc3ODg4OTk5", "MTEx***OTk5")]
    #[case::short_string("short", "s***")]
    #[case::empty_string("", "<empty>")]
    fn test_mask_token(#[case] input: &str, #[case] expected: &str) {
        let masked = mask_token(input);
        assert_eq!(masked, expected);
    }

    #[test]
    fn test_params_debug_masks_sensitive_data() {
        let params = Params {
            insecure_mode: false,
            discord_token: "MTExMjIyMzMzNDQ0NTU1NjY2Nzc3ODg4OTk5".to_string(),
            http_endpoint: "https://example.com/webhook/secret123456".to_string(),
            message_direct: None,
            message_guild: None,
            message_delete_direct: None,
            message_delete_guild: None,
            message_delete_bulk_guild: None,
            ready: None,
        };

        let debug_output = format!("{:?}", params);

        // Should contain masked discord_token
        assert!(debug_output.contains("MTEx***OTk5"));

        // Should NOT contain full discord_token
        assert!(!debug_output.contains("MTExMjIyMzMzNDQ0NTU1NjY2Nzc3ODg4OTk5"));

        // http_endpoint should be visible (not masked)
        assert!(debug_output.contains("https://example.com/webhook/secret123456"));
    }
}
