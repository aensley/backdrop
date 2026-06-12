pub mod cinnamon;
pub mod cosmic;
pub mod gnome;
pub mod kde;
pub mod lxqt;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod mate;
#[cfg(target_os = "windows")]
pub mod windows;
pub mod xfce;

use anyhow::{bail, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::{config, config::Config, image, screen, sources};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ImageMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    pub page_url: Option<String>,
}

pub enum DesktopEnv {
    Cinnamon,
    Cosmic,
    Gnome,
    Kde,
    LxQt,
    Mate,
    Xfce,
    Unknown,
}

pub fn detect_de() -> DesktopEnv {
    let combined = format!(
        "{}:{}",
        std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        std::env::var("DESKTOP_SESSION").unwrap_or_default()
    )
    .to_uppercase();

    if combined.contains("CINNAMON") {
        DesktopEnv::Cinnamon
    } else if combined.contains("COSMIC") {
        DesktopEnv::Cosmic
    } else if combined.contains("GNOME") {
        DesktopEnv::Gnome
    } else if combined.contains("KDE") {
        DesktopEnv::Kde
    } else if combined.contains("LXQT") {
        DesktopEnv::LxQt
    } else if combined.contains("MATE") {
        DesktopEnv::Mate
    } else if combined.contains("XFCE") {
        DesktopEnv::Xfce
    } else {
        DesktopEnv::Unknown
    }
}

pub fn detect_de_name() -> String {
    #[cfg(target_os = "macos")]
    return "macos".to_string();

    #[cfg(target_os = "windows")]
    return "windows".to_string();

    #[cfg(target_os = "linux")]
    match detect_de() {
        DesktopEnv::Cinnamon => "cinnamon".to_string(),
        DesktopEnv::Cosmic => "cosmic".to_string(),
        DesktopEnv::Gnome => "gnome".to_string(),
        DesktopEnv::Kde => "kde".to_string(),
        DesktopEnv::LxQt => "lxqt".to_string(),
        DesktopEnv::Mate => "mate".to_string(),
        DesktopEnv::Xfce => "xfce".to_string(),
        DesktopEnv::Unknown => "unknown".to_string(),
    }
}

pub fn pick_option(file: &Path, cfg: &Config) -> String {
    let (iw, ih) = match image::image_dims(file) {
        Some(d) => d,
        None => return "zoom".to_string(),
    };
    if iw == 0 || ih == 0 {
        return "zoom".to_string();
    }
    let sar = screen::get_ar(cfg.screen_aspect_ratio);
    let iar = iw as f64 / ih as f64;
    let cov = if iar < sar { iar / sar } else { sar / iar };
    if cov >= cfg.zoom_min_coverage { "zoom" } else { "scaled" }.to_string()
}

pub fn set(file: &Path, option: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    return macos::set(file, option);

    #[cfg(target_os = "windows")]
    return windows::set(file, option);

    #[cfg(target_os = "linux")]
    match detect_de() {
        DesktopEnv::Cinnamon => cinnamon::set(file, option),
        DesktopEnv::Cosmic => cosmic::set(file, option),
        DesktopEnv::Gnome => gnome::set(file, option),
        DesktopEnv::Kde => kde::set(file, option),
        DesktopEnv::LxQt => lxqt::set(file, option),
        DesktopEnv::Mate => mate::set(file, option),
        DesktopEnv::Xfce => xfce::set(file, option),
        DesktopEnv::Unknown => {
            // Best-effort fallback: try each method
            if cinnamon::set(file, option).is_ok() {
                return Ok(());
            }
            if gnome::set(file, option).is_ok() {
                return Ok(());
            }
            if kde::set(file, option).is_ok() {
                return Ok(());
            }
            if lxqt::set(file, option).is_ok() {
                return Ok(());
            }
            if mate::set(file, option).is_ok() {
                return Ok(());
            }
            if xfce::set(file, option).is_ok() {
                return Ok(());
            }
            bail!("unsupported desktop environment; set XDG_CURRENT_DESKTOP")
        }
    }
}

pub fn current_option() -> Option<String> {
    #[cfg(target_os = "macos")]
    return macos::current_option();

    #[cfg(target_os = "windows")]
    return windows::current_option();

    #[cfg(target_os = "linux")]
    match detect_de() {
        DesktopEnv::Cinnamon => cinnamon::current_option(),
        DesktopEnv::Cosmic => cosmic::current_option(),
        DesktopEnv::Gnome => gnome::current_option(),
        DesktopEnv::Kde => kde::current_option(),
        DesktopEnv::LxQt => lxqt::current_option(),
        DesktopEnv::Mate => mate::current_option(),
        DesktopEnv::Xfce => xfce::current_option(),
        DesktopEnv::Unknown => None,
    }
}

pub async fn apply(src: &str, cfg: &Config, force: bool) -> Result<String> {
    if !sources::is_valid(src) {
        bail!("unknown source '{src}' (valid: {})", sources::VALID_SOURCES.join(", "));
    }

    let date = Local::now().format("%Y-%m-%d");
    let dest = config::state_dir().join(format!("{src}-{date}.jpg"));

    if !force && dest.exists() {
        let meta: ImageMeta = std::fs::read_to_string(dest.with_extension("json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let option = pick_option(&dest, cfg);
        set(&dest, &option)?;
        let dims = image::image_dims(&dest)
            .map(|(w, h)| format!("{w}x{h}"))
            .unwrap_or_default();
        let mut msg = format!("backdrop: set from cache [{dims}, {option}] -> {}", dest.display());
        if let Some(ref title) = meta.title {
            msg.push('\n');
            msg.push_str(title);
        }
        if let Some(ref desc) = meta.description {
            msg.push('\n');
            msg.push_str(desc);
        }
        return Ok(msg);
    }

    let info = sources::resolve(src, cfg).await?;
    if info.urls.is_empty() {
        return Ok(format!(
            "backdrop: {src} has no image today (e.g. APOD video day); wallpaper unchanged."
        ));
    }

    let client = sources::build_client(cfg)?;
    let mut downloaded = false;
    for url in &info.urls {
        let result = client.get(url).timeout(Duration::from_secs(120)).send().await;
        if let Ok(resp) = result {
            if let Ok(bytes) = resp.bytes().await {
                if std::fs::write(&dest, &bytes).is_ok() {
                    downloaded = true;
                    break;
                }
            }
        }
    }

    if !downloaded {
        bail!("could not download any image for {src}");
    }

    let meta = ImageMeta {
        title: info.title,
        description: info.description,
        page_url: info.page_url,
    };
    if let Ok(json) = serde_json::to_string(&meta) {
        std::fs::write(dest.with_extension("json"), json).ok();
    }

    let option = pick_option(&dest, cfg);
    set(&dest, &option)?;

    cleanup_old_images();

    let dims = image::image_dims(&dest)
        .map(|(w, h)| format!("{w}x{h}"))
        .unwrap_or_default();

    let mut msg = format!("backdrop: set from {src} [{dims}, {option}] -> {}", dest.display());
    if let Some(ref title) = meta.title {
        msg.push('\n');
        msg.push_str(title);
    }
    if let Some(ref desc) = meta.description {
        msg.push('\n');
        msg.push_str(desc);
    }

    Ok(msg)
}

fn cleanup_old_images() {
    let state_dir = config::state_dir();
    let cutoff = std::time::SystemTime::now() - Duration::from_secs(14 * 24 * 3600);
    if let Ok(entries) = std::fs::read_dir(&state_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if ext == Some("jpg") || ext == Some("json") {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if modified < cutoff {
                            std::fs::remove_file(&path).ok();
                        }
                    }
                }
            }
        }
    }
}

pub fn latest_image() -> Option<std::path::PathBuf> {
    let state_dir = config::state_dir();
    std::fs::read_dir(&state_dir)
        .ok()?
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("jpg"))
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}

pub fn latest_meta() -> Option<ImageMeta> {
    let image = latest_image()?;
    let json = std::fs::read_to_string(image.with_extension("json")).ok()?;
    serde_json::from_str(&json).ok()
}

/// Picks the active source from cfg.sources based on rotation.
/// When rotate_interval is 0 (or only one source), always returns the first source.
/// Otherwise uses time-based selection so the same source is returned for the full window.
pub fn pick_source(cfg: &Config) -> &str {
    let sources = &cfg.sources;
    if sources.is_empty() {
        return "iotd";
    }
    if sources.len() == 1 || cfg.rotate_interval == 0 {
        return &sources[0];
    }
    let minutes = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 60;
    let idx = ((minutes / cfg.rotate_interval as u64) as usize) % sources.len();
    &sources[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn tmp_path(ext: &str) -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("backdrop_wptest_{id}.{ext}"))
    }

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    #[test]
    fn pick_source_empty_defaults_to_iotd() {
        let cfg = Config {
            sources: vec![],
            ..Config::default()
        };
        assert_eq!(pick_source(&cfg), "iotd");
    }

    #[test]
    fn pick_source_single_returns_that_source() {
        let cfg = Config {
            sources: vec!["apod".into()],
            ..Config::default()
        };
        assert_eq!(pick_source(&cfg), "apod");
    }

    #[test]
    fn pick_source_multiple_with_no_rotation_returns_first() {
        let cfg = Config {
            sources: vec!["iotd".into(), "apod".into()],
            rotate_interval: 0,
            ..Config::default()
        };
        assert_eq!(pick_source(&cfg), "iotd");
    }

    #[test]
    fn pick_source_multiple_with_rotation_returns_valid_source() {
        let cfg = Config {
            sources: vec!["iotd".into(), "apod".into(), "bing".into()],
            rotate_interval: 60,
            ..Config::default()
        };
        let result = pick_source(&cfg);
        assert!(cfg.sources.iter().any(|s| s == result));
    }

    #[test]
    fn pick_option_matching_aspect_ratio_returns_zoom() {
        // 16:9 image on a 16:9 screen -> coverage = 1.0 -> zoom
        let path = tmp_path("png");
        std::fs::write(&path, png_bytes(1920, 1080)).unwrap();
        let cfg = Config {
            screen_aspect_ratio: 1.7778,
            zoom_min_coverage: 0.55,
            ..Config::default()
        };
        assert_eq!(pick_option(&path, &cfg), "zoom");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn pick_option_very_narrow_image_returns_scaled() {
        // 1:2 portrait on a 16:9 screen: coverage = 0.5/1.7778 ≈ 0.28 < 0.55 -> scaled
        let path = tmp_path("png");
        std::fs::write(&path, png_bytes(500, 1000)).unwrap();
        let cfg = Config {
            screen_aspect_ratio: 1.7778,
            zoom_min_coverage: 0.55,
            ..Config::default()
        };
        assert_eq!(pick_option(&path, &cfg), "scaled");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn pick_option_missing_file_returns_zoom() {
        let path = std::path::Path::new("/tmp/backdrop_nonexistent_wptest.png");
        let cfg = Config::default();
        assert_eq!(pick_option(path, &cfg), "zoom");
    }

    #[test]
    fn detect_de_name_returns_string() {
        // Just verify it returns a non-empty string without panicking
        let name = detect_de_name();
        assert!(!name.is_empty());
    }
}

/// Returns today's cached image for the given source, if it exists.
pub fn current_image(src: &str) -> Option<std::path::PathBuf> {
    let date = Local::now().format("%Y-%m-%d");
    let path = config::state_dir().join(format!("{src}-{date}.jpg"));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Returns metadata for today's cached image for the given source.
pub fn current_meta(src: &str) -> Option<ImageMeta> {
    let image = current_image(src)?;
    let json = std::fs::read_to_string(image.with_extension("json")).ok()?;
    serde_json::from_str(&json).ok()
}
