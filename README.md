# Trafilatura Rust Port

A Rust implementation of [Python's trafilatura library](https://github.com/adbar/trafilatura) for extracting text content from web pages. This library provides efficient and accurate extraction of main content, removing boilerplate, navigation, and other non-content elements.

## Features

- Extract main content from HTML web pages
- Remove boilerplate, navigation, and other non-content elements
- Extract metadata (title, author, date, etc.)
- Support for different output formats (text, HTML, JSON, XML)
- Command-line interface similar to the Python version
- Performance optimized with Rust's zero-cost abstractions

## Installation

### From crates.io

```bash
cargo install trafilatura-rs
```

### From source

```bash
git clone https://github.com/user/trafilatura-rs
cd trafilatura-rs
cargo build --release
```

## Usage

### Command-line interface

```bash
# Extract content from a URL
trafilatura --url https://example.com

# Extract content from a local HTML file
trafilatura --file input.html

# Extract content and convert to JSON format
trafilatura --url https://example.com --output-format json

# Extract content with metadata
trafilatura --url https://example.com --extract-metadata

# Save output to a file
trafilatura --url https://example.com --output output.txt

# Show help
trafilatura --help
```

### As a library

```rust
use trafilatura::{extract_url, ExtractionConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractionConfig::default();
    
    // Extract content from a URL
    let result = extract_url("https://example.com", &config)?;
    
    println!("Title: {}", result.title.unwrap_or_default());
    println!("Content: {}", result.content);
    
    Ok(())
}
```

## Customizing Extraction

You can customize the extraction process by modifying the `ExtractionConfig` struct:

```rust
use trafilatura::{extract_url, ExtractionConfig, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExtractionConfig {
        include_links: true,
        include_images: true,
        extract_metadata: true,
        output_format: OutputFormat::Json,
        ..ExtractionConfig::default()
    };
    
    let result = extract_url("https://example.com", &config)?;
    println!("{}", result.content);
    
    Ok(())
}
```

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.

## Acknowledgements

- Original Python [trafilatura library](https://github.com/adbar/trafilatura) by Adrien Barbaresi
- Inspired by readability algorithms from Mozilla and other content extraction techniques
