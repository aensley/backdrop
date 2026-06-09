use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

pub async fn resolve(client: &Client) -> Result<Vec<String>> {
    let feed = client
        .get("https://earthobservatory.nasa.gov/feeds/image-of-the-day.rss")
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
        let base = m.as_str().to_string();
        urls.push(format!("{base}?w=3840"));
        urls.push(base);
    }

    Ok(urls)
}
