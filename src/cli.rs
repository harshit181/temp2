//! Command-line interface for Trafilatura Rust port.
//! This module provides the CLI interface for the Trafilatura library.

use std::io::{self, Read};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use clap::{Parser, ValueEnum};
use log::debug;

use crate::{ExtractionConfig, OutputFormat, TrafilaturaError};
use crate::{extract_html, extract_url, extract_file};
use crate::utils::{is_url, is_file_path, is_html_content};

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(name = "trafilatura")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "A Rust port of Python's trafilatura library for extracting text from web pages")]
pub struct Cli {
    /// Input URL, file path, or HTML content
    #[clap(index = 1, required = false)]
    input: Option<String>,
    
    /// Output format
    #[clap(short, long, value_enum, default_value = "text")]
    format: Format,
    
    /// Output file (defaults to stdout)
    #[clap(short = 'o', long)]
    output: Option<PathBuf>,
    
    /// Include tables in the extraction
    #[clap(short = 't', long, default_value = "true")]
    include_tables: bool,
    
    /// Include links in the extraction
    #[clap(short = 'l', long, default_value = "true")]
    include_links: bool,
    
    /// Include images in the extraction
    #[clap(short = 'i', long, default_value = "false")]
    include_images: bool,
    
    /// Include comments in the extraction
    #[clap(short = 'c', long, default_value = "false")]
    include_comments: bool,
    
    /// Extract metadata
    #[clap(short = 'm', long, default_value = "false")]
    extract_metadata: bool,
    
    /// User agent for HTTP requests
    #[clap(short = 'u', long)]
    user_agent: Option<String>,
    
    /// Timeout in seconds for HTTP requests
    #[clap(short = 's', long, default_value = "30")]
    timeout: u64,
    
    /// Minimum extracted content size to be considered valid
    #[clap(long, default_value = "250")]
    min_extracted_size: usize,
    
    /// Be verbose
    #[clap(short, long)]
    verbose: bool,
}

/// Output format enum
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Text,
    Html,
    Json,
    Xml,
}

impl From<Format> for OutputFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Text => OutputFormat::Text,
            Format::Html => OutputFormat::Html,
            Format::Json => OutputFormat::Json,
            Format::Xml => OutputFormat::Xml,
        }
    }
}

/// Run the CLI application
pub fn run() -> Result<(), TrafilaturaError> {
    let cli = Cli::parse();
    
    // Setup logging
    if cli.verbose {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .init();
    }
    
    // Create extraction config
    let config = ExtractionConfig {
        include_comments: cli.include_comments,
        include_tables: cli.include_tables,
        include_links: cli.include_links,
        include_images: cli.include_images,
        output_format: cli.format.into(),
        extraction_timeout: cli.timeout,
        min_extracted_size: cli.min_extracted_size,
        extract_metadata: cli.extract_metadata,
        user_agent: cli.user_agent.unwrap_or_else(|| {
            "Mozilla/5.0 (compatible; trafilatura-rs/0.1; +https://github.com/user/trafilatura-rs)".into()
        }),
    };
    
    // Get input
    let input = match cli.input {
        Some(input) => input,
        None => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };
    
    // Process input
    let result = if is_url(&input) {
        debug!("Processing URL: {}", input);
        extract_url(&input, &config)?
    } else if is_file_path(&input) {
        debug!("Processing file: {}", input);
        extract_file(&input, &config)?
    } else if is_html_content(&input) {
        debug!("Processing HTML content");
        extract_html(&input, &config)?
    } else {
        return Err(TrafilaturaError::ExtractionError(
            "Input is not a URL, file path, or HTML content".into()
        ));
    };
    
    // Format output
    let output = match config.output_format {
        OutputFormat::Text => result.content,
        OutputFormat::Html => format!(
            "<html><body>{}</body></html>",
            result.content
        ),
        OutputFormat::Json => {
            let mut json_obj = serde_json::Map::new();
            json_obj.insert("content".into(), serde_json::Value::String(result.content));
            
            if let Some(title) = result.title {
                json_obj.insert("title".into(), serde_json::Value::String(title));
            }
            
            if let Some(author) = result.author {
                json_obj.insert("author".into(), serde_json::Value::String(author));
            }
            
            if let Some(date) = result.date {
                json_obj.insert("date".into(), serde_json::Value::String(date));
            }
            
            if let Some(url) = result.url {
                json_obj.insert("url".into(), serde_json::Value::String(url));
            }
            
            if let Some(description) = result.description {
                json_obj.insert("description".into(), serde_json::Value::String(description));
            }
            
            if let Some(sitename) = result.sitename {
                json_obj.insert("sitename".into(), serde_json::Value::String(sitename));
            }
            
            if !result.categories.is_empty() {
                let categories = serde_json::Value::Array(
                    result.categories.into_iter()
                        .map(|c| serde_json::Value::String(c))
                        .collect()
                );
                json_obj.insert("categories".into(), categories);
            }
            
            serde_json::to_string_pretty(&serde_json::Value::Object(json_obj))?
        },
        OutputFormat::Xml => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document>\n");
            
            xml.push_str(&format!("  <content><![CDATA[{}]]></content>\n", result.content));
            
            if let Some(title) = result.title {
                xml.push_str(&format!("  <title><![CDATA[{}]]></title>\n", title));
            }
            
            if let Some(author) = result.author {
                xml.push_str(&format!("  <author><![CDATA[{}]]></author>\n", author));
            }
            
            if let Some(date) = result.date {
                xml.push_str(&format!("  <date><![CDATA[{}]]></date>\n", date));
            }
            
            if let Some(url) = result.url {
                xml.push_str(&format!("  <url><![CDATA[{}]]></url>\n", url));
            }
            
            if let Some(description) = result.description {
                xml.push_str(&format!("  <description><![CDATA[{}]]></description>\n", description));
            }
            
            if let Some(sitename) = result.sitename {
                xml.push_str(&format!("  <sitename><![CDATA[{}]]></sitename>\n", sitename));
            }
            
            if !result.categories.is_empty() {
                xml.push_str("  <categories>\n");
                for category in result.categories {
                    xml.push_str(&format!("    <category><![CDATA[{}]]></category>\n", category));
                }
                xml.push_str("  </categories>\n");
            }
            
            xml.push_str("</document>");
            xml
        }
    };
    
    // Output result
    match cli.output {
        Some(path) => {
            let mut file = File::create(path)?;
            file.write_all(output.as_bytes())?;
        },
        None => {
            println!("{}", output);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_conversion() {
        assert_eq!(OutputFormat::from(Format::Text), OutputFormat::Text);
        assert_eq!(OutputFormat::from(Format::Html), OutputFormat::Html);
        assert_eq!(OutputFormat::from(Format::Json), OutputFormat::Json);
        assert_eq!(OutputFormat::from(Format::Xml), OutputFormat::Xml);
    }
}