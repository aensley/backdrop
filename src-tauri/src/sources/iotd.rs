use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

use super::{extract_tag, ImageInfo};

pub async fn resolve(client: &Client) -> Result<ImageInfo> {
    let feed = client
        .get("https://www.nasa.gov/feeds/iotd-feed/")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await?;

    let item = feed
        .find("<item>")
        .and_then(|start| {
            feed[start..]
                .find("</item>")
                .map(|end| &feed[start..start + end + "</item>".len()])
        })
        .unwrap_or("");

    let url_re = Regex::new(r#"<enclosure[^>]+url="([^"]+)""#).unwrap();
    let urls: Vec<String> = url_re
        .captures_iter(&feed)
        .take(1)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    Ok(ImageInfo {
        urls,
        title: extract_tag(item, "title"),
        description: extract_tag(item, "description"),
        page_url: extract_tag(item, "link"),
    })
}
