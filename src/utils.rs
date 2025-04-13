//! Utility functions for Trafilatura Rust port.
//! This module provides various helper functions for the library.

use scraper::Html;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use url::Url;

use crate::TrafilaturaError;

/// Check if input is a URL
pub fn is_url(input: &str) -> bool {
    Url::parse(input).is_ok()
}

/// Check if input is a file path
pub fn is_file_path(input: &str) -> bool {
    Path::new(input).exists() && Path::new(input).is_file()
}

/// Check if input is likely HTML content
pub fn is_html_content(input: &str) -> bool {
    input.contains("<html") || 
    input.contains("<body") || 
    input.contains("<div") || 
    (input.contains("<") && input.contains(">"))
}

/// Read file content
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, TrafilaturaError> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Normalize HTML content
pub fn normalize_html(html: &str) -> String {
    // Remove excessive whitespace
    let mut cleaned = html.trim().to_string();
    
    // Ensure we have a proper HTML document
    if !cleaned.contains("<html") {
        if cleaned.contains("<body") {
            cleaned = format!("<html>{}</html>", cleaned);
        } else if !cleaned.contains("<head") && !cleaned.contains("<body") {
            cleaned = format!("<html><body>{}</body></html>", cleaned);
        }
    }
    
    cleaned
}

/// Parse HTML content to a document
pub fn parse_html(content: &str) -> Result<Html, TrafilaturaError> {
    let normalized = normalize_html(content);
    let document = Html::parse_document(&normalized);
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_url() {
        assert!(is_url("https://example.com"));
        assert!(is_url("http://example.com/page.html"));
        assert!(!is_url("not-a-url"));
        assert!(!is_url("/path/to/file.html"));
    }
    
    #[test]
    fn test_is_html_content() {
        assert!(is_html_content("<html><body>content</body></html>"));
        assert!(is_html_content("<div>Simple div</div>"));
        assert!(!is_html_content("Plain text without any tags"));
    }
    
    #[test]
    fn test_normalize_html() {
        assert_eq!(
            normalize_html("<div>content</div>"),
            "<html><body><div>content</div></body></html>"
        );
        
        assert_eq!(
            normalize_html("<body><div>content</div></body>"),
            "<html><body><div>content</div></body></html>"
        );
        
        assert_eq!(
            normalize_html("<html><body><div>content</div></body></html>"),
            "<html><body><div>content</div></body></html>"
        );
    }
}