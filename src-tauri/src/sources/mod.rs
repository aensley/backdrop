pub mod apod;
pub mod bing;
pub mod eo;
pub mod iotd;
pub mod wmc;

use anyhow::Result;
use regex::Regex;
use reqwest::Client;

use crate::config::Config;

pub const VALID_SOURCES: &[&str] = &["iotd", "apod", "bing", "wmc", "eo"];

pub struct ImageInfo {
    pub urls: Vec<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub page_url: Option<String>,
}

pub fn is_valid(src: &str) -> bool {
    VALID_SOURCES.contains(&src)
}

pub fn build_client(cfg: &Config) -> Result<Client> {
    Ok(Client::builder().user_agent(&cfg.user_agent).build()?)
}

pub async fn resolve(src: &str, cfg: &Config) -> Result<ImageInfo> {
    let client = build_client(cfg)?;
    match src {
        "iotd" => iotd::resolve(&client).await,
        "apod" => apod::resolve(&client).await,
        "bing" => bing::resolve(&client).await,
        "wmc" => wmc::resolve(&client).await,
        "eo" => eo::resolve(&client).await,
        _ => anyhow::bail!("unknown source '{src}'"),
    }
}

/// Strips CDATA wrappers and HTML tags, collapses whitespace.
pub(crate) fn clean_text(s: &str) -> String {
    let s = s.trim();
    let s = s
        .strip_prefix("<![CDATA[")
        .and_then(|s| s.strip_suffix("]]>"))
        .unwrap_or(s);
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let cleaned = tag_re.replace_all(s.trim(), " ");
    let ws_re = Regex::new(r"\s+").unwrap();
    ws_re.replace_all(cleaned.trim(), " ").trim().to_string()
}

/// Extracts the text content of the first occurrence of `<tag>...</tag>` in `xml`.
pub(crate) fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let content_start = start + open.len();
    let end = xml[content_start..].find(&close)?;
    let text = clean_text(&xml[content_start..content_start + end]);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
