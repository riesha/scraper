use std::net::SocketAddrV4;

use anyhow::*;
use itertools::Itertools;
use rand::seq::SliceRandom;
use reqwest::{Client, Proxy, Url};
use scraper::scrape;
use tokio::fs;

static URL: &str = "https://kaomoji-copy.com";

#[tokio::main]
async fn main() -> Result<()>
{
    let url = Url::parse(URL)?;

    let file = fs::read_to_string("proxies.txt").await?;

    let proxies = file
        .lines()
        .filter_map(|line| {
            line.parse::<SocketAddrV4>()
                .ok()
                .and_then(|addr| Url::parse(&addr.to_string()).ok())
        })
        .collect_vec();

    let client = Client::builder()
        .proxy(Proxy::custom(move |_url| {
            proxies.choose(&mut rand::thread_rng()).cloned()
        }))
        .build()?;

    let mut visited_links = vec![];
    let mut links = vec![];
    scrape::scrape_all(&client, url, &mut visited_links, &mut links).await?;
    dbg!(&links);
    dbg!(&visited_links);
    println!(
        "Visited {} links for {} links scraped",
        visited_links.len(),
        links.len()
    );
    // let images = scrape::get_all_images(&client, url).await?;
    // dbg!(&images);
    Ok(())
}
