use clap::{Arg, Command};
use simple_search::search_and_fetch_structured;
use spider::tokio;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("simple_search")
        .version("0.1.0")
        .author("Search Tool")
        .about("A simple search and fetch tool")
        .arg(
            Arg::new("query")
                .help("Search query")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("pages")
                .help("Number of pages to fetch")
                .long("pages")
                .short('p')
                .value_parser(clap::value_parser!(usize))
                .default_value("5"),
        )
        .arg(
            Arg::new("max-chars")
                .help("Maximum characters per page")
                .long("max-chars")
                .short('m')
                .value_parser(clap::value_parser!(usize))
                .default_value("5000"),
        )
        .get_matches();

    let query = matches.get_one::<String>("query").unwrap();
    let page_count = *matches.get_one::<usize>("pages").unwrap();
    let max_chars_per_page = *matches.get_one::<usize>("max-chars").unwrap();

    println!(
        "Searching for: '{}' with {} pages, max {} chars per page",
        query, page_count, max_chars_per_page
    );

    let results = search_and_fetch_structured(query, page_count, max_chars_per_page).await?;

    // Create directory for output files
    fs::create_dir_all("fetched_pages")?;

    // Generate summary content
    let mut summary_content = String::new();
    summary_content.push_str("This is the search results page.\n\n");

    // Save individual files and build summary
    for (index, page) in results.pages.iter().enumerate() {
        // Save individual HTML and MD files
        let md_filename = format!("fetched_pages/page_{}.md", index);
        let html_filename = format!("fetched_pages/page_{}.html", index);

        tokio::fs::write(&md_filename, &page.content).await?;
        tokio::fs::write(&html_filename, &page.html).await?;

        println!("Saved content to {} and {}", md_filename, html_filename);

        // Add to summary
        summary_content.push_str("<page>\n");
        summary_content.push_str(&format!("  <title>{}</title>\n", page.title));
        summary_content.push_str(&format!("  <url>{}</url>\n", page.url));
        summary_content.push_str(&format!("  <snippet>{}</snippet>\n", page.snippet));
        summary_content.push_str(&format!("  <content>{}</content>\n", page.content));
        summary_content.push_str("</page>\n\n");
    }

    // Save summary file
    tokio::fs::write("fetched_pages/search_summary.txt", &summary_content).await?;
    println!("Generated search summary at fetched_pages/search_summary.txt");

    println!("Successfully processed {} pages", results.pages.len());

    Ok(())
}
