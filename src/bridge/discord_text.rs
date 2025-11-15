//! Discord text processing utilities
//!
//! This module provides functions to handle Discord API text length limitations:
//! - Message content: 2000 characters maximum
//! - Thread names: 100 characters maximum
//!
//! All functions properly handle Unicode characters (multibyte) by counting
//! characters rather than bytes.

use tracing::warn;

/// Truncate content to Discord's 2000 character limit
///
/// If content exceeds limit, truncates to 1997 chars and appends "..."
/// Logs warning with original and truncated length.
pub fn truncate_content(content: &str) -> String {
    const MAX_LEN: usize = 2000;

    let char_count = content.chars().count();

    if char_count > MAX_LEN {
        let truncated: String = content.chars().take(MAX_LEN - 3).collect();
        let result = format!("{}...", truncated);

        warn!(
            original_len = char_count,
            truncated_len = result.chars().count(),
            "Content exceeds 2000 chars, truncated"
        );

        result
    } else {
        content.to_string()
    }
}

/// Truncate thread name to Discord's 100 character limit
///
/// If name exceeds limit, truncates to 100 chars.
pub fn truncate_thread_name(name: &str) -> String {
    const MAX_LEN: usize = 100; // Discord API maximum

    let char_count = name.chars().count();

    if char_count <= MAX_LEN {
        name.to_string()
    } else {
        // Truncate to API limit
        name.chars().take(MAX_LEN).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // Tests for truncate_content

    #[rstest]
    #[case("", "")]                           // Empty string
    #[case("Hello", "Hello")]                 // Short string
    fn test_truncate_content_no_truncation(#[case] input: &str, #[case] expected: &str) {
        let result = truncate_content(input);
        assert_eq!(result, expected);
        assert_eq!(result.chars().count(), expected.chars().count());
    }

    #[test]
    fn test_truncate_content_exactly_2000_chars() {
        let content = "a".repeat(2000);
        let result = truncate_content(&content);

        assert_eq!(result, content);
        assert_eq!(result.chars().count(), 2000);
    }

    #[test]
    fn test_truncate_content_truncates_long_content() {
        let long_content = "a".repeat(2100);
        let result = truncate_content(&long_content);

        assert_eq!(result.chars().count(), 2000);
        assert!(result.ends_with("..."));
        assert_eq!(&result[..result.len() - 3], &"a".repeat(1997));
    }

    #[test]
    fn test_truncate_content_handles_multibyte_chars() {
        // 2001 characters with emoji (multibyte)
        let content = format!("{}{}", "あ".repeat(1999), "🎉🎉");
        let result = truncate_content(&content);

        assert_eq!(result.chars().count(), 2000);
        assert!(result.ends_with("..."));
    }

    // Tests for truncate_thread_name

    #[rstest]
    #[case("", "")]                           // Empty string
    #[case("Thread", "Thread")]               // Short name
    fn test_truncate_thread_name_no_truncation(#[case] input: &str, #[case] expected: &str) {
        let result = truncate_thread_name(input);
        assert_eq!(result, expected);
        assert_eq!(result.chars().count(), expected.chars().count());
    }

    #[test]
    fn test_truncate_thread_name_exactly_100_chars() {
        let name = "a".repeat(100);
        let result = truncate_thread_name(&name);

        assert_eq!(result, name);
        assert_eq!(result.chars().count(), 100);
    }

    #[test]
    fn test_truncate_thread_name_truncates_long_name() {
        let long_name = "a".repeat(150);
        let result = truncate_thread_name(&long_name);

        assert_eq!(result.chars().count(), 100);
        assert_eq!(result, "a".repeat(100));
    }

    #[test]
    fn test_truncate_thread_name_handles_multibyte_chars() {
        // 120 characters with emoji
        let name = format!("{}{}", "あ".repeat(100), "🎉".repeat(20));
        let result = truncate_thread_name(&name);

        assert_eq!(result.chars().count(), 100);
    }

}
