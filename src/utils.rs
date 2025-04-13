//! Utility functions for the Trafilatura Rust port.
//! This module provides helper functions for URL handling, text cleaning, etc.

use regex::Regex;
use lazy_static::lazy_static;
use url::Url;

use crate::TrafilaturaError;

lazy_static! {
    /// Regex to clean up whitespace
    static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
    
    /// Regex to detect if a string is a URL
    static ref URL_RE: Regex = Regex::new(r"^https?://").unwrap();
    
    /// Regex to normalize line breaks
    static ref NEWLINES_RE: Regex = Regex::new(r"\r\n?").unwrap();
}

/// Normalize whitespace in a string
pub fn normalize_whitespace(text: &str) -> String {
    WHITESPACE_RE.replace_all(text.trim(), " ").to_string()
}

/// Normalize line breaks in a string
pub fn normalize_newlines(text: &str) -> String {
    NEWLINES_RE.replace_all(text, "\n").to_string()
}

/// Check if a string is a URL
pub fn is_url(text: &str) -> bool {
    URL_RE.is_match(text)
}

/// Create an absolute URL from a base URL and a potentially relative URL
pub fn make_absolute_url(base: &str, relative: &str) -> Result<String, TrafilaturaError> {
    if is_url(relative) {
        return Ok(relative.to_string());
    }
    
    let base_url = Url::parse(base)
        .map_err(|e| TrafilaturaError::UrlError(e))?;
    
    let absolute_url = base_url.join(relative)
        .map_err(|e| TrafilaturaError::UrlError(e))?;
    
    Ok(absolute_url.to_string())
}

/// Clean up a filename by removing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    // Replace characters that are invalid in filenames
    let invalid_chars = r#"/\:*?"<>|"#;
    let mut result = filename.to_string();
    
    for c in invalid_chars.chars() {
        result = result.replace(c, "_");
    }
    
    result
}

/// Determine if a string is likely HTML
pub fn is_html(text: &str) -> bool {
    text.contains("<html") || text.contains("<!DOCTYPE") || 
    text.contains("<body") || text.contains("<div") || 
    text.contains("<p>") || text.contains("<span")
}

/// Get file extension from URL or filename
pub fn get_file_extension(path: &str) -> Option<String> {
    let path = path.split('?').next().unwrap_or(path);
    let parts: Vec<&str> = path.rsplitn(2, '.').collect();
    
    if parts.len() > 1 {
        let ext = parts[0].to_lowercase();
        // Check if it's a reasonable extension length
        if ext.len() <= 10 {
            return Some(ext);
        }
    }
    
    None
}

/// Truncate a string to a maximum length, preserving word boundaries
pub fn truncate_string(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }
    
    let mut truncated = text[..max_length].to_string();
    
    // Find the last space to truncate at word boundary
    if let Some(last_space) = truncated.rfind(' ') {
        truncated.truncate(last_space);
    }
    
    truncated + "..."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace() {
        let text = "  This  has \t multiple    spaces  ";
        let normalized = normalize_whitespace(text);
        assert_eq!(normalized, "This has multiple spaces");
    }
    
    #[test]
    fn test_is_url() {
        assert!(is_url("https://example.com"));
        assert!(is_url("http://example.com/path"));
        assert!(!is_url("example.com"));
        assert!(!is_url("just some text"));
    }
    
    #[test]
    fn test_make_absolute_url() {
        let base = "https://example.com/page";
        
        // Absolute URL should remain unchanged
        let absolute = "https://another.com/path";
        assert_eq!(make_absolute_url(base, absolute).unwrap(), absolute);
        
        // Relative URL should be resolved
        let relative = "/images/photo.jpg";
        assert_eq!(make_absolute_url(base, relative).unwrap(), "https://example.com/images/photo.jpg");
    }
    
    #[test]
    fn test_sanitize_filename() {
        let filename = "file/with:invalid*chars?.jpg";
        let sanitized = sanitize_filename(filename);
        assert_eq!(sanitized, "file_with_invalid_chars_.jpg");
    }
    
    #[test]
    fn test_is_html() {
        assert!(is_html("<html><body>content</body></html>"));
        assert!(is_html("<!DOCTYPE html><div>content</div>"));
        assert!(!is_html("This is plain text"));
    }
    
    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("image.jpg"), Some("jpg".to_string()));
        assert_eq!(get_file_extension("document.pdf?version=1"), Some("pdf".to_string()));
        assert_eq!(get_file_extension("file"), None);
    }
    
    #[test]
    fn test_truncate_string() {
        let text = "This is a long string that should be truncated";
        assert_eq!(truncate_string(text, 10), "This is...");
        assert_eq!(truncate_string("Short", 10), "Short");
    }
}
