use anyhow::*;
use async_recursion::async_recursion;
use itertools::Itertools;
use reqwest::{Client, Url};
use select::{document::Document, predicate::Name};
pub async fn get_all_domestic_links(client: &Client, url: Url) -> Result<Vec<Url>>
{
    println!("Scraping [{}]...", url.as_str());
    let content = client.get(url).send().await?.text().await?;
    let document = Document::from(content.as_str());

    let links: Vec<Url> = document
        .find(Name("a"))
        .filter_map(|lnk| lnk.attr("href").map(|x| Url::parse(x.trim()).ok()))
        .unique()
        .filter_map(|url| url)
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
#[async_recursion]
pub async fn scrape_all(
    client: &Client, url: Url, visited: &mut Vec<Url>, links: &mut Vec<String>,
) -> Result<()>
{
    let std::result::Result::Ok(seed_links) = get_all_domestic_links(&client, url.clone()).await else {return Ok(())};
    let og = Url::parse(&url.clone().origin().ascii_serialization())?;

    let tasks: Vec<_> = seed_links
        .into_iter()
        .map(|lnk| {
            let cl = client.clone();
            let ur = if lnk.cannot_be_a_base()
            {
                og.join(lnk.as_str()).unwrap()
            }
            else
            {
                lnk
            };
            if !visited.contains(&ur)
            {
                println!("new link to visit! {}", ur.as_str());
                visited.push(ur.clone());

                Some(tokio::spawn(async move {
                    get_all_domestic_links(&cl, ur).await
                }))
            }
            else
            {
                None
            }
        })
        .filter_map(|task| task)
        .collect();

    if tasks.is_empty()
    {
        links.sort();
        links.dedup();
        return Ok(());
    }
    for task in tasks
    {
        let std::result::Result::Ok(task) = &mut task.await? else {continue};
        links.append(&mut task.iter().map(|x| x.to_string()).collect_vec());
    }

    scrape_all(client, url, visited, links).await
}
