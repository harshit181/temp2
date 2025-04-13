//! Metadata extraction for Trafilatura Rust port.
//! This module contains utilities for extracting metadata from HTML documents.

use scraper::{Html, Selector};
use regex::Regex;
use lazy_static::lazy_static;

use crate::{ExtractionResult, TrafilaturaError};

lazy_static! {
    /// Regex to match dates in common formats
    static ref DATE_REGEX: Regex = Regex::new(
        r"(?i)\d{4}[-/]\d{1,2}[-/]\d{1,2}|\d{1,2}[-/]\d{1,2}[-/]\d{4}|(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]* \d{1,2},? \d{4}"
    ).unwrap();
}

/// Extract metadata from a document
pub fn extract_metadata(document: &Html, mut result: ExtractionResult) -> Result<ExtractionResult, TrafilaturaError> {
    // Extract title if not already set
    if result.title.is_none() {
        result.title = extract_title(document);
    }
    
    // Extract author if not already set
    if result.author.is_none() {
        result.author = extract_author(document);
    }
    
    // Extract date if not already set
    if result.date.is_none() {
        result.date = extract_date(document);
    }
    
    // Extract description if not already set
    if result.description.is_none() {
        result.description = extract_description(document);
    }
    
    // Extract sitename if not already set
    if result.sitename.is_none() {
        result.sitename = extract_sitename(document);
    }
    
    // Extract categories
    result.categories = extract_categories(document);
    
    Ok(result)
}

/// Extract the title from a document
fn extract_title(document: &Html) -> Option<String> {
    // Try Open Graph title
    let og_title_selector = Selector::parse("meta[property='og:title']").unwrap();
    if let Some(og_title) = document.select(&og_title_selector).next() {
        if let Some(content) = og_title.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try Twitter title
    let twitter_title_selector = Selector::parse("meta[name='twitter:title']").unwrap();
    if let Some(twitter_title) = document.select(&twitter_title_selector).next() {
        if let Some(content) = twitter_title.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try standard title tag
    let title_selector = Selector::parse("title").unwrap();
    if let Some(title) = document.select(&title_selector).next() {
        let title_text = title.text().collect::<Vec<_>>().join(" ");
        if !title_text.is_empty() {
            return Some(title_text);
        }
    }
    
    // Try h1
    let h1_selector = Selector::parse("h1").unwrap();
    if let Some(h1) = document.select(&h1_selector).next() {
        let h1_text = h1.text().collect::<Vec<_>>().join(" ");
        if !h1_text.is_empty() {
            return Some(h1_text);
        }
    }
    
    None
}

/// Extract the author from a document
fn extract_author(document: &Html) -> Option<String> {
    // Try meta author
    let meta_author_selector = Selector::parse("meta[name='author']").unwrap();
    if let Some(meta_author) = document.select(&meta_author_selector).next() {
        if let Some(content) = meta_author.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try article:author
    let og_author_selector = Selector::parse("meta[property='article:author']").unwrap();
    if let Some(og_author) = document.select(&og_author_selector).next() {
        if let Some(content) = og_author.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try common author classes
    for class_name in &["author", "byline", "dc-creator"] {
        let selector = Selector::parse(&format!(".{}", class_name)).unwrap();
        if let Some(author_elem) = document.select(&selector).next() {
            let author_text = author_elem.text().collect::<Vec<_>>().join(" ");
            if !author_text.is_empty() {
                return Some(author_text);
            }
        }
    }
    
    None
}

/// Extract the date from a document
fn extract_date(document: &Html) -> Option<String> {
    // Try published date meta
    let published_time_selector = Selector::parse("meta[property='article:published_time']").unwrap();
    if let Some(meta_date) = document.select(&published_time_selector).next() {
        if let Some(content) = meta_date.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try date meta
    let date_selector = Selector::parse("meta[name='date']").unwrap();
    if let Some(meta_date) = document.select(&date_selector).next() {
        if let Some(content) = meta_date.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try time elements
    let time_selector = Selector::parse("time").unwrap();
    if let Some(time) = document.select(&time_selector).next() {
        if let Some(datetime) = time.value().attr("datetime") {
            if !datetime.is_empty() {
                return Some(datetime.to_string());
            }
        }
        
        let time_text = time.text().collect::<Vec<_>>().join(" ");
        if !time_text.is_empty() {
            if let Some(date_match) = DATE_REGEX.find(&time_text) {
                return Some(date_match.as_str().to_string());
            }
        }
    }
    
    // Try date classes
    for class_name in &["date", "published", "timestamp", "post-date"] {
        let selector = Selector::parse(&format!(".{}", class_name)).unwrap();
        if let Some(date_elem) = document.select(&selector).next() {
            let date_text = date_elem.text().collect::<Vec<_>>().join(" ");
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
fn extract_description(document: &Html) -> Option<String> {
    // Try Open Graph description
    let og_desc_selector = Selector::parse("meta[property='og:description']").unwrap();
    if let Some(og_desc) = document.select(&og_desc_selector).next() {
        if let Some(content) = og_desc.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try meta description
    let meta_desc_selector = Selector::parse("meta[name='description']").unwrap();
    if let Some(meta_desc) = document.select(&meta_desc_selector).next() {
        if let Some(content) = meta_desc.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try Twitter description
    let twitter_desc_selector = Selector::parse("meta[name='twitter:description']").unwrap();
    if let Some(twitter_desc) = document.select(&twitter_desc_selector).next() {
        if let Some(content) = twitter_desc.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    None
}

/// Extract the site name from a document
fn extract_sitename(document: &Html) -> Option<String> {
    // Try Open Graph site name
    let og_site_selector = Selector::parse("meta[property='og:site_name']").unwrap();
    if let Some(og_site) = document.select(&og_site_selector).next() {
        if let Some(content) = og_site.value().attr("content") {
            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }
    
    // Try copyright
    let copyright_selector = Selector::parse(".copyright").unwrap();
    if let Some(copyright) = document.select(&copyright_selector).next() {
        let text = copyright.text().collect::<Vec<_>>().join(" ");
        if !text.is_empty() {
            return Some(text);
        }
    }
    
    None
}

/// Extract categories and tags from a document
fn extract_categories(document: &Html) -> Vec<String> {
    let mut categories = Vec::new();
    
    // Try article:section
    let section_selector = Selector::parse("meta[property='article:section']").unwrap();
    if let Some(section) = document.select(&section_selector).next() {
        if let Some(content) = section.value().attr("content") {
            if !content.is_empty() {
                categories.push(content.to_string());
            }
        }
    }
    
    // Try article:tag
    let tag_selector = Selector::parse("meta[property='article:tag']").unwrap();
    for tag in document.select(&tag_selector) {
        if let Some(content) = tag.value().attr("content") {
            if !content.is_empty() {
                categories.push(content.to_string());
            }
        }
    }
    
    // Try common tag classes
    for class_name in &["tags", "categories", "category", "topics"] {
        let selector = Selector::parse(&format!(".{} a", class_name)).unwrap();
        for link in document.select(&selector) {
            let text = link.text().collect::<Vec<_>>().join(" ");
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
    use scraper::Html;

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
        
        let document = Html::parse_document(html);
        
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
        
        let document = Html::parse_document(html);
        
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
        
        let document = Html::parse_document(html);
        
        // Should prefer article:published_time
        assert_eq!(extract_date(&document), Some("2023-09-01".to_string()));
    }
}