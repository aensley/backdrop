use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub async fn resolve(client: &Client) -> Result<Vec<String>> {
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
        _ => return Ok(vec![]),
    };

    let encoded = urlencoding::encode(&file);

    let resp2: Value = client
        .get(format!(
            "https://commons.wikimedia.org/w/api.php?action=query&format=json&prop=imageinfo&iiprop=url&iiurlwidth=3840&titles=File:{encoded}"
        ))
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .json()
        .await?;

    let mut urls = Vec::new();
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
            }
        }
    }

    Ok(urls)
}
