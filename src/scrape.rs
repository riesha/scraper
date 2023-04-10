use anyhow::*;
use async_recursion::async_recursion;
use reqwest::{Client, Url};
use select::{document::Document, predicate::Name};

pub async fn get_all_domestic_links(client: &Client, url: Url) -> Result<Vec<String>>
{
    println!("Scraping [{}]...", url.as_str());
    let content = client.get(url).send().await?.text().await?;
    let document = Document::from(content.as_str());

    let mut links: Vec<String> = document
        .find(Name("a"))
        .filter_map(|lnk| lnk.attr("href").map(|x| x.trim().to_string()))
        .filter(|lnk| {
            !lnk.contains("//")
                && !lnk.contains("javascript:void")
                && lnk.len() > 1
                && !lnk.contains(".php")
        })
        .collect();

    links.sort();
    links.dedup();

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
    let og = url.clone().origin().ascii_serialization();

    let tasks: Vec<_> = seed_links
        .into_iter()
        .map(|lnk| {
            let cl = client.clone();
            let ur = Url::parse(&og).unwrap().join(&lnk).unwrap();
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
        return Ok(());
    }
    for task in tasks
    {
        let std::result::Result::Ok(task) = &mut task.await? else {continue};
        links.append(task);
    }
    links.sort();
    links.dedup();

    scrape_all(client, url, visited, links).await
}
