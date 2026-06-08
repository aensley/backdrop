use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct BingResponse {
    images: Vec<BingImage>,
}

#[derive(Deserialize)]
struct BingImage {
    urlbase: String,
    url: String,
}

pub async fn resolve(client: &Client) -> Result<Vec<String>> {
    let resp: BingResponse = client
        .get("https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .json()
        .await?;

    let mut urls = Vec::new();
    if let Some(img) = resp.images.first() {
        urls.push(format!("https://www.bing.com{}_UHD.jpg", img.urlbase));
        urls.push(format!("https://www.bing.com{}", img.url));
    }

    Ok(urls)
}
