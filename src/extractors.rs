//! Content extraction algorithms for Trafilatura Rust port.
//! This module implements various extraction strategies to identify the main content.

use scraper::{Html, Selector, ElementRef, Element};
use lazy_static::lazy_static;
use log::debug;

use crate::{ExtractionConfig, TrafilaturaError};
use crate::html::{clean_html, get_text_content, has_class_hint, has_id_hint};

lazy_static! {
    /// Content element hints - classes that suggest main content
    static ref CONTENT_CLASSES: Vec<&'static str> = vec![
        "article", "post", "content", "entry", "main", "text", "blog", "story", "body",
        "column", "section", "post-content", "main-content", "article-content",
        "story-content", "story-body", "news-content", "news-article", "news-story",
        "entry-content", "article-body", "article-text", "articleBody", "content-article",
        "post-text", "post-body", "content-text", "content-body", "rich-text", "page-content"
    ];

    /// Content element hints - IDs that suggest main content
    static ref CONTENT_IDS: Vec<&'static str> = vec![
        "article", "post", "content", "entry", "main", "text", "blog", "story", "body",
        "column", "section", "post-content", "main-content", "article-content",
        "story-content", "story-body", "news-content", "news-article", "news-story",
        "entry-content", "article-body", "article-text", "articleBody", "content-article",
        "post-text", "post-body", "content-text", "content-body", "story-text", "page-content"
    ];

    /// Tag weights for scoring potential content containers
    static ref TAG_WEIGHTS: Vec<(&'static str, i32)> = vec![
        ("div", 5),
        ("p", 15),         // Increased paragraph weight
        ("h1", 10),        // Increased heading weights
        ("h2", 8),
        ("h3", 6),
        ("h4", 4),
        ("h5", 3),
        ("article", 25),   // Increased article weight
        ("section", 15),   // Increased section weight
        ("main", 25),      // Increased main weight
        ("content", 20),   // Increased content weight
        ("li", 1),
        ("td", 1),
        ("a", -5),         // Increased penalty for links
        ("script", -50),   // Increased penalties for non-content elements
        ("style", -50),
        ("header", -25),
        ("footer", -50),
        ("nav", -50),
        ("aside", -30),
        ("iframe", -40),
        ("form", -20),
        ("button", -15),
        ("banner", -30),
        ("social", -25),
        ("share", -25),
        ("comment", -25),
        ("advertisement", -50),
        ("meta", -25),
        ("widget", -25),
    ];

    /// Boilerplate link density threshold - links above this ratio are likely navigation
    static ref LINK_DENSITY_THRESHOLD: f64 = 0.33;  // Lowered from 0.5 to be more aggressive at filtering
}

/// Extract content from Wikipedia pages using their specific structure
fn extract_wikipedia_content(document: &Html, _config: &ExtractionConfig) -> Option<String> {
    // Check if this is a Wikipedia page (looking for specific elements or patterns)
    // Wikipedia pages have a specific structure with id="content" and class="mw-parser-output"
    
    // First check for the main content wrapper
    let main_content_selector = Selector::parse("#mw-content-text").unwrap();
    let main_content = document.select(&main_content_selector).next()?;
    
    // Find the parser output div which contains all the article content
    let parser_output_selector = Selector::parse(".mw-parser-output").unwrap();
    let parser_output = main_content.select(&parser_output_selector).next()?;
    
    // Remove unwanted elements specific to Wikipedia
    // - Table of contents
    // - Navigation boxes
    // - Infoboxes
    // - References section
    // - External links section
    // - See also section
    // Prepare text extraction

    // Extract all paragraphs first
    let mut content = String::new();
    
    // Add the title
    let title_selector = Selector::parse("#firstHeading").unwrap();
    if let Some(title) = document.select(&title_selector).next() {
        content.push_str(&title.text().collect::<String>());
        content.push_str("\n\n");
    }
    
    // Process sections and paragraphs
    let section_selector = Selector::parse("h1, h2, h3, h4, h5, h6, p, ul, ol").unwrap();
    let mut skip_section = false;
    
    for element in parser_output.select(&section_selector) {
        let tag_name = element.value().name();
        let element_text = element.text().collect::<String>().trim().to_string();
        
        // Skip empty elements
        if element_text.is_empty() {
            continue;
        }
        
        // Check for heading indicating sections to skip
        if tag_name.starts_with('h') {
            skip_section = element_text == "References" || 
                          element_text == "External links" || 
                          element_text == "See also" || 
                          element_text == "Further reading" ||
                          element_text == "Notes" ||
                          element_text.contains("Bibliography") ||
                          element_text.contains("Sources");
            
            if !skip_section {
                content.push_str(&element_text);
                content.push_str("\n\n");
            }
            continue;
        }
        
        if skip_section {
            continue;
        }
        
        // Process paragraphs and lists
        if tag_name == "p" {
            // Skip very short paragraphs that are likely metadata
            if element_text.len() < 20 && (
                element_text.contains("Redirected from") || 
                element_text.contains("Jump to navigation") || 
                element_text.contains("From Wikipedia")
            ) {
                continue;
            }
            
            content.push_str(&element_text);
            content.push_str("\n\n");
        } else if tag_name == "ul" || tag_name == "ol" {
            // Process lists
            let li_selector = Selector::parse("li").unwrap();
            for li in element.select(&li_selector) {
                let li_text = li.text().collect::<String>().trim().to_string();
                if !li_text.is_empty() {
                    content.push_str("• ");
                    content.push_str(&li_text);
                    content.push_str("\n");
                }
            }
            content.push_str("\n");
        }
    }
    
    if !content.is_empty() {
        Some(content.trim().to_string())
    } else {
        None
    }
}

/// Extract content from a document using multiple strategies
pub fn extract_content(document: &Html, config: &ExtractionConfig) -> Result<String, TrafilaturaError> {
    // First clean the document
    let cleaned_document = clean_html(document, config)?;
    
    // Check if this is a Wikipedia page and use specialized extraction
    if let Some(content) = extract_wikipedia_content(&cleaned_document, config) {
        if !content.is_empty() && content.len() >= config.min_extracted_size {
            debug!("Content extracted using Wikipedia-specific strategy");
            return Ok(content);
        }
    }
    
    // Try to extract content using different strategies in order
    
    // 1. Try with article tag - semantic HTML is the most reliable indicator
    let article_selector = Selector::parse("article").unwrap();
    let articles = cleaned_document.select(&article_selector);
    
    // Find the longest and most content-rich article element
    let mut best_article_text = String::new();
    let mut best_article_score = 0;
    
    for article in articles {
        let text = get_text_content(&article, config);
        if !text.is_empty() && text.len() >= config.min_extracted_size {
            let score = score_node(&article, config);
            if score > best_article_score {
                best_article_text = text;
                best_article_score = score;
            }
        }
    }
    
    if !best_article_text.is_empty() {
        debug!("Content extracted using article tag strategy");
        return Ok(best_article_text);
    }
    
    // 2. Try with content hints - classes and IDs that suggest content
    if let Some(content) = extract_by_hints(&cleaned_document, config) {
        if !content.is_empty() && content.len() >= config.min_extracted_size {
            debug!("Content extracted using hints strategy");
            return Ok(content);
        }
    }
    
    // 3. Try with content density - most reliable fallback
    if let Some(content) = extract_by_density(&cleaned_document, config) {
        if !content.is_empty() && content.len() >= config.min_extracted_size {
            debug!("Content extracted using density strategy");
            return Ok(content);
        }
    }
    
    // 4. Extract paragraphs as fallback, but be more selective
    let mut paragraphs = Vec::new();
    
    // Get all paragraphs
    let p_selector = Selector::parse("p").unwrap();
    for p in cleaned_document.select(&p_selector) {
        // Skip very short paragraphs that are likely menu items or buttons
        let text = p.text().collect::<String>();
        if text.len() < 20 {
            continue;
        }
        
        // Skip paragraphs with high link density
        let link_density = calculate_link_density(&p);
        if link_density > *LINK_DENSITY_THRESHOLD {
            continue;
        }
        
        // Skip paragraphs in unwanted containers
        if let Some(parent) = p.parent_element() {
            if has_class_hint(&parent, &["nav", "menu", "footer", "header", "sidebar", "comment"]) {
                continue;
            }
        }
        
        paragraphs.push(p);
    }
    
    // If we have multiple paragraphs, try to find clusters of them
    if paragraphs.len() >= 3 {
        // Group consecutive paragraphs that are likely part of the main content
        let mut text = String::new();
        for p in paragraphs {
            let paragraph_text = get_text_content(&p, config);
            if !paragraph_text.trim().is_empty() {
                text.push_str(&paragraph_text);
                text.push('\n');
            }
        }
        
        if text.len() >= config.min_extracted_size {
            debug!("Content extracted using filtered paragraphs strategy");
            return Ok(text.trim().to_string());
        }
    }
    
    // 5. Last resort - just try to get any text
    let mut text = String::new();
    for p in cleaned_document.select(&p_selector) {
        let paragraph_text = get_text_content(&p, config);
        if !paragraph_text.trim().is_empty() {
            text.push_str(&paragraph_text);
            text.push('\n');
        }
    }
    
    debug!("Content extracted using last-resort paragraphs strategy");
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
    
    // Common unwanted classes to filter out
    let unwanted_classes = vec![
        "nav", "navbar", "navigation", "menu", "footer", "header", "sidebar",
        "advertisement", "ad", "social", "share", "sharing", "comment", "comments",
        "related", "recommended", "promotion", "promo", "subscribe", "subscription",
        "download", "copyright", "tags", "tag-cloud", "breadcrumb", "pagination",
        "pager", "widget", "banner"
    ];
    
    // Common unwanted IDs to filter out
    let unwanted_ids = vec![
        "nav", "navbar", "navigation", "menu", "footer", "header", "sidebar",
        "advertisement", "ad", "social", "share", "sharing", "comment", "comments",
        "related", "recommended", "promotion", "promo", "subscribe", "subscription",
        "download", "copyright", "tags", "tag-cloud", "breadcrumb", "pagination",
        "pager", "widget", "banner"
    ];
    
    // Look for common content containers - prioritizing semantic tags first
    for &tag in &["article", "main", "section", "div", "body"] {
        let selector = Selector::parse(tag).unwrap();
        for element in document.select(&selector) {
            // Skip elements that are likely navigation or other non-content
            if has_class_hint(&element, &unwanted_classes) || has_id_hint(&element, &unwanted_ids) {
                continue;
            }
            
            // Skip elements that have too many links (likely navigation)
            let link_density = calculate_link_density(&element);
            if link_density > *LINK_DENSITY_THRESHOLD {
                continue;
            }
            
            // Check paragraph count - content likely has multiple paragraphs
            let p_selector = Selector::parse("p").unwrap();
            let p_count = element.select(&p_selector).count();
            
            // Check if this element has enough text content
            let text_content = element.text().collect::<String>();
            let text_length = text_content.len();
            
            // Prioritize elements with good content indicators
            if (text_length > 250 && p_count >= 2) || 
               (text_length > 500) || 
               (p_count >= 4) || 
               has_class_hint(&element, &CONTENT_CLASSES) || 
               has_id_hint(&element, &CONTENT_IDS) {
                candidates.push(element);
                
                // For article tags, give them higher priority by adding them earlier in the list
                if tag == "article" || tag == "main" {
                    candidates.insert(0, element);
                }
            } else if text_length > 100 {
                // Lower-quality candidates still get added
                candidates.push(element);
            }
        }
    }
    
    candidates
}

/// Score a node based on its content
fn score_node(element: &ElementRef, _config: &ExtractionConfig) -> i32 {
    let mut score = 0;
    
    // Score based on text length (more text = more likely to be content)
    let text_content: String = element.text().collect();
    score += (text_content.len() / 20) as i32; // Increased the text weight factor
    
    // Bonus for content class/id hints
    if has_class_hint(element, &CONTENT_CLASSES) {
        score += 75; // Increased the bonus for content class hints
    }
    
    if has_id_hint(element, &CONTENT_IDS) {
        score += 75; // Increased the bonus for content ID hints
    }
    
    // Count paragraphs - articles typically have several paragraphs
    let p_selector = Selector::parse("p").unwrap();
    let p_count = element.select(&p_selector).count();
    score += p_count as i32 * 10; // Each paragraph adds to the score
    
    // Count text-heavy elements that suggest content (paragraphs, headings, list items)
    let content_elements_selector = Selector::parse("p, h1, h2, h3, h4, h5, h6, li").unwrap();
    let content_elements_count = element.select(&content_elements_selector).count();
    score += content_elements_count as i32 * 5;
    
    // Penalize elements with non-content hints
    let unwanted_classes = vec![
        "nav", "navbar", "navigation", "menu", "footer", "header", "sidebar", 
        "advertisement", "social", "sharing", "comment", "related", "recommendation"
    ];
    if has_class_hint(element, &unwanted_classes) {
        score -= 50;
    }
    
    let unwanted_ids = vec![
        "nav", "navbar", "navigation", "menu", "footer", "header", "sidebar",
        "advertisement", "social", "sharing", "comment", "related", "recommendation"
    ];
    if has_id_hint(element, &unwanted_ids) {
        score -= 50;
    }
    
    // Score based on child elements' tag types
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
    
    // Penalize for high link density (navigation-heavy content)
    let link_density = calculate_link_density(element);
    if link_density > *LINK_DENSITY_THRESHOLD {
        score -= (link_density * 150.0) as i32; // Increased penalty for link-heavy content
    }
    
    // Bonus for elements with common article structure (heading followed by paragraphs)
    let heading_selector = Selector::parse("h1, h2, h3").unwrap();
    if element.select(&heading_selector).next().is_some() && p_count >= 2 {
        score += 30; // Bonus for having a heading and multiple paragraphs
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