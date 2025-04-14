//! # Trafilatura Rust Port
//!
//! A Rust implementation of Python's trafilatura library for extracting text content from web pages.
//! This library provides functionality to extract the main content from HTML documents,
//! removing boilerplate, navigation, and other non-content elements.

pub mod cli;
pub mod extractors;
pub mod html;
pub mod metadata;
pub mod readability;
pub mod utils;
pub mod xpath;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use reqwest::blocking::Client;
use scraper::Html;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum TrafilaturaError {
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("HTML parsing error: {0}")]
    ParsingError(String),

    #[error("Extraction error: {0}")]
    ExtractionError(String),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("CSS selector error: {0}")]
    SelectorError(String),
}

/// Output format options for extracted content
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Html,
    Json,
    Xml,
}

/// Configuration options for extraction
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Include comments in the extraction
    pub include_comments: bool,
    /// Include tables in the extraction
    pub include_tables: bool,
    /// Include links in the extraction
    pub include_links: bool,
    /// Include images in the extraction
    pub include_images: bool,
    /// Output format
    pub output_format: OutputFormat,
    /// Extraction fallback order
    pub extraction_timeout: u64,
    /// Min extracted text length to be considered valid
    pub min_extracted_size: usize,
    /// Whether to extract metadata
    pub extract_metadata: bool,
    /// User agent string for HTTP requests
    pub user_agent: String,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            include_comments: false,
            include_tables: true,
            include_links: true,
            include_images: false,
            output_format: OutputFormat::Text,
            extraction_timeout: 30,
            min_extracted_size: 250,
            extract_metadata: false,
            user_agent: "Mozilla/5.0 (compatible; trafilatura-rs/0.1; +https://github.com/user/trafilatura-rs)".into(),
        }
    }
}

/// Extraction result containing the main content and optional metadata
#[derive(Debug, Clone, Default)]
pub struct ExtractionResult {
    /// Main content
    pub content: String,
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document date
    pub date: Option<String>,
    /// Document URL
    pub url: Option<String>,
    /// Document description
    pub description: Option<String>,
    /// Document sitename
    pub sitename: Option<String>,
    /// Document categories/tags
    pub categories: Vec<String>,
}

/// Extract text from a URL
pub fn extract_url(url: &str, config: &ExtractionConfig) -> Result<ExtractionResult, TrafilaturaError> {
    let url = Url::parse(url)?;
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(config.extraction_timeout))
        .user_agent(&config.user_agent)
        .build()?;
    
    let response = client.get(url.clone()).send()?;
    
    if !response.status().is_success() {
        return Err(TrafilaturaError::RequestError(reqwest::Error::from(
            response.error_for_status().unwrap_err()
        )));
    }
    
    let html = response.text()?;
    let mut result = extract_html(&html, config)?;
    
    // Set the URL in the result
    result.url = Some(url.to_string());
    
    Ok(result)
}

/// Extract text from a local HTML file
pub fn extract_file<P: AsRef<Path>>(path: P, config: &ExtractionConfig) -> Result<ExtractionResult, TrafilaturaError> {
    let mut file = File::open(path)?;
    let mut html = String::new();
    file.read_to_string(&mut html)?;
    
    extract_html(&html, config)
}

/// Extract text from an HTML string
pub fn extract_html(html: &str, config: &ExtractionConfig) -> Result<ExtractionResult, TrafilaturaError> {
    let document = Html::parse_document(html);
    
    let mut result = ExtractionResult::default();
    
    // Extract metadata if configured
    if config.extract_metadata {
        result = metadata::extract_metadata(&document, result)?;
    }
    
    // First try using XPath-based extraction (similar to Python trafilatura)
    let xpath_content = xpath::extract_with_xpath(html, config)?;
    
    if !xpath_content.is_empty() && xpath_content.len() >= config.min_extracted_size {
        result.content = xpath_content;
        return Ok(result);
    }
    
    // Try original extraction methods as fallback
    let content = extractors::extract_content(&document, config)?;
    
    if content.is_empty() || content.len() < config.min_extracted_size {
        // Try readability algorithm as fallback
        let readability_content = readability::extract_with_readability(&document, config)?;
        
        if !readability_content.is_empty() && readability_content.len() >= config.min_extracted_size {
            result.content = readability_content;
        } else {
            result.content = content;
        }
    } else {
        result.content = content;
    }
    
    // If the content is still too short, return extraction error
    if result.content.is_empty() || result.content.len() < config.min_extracted_size {
        return Err(TrafilaturaError::ExtractionError(
            format!("Extracted content too short: {} chars", result.content.len())
        ));
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_config_default() {
        let config = ExtractionConfig::default();
        assert_eq!(config.include_comments, false);
        assert_eq!(config.include_tables, true);
        assert_eq!(config.include_links, true);
        assert_eq!(config.include_images, false);
        assert_eq!(config.output_format, OutputFormat::Text);
        assert_eq!(config.min_extracted_size, 250);
    }
    
    #[test]
    fn test_xpath_extraction() {
        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Test Page</title>
            <meta property="og:site_name" content="Wikipedia" />
        </head>
        <body>
            <div id="content">
                <h1>Main Heading</h1>
                <div id="mw-content-text">
                    <div class="mw-parser-output">
                        <p>This is the main paragraph of content that should be extracted.</p>
                        <p>This is a second paragraph.</p>
                        <ul>
                            <li>List item 1</li>
                            <li>List item 2</li>
                        </ul>
                        <h2>References</h2>
                        <div class="references">
                            <p>Reference 1</p>
                            <p>Reference 2</p>
                        </div>
                    </div>
                </div>
            </div>
        </body>
        </html>"#;
        
        let config = ExtractionConfig::default();
        let result = extract_html(html, &config).unwrap();
        
        assert!(result.content.contains("Main Heading"));
        assert!(result.content.contains("main paragraph of content"));
        assert!(result.content.contains("second paragraph"));
        assert!(result.content.contains("List item 1"));
        assert!(result.content.contains("List item 2"));
        
        // References should be excluded
        assert!(!result.content.contains("Reference 1"));
    }
}
