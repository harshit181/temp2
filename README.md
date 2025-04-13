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
trafilatura https://example.com

# Extract content from a local HTML file
trafilatura path/to/file.html

# Extract content from HTML passed via stdin
cat file.html | trafilatura

# Extract content with specified minimum length
trafilatura --min-extracted-size 100 https://example.com

# Extract content and metadata in JSON format
trafilatura -f json -m https://example.com

# Extract content and metadata in XML format
trafilatura -f xml -m https://example.com

# Save output to a file
trafilatura -o output.txt https://example.com

# Show help for all options
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
        min_extracted_size: 100,
        ..ExtractionConfig::default()
    };
    
    let result = extract_url("https://example.com", &config)?;
    println!("{}", result.content);
    
    Ok(())
}
```

## Implementation Details

This port uses the `scraper` library (based on `html5ever`) for HTML parsing, instead of the outdated `kuchiki` library. The main extraction algorithms follow the same approach as the Python original:

1. First attempt extraction using semantic HTML elements (article tags)
2. If that fails, try content extraction based on class/ID hints
3. If that fails, try extraction based on content density calculation
4. If all else fails, fall back to the readability algorithm

The command-line interface supports all the same options as the Python version, with a similar usage pattern.

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.

## Acknowledgements

- Original Python [trafilatura library](https://github.com/adbar/trafilatura) by Adrien Barbaresi
- Inspired by readability algorithms from Mozilla and other content extraction techniques
