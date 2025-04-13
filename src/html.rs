//! HTML processing functions for Trafilatura Rust port.
//! This module contains utilities for cleaning and normalizing HTML content.

use kuchiki::traits::*;
use kuchiki::{ElementData, NodeData, NodeRef};
use regex::Regex;
use lazy_static::lazy_static;

use crate::ExtractionConfig;
use crate::TrafilaturaError;

lazy_static! {
    /// Common elements that should be removed during cleaning
    static ref UNWANTED_ELEMENTS: Vec<&'static str> = vec![
        "script", "style", "noscript", "iframe", "footer", "nav", "aside",
        "form", "button", "svg", "head", "header", "meta", "link", "comment"
    ];

    /// Common class names that indicate navigation, ads, or other non-content elements
    static ref UNWANTED_CLASSES: Vec<&'static str> = vec![
        "nav", "navbar", "navigation", "menu", "footer", "comment", "widget", 
        "sidebar", "advertisement", "ad", "advert", "popup", "banner", "social",
        "sharing", "share", "related", "recommend", "promotion", "shopping"
    ];

    /// Common ID names that indicate navigation, ads, or other non-content elements
    static ref UNWANTED_IDS: Vec<&'static str> = vec![
        "nav", "navbar", "navigation", "menu", "footer", "comments", "sidebar",
        "advertisement", "related", "recommend", "sidebar", "social", "sharing"
    ];

    /// Regex to match multiple spaces
    static ref MULTIPLE_SPACES_RE: Regex = Regex::new(r"\s+").unwrap();

    /// Regex to match line breaks
    static ref LINE_BREAKS_RE: Regex = Regex::new(r"(\r\n|\r|\n)+").unwrap();
}

/// Clean an HTML document by removing unwanted elements
pub fn clean_html(document: &NodeRef, config: &ExtractionConfig) -> Result<NodeRef, TrafilaturaError> {
    let document_clone = document.clone();
    
    // Remove unwanted elements
    for element_name in UNWANTED_ELEMENTS.iter() {
        let elements = document_clone.select(*element_name).unwrap();
        for element in elements {
            if let Ok(node) = element.as_node().try_into_node_ref() {
                if let Some(parent) = node.parent() {
                    parent.children().remove_from_parent(&node);
                }
            }
        }
    }
    
    // Remove elements with unwanted classes
    for class_hint in UNWANTED_CLASSES.iter() {
        let selector = format!("[class*='{}']", class_hint);
        let elements = document_clone.select(&selector).unwrap();
        for element in elements {
            if let Ok(node) = element.as_node().try_into_node_ref() {
                if let Some(parent) = node.parent() {
                    parent.children().remove_from_parent(&node);
                }
            }
        }
    }
    
    // Remove elements with unwanted IDs
    for id_hint in UNWANTED_IDS.iter() {
        let selector = format!("[id*='{}']", id_hint);
        let elements = document_clone.select(&selector).unwrap();
        for element in elements {
            if let Ok(node) = element.as_node().try_into_node_ref() {
                if let Some(parent) = node.parent() {
                    parent.children().remove_from_parent(&node);
                }
            }
        }
    }
    
    // Remove comments if not configured to include them
    if !config.include_comments {
        remove_all_comments(&document_clone);
    }
    
    // Remove tables if not configured to include them
    if !config.include_tables {
        let table_elements = document_clone.select("table").unwrap();
        for element in table_elements {
            if let Ok(node) = element.as_node().try_into_node_ref() {
                if let Some(parent) = node.parent() {
                    parent.children().remove_from_parent(&node);
                }
            }
        }
    }
    
    Ok(document_clone)
}

/// Remove all HTML comments from a document
fn remove_all_comments(node: &NodeRef) {
    // Collect nodes to remove first to avoid borrowing issues
    let mut nodes_to_remove = Vec::new();
    
    for child in node.children() {
        match child.data() {
            NodeData::Comment(_) => {
                nodes_to_remove.push(child.clone());
            }
            NodeData::Element(_) => {
                remove_all_comments(&child);
            }
            _ => {}
        }
    }
    
    // Now remove the comment nodes
    for node_to_remove in nodes_to_remove {
        if let Some(parent) = node_to_remove.parent() {
            parent.children().remove_from_parent(&node_to_remove);
        }
    }
}

/// Get the text content of a node, preserving some formatting
pub fn get_text_content(node: &NodeRef, config: &ExtractionConfig) -> String {
    let mut text = String::new();
    
    extract_text_content(node, &mut text, config);
    
    // Normalize spaces
    let text = MULTIPLE_SPACES_RE.replace_all(&text, " ").to_string();
    
    // Normalize line breaks
    let text = LINE_BREAKS_RE.replace_all(&text, "\n").to_string();
    
    // Trim whitespace
    text.trim().to_string()
}

/// Extract text content from a node and its children
fn extract_text_content(node: &NodeRef, text: &mut String, config: &ExtractionConfig) {
    match node.data() {
        NodeData::Text(text_ref) => {
            let content = text_ref.borrow();
            if !content.trim().is_empty() {
                text.push_str(&content);
                text.push(' ');
            }
        }
        NodeData::Element(element_ref) => {
            let element = element_ref.borrow();
            
            // Handle specific elements
            match element.name.local.as_ref() {
                "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" => {
                    // Extract text from children
                    for child in node.children() {
                        extract_text_content(&child, text, config);
                    }
                    // Add a line break after block elements
                    text.push('\n');
                }
                "br" => {
                    text.push('\n');
                }
                "a" => {
                    if config.include_links {
                        let mut link_text = String::new();
                        for child in node.children() {
                            extract_text_content(&child, &mut link_text, config);
                        }
                        
                        if !link_text.trim().is_empty() {
                            text.push_str(&link_text);
                            
                            // Optionally add the href in parentheses
                            if let Some(href) = element.attributes.borrow().get("href") {
                                text.push_str(&format!(" ({})", href));
                            }
                            
                            text.push(' ');
                        }
                    } else {
                        // Just extract text without href
                        for child in node.children() {
                            extract_text_content(&child, text, config);
                        }
                    }
                }
                "img" => {
                    if config.include_images {
                        let attrs = element.attributes.borrow();
                        if let Some(alt) = attrs.get("alt") {
                            if !alt.trim().is_empty() {
                                text.push_str(&format!("[Image: {}]", alt));
                                text.push(' ');
                            } else if let Some(src) = attrs.get("src") {
                                text.push_str(&format!("[Image: {}]", src));
                                text.push(' ');
                            }
                        }
                    }
                }
                _ => {
                    // Generic element processing
                    for child in node.children() {
                        extract_text_content(&child, text, config);
                    }
                }
            }
        }
        NodeData::Comment(_) => {
            if config.include_comments {
                // Include comment content if configured
                text.push_str("[Comment]");
                text.push(' ');
            }
        }
        _ => {}
    }
}

/// Convert the node to an HTML string
pub fn node_to_html(node: &NodeRef) -> Result<String, TrafilaturaError> {
    let mut html = Vec::new();
    node.serialize(&mut html).map_err(|e| TrafilaturaError::ParsingError(e.to_string()))?;
    
    let html_string = String::from_utf8(html)
        .map_err(|e| TrafilaturaError::ParsingError(e.to_string()))?;
    
    Ok(html_string)
}

/// Check if a node has any of the given class hints
pub fn has_class_hint(node: &NodeRef, class_hints: &[&str]) -> bool {
    if let Ok(element) = node.as_element() {
        let attributes = element.attributes.borrow();
        if let Some(class_attr) = attributes.get("class") {
            for hint in class_hints {
                if class_attr.contains(hint) {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a node has any of the given ID hints
pub fn has_id_hint(node: &NodeRef, id_hints: &[&str]) -> bool {
    if let Ok(element) = node.as_element() {
        let attributes = element.attributes.borrow();
        if let Some(id_attr) = attributes.get("id") {
            for hint in id_hints {
                if id_attr.contains(hint) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuchiki::parse_html;

    #[test]
    fn test_clean_html_removes_scripts() {
        let html = r#"<html><body><p>Text</p><script>alert(1);</script></body></html>"#;
        let document = parse_html().one(html);
        let config = ExtractionConfig::default();
        
        let cleaned = clean_html(&document, &config).unwrap();
        let cleaned_html = node_to_html(&cleaned).unwrap();
        
        assert!(!cleaned_html.contains("<script>"));
        assert!(cleaned_html.contains("<p>Text</p>"));
    }

    #[test]
    fn test_get_text_content() {
        let html = r#"<html><body><h1>Title</h1><p>Paragraph <a href="http://example.com">with link</a></p></body></html>"#;
        let document = parse_html().one(html);
        let config = ExtractionConfig::default();
        
        let text = get_text_content(&document, &config);
        
        assert!(text.contains("Title"));
        assert!(text.contains("Paragraph"));
        assert!(text.contains("with link"));
        
        // Test with links inclusion
        let mut config_with_links = config.clone();
        config_with_links.include_links = true;
        
        let text_with_links = get_text_content(&document, &config_with_links);
        assert!(text_with_links.contains("(http://example.com)"));
    }
}
