use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

pub async fn resolve(client: &Client) -> Result<Vec<String>> {
    let feed = client
        .get("https://www.nasa.gov/feeds/iotd-feed/")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await?;

    let re = Regex::new(r#"<enclosure[^>]+url="([^"]+)""#).unwrap();
    let urls: Vec<String> = re
        .captures_iter(&feed)
        .take(1)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    Ok(urls)
}
