//! Readability implementation for Trafilatura Rust port.
//! This module provides a fallback extraction method based on a simplified readability algorithm.

use scraper::{Html, Selector, ElementRef};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

use crate::{ExtractionConfig, TrafilaturaError};
use crate::html::{get_text_content, clean_html};

lazy_static! {
    /// Regex to match unlikely content candidates
    static ref UNLIKELY_CANDIDATES_RE: Regex = Regex::new(
        r"(?i)combx|comment|community|disqus|extra|foot|header|menu|remark|rss|shoutbox|sidebar|sponsor|ad-break|agegate|pagination|pager|popup|tweet|twitter|social|share"
    ).unwrap();
    
    /// Regex to match positive candidates
    static ref POSITIVE_CANDIDATES_RE: Regex = Regex::new(
        r"(?i)article|body|content|entry|hentry|main|page|pagination|post|text|blog|story"
    ).unwrap();
    
    /// Regex to match negative candidates
    static ref NEGATIVE_CANDIDATES_RE: Regex = Regex::new(
        r"(?i)hidden|^hid$|combx|comment|com-|contact|foot|footer|footnote|masthead|media|meta|outbrain|promo|related|scroll|shoutbox|sidebar|sponsor|shopping|tags|tool|widget"
    ).unwrap();
}

/// Extract content using readability algorithm
pub fn extract_with_readability(document: &Html, config: &ExtractionConfig) -> Result<String, TrafilaturaError> {
    // Clean the document first
    let working_document = clean_html(document, config)?;
    
    // Find all paragraphs
    let p_selector = Selector::parse("p").unwrap();
    let paragraphs: Vec<ElementRef> = working_document.select(&p_selector).collect();
    
    if paragraphs.is_empty() {
        return Ok(String::new());
    }
    
    // Score paragraphs
    let mut paragraph_scores: HashMap<String, f64> = HashMap::new();
    for paragraph in &paragraphs {
        let text = paragraph.text().collect::<String>();
        let words = text.split_whitespace().count();
        
        if words < 20 {
            continue;
        }
        
        // Find parent element to score
        let parent_selector = Selector::parse("body").unwrap();
        if let Some(_parent) = working_document.select(&parent_selector).next() {
            // Use a simple string identifier for the body element
            let parent_id = "body_element".to_string();
            let score = paragraph_scores.entry(parent_id).or_insert(0.0);
            *score += words as f64 / 20.0;
        }
    }
    
    // Find the top-scoring parent
    let mut top_parent = None;
    let mut top_score = 0.0;
    
    for (_parent_hash, score) in &paragraph_scores {
        if *score > top_score {
            top_score = *score;
            
            // Find the parent by hash
            // Note: In a real implementation, we would need a better way to find
            // the element by hash. This is just a placeholder.
            let body_selector = Selector::parse("body").unwrap();
            if let Some(body) = working_document.select(&body_selector).next() {
                top_parent = Some(body);
            }
        }
    }
    
    // Extract content from top parent
    if let Some(parent) = top_parent {
        let content = get_text_content(&parent, config);
        Ok(content)
    } else {
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readability_extraction() {
        let html = r#"
        <html>
            <body>
                <div class="header">Site Header</div>
                <div class="content">
                    <h1>Article Title</h1>
                    <p>This is a long paragraph with enough text to meet the minimum word count threshold. 
                    It contains meaningful content that should be extracted by the readability algorithm.
                    The algorithm should recognize this as the main content of the page and score it highly.</p>
                    <p>This is another paragraph with more meaningful content that contributes to the overall
                    score of the parent element. Together with the previous paragraph, it should help identify
                    this div as the main content container.</p>
                </div>
                <div class="footer">Site Footer</div>
            </body>
        </html>
        "#;
        
        let document = Html::parse_document(html);
        let config = ExtractionConfig::default();
        
        let content = extract_with_readability(&document, &config).unwrap();
        
        assert!(content.contains("Article Title"));
        assert!(content.contains("long paragraph"));
        assert!(content.contains("main content"));
        assert!(content.contains("another paragraph"));
    }
}