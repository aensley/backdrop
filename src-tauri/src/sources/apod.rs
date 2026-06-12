use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

use super::{clean_text, ImageInfo};

pub async fn resolve(client: &Client) -> Result<ImageInfo> {
    let page = client
        .get("https://apod.nasa.gov/apod/astropix.html")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await?;

    let url_re = Regex::new(r#"(?i)href="(image/[^"]+\.(jpg|jpeg|png|gif))""#).unwrap();
    let urls: Vec<String> = url_re
        .captures_iter(&page)
        .take(1)
        .filter_map(|c| c.get(1))
        .map(|m| format!("https://apod.nasa.gov/apod/{}", m.as_str()))
        .collect();

    // Title is the first <b> inside a <center> block; credits follow in the same block
    let block_re = Regex::new(r"(?is)<center>\s*<b>\s*([^<]+?)\s*</b>(.*?)</center>").unwrap();
    let (title, description) = block_re
        .captures(&page)
        .map(|c| {
            let t = c
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty());
            let d = c.get(2).map(|m| clean_text(m.as_str())).filter(|s| !s.is_empty());
            (t, d)
        })
        .unwrap_or((None, None));

    Ok(ImageInfo {
        urls,
        title,
        description,
        page_url: Some("https://apod.nasa.gov/apod/astropix.html".to_string()),
    })
}
