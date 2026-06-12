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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_accepts_all_known_sources() {
        for src in VALID_SOURCES {
            assert!(is_valid(src), "expected '{src}' to be valid");
        }
    }

    #[test]
    fn is_valid_rejects_unknown_sources() {
        assert!(!is_valid("facebook"));
        assert!(!is_valid(""));
        assert!(!is_valid("IOTD")); // case-sensitive
    }

    #[test]
    fn clean_text_strips_html_tags() {
        assert_eq!(clean_text("<p>Hello <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn clean_text_strips_cdata_wrapper() {
        assert_eq!(clean_text("<![CDATA[some content]]>"), "some content");
    }

    #[test]
    fn clean_text_collapses_internal_whitespace() {
        assert_eq!(clean_text("hello   world"), "hello world");
    }

    #[test]
    fn clean_text_trims_outer_whitespace() {
        assert_eq!(clean_text("  hello  "), "hello");
    }

    #[test]
    fn clean_text_decodes_amp() {
        assert_eq!(clean_text("fish &amp; chips"), "fish & chips");
    }

    #[test]
    fn clean_text_decodes_lt_gt() {
        assert_eq!(clean_text("&lt;tag&gt;"), "<tag>");
    }

    #[test]
    fn clean_text_decodes_quot_apos() {
        assert_eq!(clean_text("&quot;it&#39;s&quot;"), "\"it's\"");
    }

    #[test]
    fn clean_text_decodes_decimal_entity() {
        assert_eq!(clean_text("&#65;"), "A");
    }

    #[test]
    fn clean_text_decodes_hex_entity() {
        assert_eq!(clean_text("&#x41;"), "A");
    }

    #[test]
    fn extract_tag_returns_content() {
        assert_eq!(
            extract_tag("<title>Hello World</title>", "title"),
            Some("Hello World".to_string()),
        );
    }

    #[test]
    fn extract_tag_strips_inner_html() {
        assert_eq!(
            extract_tag("<desc><b>bold</b> text</desc>", "desc"),
            Some("bold text".to_string()),
        );
    }

    #[test]
    fn extract_tag_missing_returns_none() {
        assert_eq!(extract_tag("<other>content</other>", "title"), None);
    }

    #[test]
    fn extract_tag_empty_content_returns_none() {
        assert_eq!(extract_tag("<title></title>", "title"), None);
    }

    #[test]
    fn extract_tag_first_occurrence_only() {
        let xml = "<title>First</title><title>Second</title>";
        assert_eq!(extract_tag(xml, "title"), Some("First".to_string()));
    }
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
