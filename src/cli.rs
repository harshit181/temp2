//! Command-line interface for Trafilatura Rust port.
//! This module provides the CLI functionality for extracting content from URLs or files.

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Arg, Command, ArgAction, ArgMatches};
use log::{info, debug, error};
use url::Url;

use crate::{
    extract_file, extract_url, extract_html,
    ExtractionConfig, ExtractionResult, OutputFormat, TrafilaturaError
};

/// Build the command-line interface
pub fn build_cli() -> Command {
    Command::new("trafilatura")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Trafilatura Rust Port Contributors")
        .about("Extract content from web pages")
        .arg(
            Arg::new("input")
                .help("URL, file path, or HTML string to process")
                .required(false)
                .index(1)
        )
        .arg(
            Arg::new("url")
                .long("url")
                .short('u')
                .help("URL to download and process")
                .value_name("URL")
                .conflicts_with("input")
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .help("HTML file to process")
                .value_name("FILE")
                .conflicts_with_all(["input", "url"])
        )
        .arg(
            Arg::new("input_format")
                .long("input-format")
                .short('i')
                .help("Input format (auto, html, or txt)")
                .default_value("auto")
                .value_parser(["auto", "html", "txt"])
        )
        .arg(
            Arg::new("output_format")
                .long("output-format")
                .short('o')
                .help("Output format (txt, html, json, or xml)")
                .default_value("txt")
                .value_parser(["txt", "html", "json", "xml"])
        )
        .arg(
            Arg::new("output_file")
                .long("output")
                .short('O')
                .help("Output file (default: stdout)")
                .value_name("FILE")
        )
        .arg(
            Arg::new("include_comments")
                .long("include-comments")
                .help("Include comments in the extraction")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("include_tables")
                .long("include-tables")
                .help("Include tables in the extraction")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("include_links")
                .long("include-links")
                .help("Include links in the extraction")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("include_images")
                .long("include-images")
                .help("Include images in the extraction")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("extract_metadata")
                .long("extract-metadata")
                .help("Extract metadata from the document")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("min_extracted_size")
                .long("min-extracted-size")
                .help("Minimum size of extracted content to be considered valid")
                .default_value("250")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .help("Timeout for HTTP requests in seconds")
                .default_value("30")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("user_agent")
                .long("user-agent")
                .help("User-Agent string for HTTP requests")
                .default_value("Mozilla/5.0 (compatible; trafilatura-rs/0.1; +https://github.com/user/trafilatura-rs)")
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Verbose output")
                .action(ArgAction::SetTrue)
        )
}

/// Parse command-line arguments and set up configuration
pub fn parse_args(matches: &ArgMatches) -> Result<(ExtractionConfig, String, Option<PathBuf>), TrafilaturaError> {
    // Set up extraction configuration
    let mut config = ExtractionConfig {
        include_comments: matches.get_flag("include_comments"),
        include_tables: matches.get_flag("include_tables"),
        include_links: matches.get_flag("include_links"),
        include_images: matches.get_flag("include_images"),
        extract_metadata: matches.get_flag("extract_metadata"),
        min_extracted_size: *matches.get_one::<usize>("min_extracted_size").unwrap(),
        extraction_timeout: *matches.get_one::<u64>("timeout").unwrap(),
        user_agent: matches.get_one::<String>("user_agent").unwrap().clone(),
        output_format: match matches.get_one::<String>("output_format").unwrap().as_str() {
            "txt" => OutputFormat::Text,
            "html" => OutputFormat::Html,
            "json" => OutputFormat::Json,
            "xml" => OutputFormat::Xml,
            _ => OutputFormat::Text,
        },
    };

    // Get input source
    let input_source = if let Some(url) = matches.get_one::<String>("url") {
        url.clone()
    } else if let Some(file) = matches.get_one::<String>("file") {
        file.clone()
    } else if let Some(input) = matches.get_one::<String>("input") {
        input.clone()
    } else {
        return Err(TrafilaturaError::ExtractionError(
            "No input provided. Use --url, --file, or positional argument".to_string(),
        ));
    };

    // Get output file if specified
    let output_file = matches
        .get_one::<String>("output_file")
        .map(|path| PathBuf::from(path));

    Ok((config, input_source, output_file))
}

/// Process the input according to its type (URL, file, HTML string)
pub fn process_input(config: &ExtractionConfig, input: &str) -> Result<ExtractionResult, TrafilaturaError> {
    // Determine if input is a URL
    if input.starts_with("http://") || input.starts_with("https://") {
        debug!("Processing input as URL: {}", input);
        extract_url(input, config)
    }
    // Determine if input is a file path
    else if std::path::Path::new(input).exists() {
        debug!("Processing input as file: {}", input);
        extract_file(input, config)
    }
    // Otherwise, treat as HTML string
    else {
        debug!("Processing input as HTML string");
        extract_html(input, config)
    }
}

/// Format the extraction result according to the specified output format
pub fn format_result(result: &ExtractionResult, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => result.content.clone(),
        
        OutputFormat::Html => {
            let mut html = String::new();
            
            if let Some(title) = &result.title {
                html.push_str(&format!("<h1>{}</h1>\n", html_escape::encode_text(title)));
            }
            
            if let Some(author) = &result.author {
                html.push_str(&format!("<p class=\"author\">By: {}</p>\n", html_escape::encode_text(author)));
            }
            
            if let Some(date) = &result.date {
                html.push_str(&format!("<p class=\"date\">Date: {}</p>\n", html_escape::encode_text(date)));
            }
            
            html.push_str(&format!("<div class=\"content\">{}</div>\n", result.content));
            
            html
        },
        
        OutputFormat::Json => {
            let mut json_obj = serde_json::json!({
                "content": result.content,
            });
            
            if let Some(title) = &result.title {
                json_obj["title"] = serde_json::Value::String(title.clone());
            }
            
            if let Some(author) = &result.author {
                json_obj["author"] = serde_json::Value::String(author.clone());
            }
            
            if let Some(date) = &result.date {
                json_obj["date"] = serde_json::Value::String(date.clone());
            }
            
            if let Some(description) = &result.description {
                json_obj["description"] = serde_json::Value::String(description.clone());
            }
            
            if let Some(sitename) = &result.sitename {
                json_obj["sitename"] = serde_json::Value::String(sitename.clone());
            }
            
            if let Some(url) = &result.url {
                json_obj["url"] = serde_json::Value::String(url.clone());
            }
            
            if !result.categories.is_empty() {
                json_obj["categories"] = serde_json::Value::Array(
                    result.categories.iter().map(|c| serde_json::Value::String(c.clone())).collect()
                );
            }
            
            serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
        },
        
        OutputFormat::Xml => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document>\n");
            
            if let Some(title) = &result.title {
                xml.push_str(&format!("  <title>{}</title>\n", html_escape::encode_text(title)));
            }
            
            if let Some(author) = &result.author {
                xml.push_str(&format!("  <author>{}</author>\n", html_escape::encode_text(author)));
            }
            
            if let Some(date) = &result.date {
                xml.push_str(&format!("  <date>{}</date>\n", html_escape::encode_text(date)));
            }
            
            if let Some(description) = &result.description {
                xml.push_str(&format!("  <description>{}</description>\n", html_escape::encode_text(description)));
            }
            
            if let Some(sitename) = &result.sitename {
                xml.push_str(&format!("  <sitename>{}</sitename>\n", html_escape::encode_text(sitename)));
            }
            
            if let Some(url) = &result.url {
                xml.push_str(&format!("  <url>{}</url>\n", html_escape::encode_text(url)));
            }
            
            if !result.categories.is_empty() {
                xml.push_str("  <categories>\n");
                for category in &result.categories {
                    xml.push_str(&format!("    <category>{}</category>\n", html_escape::encode_text(category)));
                }
                xml.push_str("  </categories>\n");
            }
            
            xml.push_str(&format!("  <content>{}</content>\n", html_escape::encode_text(&result.content)));
            xml.push_str("</document>");
            
            xml
        },
    }
}

/// Write output to a file or stdout
pub fn write_output(output: &str, output_file: Option<PathBuf>) -> Result<(), TrafilaturaError> {
    match output_file {
        Some(path) => {
            let mut file = File::create(path)?;
            file.write_all(output.as_bytes())?;
            Ok(())
        },
        None => {
            // Write to stdout
            io::stdout().write_all(output.as_bytes())?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;
    use tempfile::NamedTempFile;

    #[test]
    fn test_build_cli() {
        let app = build_cli();
        assert_eq!(app.get_name(), "trafilatura");
    }

    #[test]
    fn test_format_result() {
        let result = ExtractionResult {
            content: "Test content".to_string(),
            title: Some("Test Title".to_string()),
            author: Some("Test Author".to_string()),
            date: Some("2023-01-01".to_string()),
            url: Some("https://example.com".to_string()),
            description: Some("Test Description".to_string()),
            sitename: Some("Example".to_string()),
            categories: vec!["test".to_string(), "example".to_string()],
        };
        
        // Test text format
        let text_output = format_result(&result, OutputFormat::Text);
        assert_eq!(text_output, "Test content");
        
        // Test HTML format
        let html_output = format_result(&result, OutputFormat::Html);
        assert!(html_output.contains("<h1>Test Title</h1>"));
        assert!(html_output.contains("<p class=\"author\">By: Test Author</p>"));
        
        // Test JSON format
        let json_output = format_result(&result, OutputFormat::Json);
        assert!(json_output.contains("\"title\""));
        assert!(json_output.contains("\"author\""));
        assert!(json_output.contains("\"categories\""));
        
        // Test XML format
        let xml_output = format_result(&result, OutputFormat::Xml);
        assert!(xml_output.contains("<title>Test Title</title>"));
        assert!(xml_output.contains("<author>Test Author</author>"));
        assert!(xml_output.contains("<category>test</category>"));
    }

    #[test]
    fn test_write_output() -> Result<(), Box<dyn std::error::Error>> {
        let content = "Test output content";
        
        // Test writing to file
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_path_buf();
        
        write_output(content, Some(temp_path.clone()))?;
        
        let file_content = std::fs::read_to_string(temp_path)?;
        assert_eq!(file_content, content);
        
        Ok(())
    }
}
