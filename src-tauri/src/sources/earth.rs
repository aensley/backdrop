use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

use super::{clean_text, ImageInfo};

pub async fn resolve(client: &Client) -> Result<ImageInfo> {
    let homepage = client
        .get("https://www.earth.com/")
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await?;

    // Next.js outputs relative hrefs (/image/slug/) so match both forms.
    // Case-insensitive to handle "Image of the Day" vs "IMAGE OF THE DAY".
    let section_re =
        Regex::new(r#"(?si)image of the day.{0,2000}?href=["']((?:https://www\.earth\.com)?/image/[^"']+)["']"#)
            .unwrap();
    // Fallback: first /image/ article link anywhere on the page (consistently the IOTD).
    let any_re = Regex::new(r#"href=["']((?:https://www\.earth\.com)?/image/[^"']+)["']"#).unwrap();

    let raw_url = section_re
        .captures(&homepage)
        .or_else(|| any_re.captures(&homepage))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow!("earth: image link not found on homepage"))?;

    let article_url = if raw_url.starts_with('/') {
        format!("https://www.earth.com{raw_url}")
    } else {
        raw_url
    };

    let article = client
        .get(&article_url)
        .timeout(Duration::from_secs(30))
        .send()
        .await?
        .text()
        .await?;

    let raw_image_url = extract_cdn_url(&article).ok_or_else(|| anyhow!("earth: image URL not found"))?;

    // Strip the WIDTHxHEIGHT resize suffix before the extension, e.g. -960x640.jpg -> .jpg
    let size_re = Regex::new(r"-\d+x\d+(\.[a-zA-Z]+)$").unwrap();
    let image_url = size_re.replace(&raw_image_url, "$1").to_string();

    let title = extract_og_meta(&article, "og:title")
        .or_else(|| {
            Regex::new(r"<title>([^|<]+)")
                .ok()?
                .captures(&article)?
                .get(1)
                .map(|m| m.as_str().trim().to_string())
        })
        .filter(|s| !s.is_empty());

    let description = extract_og_meta(&article, "og:description")
        .map(|s| clean_text(&s))
        .filter(|s| !s.is_empty());

    Ok(ImageInfo {
        urls: vec![image_url],
        title,
        description,
        page_url: Some(article_url),
    })
}

fn extract_cdn_url(html: &str) -> Option<String> {
    // Try og:image (direct full-res URL)
    if let Some(url) = extract_og_meta(html, "og:image") {
        if url.contains("cff2.earth.com") {
            return Some(url);
        }
    }
    // Try direct CDN URL anywhere in the page
    let direct_re = Regex::new(r#"(https://cff2\.earth\.com/uploads/[^\s"&]+)"#).unwrap();
    if let Some(m) = direct_re.find(html) {
        return Some(m.as_str().to_string());
    }
    // Try URL-encoded form inside a Next.js image proxy URL
    let encoded_re = Regex::new(r#"url=(https%3A%2F%2Fcff2\.earth\.com%2Fuploads%2F[^&"]+)"#).unwrap();
    encoded_re
        .captures(html)?
        .get(1)
        .and_then(|m| urlencoding::decode(m.as_str()).ok().map(|s| s.into_owned()))
}

fn extract_og_meta(html: &str, property: &str) -> Option<String> {
    let p1 = format!(r#"<meta[^>]+property="{property}"[^>]+content="([^"]+)""#);
    let p2 = format!(r#"<meta[^>]+content="([^"]+)"[^>]+property="{property}""#);
    Regex::new(&p1)
        .ok()?
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .or_else(|| {
            Regex::new(&p2)
                .ok()?
                .captures(html)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
        })
}
