use anyhow::Context as _;
use serde::Deserialize;
use crate::bridge::sender_filter::SenderFilterPolicy;

/// Default HTTP request timeout in seconds (5 minutes)
fn default_http_timeout() -> u64 {
    300
}

/// Default HTTP connection timeout in seconds
fn default_http_connect_timeout() -> u64 {
    10
}

/// Default maximum number of actions to execute per event
fn default_max_actions() -> usize {
    5
}

/// Default maximum HTTP response body size in bytes (128KB)
fn default_max_response_body_size() -> usize {
    131_072
}

/// Deserialize environment variable string into SenderFilterPolicy
fn deserialize_sender_filter_policy<'de, D>(
    deserializer: D,
) -> Result<Option<SenderFilterPolicy>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.map(|policy| SenderFilterPolicy::from_policy(&policy)))
}

#[derive(Deserialize, Clone)]
pub struct Params {
    #[serde(default)]
    pub insecure_mode: bool,
    pub discord_token: String,
    pub http_endpoint: String,

    // HTTP Client Configuration
    #[serde(default = "default_http_timeout")]
    pub http_timeout: u64,
    #[serde(default = "default_http_connect_timeout")]
    pub http_connect_timeout: u64,
    #[serde(default = "default_max_response_body_size")]
    pub max_response_body_size: usize,

    // Action Execution Configuration
    #[serde(default = "default_max_actions")]
    pub max_actions: usize,

    // ========================================
    // Event Configuration
    // ========================================
    // Direct Message Events
    #[serde(default, deserialize_with = "deserialize_sender_filter_policy")]
    pub message_direct: Option<SenderFilterPolicy>,

    // Guild Events
    #[serde(default, deserialize_with = "deserialize_sender_filter_policy")]
    pub message_guild: Option<SenderFilterPolicy>,

    // Message Delete Events
    #[serde(default)]
    pub message_delete_direct: Option<String>,
    #[serde(default)]
    pub message_delete_guild: Option<String>,
    #[serde(default)]
    pub message_delete_bulk_guild: Option<String>,

    // Message Update Events
    #[serde(default)]
    pub message_update_direct: Option<String>,
    #[serde(default)]
    pub message_update_guild: Option<String>,

    // Reaction Add Events
    #[serde(default, deserialize_with = "deserialize_sender_filter_policy")]
    pub reaction_add_direct: Option<SenderFilterPolicy>,
    #[serde(default, deserialize_with = "deserialize_sender_filter_policy")]
    pub reaction_add_guild: Option<SenderFilterPolicy>,

    // Reaction Remove Events
    #[serde(default, deserialize_with = "deserialize_sender_filter_policy")]
    pub reaction_remove_direct: Option<SenderFilterPolicy>,
    #[serde(default, deserialize_with = "deserialize_sender_filter_policy")]
    pub reaction_remove_guild: Option<SenderFilterPolicy>,

    // Context-Independent Events
    #[serde(default)]
    pub ready: Option<String>,
    #[serde(default)]
    pub resumed: Option<String>,
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
            .field("http_timeout", &self.http_timeout)
            .field("http_connect_timeout", &self.http_connect_timeout)
            .field("max_response_body_size", &self.max_response_body_size)
            .field("max_actions", &self.max_actions)
            .field("message_direct", &self.message_direct)
            .field("message_guild", &self.message_guild)
            .field("message_delete_direct", &self.message_delete_direct)
            .field("message_delete_guild", &self.message_delete_guild)
            .field("message_delete_bulk_guild", &self.message_delete_bulk_guild)
            .field("message_update_direct", &self.message_update_direct)
            .field("message_update_guild", &self.message_update_guild)
            .field("reaction_add_direct", &self.reaction_add_direct)
            .field("reaction_add_guild", &self.reaction_add_guild)
            .field("reaction_remove_direct", &self.reaction_remove_direct)
            .field("reaction_remove_guild", &self.reaction_remove_guild)
            .field("ready", &self.ready)
            .field("resumed", &self.resumed)
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

    /// Check if Direct Reaction Add events are enabled
    pub fn has_direct_reaction_add_events(&self) -> bool {
        self.reaction_add_direct.is_some()
    }

    /// Check if Guild Reaction Add events are enabled
    pub fn has_guild_reaction_add_events(&self) -> bool {
        self.reaction_add_guild.is_some()
    }

    /// Check if Direct Reaction Remove events are enabled
    pub fn has_direct_reaction_remove_events(&self) -> bool {
        self.reaction_remove_direct.is_some()
    }

    /// Check if Guild Reaction Remove events are enabled
    pub fn has_guild_reaction_remove_events(&self) -> bool {
        self.reaction_remove_guild.is_some()
    }

    /// Check if any MESSAGE_DELETE events are enabled
    pub fn has_message_delete_events(&self) -> bool {
        self.message_delete_direct.is_some() || self.message_delete_guild.is_some()
    }

    /// Check if MESSAGE_DELETE_BULK event is enabled
    pub fn has_message_delete_bulk_events(&self) -> bool {
        self.message_delete_bulk_guild.is_some()
    }

    /// Check if any MESSAGE_UPDATE events are enabled
    pub fn has_message_update_events(&self) -> bool {
        self.message_update_direct.is_some() || self.message_update_guild.is_some()
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
            http_timeout: default_http_timeout(),
            http_connect_timeout: default_http_connect_timeout(),
            max_response_body_size: default_max_response_body_size(),
            max_actions: default_max_actions(),
            message_direct: None,
            message_guild: None,
            message_delete_direct: None,
            message_delete_guild: None,
            message_delete_bulk_guild: None,
            message_update_direct: None,
            message_update_guild: None,
            reaction_add_direct: None,
            reaction_add_guild: None,
            reaction_remove_direct: None,
            reaction_remove_guild: None,
            ready: None,
            resumed: None,
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
