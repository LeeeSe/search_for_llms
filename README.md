# simple_search

A Rust library and command-line tool for searching web pages and fetching their content.

## Overview

`simple_search` is a tool that performs web searches and retrieves content from the resulting pages. It combines search engine querying with web scraping capabilities to gather structured information from search results.

## Features

- Search using Google search engine
- Concurrently fetch multiple web pages
- Extract main content from web pages
- Return structured data or formatted text
- Save content to local files
- Configurable number of pages and content length

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
simple_search = "0.1.0"
```

Or install the command-line tool:

```bash
cargo install simple_search
```

## Usage

### As a Library

```rust
use simple_search::{search_and_fetch_structured, search_and_fetch_summary};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get structured results
    let results = search_and_fetch_structured("Rust programming", 5, 5000).await?;
    
    // Or get a formatted summary string
    let summary = search_and_fetch_summary("Rust programming", 5, 5000).await?;
    
    println!("{}", summary);
    Ok(())
}
```

### Command Line

```bash
# Basic search
simple_search "Rust programming"

# Search with options
simple_search "Rust programming" --pages 10 --max-chars 10000
```

### Command Line Options

- `query` - Search query (required)
- `--pages` / `-p` - Number of pages to fetch (default: 5)
- `--max-chars` / `-m` - Maximum characters per page (default: 5000)

## Output

The tool creates a `fetched_pages` directory with:

- Individual HTML and Markdown files for each page
- A summary file with all results

## Library Functions

### `search_and_fetch_structured`

Returns structured data in a `SearchResults` struct:

```rust
pub async fn search_and_fetch_structured(
    query: &str,
    page_count: usize,
    max_chars_per_page: usize,
) -> Result<SearchResults, Box<dyn std::error::Error>>
```

### `search_and_fetch_summary`

Returns a formatted string summary:

```rust
pub async fn search_and_fetch_summary(
    query: &str,
    page_count: usize,
    max_chars_per_page: usize,
) -> Result<String, Box<dyn std::error::Error>>
```

## Dependencies

- `clap` - Command line argument parsing
- `serde` - Serialization framework
- `simple_google` - Google search functionality
- `spider` - Web scraping framework
- `futures` - Asynchronous programming utilities

## License

MIT