use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

use super::{extract_tag, ImageInfo};

pub async fn resolve(client: &Client) -> Result<ImageInfo> {
    let feed = client
        .get("https://science.nasa.gov/feed/earth-observatory/image-of-the-day")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await?;

    let item = match feed.find("<item>") {
        Some(start) => match feed[start..].find("</item>") {
            Some(end) => &feed[start..start + end + "</item>".len()],
            None => "",
        },
        None => "",
    };

    let re = Regex::new(r#"(?i)https://assets\.science\.nasa\.gov/dynamicimage/[^"?]+\.(jpg|jpeg|png)"#).unwrap();

    let mut urls = Vec::new();
    if let Some(m) = re.find(item) {
        let base = m
            .as_str()
            .replace("&#039;", "'")
            .replace("&amp;", "&")
            .replace("&quot;", "\"");
        urls.push(format!("{base}?w=3840"));
        urls.push(base);
    }

    let footer_re = Regex::new(r"\s*The post .+ appeared first on [^.]+\.\s*$").unwrap();
    let description = extract_tag(item, "description").and_then(|d| {
        let trimmed = footer_re.replace(&d, "").trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    Ok(ImageInfo {
        urls,
        title: extract_tag(item, "title"),
        description,
        page_url: extract_tag(item, "link"),
    })
}
