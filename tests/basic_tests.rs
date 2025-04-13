//! Integration tests for trafilatura-rs
//!
//! These tests verify that the main functionality of the library works as expected.

use trafilatura::{
    extract_html, extract_url, extract_file,
    ExtractionConfig, OutputFormat, TrafilaturaError
};

// A basic HTML document for testing
const TEST_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Document</title>
    <meta name="author" content="Test Author">
    <meta name="description" content="Test Description">
</head>
<body>
    <header>
        <nav>
            <ul>
                <li><a href="#">Home</a></li>
                <li><a href="#">About</a></li>
            </ul>
        </nav>
    </header>
    
    <main>
        <article>
            <h1>Test Heading</h1>
            <p>This is a test paragraph with <a href="https://example.com">a link</a>.</p>
            <p>This is another paragraph with more content.</p>
        </article>
    </main>
    
    <footer>
        <p>Copyright 2023</p>
    </footer>
</body>
</html>
"#;

#[test]
fn test_extract_html_basic() {
    let config = ExtractionConfig::default();
    let result = extract_html(TEST_HTML, &config).unwrap();
    
    assert!(result.content.contains("Test Heading"));
    assert!(result.content.contains("This is a test paragraph"));
    assert!(result.content.contains("another paragraph"));
    assert!(!result.content.contains("Copyright 2023"));
}

#[test]
fn test_extract_html_with_metadata() {
    let mut config = ExtractionConfig::default();
    config.extract_metadata = true;
    
    let result = extract_html(TEST_HTML, &config).unwrap();
    
    assert_eq!(result.title, Some("Test Document".to_string()));
    assert_eq!(result.author, Some("Test Author".to_string()));
    assert_eq!(result.description, Some("Test Description".to_string()));
}

#[test]
fn test_extract_html_with_links() {
    let mut config = ExtractionConfig::default();
    config.include_links = true;
    
    let result = extract_html(TEST_HTML, &config).unwrap();
    
    assert!(result.content.contains("a link"));
    assert!(result.content.contains("https://example.com"));
}

#[test]
fn test_extract_html_without_links() {
    let mut config = ExtractionConfig::default();
    config.include_links = false;
    
    let result = extract_html(TEST_HTML, &config).unwrap();
    
    assert!(result.content.contains("a link"));
    assert!(!result.content.contains("https://example.com"));
}

#[test]
fn test_min_extracted_size() {
    let mut config = ExtractionConfig::default();
    config.min_extracted_size = 1000; // Set to a very high value
    
    let result = extract_html(TEST_HTML, &config);
    
    assert!(result.is_err());
    match result {
        Err(TrafilaturaError::ExtractionError(msg)) => {
            assert!(msg.contains("Extracted content too short"));
        }
        _ => panic!("Expected ExtractionError"),
    }
}

// This test is commented out because it requires network access
// Uncomment to run manually
/*
#[test]
fn test_extract_url() {
    let config = ExtractionConfig::default();
    let result = extract_url("https://example.com", &config);
    
    assert!(result.is_ok());
    let content = result.unwrap();
    
    assert!(!content.content.is_empty());
    assert_eq!(content.url, Some("https://example.com/".to_string()));
}
*/

// This test creates a temporary file and tests the extract_file function
#[test]
fn test_extract_file() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    // Create a temporary file with our test HTML
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(TEST_HTML.as_bytes())?;
    
    let config = ExtractionConfig::default();
    let result = extract_file(temp_file.path(), &config)?;
    
    assert!(result.content.contains("Test Heading"));
    assert!(result.content.contains("This is a test paragraph"));
    
    Ok(())
}
