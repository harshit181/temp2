//! Metadata extraction for web pages
//! This module handles extracting metadata such as title, author, date, etc.

use chrono::NaiveDate;
use kuchiki::NodeRef;
use regex::Regex;
use lazy_static::lazy_static;

use crate::{ExtractionResult, TrafilaturaError};

lazy_static! {
    // Regex for extracting dates from text
    static ref DATE_REGEX: Regex = Regex::new(
        r"(?i)(?:\d{4}[-/]\d{1,2}[-/]\d{1,2}|\d{1,2}[-/]\d{1,2}[-/]\d{4})"
    ).unwrap();
    
    // Common date formats
    static ref DATE_FORMATS: Vec<&'static str> = vec![
        "%Y-%m-%d", "%d-%m-%Y", "%Y/%m/%d", "%d/%m/%Y",
    ];
}

/// Extract metadata from a document
pub fn extract_metadata(document: &NodeRef, mut result: ExtractionResult) -> Result<ExtractionResult, TrafilaturaError> {
    // Extract title
    result.title = extract_title(document);
    
    // Extract author
    result.author = extract_author(document);
    
    // Extract date
    result.date = extract_date(document);
    
    // Extract description
    result.description = extract_description(document);
    
    // Extract site name
    result.sitename = extract_sitename(document);
    
    // Extract categories/tags
    result.categories = extract_categories(document);
    
    Ok(result)
}

/// Extract the title from a document
fn extract_title(document: &NodeRef) -> Option<String> {
    // Try Open Graph title
    if let Ok(og_title) = document.select_first("meta[property='og:title']") {
        let node = og_title.as_node();
        if let Some(element) = node.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try Twitter title
    if let Ok(twitter_title) = document.select_first("meta[name='twitter:title']") {
        let node = twitter_title.as_node();
        if let Some(element) = node.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try standard title tag
    if let Ok(title) = document.select_first("title") {
        let title_text = title.text_contents();
        if !title_text.is_empty() {
            return Some(title_text);
        }
    }
    
    // Try h1
    if let Ok(h1) = document.select_first("h1") {
        let h1_text = h1.text_contents();
        if !h1_text.is_empty() {
            return Some(h1_text);
        }
    }
    
    None
}

/// Extract the author from a document
fn extract_author(document: &NodeRef) -> Option<String> {
    // Try meta author
    if let Ok(meta_author) = document.select_first("meta[name='author']") {
        if let Ok(element) = meta_author.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try article:author
    if let Ok(og_author) = document.select_first("meta[property='article:author']") {
        if let Ok(element) = og_author.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try common author classes
    for class_name in &["author", "byline", "dc-creator"] {
        let selector = format!(".{}", class_name);
        if let Ok(author_elem) = document.select_first(&selector) {
            let author_text = author_elem.text_contents();
            if !author_text.is_empty() {
                return Some(author_text);
            }
        }
    }
    
    None
}

/// Extract the date from a document
fn extract_date(document: &NodeRef) -> Option<String> {
    // Try published date meta
    if let Ok(meta_date) = document.select_first("meta[property='article:published_time']") {
        if let Ok(element) = meta_date.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try date meta
    if let Ok(meta_date) = document.select_first("meta[name='date']") {
        if let Ok(element) = meta_date.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try time elements
    if let Ok(time) = document.select_first("time") {
        if let Ok(element) = time.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(datetime) = attributes.get("datetime") {
                if !datetime.is_empty() {
                    return Some(datetime.to_string());
                }
            }
        }
        
        let time_text = time.text_contents();
        if !time_text.is_empty() {
            if let Some(date_match) = DATE_REGEX.find(&time_text) {
                return Some(date_match.as_str().to_string());
            }
        }
    }
    
    // Try date classes
    for class_name in &["date", "published", "timestamp", "post-date"] {
        let selector = format!(".{}", class_name);
        if let Ok(date_elem) = document.select_first(&selector) {
            let date_text = date_elem.text_contents();
            if !date_text.is_empty() {
                if let Some(date_match) = DATE_REGEX.find(&date_text) {
                    return Some(date_match.as_str().to_string());
                }
                return Some(date_text);
            }
        }
    }
    
    None
}

/// Extract the description from a document
fn extract_description(document: &NodeRef) -> Option<String> {
    // Try Open Graph description
    if let Ok(og_desc) = document.select_first("meta[property='og:description']") {
        if let Ok(element) = og_desc.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try meta description
    if let Ok(meta_desc) = document.select_first("meta[name='description']") {
        if let Ok(element) = meta_desc.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try Twitter description
    if let Ok(twitter_desc) = document.select_first("meta[name='twitter:description']") {
        if let Ok(element) = twitter_desc.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    None
}

/// Extract the site name from a document
fn extract_sitename(document: &NodeRef) -> Option<String> {
    // Try Open Graph site name
    if let Ok(og_site) = document.select_first("meta[property='og:site_name']") {
        if let Ok(element) = og_site.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    
    // Try copyright
    if let Ok(copyright) = document.select_first(".copyright") {
        let text = copyright.text_contents();
        if !text.is_empty() {
            return Some(text);
        }
    }
    
    None
}

/// Extract categories and tags from a document
fn extract_categories(document: &NodeRef) -> Vec<String> {
    let mut categories = Vec::new();
    
    // Try article:section
    if let Ok(section) = document.select_first("meta[property='article:section']") {
        if let Ok(element) = section.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    categories.push(content.to_string());
                }
            }
        }
    }
    
    // Try article:tag
    let tags = document.select("meta[property='article:tag']").unwrap();
    for tag in tags {
        if let Ok(element) = tag.as_element() {
            let attributes = element.attributes.borrow();
            if let Some(content) = attributes.get("content") {
                if !content.is_empty() {
                    categories.push(content.to_string());
                }
            }
        }
    }
    
    // Try common tag classes
    for class_name in &["tags", "categories", "category", "topics"] {
        let selector = format!(".{} a", class_name);
        let links = document.select(&selector).unwrap();
        for link in links {
            let text = link.text_contents();
            if !text.is_empty() {
                categories.push(text);
            }
        }
    }
    
    categories
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuchiki::parse_html;

    #[test]
    fn test_extract_title() {
        let html = r#"
        <html>
            <head>
                <title>Page Title</title>
                <meta property="og:title" content="OG Title">
            </head>
            <body>
                <h1>H1 Title</h1>
            </body>
        </html>
        "#;
        
        let document = parse_html().one(html);
        
        // Should prefer OG title
        assert_eq!(extract_title(&document), Some("OG Title".to_string()));
    }

    #[test]
    fn test_extract_author() {
        let html = r#"
        <html>
            <head>
                <meta name="author" content="John Doe">
            </head>
            <body>
                <div class="author">Jane Smith</div>
            </body>
        </html>
        "#;
        
        let document = parse_html().one(html);
        
        // Should prefer meta author
        assert_eq!(extract_author(&document), Some("John Doe".to_string()));
    }

    #[test]
    fn test_extract_date() {
        let html = r#"
        <html>
            <head>
                <meta property="article:published_time" content="2023-09-01">
            </head>
            <body>
                <time datetime="2023-08-15">August 15, 2023</time>
            </body>
        </html>
        "#;
        
        let document = parse_html().one(html);
        
        // Should prefer article:published_time
        assert_eq!(extract_date(&document), Some("2023-09-01".to_string()));
    }
}
