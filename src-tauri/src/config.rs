use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub sources: Vec<String>,
    pub rotate_interval: u32,
    pub screen_aspect_ratio: f64,
    pub zoom_min_coverage: f64,
    pub user_agent: String,
    pub timer_time: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sources: vec!["iotd".to_string()],
            rotate_interval: 0,
            screen_aspect_ratio: 1.7778,
            zoom_min_coverage: 0.55,
            user_agent: "backdrop/2.0 (personal daily wallpaper app)".to_string(),
            timer_time: "08:00".to_string(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir().expect("no config dir").join("backdrop")
}

pub fn state_dir() -> PathBuf {
    dirs::data_local_dir().expect("no data dir").join("backdrop")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config")
}

pub fn ensure_dirs() -> Result<()> {
    fs::create_dir_all(config_dir())?;
    fs::create_dir_all(state_dir())?;
    Ok(())
}

pub fn ensure_config() -> Result<()> {
    ensure_dirs()?;
    let path = config_file();
    if path.exists() {
        return Ok(());
    }

    let legacy = config_dir().join("source");
    let seed = if legacy.exists() {
        fs::read_to_string(&legacy).unwrap_or_default().trim().to_string()
    } else {
        String::new()
    };
    let source = if seed.is_empty() { "iotd".to_string() } else { seed };
    let d = Config::default();

    let content = format!(
        "# backdrop configuration  (key = value; lines starting with # are ignored)\n\n\
         # Active wallpaper source(s): comma-separated list, e.g. iotd,apod\n\
         # Valid values: iotd | apod | bing | wmc | eo\n\
         # Also settable with: backdrop set <source>\n\
         sources = {source}\n\n\
         # Minutes between source rotations when multiple sources are selected (0 = off).\n\
         rotate_interval = {ri}\n\n\
         # Screen aspect ratio used only if auto-detection fails.\n\
         # 16:9 = 1.7778   16:10 = 1.6   21:9 = 2.3333   4:3 = 1.3333\n\
         screen_aspect_ratio = {sar}\n\n\
         # Crop tolerance for choosing zoom vs scaled.\n\
         zoom_min_coverage = {zmc}\n\n\
         # HTTP User-Agent string sent with all requests.\n\
         user_agent = {ua}\n\n\
         # Time of day to run the daily wallpaper update (HH:MM, 24-hour format).\n\
         # Also settable with: backdrop set-time HH:MM\n\
         timer_time = {timer}\n",
        ri = d.rotate_interval,
        sar = d.screen_aspect_ratio,
        zmc = d.zoom_min_coverage,
        ua = d.user_agent,
        timer = d.timer_time,
    );

    fs::write(&path, content)?;
    Ok(())
}

fn parse_value(line: &str, key: &str) -> Option<String> {
    let line = line.trim();
    if line.starts_with('#') {
        return None;
    }
    let (k, rest) = line.split_once('=')?;
    if k.trim() != key {
        return None;
    }
    let v = rest.trim();
    let v = if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
        &v[1..v.len() - 1]
    } else {
        v
    };
    Some(v.trim_end().to_string())
}

pub fn cfg_get(key: &str) -> Option<String> {
    let content = fs::read_to_string(config_file()).ok()?;
    content
        .lines()
        .filter_map(|l| parse_value(l, key))
        .next_back()
        .filter(|v| !v.is_empty())
}

pub fn cfg_set(key: &str, value: &str) -> Result<()> {
    ensure_config()?;
    let path = config_file();
    let content = fs::read_to_string(&path)?;

    let mut found = false;
    let mut lines: Vec<String> = content
        .lines()
        .map(|line| {
            if !found && parse_value(line, key).is_some() {
                found = true;
                format!("{key} = {value}")
            } else {
                line.to_string()
            }
        })
        .collect();

    if !found {
        lines.push(format!("{key} = {value}"));
    }

    let mut result = lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }

    fs::write(&path, result)?;
    Ok(())
}

pub fn load() -> Result<Config> {
    ensure_config()?;
    let d = Config::default();

    // Prefer `sources` (comma-separated list); fall back to legacy single `source` key.
    let sources: Vec<String> = if let Some(v) = cfg_get("sources") {
        v.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if let Some(v) = cfg_get("source") {
        vec![v]
    } else {
        d.sources.clone()
    };
    let sources = if sources.is_empty() { d.sources.clone() } else { sources };

    Ok(Config {
        sources,
        rotate_interval: cfg_get("rotate_interval").and_then(|v| v.parse().ok()).unwrap_or(0),
        screen_aspect_ratio: cfg_get("screen_aspect_ratio")
            .and_then(|v| v.parse().ok())
            .unwrap_or(d.screen_aspect_ratio),
        zoom_min_coverage: cfg_get("zoom_min_coverage")
            .and_then(|v| v.parse().ok())
            .unwrap_or(d.zoom_min_coverage),
        user_agent: cfg_get("user_agent").unwrap_or(d.user_agent),
        timer_time: cfg_get("timer_time").unwrap_or(d.timer_time),
    })
}
