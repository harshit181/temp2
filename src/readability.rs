//! Readability algorithm implementation for content extraction.
//! Based on the Mozilla Readability algorithm used in Firefox Reader Mode.

use kuchiki::NodeRef;
use regex::Regex;
use lazy_static::lazy_static;
use log::debug;

use crate::{ExtractionConfig, TrafilaturaError};
use crate::html::{clean_html, get_text_content, has_class_hint, has_id_hint};

lazy_static! {
    /// Positive indicators for content based on class/id
    static ref POSITIVE_INDICATORS: Vec<&'static str> = vec![
        "article", "body", "content", "entry", "main", "page", "post", 
        "text", "blog", "story", "container", "readable"
    ];

    /// Negative indicators for non-content based on class/id
    static ref NEGATIVE_INDICATORS: Vec<&'static str> = vec![
        "advert", "ad-", "banner", "combx", "comment", "community", "disqus",
        "extra", "foot", "header", "menu", "meta", "nav", "popup", "related",
        "remark", "rss", "share", "shoutbox", "sidebar", "sponsor", "shopping",
        "widget", "hidden", "js", "modal", "login"
    ];

    /// Regex patterns for unlikely content (from Readability)
    static ref UNLIKELY_PATTERNS: Regex = Regex::new(
        r"(?i)banner|breadcrumbs|combx|comment|community|cover-wrap|disqus|extra|foot|header|legends|menu|related|remark|replies|rss|shoutbox|sidebar|skyscraper|social|sponsor|supplemental|ad-break|agegate|pagination|pager|popup|yom-remote"
    ).unwrap();

    /// Regex patterns for likely content (from Readability)
    static ref LIKELY_PATTERNS: Regex = Regex::new(
        r"(?i)article|body|content|entry|main|news|pag(?:e|ination)|post|text|blog|story"
    ).unwrap();

    /// Regex for empty nodes
    static ref EMPTY_NODE_RE: Regex = Regex::new(r"^\s*$").unwrap();
}

/// Extract content using readability algorithm
pub fn extract_with_readability(document: &NodeRef, config: &ExtractionConfig) -> Result<String, TrafilaturaError> {
    // First clean the document
    let cleaned_document = clean_html(document, config)?;
    
    // Create a clone to work with
    let working_document = cleaned_document.clone();
    
    // Prepare the document by removing unlikely candidates
    prepare_document(&working_document);
    
    // Find all paragraphs
    let paragraphs = working_document.select("p").unwrap();
    
    // Score paragraphs and their parent nodes
    let mut candidates = Vec::new();
    
    for paragraph in paragraphs {
        let paragraph_node = paragraph.as_node();
        let paragraph_text = paragraph_node.text_contents();
        
        // Skip if too short
        if paragraph_text.len() < 25 {
            continue;
        }
        
        // Find parent to score
        let mut parent = paragraph_node.parent().and_then(|p| p.into_node_ref());
        if parent.is_none() {
            continue;
        }
        
        // Score the paragraph's parent
        let parent_node = parent.unwrap();
        
        // Add to candidates if not already there
        if !candidates.iter().any(|(node, _)| node.address() == parent_node.address()) {
            let score = score_node(&parent_node);
            candidates.push((parent_node, score));
        }
    }
    
    // Find the best candidate
    if candidates.is_empty() {
        // Fallback: use the body
        let body = working_document.select_first("body").map_err(|_| {
            TrafilaturaError::ExtractionError("No body element found".to_string())
        })?;
        
        let text = get_text_content(body.as_node(), config);
        return Ok(text);
    }
    
    // Sort candidates by score
    candidates.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    
    // Get the best candidate
    let (best_candidate, _) = &candidates[0];
    
    // Get the article text
    let text = get_text_content(best_candidate, config);
    
    Ok(text)
}

/// Prepare document by removing unlikely candidates
fn prepare_document(document: &NodeRef) {
    // Remove unlikely candidates
    let mut nodes_to_remove = Vec::new();
    
    if let Ok(elements) = document.select("*") {
        for element in elements {
            let node = element.as_node();
            
            // Skip nodes that are certain elements we want to keep
            if let Ok(element_data) = node.as_element() {
                let name = element_data.name.local.to_string();
                if ["html", "body", "article", "section", "main"].contains(&name.as_str()) {
                    continue;
                }
            }
            
            // Check attributes for unlikeliness
            if let Ok(element_ref) = node.as_element() {
                let element = element_ref.attributes.borrow();
                
                if let Some(class) = element.get("class") {
                    if UNLIKELY_PATTERNS.is_match(class) && !LIKELY_PATTERNS.is_match(class) {
                        nodes_to_remove.push(node.clone());
                        continue;
                    }
                }
                
                if let Some(id) = element.get("id") {
                    if UNLIKELY_PATTERNS.is_match(id) && !LIKELY_PATTERNS.is_match(id) {
                        nodes_to_remove.push(node.clone());
                        continue;
                    }
                }
            }
        }
    }
    
    // Remove the nodes
    for node in nodes_to_remove {
        if let Some(parent) = node.parent() {
            parent.children().remove_from_parent(&node);
        }
    }
}

/// Score a node based on its content and attributes
fn score_node(node: &NodeRef) -> f64 {
    let mut score = 1.0;
    
    // Get the tag name
    let tag_name = match node.as_element() {
        Ok(element) => element.name.local.to_string(),
        Err(_) => return 0.0,
    };
    
    // Adjust score based on tag
    match tag_name.as_str() {
        "div" => score += 5.0,
        "article" | "section" | "main" => score += 10.0,
        "p" => score += 3.0,
        "pre" | "td" | "blockquote" => score += 3.0,
        _ => {}
    }
    
    // Check class and id for indicators
    if has_class_hint(node, &POSITIVE_INDICATORS) {
        score += 25.0;
    }
    
    if has_id_hint(node, &POSITIVE_INDICATORS) {
        score += 25.0;
    }
    
    if has_class_hint(node, &NEGATIVE_INDICATORS) {
        score -= 25.0;
    }
    
    if has_id_hint(node, &NEGATIVE_INDICATORS) {
        score -= 25.0;
    }
    
    // Text density
    let text_length = node.text_contents().len();
    score += text_length as f64 / 100.0;
    
    // Adjust score based on link density
    let link_density = calculate_link_density(node);
    score *= (1.0 - link_density);
    
    score
}

/// Calculate the link density of a node (text in links / total text)
fn calculate_link_density(node: &NodeRef) -> f64 {
    let total_text_length = node.text_contents().len();
    
    if total_text_length == 0 {
        return 0.0;
    }
    
    let links = node.select("a").unwrap();
    let mut link_text_length = 0;
    
    for link in links {
        link_text_length += link.as_node().text_contents().len();
    }
    
    link_text_length as f64 / total_text_length as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuchiki::parse_html;

    #[test]
    fn test_readability_extraction() {
        let html = r#"
        <html>
            <body>
                <div id="header">Header content</div>
                <div id="content">
                    <h1>Article Title</h1>
                    <p>This is the main content of the article with a decent amount of text to make it be selected as the main content.</p>
                    <p>Another paragraph with more meaningful content that should be extracted properly by the readability algorithm.</p>
                </div>
                <div id="sidebar">Sidebar content</div>
                <div id="footer">Footer content</div>
            </body>
        </html>
        "#;
        
        let document = parse_html().one(html);
        let config = ExtractionConfig::default();
        
        let content = extract_with_readability(&document, &config).unwrap();
        
        assert!(content.contains("Article Title"));
        assert!(content.contains("main content of the article"));
        assert!(!content.contains("Sidebar content"));
        assert!(!content.contains("Footer content"));
    }

    #[test]
    fn test_score_node() {
        let html = r#"<article class="content"><p>Content paragraph.</p></article>"#;
        let document = parse_html().one(html);
        
        let article = document.select_first("article").unwrap();
        let score = score_node(article.as_node());
        
        // Should have a high score due to article tag and content class
        assert!(score > 30.0);
    }
}
