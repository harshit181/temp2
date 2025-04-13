# Trafilatura Rust Port

A Rust implementation of [Python's trafilatura library](https://github.com/adbar/trafilatura) for extracting text content from web pages.

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
