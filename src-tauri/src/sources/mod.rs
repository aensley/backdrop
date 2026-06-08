pub mod apod;
pub mod bing;
pub mod eo;
pub mod iotd;
pub mod wmc;

use anyhow::Result;
use reqwest::Client;

use crate::config::Config;

pub const VALID_SOURCES: &[&str] = &["iotd", "apod", "bing", "wmc", "eo"];

pub fn is_valid(src: &str) -> bool {
    VALID_SOURCES.contains(&src)
}

pub fn build_client(cfg: &Config) -> Result<Client> {
    Ok(Client::builder().user_agent(&cfg.user_agent).build()?)
}

pub async fn resolve(src: &str, cfg: &Config) -> Result<Vec<String>> {
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
