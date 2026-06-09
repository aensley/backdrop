pub mod cinnamon;
pub mod cosmic;
pub mod gnome;
pub mod kde;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod mate;
#[cfg(target_os = "windows")]
pub mod windows;
pub mod xfce;

use anyhow::{bail, Result};
use chrono::Local;
use std::path::Path;
use std::time::Duration;

use crate::{config, config::Config, image, screen, sources};

pub enum DesktopEnv {
    Cinnamon,
    Cosmic,
    Gnome,
    Kde,
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
        DesktopEnv::Mate => mate::current_option(),
        DesktopEnv::Xfce => xfce::current_option(),
        DesktopEnv::Unknown => None,
    }
}

pub async fn apply(src: &str, cfg: &Config) -> Result<String> {
    if !sources::is_valid(src) {
        bail!("unknown source '{src}' (valid: {})", sources::VALID_SOURCES.join(", "));
    }

    let candidates = sources::resolve(src, cfg).await?;
    if candidates.is_empty() {
        return Ok(format!(
            "backdrop: {src} has no image today (e.g. APOD video day); wallpaper unchanged."
        ));
    }

    let date = Local::now().format("%Y-%m-%d");
    let dest = config::state_dir().join(format!("{src}-{date}.jpg"));

    let client = sources::build_client(cfg)?;
    let mut downloaded = false;
    for url in &candidates {
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

    let option = pick_option(&dest, cfg);
    set(&dest, &option)?;

    cleanup_old_images();

    let dims = image::image_dims(&dest)
        .map(|(w, h)| format!("{w}x{h}"))
        .unwrap_or_default();

    Ok(format!(
        "backdrop: set from {src} [{dims}, {option}] -> {}",
        dest.display()
    ))
}

fn cleanup_old_images() {
    let state_dir = config::state_dir();
    let cutoff = std::time::SystemTime::now() - Duration::from_secs(14 * 24 * 3600);
    if let Ok(entries) = std::fs::read_dir(&state_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jpg") {
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
