//! Content extraction algorithms for Trafilatura Rust port.
//! This module implements various extraction strategies to identify the main content.

use scraper::{Html, Selector, ElementRef};
use lazy_static::lazy_static;
use log::debug;

use crate::{ExtractionConfig, TrafilaturaError};
use crate::html::{clean_html, get_text_content, has_class_hint, has_id_hint};

lazy_static! {
    /// Content element hints - classes that suggest main content
    static ref CONTENT_CLASSES: Vec<&'static str> = vec![
        "article", "post", "content", "entry", "main", "text", "blog", "story", "body",
        "column", "section", "post-content", "main-content", "article-content"
    ];

    /// Content element hints - IDs that suggest main content
    static ref CONTENT_IDS: Vec<&'static str> = vec![
        "article", "post", "content", "entry", "main", "text", "blog", "story", "body",
        "column", "section", "post-content", "main-content", "article-content"
    ];

    /// Tag weights for scoring potential content containers
    static ref TAG_WEIGHTS: Vec<(&'static str, i32)> = vec![
        ("div", 5),
        ("p", 10),
        ("h1", 5),
        ("h2", 5),
        ("h3", 3),
        ("h4", 2),
        ("h5", 1),
        ("article", 15),
        ("section", 10),
        ("main", 15),
        ("content", 15),
        ("li", 1),
        ("td", 1),
        ("a", -2),
        ("script", -20),
        ("style", -20),
        ("header", -10),
        ("footer", -10),
        ("nav", -10),
        ("aside", -10),
        ("iframe", -15),
        ("form", -10),
        ("button", -5),
    ];

    /// Boilerplate link density threshold - links above this ratio are likely navigation
    static ref LINK_DENSITY_THRESHOLD: f64 = 0.5;
}

/// Extract content from a document using multiple strategies
pub fn extract_content(document: &Html, config: &ExtractionConfig) -> Result<String, TrafilaturaError> {
    // First clean the document
    let cleaned_document = clean_html(document, config)?;
    
    // Try to extract content using different strategies in order
    
    // 1. Try with article tag
    let article_selector = Selector::parse("article").unwrap();
    if let Some(article) = cleaned_document.select(&article_selector).next() {
        let text = get_text_content(&article, config);
        if !text.is_empty() && text.len() >= config.min_extracted_size {
            debug!("Content extracted using article tag strategy");
            return Ok(text);
        }
    }
    
    // 2. Try with content hints
    if let Some(content) = extract_by_hints(&cleaned_document, config) {
        if !content.is_empty() && content.len() >= config.min_extracted_size {
            debug!("Content extracted using hints strategy");
            return Ok(content);
        }
    }
    
    // 3. Try with content density
    if let Some(content) = extract_by_density(&cleaned_document, config) {
        if !content.is_empty() && content.len() >= config.min_extracted_size {
            debug!("Content extracted using density strategy");
            return Ok(content);
        }
    }
    
    // 4. Just extract <p> tags as fallback
    let p_selector = Selector::parse("p").unwrap();
    let paragraphs = cleaned_document.select(&p_selector);
    let mut text = String::new();
    
    for paragraph in paragraphs {
        let paragraph_text = get_text_content(&paragraph, config);
        if !paragraph_text.trim().is_empty() {
            text.push_str(&paragraph_text);
            text.push('\n');
        }
    }
    
    debug!("Content extracted using paragraphs fallback strategy");
    Ok(text.trim().to_string())
}

/// Extract content based on class and ID hints
fn extract_by_hints(document: &Html, config: &ExtractionConfig) -> Option<String> {
    // Try to find elements with content class hints
    for class_hint in CONTENT_CLASSES.iter() {
        let selector = Selector::parse(&format!("[class*='{}']", class_hint)).unwrap();
        if let Some(element) = document.select(&selector).next() {
            let text = get_text_content(&element, config);
            if !text.is_empty() && text.len() >= config.min_extracted_size {
                return Some(text);
            }
        }
    }
    
    // Try to find elements with content ID hints
    for id_hint in CONTENT_IDS.iter() {
        let selector = Selector::parse(&format!("[id*='{}']", id_hint)).unwrap();
        if let Some(element) = document.select(&selector).next() {
            let text = get_text_content(&element, config);
            if !text.is_empty() && text.len() >= config.min_extracted_size {
                return Some(text);
            }
        }
    }
    
    None
}

/// Extract content based on text density
fn extract_by_density(document: &Html, config: &ExtractionConfig) -> Option<String> {
    // Find all potential content containers
    let candidates = find_content_candidates(document);
    
    // If we found candidates, return the best one
    if !candidates.is_empty() {
        let mut best_candidate = &candidates[0];
        let mut best_score = score_node(best_candidate, config);
        
        for candidate in &candidates[1..] {
            let score = score_node(candidate, config);
            if score > best_score {
                best_candidate = candidate;
                best_score = score;
            }
        }
        
        let text = get_text_content(best_candidate, config);
        if !text.is_empty() {
            return Some(text);
        }
    }
    
    None
}

/// Find potential content containers in the document
fn find_content_candidates(document: &Html) -> Vec<ElementRef> {
    let mut candidates = Vec::new();
    
    // Look for common content containers
    for &tag in &["article", "section", "main", "div", "body"] {
        let selector = Selector::parse(tag).unwrap();
        for element in document.select(&selector) {
            // Skip elements that are likely navigation or footer
            if has_class_hint(&element, &["nav", "navigation", "menu", "footer", "header", "sidebar"]) {
                continue;
            }
            
            if has_id_hint(&element, &["nav", "navigation", "menu", "footer", "header", "sidebar"]) {
                continue;
            }
            
            // Check if this element has enough text content
            let text_length = element.text().collect::<String>().len();
            if text_length > 100 {
                candidates.push(element);
            }
        }
    }
    
    candidates
}

/// Score a node based on its content
fn score_node(element: &ElementRef, _config: &ExtractionConfig) -> i32 {
    let mut score = 0;
    
    // Score based on text length
    let text_content: String = element.text().collect();
    score += (text_content.len() / 25) as i32;
    
    // Bonus for content class/id hints
    if has_class_hint(element, &CONTENT_CLASSES) {
        score += 50;
    }
    
    if has_id_hint(element, &CONTENT_IDS) {
        score += 50;
    }
    
    // Score based on child elements
    let all_selector = Selector::parse("*").unwrap();
    for child in element.select(&all_selector) {
        let tag_name = child.value().name();
        
        // Add weight based on tag
        for &(tag, weight) in TAG_WEIGHTS.iter() {
            if tag_name == tag {
                score += weight;
                break;
            }
        }
    }
    
    // Penalize for high link density
    let link_density = calculate_link_density(element);
    if link_density > *LINK_DENSITY_THRESHOLD {
        score -= (link_density * 100.0) as i32;
    }
    
    score
}

/// Calculate the link density of a node (text in links / total text)
fn calculate_link_density(element: &ElementRef) -> f64 {
    let total_text_length = element.text().collect::<String>().len();
    
    if total_text_length == 0 {
        return 0.0;
    }
    
    let a_selector = Selector::parse("a").unwrap();
    let links = element.select(&a_selector);
    let mut link_text_length = 0;
    
    for link in links {
        link_text_length += link.text().collect::<String>().len();
    }
    
    link_text_length as f64 / total_text_length as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_extract_content_with_article() {
        let html = r#"<html><body><article><h1>Title</h1><p>Main content paragraph.</p></article><div>Some other content</div></body></html>"#;
        let document = Html::parse_document(html);
        let config = ExtractionConfig::default();
        
        let content = extract_content(&document, &config).unwrap();
        
        assert!(content.contains("Title"));
        assert!(content.contains("Main content paragraph"));
    }

    #[test]
    fn test_extract_content_with_hints() {
        let html = r#"<html><body><div class="content"><h1>Title</h1><p>Main content paragraph.</p></div><div>Some other content</div></body></html>"#;
        let document = Html::parse_document(html);
        let config = ExtractionConfig::default();
        
        let content = extract_content(&document, &config).unwrap();
        
        assert!(content.contains("Title"));
        assert!(content.contains("Main content paragraph"));
    }

    #[test]
    fn test_calculate_link_density() {
        let html = "<div>This is a <a href=\"#\">link</a> in some text.</div>";
        let document = Html::parse_document(html);
        
        let div_selector = Selector::parse("div").unwrap();
        let div = document.select(&div_selector).next().unwrap();
        let density = calculate_link_density(&div);
        
        // Link text "link" is 4 chars, total text is "This is a link in some text." (27 chars)
        assert!((density - 4.0/27.0).abs() < 0.01);
    }
}