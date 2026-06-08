use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

pub async fn resolve(client: &Client) -> Result<Vec<String>> {
    let page = client
        .get("https://apod.nasa.gov/apod/astropix.html")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await?;

    let re = Regex::new(r#"(?i)href="(image/[^"]+\.(jpg|jpeg|png|gif))""#).unwrap();
    let urls: Vec<String> = re
        .captures_iter(&page)
        .take(1)
        .filter_map(|c| c.get(1))
        .map(|m| format!("https://apod.nasa.gov/apod/{}", m.as_str()))
        .collect();

    Ok(urls)
}
