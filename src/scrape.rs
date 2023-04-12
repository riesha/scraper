use anyhow::*;
use async_recursion::async_recursion;
use itertools::Itertools;

use rand::seq::IteratorRandom;
use reqwest::{Client, Url};
use select::{document::Document, predicate::Name};
use std::result::Result::Ok as StdOk;

#[macro_export]
macro_rules! bench {
    ($func:stmt,$name:literal) => {
        let start = Instant::now();
        $func
        let duration = start.elapsed();
        println!("Time elapsed in {} is: {:?}", $name, duration);
    };
}

pub async fn get_all_domestic_links(client: &Client, url: Url) -> Result<Vec<Url>>
{
    //println!("Scraping [{}]...", url.as_str());
    let content = client.get(url).send().await?.text().await?;
    let document = Document::from(content.as_str());

    let links: Vec<Url> = document
        .find(Name("a"))
        .filter_map(|lnk| lnk.attr("href").and_then(|x| Url::parse(x.trim()).ok()))
        .unique()
        .collect();

    if links.is_empty()
    {
        Err(anyhow!("No links found on page!"))
    }
    else
    {
        Ok(links)
    }
}
pub async fn get_all_images(client: &Client, url: Url) -> Result<Vec<Url>>
{
    let content = client.get(url).send().await?.text().await?;
    let document = Document::from(content.as_str());

    let images = document
        .find(Name("img"))
        .filter_map(|img| img.attr("src").and_then(|x| Url::parse(x.trim()).ok()))
        .unique()
        .collect_vec();
    Ok(images)
}
#[async_recursion]
pub async fn scrape_all(
    client: &Client, url: Url, visited: &mut Vec<Url>, links: &mut Vec<String>,
) -> Result<()>
{
    if !visited.contains(&url)
    {
        visited.push(url.clone());
        println!("scrape_all with {}", url);

        let StdOk(seed_links) = get_all_domestic_links(client, url.clone()).await else {return Ok(())};
        let og = Url::parse(&url.clone().origin().ascii_serialization())?;

        let tasks: Vec<_> = seed_links
            .into_iter()
            .filter_map(|lnk| {
                let cl = client.clone();
                let ur = if lnk.cannot_be_a_base()
                {
                    og.join(lnk.as_str()).unwrap()
                }
                else
                {
                    lnk
                };
                if !links.contains(&ur.to_string())
                {
                    println!("new link to visit! {}", ur.as_str());
                    links.push(ur.to_string());

                    Some(tokio::spawn(async move {
                        get_all_domestic_links(&cl, ur).await
                    }))
                }
                else
                {
                    None
                }
            })
            .collect();

        if tasks.is_empty()
        {
            links.sort();
            links.dedup();
            return Ok(());
        }
        for task in tasks
        {
            let StdOk(task) = &mut task.await? else {continue};
            links.append(&mut task.iter().map(|x| x.to_string()).collect_vec());
        }
    }
    let url = links
        .iter()
        .filter(|x| !visited.contains(&Url::parse(x).unwrap()))
        .choose(&mut rand::thread_rng())
        .and_then(|x| Url::parse(x).ok())
        .unwrap();
    scrape_all(client, url, visited, links).await
}
