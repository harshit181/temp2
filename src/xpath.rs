//! CSS selector extraction module for Trafilatura Rust port.
//! This module uses CSS selectors for HTML element selection.
//! The module name remains xpath.rs for compatibility with the original design.

use log::debug;
use scraper::{Html, Selector, ElementRef};
use regex::Regex;

use crate::ExtractionConfig;
use crate::TrafilaturaError;

/// CSS selectors used for content extraction
pub struct XPaths {
    /// CSS selector for the main content area
    pub main_content: &'static str,
    /// CSS selector for paragraphs
    pub paragraphs: &'static str,
    /// CSS selector for headings
    pub headings: &'static str,
    /// CSS selector for lists
    pub lists: &'static str,
    /// CSS selector for list items
    pub list_items: &'static str,
    /// CSS selector for tables
    pub tables: &'static str,
    /// CSS selector for images
    pub images: &'static str,
    /// CSS selector for captions
    pub captions: &'static str,
    /// CSS selector for anchors/links
    pub anchors: &'static str,
}

/// Default CSS selector expressions for content extraction
pub const DEFAULT_XPATHS: XPaths = XPaths {
    // Complex selectors for main content, ported from Python trafilatura's BODY_XPATH
    main_content: concat!(
        // Common article containers
        "article, main, ",
        
        // Post and entry classes
        "div.post, div.entry, ",
        "div[class*='post-text'], div[class*='post_text'], ",
        "div[class*='post-body'], div[class*='post-entry'], div[class*='postentry'], ",
        "div[class*='post-content'], div[class*='post_content'], ",
        "div[class*='postcontent'], div[class*='postContent'], div[class*='post_inner_wrapper'], ",
        
        // Article text classes
        "div[class*='article-text'], div[class*='articletext'], div[class*='articleText'], ",
        
        // Content containers
        "div[id*='entry-content'], ",
        "div[class*='entry-content'], div[id*='article-content'], ",
        "div[class*='article-content'], div[id*='article__content'], ",
        "div[class*='article__content'], div[id*='article-body'], ",
        "div[class*='article-body'], div[id*='article__body'], ",
        "div[class*='article__body'], div[itemprop='articleBody'], ",
        "div[id*='articlebody' i], div[class*='articlebody' i], ",  // case insensitive
        "div#articleContent, div[class*='ArticleContent'], ",
        "div[class*='page-content'], div[class*='text-content'], ",
        "div[id*='body-text'], div[class*='body-text'], ",
        "div[class*='article__container'], div[id*='art-content'], div[class*='art-content'], ",
        
        // Secondary content selectors
        "div[class*='post-bodycopy'], ",
        "div[class*='storycontent'], div[class*='story-content'], ",
        "div.postarea, div.art-postcontent, ",
        "div[class*='theme-content'], div[class*='blog-content'], ",
        "div[class*='section-content'], div[class*='single-content'], ",
        "div[class*='single-post'], ",
        "div[class*='main-column'], div[class*='wpb_text_column'], ",
        "div[id^='primary'], div[class^='article '], div.text, ",
        "div#article, div.cell, div#story, div.story, ",
        "div[class*='story-body'], div[id*='story-body'], div[class*='field-body'], ",
        "div[class*='fulltext' i], ",  // case insensitive
        "div[role='article'], ",
        
        // Content main selectors
        "div[id*='content-main'], div[class*='content-main'], div[class*='content_main'], ",
        "div[id*='content-body'], div[class*='content-body'], div[id*='contentBody'], ",
        "div[class*='content__body'], div[id*='main-content' i], div[class*='main-content' i], ",  // case insensitive
        "div[class*='page-content' i], ", // case insensitive
        "div#content, div.content, ",
        
        // Main selectors
        "section[class^='main'], section[id^='main'], section[role^='main'], ",
        "div[class^='main'], div[id^='main'], div[role^='main']"
    ),
    paragraphs: "p, div[class*='paragraph'], div[class*='text-block'], div[class*='post-block'], div[class*='entry-block'], div.post-text, div.text, div[class*='article-text'], section[class*='paragraph']",
    headings: "h1, h2, h3, h4, h5, h6",
    lists: "ul, ol, dl",
    list_items: "li, dt, dd",
    tables: "table",
    images: "img",
    captions: "figcaption",
    anchors: "a",
};

/// Wikipedia-specific CSS selector expressions for content extraction
pub const WIKI_XPATHS: XPaths = XPaths {
    main_content: "div#content, div#bodyContent, div#mw-content-text, div.mw-parser-output",
    paragraphs: "div#mw-content-text p, div.mw-parser-output p, .mw-parser-output .mw-empty-elt",
    headings: "div#mw-content-text h1, div#mw-content-text h2, div#mw-content-text h3, div#mw-content-text h4, div#mw-content-text h5, div#mw-content-text h6, div.mw-parser-output h1, div.mw-parser-output h2, div.mw-parser-output h3, div.mw-parser-output h4, div.mw-parser-output h5, div.mw-parser-output h6",
    lists: "div#mw-content-text ul, div#mw-content-text ol, div.mw-parser-output ul, div.mw-parser-output ol",
    list_items: "div#mw-content-text li, div.mw-parser-output li",
    tables: "div#mw-content-text table, div.mw-parser-output table",
    images: "div#mw-content-text img, div.mw-parser-output img",
    captions: "div#mw-content-text figcaption, div.mw-parser-output figcaption",
    anchors: "div#mw-content-text a, div.mw-parser-output a",
};

/// Helper function to create a Selector from a CSS selector string
pub fn create_selector(selector: &str) -> Result<Selector, TrafilaturaError> {
    Selector::parse(selector)
        .map_err(|e| TrafilaturaError::SelectorError(format!("Invalid selector: {} - {:?}", selector, e)))
}

/// Text content of section headers to skip in Wikipedia articles
pub const WIKI_SKIP_SECTION_TITLES: [&str; 13] = [
    "References",
    "External links",
    "See also",
    "Further reading",
    "Notes",
    "Bibliography",
    "Sources",
    "Citations",
    "Footnotes",
    "Literature",
    "Literatur", // German Wikipedia
    "Weblinks",  // German Wikipedia
    "Enlaces externos", // Spanish Wikipedia
];

/// Names of elements to exclude from extraction
pub const EXCLUDE_ELEMENTS: [&str; 12] = [
    "nav", "aside", "footer", "menu", "header", "form", 
    "script", "style", "noscript", "figcaption", "iframe", "toc"
];

/// Classes of elements to exclude from extraction
pub const EXCLUDE_CLASSES: [&str; 35] = [
    "nav", "navbar", "menu", "footer", "sidebar", "comment", "widget", 
    "advertisement", "ad", "advert", "popup", "banner", "social",
    "sharing", "share", "related", "recommend", "promotion", "shopping",
    "subscribe", "subscription", "newsletter", "promo", "masthead", "aux",
    "breadcrumb", "byline", "metadata", "date", "tags", "cloud", "topics",
    "author", "copyright", "disclaimer"
];

/// IDs of elements to exclude from extraction
pub const EXCLUDE_IDS: [&str; 30] = [
    "nav", "navbar", "menu", "footer", "sidebar", "comment", "comments", 
    "advertisement", "social", "sharing", "share", "related", "recommend",
    "newsletter", "promo", "masthead", "breadcrumb", "byline", "metadata",
    "pagination", "pager", "tags", "tag-cloud", "topics", "topic-list",
    "category", "categories", "search", "sidebar", "toc"
];

/// Extract content using CSS selector expressions (simplified XPath-like approach)
pub fn extract_with_xpath(html_content: &str, config: &ExtractionConfig) -> Result<String, TrafilaturaError> {
    // Parse the HTML document
    let document = Html::parse_document(html_content);
    
    // Determine if this is a Wikipedia page
    let is_wiki = is_wikipedia_page(&document);
    let xpaths = if is_wiki { &WIKI_XPATHS } else { &DEFAULT_XPATHS };
    
    debug!("Using CSS selector extraction with {} selectors", if is_wiki { "Wikipedia" } else { "default" });
    
    // Find the main content
    let mut content = String::new();
    
    // Create the selector for main content
    let main_content_selector = create_selector(xpaths.main_content)?;
    let mut elements = document.select(&main_content_selector).collect::<Vec<_>>();
    
    // If we didn't find a main content area, try with a broader approach
    if elements.is_empty() {
        let body_selector = create_selector("body")?;
        elements = document.select(&body_selector).collect::<Vec<_>>();
    }
    
    if elements.is_empty() {
        return Err(TrafilaturaError::ExtractionError("No content elements found".to_string()));
    }
    
    // Process the main content
    let main_element = &elements[0];
    
    // Extract headings and content
    let headings_selector = create_selector(xpaths.headings)?;
    for element in main_element.select(&headings_selector) {
        let text = element.text().collect::<String>();
        let is_skip_section = is_wiki && should_skip_section(&text);
        
        // If it's not a section to skip
        if !is_skip_section && is_heading(element) {
            if !text.trim().is_empty() {
                content.push_str(&text);
                content.push_str("\n\n");
            }
        }
    }
    
    // Extract paragraphs, checking if we're in a section to skip
    let paragraphs_selector = create_selector(xpaths.paragraphs)?;
    for element in main_element.select(&paragraphs_selector) {
        // Check if preceding heading is in skip section
        let mut should_skip = false;
        if is_wiki {
            if let Some(heading_text) = find_preceding_heading_text(&document, &element) {
                should_skip = should_skip_section(&heading_text);
            }
        }
        
        if should_skip || should_exclude(&element) {
            continue;
        }
        
        let text = element.text().collect::<String>();
        let trimmed = text.trim();
        if !trimmed.is_empty() && trimmed.len() > 10 {  // Exclude very short paragraphs
            content.push_str(trimmed);
            content.push_str("\n\n");
        }
    }
    
    // Extract lists
    if config.include_tables {
        let lists_selector = create_selector(xpaths.lists)?;
        for element in main_element.select(&lists_selector) {
            // Check if preceding heading is in skip section
            let mut should_skip = false;
            if is_wiki {
                if let Some(heading_text) = find_preceding_heading_text(&document, &element) {
                    should_skip = should_skip_section(&heading_text);
                }
            }
            
            if should_skip || should_exclude(&element) {
                continue;
            }
            
            // Extract list items
            let list_items_selector = create_selector(xpaths.list_items)?;
            for item in element.select(&list_items_selector) {
                if should_exclude(&item) {
                    continue;
                }
                
                let text = item.text().collect::<String>();
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    content.push_str("• ");
                    content.push_str(trimmed);
                    content.push_str("\n");
                }
            }
            content.push_str("\n");
        }
    }
    
    // Extract tables
    if config.include_tables {
        let tables_selector = create_selector(xpaths.tables)?;
        for element in main_element.select(&tables_selector) {
            // Check if preceding heading is in skip section
            let mut should_skip = false;
            if is_wiki {
                if let Some(heading_text) = find_preceding_heading_text(&document, &element) {
                    should_skip = should_skip_section(&heading_text);
                }
            }
            
            if should_skip || should_exclude(&element) {
                continue;
            }
            
            // Simple extraction of table text
            let text = element.text().collect::<String>();
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                content.push_str("[Table: ");
                content.push_str(trimmed);
                content.push_str("]\n\n");
            }
        }
    }
    
    // Extract images
    if config.include_images {
        let images_selector = create_selector(xpaths.images)?;
        for element in main_element.select(&images_selector) {
            if should_exclude(&element) {
                continue;
            }
            
            let alt = element.value().attr("alt").unwrap_or("");
            let src = element.value().attr("src").unwrap_or("");
            
            if !alt.is_empty() || !src.is_empty() {
                content.push_str("[Image: ");
                if !alt.is_empty() {
                    content.push_str(alt);
                } else {
                    content.push_str(src);
                }
                content.push_str("]\n\n");
            }
        }
    }
    
    // Clean up the content
    let mut cleaned_content = content.trim().to_string();
    
    // Replace consecutive newlines with just two
    let multiple_newlines_re = Regex::new(r"\n{3,}").unwrap();
    cleaned_content = multiple_newlines_re.replace_all(&cleaned_content, "\n\n").to_string();
    
    Ok(cleaned_content)
}

/// Find the text of the preceding heading of an element
fn find_preceding_heading_text(document: &Html, element: &ElementRef) -> Option<String> {
    // Try to find headings by traversing the DOM upwards
    let h_selector = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
    
    // Find parent section or article
    let _section_selector = Selector::parse("section, article, div").unwrap();
    let mut current = element.clone();
    
    // First check if we can find a heading within the same parent
    while let Some(parent_ref) = current.parent().and_then(ElementRef::wrap) {
        // Check direct children of parent for headings that come before our element
        for heading in parent_ref.select(&h_selector) {
            // Check if heading is before our element in document order
            if is_before(document, &heading, element) {
                return Some(heading.text().collect());
            }
        }
        
        // Move up to parent
        current = parent_ref;
        
        // If we've reached a section or article, stop
        if current.value().name.local.eq_str_ignore_ascii_case("section") || 
           current.value().name.local.eq_str_ignore_ascii_case("article") {
            break;
        }
    }
    
    None
}

/// Check if element a is before element b in document order
fn is_before(document: &Html, a: &ElementRef, b: &ElementRef) -> bool {
    // Simplistic approach: compare the source positions
    // This is a heuristic that works in most cases but not all
    let all_elements: Vec<_> = document.tree.nodes().collect();
    let a_pos = all_elements.iter().position(|n| n.id() == a.id());
    let b_pos = all_elements.iter().position(|n| n.id() == b.id());
    
    match (a_pos, b_pos) {
        (Some(a_idx), Some(b_idx)) => a_idx < b_idx,
        _ => false,
    }
}

/// Check if a section should be skipped based on heading text
fn should_skip_section(heading_text: &str) -> bool {
    let heading_lower = heading_text.to_lowercase();
    
    WIKI_SKIP_SECTION_TITLES.iter().any(|&title| {
        heading_lower.contains(&title.to_lowercase())
    })
}

/// Check if the page is a Wikipedia page
fn is_wikipedia_page(document: &Html) -> bool {
    // Check meta tags for Wikipedia
    let meta_selector = Selector::parse("meta[property='og:site_name']").unwrap();
    for meta in document.select(&meta_selector) {
        if let Some(content) = meta.value().attr("content") {
            if content.contains("Wikipedia") {
                return true;
            }
        }
    }
    
    // Check domain in canonical link
    let link_selector = Selector::parse("link[rel='canonical']").unwrap();
    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            if href.contains("wikipedia.org") {
                return true;
            }
        }
    }
    
    false
}

/// Check if an element is a heading
fn is_heading(element: ElementRef) -> bool {
    let name = element.value().name.local.to_lowercase();
    matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

/// Check if an element should be excluded based on its tag, class, or ID
fn should_exclude(element: &ElementRef) -> bool {
    // Check element itself
    let el = element.value();
    
    // Check tag name
    let tag_name = el.name.local.to_lowercase();
    if EXCLUDE_ELEMENTS.iter().any(|&tag| tag.eq_ignore_ascii_case(&tag_name)) {
        return true;
    }
    
    // Check classes
    if let Some(class_attr) = el.attr("class") {
        let classes: Vec<&str> = class_attr.split_whitespace().collect();
        for class in classes {
            if EXCLUDE_CLASSES.iter().any(|&excl| class.eq_ignore_ascii_case(excl)) {
                return true;
            }
        }
    }
    
    // Check id
    if let Some(id) = el.attr("id") {
        if EXCLUDE_IDS.iter().any(|&excl_id| id.eq_ignore_ascii_case(excl_id)) {
            return true;
        }
    }
    
    // Check parent elements
    if let Some(parent_ref) = element.parent().and_then(ElementRef::wrap) {
        let parent = parent_ref.value();
        
        // Check parent tag
        let parent_tag = parent.name.local.to_lowercase();
        if EXCLUDE_ELEMENTS.iter().any(|&tag| tag.eq_ignore_ascii_case(&parent_tag)) {
            return true;
        }
        
        // Check parent classes
        if let Some(parent_class) = parent.attr("class") {
            let parent_classes: Vec<&str> = parent_class.split_whitespace().collect();
            for class in parent_classes {
                if EXCLUDE_CLASSES.iter().any(|&excl| class.eq_ignore_ascii_case(excl)) {
                    return true;
                }
            }
        }
        
        // Check parent id
        if let Some(parent_id) = parent.attr("id") {
            if EXCLUDE_IDS.iter().any(|&excl_id| parent_id.eq_ignore_ascii_case(excl_id)) {
                return true;
            }
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wikipedia_page_detection() {
        // Test with a Wikipedia page
        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <meta property="og:site_name" content="Wikipedia" />
        </head>
        <body></body>
        </html>"#;
        
        let document = Html::parse_document(html);
        assert!(is_wikipedia_page(&document));
        
        // Test with a non-Wikipedia page
        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <meta property="og:site_name" content="Not Wikipedia" />
        </head>
        <body></body>
        </html>"#;
        
        let document = Html::parse_document(html);
        assert!(!is_wikipedia_page(&document));
    }
    
    #[test]
    fn test_should_skip_section() {
        assert!(should_skip_section("References"));
        assert!(should_skip_section("External links"));
        assert!(should_skip_section("See also"));
        assert!(!should_skip_section("Introduction"));
        assert!(!should_skip_section("Main content"));
    }
}