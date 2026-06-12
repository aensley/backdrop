use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use super::ImageInfo;

#[derive(Deserialize)]
struct BingResponse {
    images: Vec<BingImage>,
}

#[derive(Deserialize)]
struct BingImage {
    urlbase: String,
    url: String,
    title: Option<String>,
    copyright: Option<String>,
    copyrightlink: Option<String>,
}

pub async fn resolve(client: &Client) -> Result<ImageInfo> {
    let resp: BingResponse = client
        .get("https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .json()
        .await?;

    let mut info = ImageInfo {
        urls: Vec::new(),
        title: None,
        description: None,
        page_url: None,
    };

    if let Some(img) = resp.images.first() {
        info.urls.push(format!("https://www.bing.com{}_UHD.jpg", img.urlbase));
        info.urls.push(format!("https://www.bing.com{}", img.url));
        info.title = img.title.clone().filter(|s| !s.is_empty());
        info.description = img.copyright.clone().filter(|s| !s.is_empty());
        info.page_url = img.copyrightlink.clone().filter(|s| !s.is_empty());
    }

    Ok(info)
}
