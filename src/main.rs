use anyhow::*;
use reqwest::{Client, Url};
use scraper::scrape;

static URL: &str = "https://kaomoji-copy.com";

#[tokio::main]
async fn main() -> Result<()>
{
    let url = Url::parse(URL)?;
    let client = Client::builder().build()?;

    let mut visited_links = vec![];
    let mut links = vec![];
    let _ = scrape::scrape_all(&client, url, &mut visited_links, &mut links).await?;
    dbg!(&links);
    println!(
        "Visited {} links for {} links scraped",
        visited_links.len(),
        links.len()
    );
    Ok(())
}
