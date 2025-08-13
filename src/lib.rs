use serde::{Deserialize, Serialize};
use simple_google::search_and_parse;
use simple_google::SearchResult;

use spider::tokio::task;
use spider::website::Website;
use spider_transformations::transformation::content;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultPage {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub content: String,
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub pages: Vec<SearchResultPage>,
}

/// Search and fetch pages, returning a formatted summary string
pub async fn search_and_fetch_summary(
    query: &str,
    page_count: usize,
    max_chars_per_page: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let results = search_and_fetch_structured(query, page_count, max_chars_per_page).await?;

    let mut summary_content = String::new();
    summary_content.push_str("This is the search results page.\n\n");

    for page in results.pages {
        summary_content.push_str("<page>\n");
        summary_content.push_str(&format!("  <title>{}</title>\n", page.title));
        summary_content.push_str(&format!("  <url>{}</url>\n", page.url));
        summary_content.push_str(&format!("  <snippet>{}</snippet>\n", page.snippet));
        summary_content.push_str(&format!("  <content>{}</content>\n", page.content));
        summary_content.push_str("</page>\n\n");
    }

    Ok(summary_content)
}

/// Search and fetch pages, returning structured data
pub async fn search_and_fetch_structured(
    query: &str,
    page_count: usize,
    max_chars_per_page: usize,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    // Calculate how many Google search pages we need (10 results per page)
    let google_pages = (page_count + 9) / 10;

    let google_results: Vec<SearchResult> = search_and_parse(query, google_pages as u32).await?;

    // Get the required number of links
    let links: Vec<String> = google_results
        .iter()
        .map(|result| result.link.clone())
        .take(page_count)
        .collect();

    println!(
        "Found {} links to fetch (requested: {})",
        links.len(),
        page_count
    );

    // Setup transformation config
    let mut conf = content::TransformConfig::default();
    conf.return_format = content::ReturnFormat::Markdown;
    conf.clean_html = true;
    conf.main_content = true;

    // Concurrent fetch all links
    let mut handles = vec![];

    for (index, link) in links.iter().enumerate() {
        let link_clone = link.clone();
        let google_result = google_results[index].clone();
        let handle = task::spawn(async move {
            let mut website: Website = Website::new(&link_clone);

            website.with_limit(1);
            website.with_depth(1);
            website.configuration.respect_robots_txt = true;
            website.configuration.subdomains = false;
            website.configuration.delay = 0;
            website.configuration.user_agent = Some(Box::new("SpiderBot".into()));

            let start = Instant::now();
            website.scrape().await;
            let duration = start.elapsed();

            match website.get_pages() {
                Some(pages) => {
                    if !pages.is_empty() {
                        println!(
                            "Fetched {} in {:?}, got {} pages",
                            link_clone,
                            duration,
                            pages.len()
                        );

                        for (page_index, page) in pages.iter().enumerate() {
                            let page_markdown =
                                content::transform_content(&page, &conf, &None, &None, &None);
                            let html_content = page.get_html();

                            if page_index == 0 {
                                return Some((google_result, page_markdown, html_content));
                            }
                        }
                    } else {
                        println!("Fetched {} in {:?}, but got no pages", link_clone, duration);
                    }
                }
                None => {
                    println!("Failed to fetch {} in {:?}", link_clone, duration);
                }
            }
            None
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;

    let mut result_pages = Vec::new();
    for result in results {
        if let Ok(Some((search_result, markdown_content, html_content))) = result {
            let trimmed_content = trim_content(&markdown_content, max_chars_per_page);
            result_pages.push(SearchResultPage {
                title: search_result.title,
                url: search_result.link,
                snippet: search_result.snippet,
                content: trimmed_content,
                html: html_content,
            });
        }
    }

    Ok(SearchResults {
        pages: result_pages,
    })
}

/// Trim content to max characters (excluding spaces and newlines)
fn trim_content(content: &str, max_chars: usize) -> String {
    let content_without_whitespace: String =
        content.chars().filter(|c| !c.is_whitespace()).collect();

    if content_without_whitespace.len() <= max_chars {
        return content.to_string();
    }

    let mut char_count = 0;
    let mut result = String::new();

    for ch in content.chars() {
        result.push(ch);
        if !ch.is_whitespace() {
            char_count += 1;
            if char_count >= max_chars {
                break;
            }
        }
    }

    result
}
