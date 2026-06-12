use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use super::{clean_text, ImageInfo};

pub async fn resolve(client: &Client) -> Result<ImageInfo> {
    let date = chrono::Local::now().format("%Y-%m-%d");
    let template = format!("{{{{Potd/{date}}}}}");

    let resp: Value = client
        .get(format!(
            "https://commons.wikimedia.org/w/api.php?action=expandtemplates&format=json&prop=wikitext&text={template}"
        ))
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .json()
        .await?;

    let file = match resp["expandtemplates"]["wikitext"].as_str() {
        Some(f) if !f.trim().is_empty() => f.trim().to_string(),
        _ => {
            return Ok(ImageInfo {
                urls: vec![],
                title: None,
                description: None,
                page_url: None,
            })
        }
    };

    let encoded = urlencoding::encode(&file);

    let resp2: Value = client
        .get(format!(
            "https://commons.wikimedia.org/w/api.php?action=query&format=json&prop=imageinfo&iiprop=url|extmetadata&iiurlwidth=3840&titles=File:{encoded}"
        ))
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .json()
        .await?;

    let mut urls = Vec::new();
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;

    if let Some(pages) = resp2["query"]["pages"].as_object() {
        if let Some(page) = pages.values().next() {
            if let Some(info) = page["imageinfo"][0].as_object() {
                if let Some(thumb) = info.get("thumburl").and_then(|v| v.as_str()) {
                    if !thumb.is_empty() {
                        urls.push(thumb.to_string());
                    }
                }
                if let Some(url) = info.get("url").and_then(|v| v.as_str()) {
                    urls.push(url.to_string());
                }
                if let Some(meta) = info.get("extmetadata") {
                    title = meta["ObjectName"]["value"]
                        .as_str()
                        .map(clean_text)
                        .filter(|s| !s.is_empty());
                    description = meta["ImageDescription"]["value"]
                        .as_str()
                        .map(clean_text)
                        .filter(|s| !s.is_empty());
                }
            }
        }
    }

    // Derive title from filename if not in extmetadata
    if title.is_none() {
        title = file
            .strip_prefix("File:")
            .map(|s| {
                let stem = s.rsplit_once('.').map(|(base, _)| base).unwrap_or(s);
                stem.replace('_', " ")
            })
            .filter(|s| !s.is_empty());
    }

    let page_url = Some(format!(
        "https://commons.wikimedia.org/wiki/File:{}",
        urlencoding::encode(file.strip_prefix("File:").unwrap_or(&file))
    ));

    Ok(ImageInfo {
        urls,
        title,
        description,
        page_url,
    })
}
