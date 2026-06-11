use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

use super::{clean_text, ImageInfo};

pub async fn resolve(client: &Client) -> Result<ImageInfo> {
    let html = client
        .get("https://www.nationalgeographic.com/photo-of-the-day/")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await?;

    // Each POTD entry embeds a "crps" array with a "nm":"raw" full-res URL,
    // followed by a "caption" block. The first entry is today's photo.
    let entry_re = Regex::new(
        r#"(?s)"nm":"raw"[^}]*"url":"(https://i\.natgeofe\.com/n/[^"]+\.jpg)".*?"caption":\{"credit":"(?:[^"\\]|\\.)*","text":"((?:[^"\\]|\\.)*)","title":"([^"]+)""#,
    )
    .unwrap();

    let caps = entry_re
        .captures(&html)
        .ok_or_else(|| anyhow!("natgeo: photo entry not found"))?;

    let url = caps[1].to_string();

    // The text field is JSON-encoded; unescape embedded backslash sequences before cleaning.
    let raw_text = caps[2].replace(r#"\""#, "\"").replace(r"\\", "\\");
    let description = clean_text(&raw_text);

    // Title format: "<Month Day, Year> | <Photo Title>" — strip the date prefix.
    let raw_title = &caps[3];
    let title = raw_title
        .find(" | ")
        .map(|i| raw_title[i + 3..].to_string())
        .unwrap_or_else(|| raw_title.to_string());

    Ok(ImageInfo {
        urls: vec![url],
        title: Some(title).filter(|s| !s.is_empty()),
        description: Some(description).filter(|s| !s.is_empty()),
        page_url: Some("https://www.nationalgeographic.com/photo-of-the-day/".to_string()),
    })
}
