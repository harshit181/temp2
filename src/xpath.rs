//! CSS selector extraction module for Trafilatura Rust port.
//! This module uses CSS selectors instead of XPath expressions for better compatibility with kuchiki.
//! The module name remains xpath.rs for compatibility with the original design.

use kuchiki::{ElementData, NodeDataRef, NodeRef, parse_html};
use kuchiki::traits::*;
use log::debug;

use crate::ExtractionConfig;
use crate::TrafilaturaError;

/// CSS selectors used for content extraction (used instead of XPath for better compatibility)
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
    main_content: "main, article, div.article, div.content, div.document, div#content, div#article",
    paragraphs: "p",
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
    main_content: "div#content, div#mw-content-text",
    paragraphs: "div#mw-content-text p",
    headings: "div#mw-content-text h1, div#mw-content-text h2, div#mw-content-text h3, div#mw-content-text h4, div#mw-content-text h5, div#mw-content-text h6",
    lists: "div#mw-content-text ul, div#mw-content-text ol",
    list_items: "div#mw-content-text li",
    tables: "div#mw-content-text table",
    images: "div#mw-content-text img",
    captions: "div#mw-content-text figcaption",
    anchors: "div#mw-content-text a",
};

/// CSS selectors to identify sections to skip in Wikipedia articles
pub const WIKI_SKIP_SECTIONS: [&str; 6] = [
    "div#mw-content-text h2:contains('References')",
    "div#mw-content-text h2:contains('External links')",
    "div#mw-content-text h2:contains('See also')",
    "div#mw-content-text h2:contains('Further reading')",
    "div#mw-content-text h2:contains('Notes')",
    "div#mw-content-text h2:contains('Bibliography')",
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
    let document = parse_html().one(html_content);
    
    // Determine if this is a Wikipedia page
    let is_wiki = is_wikipedia_page(&document);
    let xpaths = if is_wiki { &WIKI_XPATHS } else { &DEFAULT_XPATHS };
    
    debug!("Using CSS selector extraction with {} selectors", if is_wiki { "Wikipedia" } else { "default" });
    
    // Find the main content
    let mut content = String::new();
    let mut elements = document.select(xpaths.main_content)
        .map_err(|_| TrafilaturaError::ExtractionError("CSS selection error for main content".to_string()))?
        .collect::<Vec<_>>();
    
    // If we didn't find a main content area, try with a broader approach
    if elements.is_empty() {
        elements = document.select("body")
            .map_err(|_| TrafilaturaError::ExtractionError("CSS selection error for body".to_string()))?
            .collect::<Vec<_>>();
    }
    
    if elements.is_empty() {
        return Err(TrafilaturaError::ExtractionError("No content elements found".to_string()));
    }
    
    // Process the main content
    let main_element = &elements[0];
    
    // For Wikipedia, we'll track if we're in a section that should be skipped
    let mut in_skip_section;
    
    // Extract headings and content
    if let Ok(elements) = main_element.as_node().select(xpaths.headings) {
        for element in elements {
            let text = get_element_text(&element).unwrap_or_default();
            let is_skip_section = is_wiki && should_skip_section(&text);
            
            // If it's not a section to skip and it's a heading
            if !is_skip_section && is_heading(&element.as_node()) {
                if !text.trim().is_empty() {
                    content.push_str(&text);
                    content.push_str("\n\n");
                }
            }
        }
    }
    
    // Reset the flag
    in_skip_section = false;
    
    // Extract paragraphs, checking if we're in a section to skip
    if let Ok(elements) = main_element.as_node().select(xpaths.paragraphs) {
        for element in elements {
            // Check if preceding heading is in skip section
            if is_wiki {
                let prev = find_preceding_heading(&element);
                if let Some(heading) = prev {
                    let heading_text = heading.text_contents();
                    in_skip_section = should_skip_section(&heading_text);
                }
            }
            
            if in_skip_section || should_exclude(&element) {
                continue;
            }
            
            if let Some(text) = get_element_text(&element) {
                let trimmed = text.trim();
                if !trimmed.is_empty() && trimmed.len() > 10 {  // Exclude very short paragraphs
                    content.push_str(trimmed);
                    content.push_str("\n\n");
                }
            }
        }
    }
    
    // Reset the flag
    in_skip_section = false;
    
    // Extract lists
    if config.include_tables {
        if let Ok(elements) = main_element.as_node().select(xpaths.lists) {
            for element in elements {
                // Check if preceding heading is in skip section
                if is_wiki {
                    let prev = find_preceding_heading(&element);
                    if let Some(heading) = prev {
                        let heading_text = heading.text_contents();
                        in_skip_section = should_skip_section(&heading_text);
                    }
                }
                
                if in_skip_section || should_exclude(&element) {
                    continue;
                }
                
                // Extract list items
                if let Ok(list_items) = element.as_node().select(xpaths.list_items) {
                    for item in list_items {
                        if should_exclude(&item) {
                            continue;
                        }
                        
                        if let Some(text) = get_element_text(&item) {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                content.push_str("• ");
                                content.push_str(trimmed);
                                content.push_str("\n");
                            }
                        }
                    }
                    content.push_str("\n");
                }
            }
        }
    }
    
    // Reset the flag
    in_skip_section = false;
    
    // Extract tables
    if config.include_tables {
        if let Ok(elements) = main_element.as_node().select(xpaths.tables) {
            for element in elements {
                // Check if preceding heading is in skip section
                if is_wiki {
                    let prev = find_preceding_heading(&element);
                    if let Some(heading) = prev {
                        let heading_text = heading.text_contents();
                        in_skip_section = should_skip_section(&heading_text);
                    }
                }
                
                if in_skip_section || should_exclude(&element) {
                    continue;
                }
                
                // Simple extraction of table text
                if let Some(text) = get_element_text(&element) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        content.push_str("[Table: ");
                        content.push_str(trimmed);
                        content.push_str("]\n\n");
                    }
                }
            }
        }
    }
    
    // Extract images
    if config.include_images {
        if let Ok(elements) = main_element.as_node().select(xpaths.images) {
            for element in elements {
                if should_exclude(&element) {
                    continue;
                }
                
                let node = element.as_node();
                if let Some(element_data) = node.as_element() {
                    let attrs = element_data.attributes.borrow();
                    let alt = attrs.get("alt").unwrap_or("");
                    let src = attrs.get("src").unwrap_or("");
                    
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
        }
    }
    
    // Clean up the content
    let mut cleaned_content = content.trim().to_string();
    
    // Replace consecutive newlines with just two
    let multiple_newlines_re = regex::Regex::new(r"\n{3,}").unwrap();
    cleaned_content = multiple_newlines_re.replace_all(&cleaned_content, "\n\n").to_string();
    
    Ok(cleaned_content)
}

/// Find the preceding heading element
fn find_preceding_heading(element: &NodeDataRef<ElementData>) -> Option<NodeRef> {
    let node = element.as_node();
    let mut current = node.clone();
    
    // Check previous siblings first
    loop {
        if let Some(prev) = current.previous_sibling() {
            if is_heading(&prev) {
                return Some(prev);
            }
            current = prev;
        } else {
            break;
        }
    }
    
    // If no heading found among siblings, check parent's previous siblings
    if let Some(parent) = node.parent() {
        let mut current = parent.clone();
        loop {
            if let Some(prev) = current.previous_sibling() {
                if is_heading(&prev) {
                    return Some(prev);
                }
                
                // Check if any descendants are headings
                if let Ok(headings) = prev.select("h1,h2,h3,h4,h5,h6") {
                    if let Some(last_heading) = headings.last() {
                        return Some(last_heading.as_node().clone());
                    }
                }
                
                current = prev;
            } else {
                break;
            }
        }
    }
    
    None
}

/// Check if a section should be skipped based on heading text
fn should_skip_section(heading_text: &str) -> bool {
    let heading_lower = heading_text.to_lowercase();
    heading_lower.contains("references") ||
    heading_lower.contains("external links") ||
    heading_lower.contains("see also") ||
    heading_lower.contains("further reading") ||
    heading_lower.contains("notes") ||
    heading_lower.contains("bibliography") ||
    heading_lower.contains("sources") ||
    heading_lower.contains("citations")
}

/// Check if the page is a Wikipedia page
fn is_wikipedia_page(document: &NodeRef) -> bool {
    if let Ok(meta_tags) = document.select("meta[property='og:site_name']") {
        for meta in meta_tags {
            let node = meta.as_node();
            if let Some(element_data) = node.as_element() {
                let attrs = element_data.attributes.borrow();
                if let Some(content) = attrs.get("content") {
                    if content.contains("Wikipedia") {
                        return true;
                    }
                }
            }
        }
    }
    
    // Check domain in canonical link
    if let Ok(links) = document.select("link[rel='canonical']") {
        for link in links {
            let node = link.as_node();
            if let Some(element_data) = node.as_element() {
                let attrs = element_data.attributes.borrow();
                if let Some(href) = attrs.get("href") {
                    if href.contains("wikipedia.org") {
                        return true;
                    }
                }
            }
        }
    }
    
    false
}

/// Check if a node is a heading element
fn is_heading(node: &NodeRef) -> bool {
    if let Some(element_ref) = node.as_element() {
        let name = element_ref.name.local.to_string();
        return matches!(name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6");
    }
    false
}

/// Check if an element should be excluded based on its tag, class, or ID
fn should_exclude(element: &NodeDataRef<ElementData>) -> bool {
    let node = element.as_node();
    
    // Check parent for exclusion first
    if let Some(parent) = node.parent() {
        if let Some(parent_element) = parent.as_element() {
            // Check if parent is in exclude list
            let name = parent_element.name.local.to_string();
            if EXCLUDE_ELEMENTS.contains(&name.as_str()) {
                return true;
            }
            
            // Check parent classes and IDs
            let attrs = parent_element.attributes.borrow();
            
            if let Some(class) = attrs.get("class") {
                for exclude_class in EXCLUDE_CLASSES.iter() {
                    if class.contains(exclude_class) {
                        return true;
                    }
                }
            }
            
            if let Some(id) = attrs.get("id") {
                for exclude_id in EXCLUDE_IDS.iter() {
                    if id.contains(exclude_id) {
                        return true;
                    }
                }
            }
        }
    }
    
    // Check the element itself
    if let Some(element_data) = node.as_element() {
        // Check if element is in exclude list
        let name = element_data.name.local.to_string();
        if EXCLUDE_ELEMENTS.contains(&name.as_str()) {
            return true;
        }
        
        // Check classes and IDs
        let attrs = element_data.attributes.borrow();
        
        if let Some(class) = attrs.get("class") {
            for exclude_class in EXCLUDE_CLASSES.iter() {
                if class.contains(exclude_class) {
                    return true;
                }
            }
        }
        
        if let Some(id) = attrs.get("id") {
            for exclude_id in EXCLUDE_IDS.iter() {
                if id.contains(exclude_id) {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Get the text content of an element
fn get_element_text(element: &NodeDataRef<ElementData>) -> Option<String> {
    let node = element.as_node();
    let text = node.text_contents();
    
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}