pub mod apod;
pub mod bing;
pub mod earth;
pub mod eo;
pub mod iotd;
pub mod natgeo;
pub mod wmc;

use anyhow::Result;
use regex::Regex;
use reqwest::Client;

use crate::config::Config;

pub const VALID_SOURCES: &[&str] = &["iotd", "apod", "bing", "earth", "wmc", "eo", "natgeo"];

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
        "earth" => earth::resolve(&client).await,
        "wmc" => wmc::resolve(&client).await,
        "eo" => eo::resolve(&client).await,
        "natgeo" => natgeo::resolve(&client).await,
        _ => anyhow::bail!("unknown source '{src}'"),
    }
}

/// Strips CDATA wrappers, HTML tags, and HTML entities, then collapses whitespace.
pub(crate) fn clean_text(s: &str) -> String {
    let s = s.trim();
    let s = s
        .strip_prefix("<![CDATA[")
        .and_then(|s| s.strip_suffix("]]>"))
        .unwrap_or(s);
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let cleaned = tag_re.replace_all(s.trim(), " ");
    let decoded = decode_entities(&cleaned);
    let ws_re = Regex::new(r"\s+").unwrap();
    ws_re.replace_all(decoded.trim(), " ").trim().to_string()
}

fn decode_entities(s: &str) -> String {
    let hex_re = Regex::new(r"&#x([0-9a-fA-F]+);").unwrap();
    let s = hex_re.replace_all(s, |caps: &regex::Captures| {
        u32::from_str_radix(&caps[1], 16)
            .ok()
            .and_then(char::from_u32)
            .map(|c| c.to_string())
            .unwrap_or_else(|| caps[0].to_string())
    });
    let dec_re = Regex::new(r"&#([0-9]+);").unwrap();
    let s = dec_re.replace_all(&s, |caps: &regex::Captures| {
        caps[1]
            .parse::<u32>()
            .ok()
            .and_then(char::from_u32)
            .map(|c| c.to_string())
            .unwrap_or_else(|| caps[0].to_string())
    });
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
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
