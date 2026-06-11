use serde_json::Value;
use tauri::command;

use crate::{config, screen, sources, timer, wallpaper};

#[command]
pub async fn update(force: Option<bool>) -> Result<String, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    wallpaper::apply(&cfg.source, &cfg, force.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn set_source(source: String, force: Option<bool>) -> Result<String, String> {
    if !sources::is_valid(&source) {
        return Err(format!(
            "unknown source '{source}' (valid: {})",
            sources::VALID_SOURCES.join(", ")
        ));
    }
    config::cfg_set("source", &source).map_err(|e| e.to_string())?;
    let cfg = config::load().map_err(|e| e.to_string())?;
    let msg = wallpaper::apply(&source, &cfg, force.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("Active source is now '{source}'. {msg}"))
}

#[command]
pub async fn set_time(time: String) -> Result<String, String> {
    let re = regex::Regex::new(r"^([01][0-9]|2[0-3]):[0-5][0-9]$").unwrap();
    if !re.is_match(&time) {
        return Err("Expected HH:MM (24-hour format), e.g. 08:00".to_string());
    }
    config::cfg_set("timer_time", &time).map_err(|e| e.to_string())?;
    timer::apply_timer_time(&time).map_err(|e| e.to_string())?;
    if timer::is_active() {
        timer::restart().map_err(|e| e.to_string())?;
        Ok(format!("Timer time set to {time} and timer restarted."))
    } else {
        Ok(format!("Timer time set to {time}. Run 'enable' to start the timer."))
    }
}

#[command]
pub async fn get_status() -> Result<Value, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    let latest = wallpaper::current_image(&cfg.source)
        .or_else(wallpaper::latest_image)
        .map(|p| p.to_string_lossy().to_string());
    let sar = screen::get_ar(cfg.screen_aspect_ratio);
    Ok(serde_json::json!({
        "source": cfg.source,
        "latest_image": latest,
        "desktop_env": wallpaper::detect_de_name(),
        "display_method": wallpaper::current_option(),
        "zoom_min_coverage": cfg.zoom_min_coverage,
        "screen_aspect_ratio": sar,
        "config_ar_fallback": cfg.screen_aspect_ratio,
        "config_file": config::config_file().to_string_lossy(),
        "timer_time": cfg.timer_time,
        "timer_active": timer::is_active(),
        "user_agent": cfg.user_agent,
    }))
}

#[command]
pub async fn random_wallpaper(force: Option<bool>) -> Result<String, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize)
        % sources::VALID_SOURCES.len();
    let src = sources::VALID_SOURCES[idx];
    wallpaper::apply(src, &cfg, force.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn enable_timer() -> Result<String, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    timer::enable(&cfg.timer_time).map_err(|e| e.to_string())?;
    Ok(format!("Daily timer enabled (runs at {}).", cfg.timer_time))
}

#[command]
pub async fn disable_timer() -> Result<String, String> {
    timer::disable().map_err(|e| e.to_string())?;
    Ok("Daily timer disabled.".to_string())
}

#[command]
pub fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", "", &url])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn get_image_meta() -> Result<serde_json::Value, String> {
    let cfg = config::load().map_err(|e| e.to_string())?;
    let meta = wallpaper::current_meta(&cfg.source)
        .or_else(wallpaper::latest_meta)
        .unwrap_or_default();
    Ok(serde_json::json!({
        "title": meta.title,
        "description": meta.description,
        "page_url": meta.page_url,
    }))
}

#[command]
pub async fn set_config_value(key: String, value: String) -> Result<String, String> {
    match key.as_str() {
        "screen_aspect_ratio" => {
            let v: f64 = value.parse().map_err(|_| format!("'{value}' is not a valid number"))?;
            if v <= 0.0 {
                return Err("screen_aspect_ratio must be a positive number".to_string());
            }
            config::cfg_set(&key, &value).map_err(|e| e.to_string())?;
            Ok("Screen aspect ratio updated.".to_string())
        }
        "zoom_min_coverage" => {
            let v: f64 = value.parse().map_err(|_| format!("'{value}' is not a valid number"))?;
            if v <= 0.0 || v > 1.0 {
                return Err("zoom_min_coverage must be between 0 and 1".to_string());
            }
            config::cfg_set(&key, &value).map_err(|e| e.to_string())?;
            Ok("Zoom min coverage updated.".to_string())
        }
        "user_agent" => {
            if value.trim().is_empty() {
                return Err("user_agent cannot be empty".to_string());
            }
            config::cfg_set(&key, &value).map_err(|e| e.to_string())?;
            Ok("User agent updated.".to_string())
        }
        _ => Err(format!("unknown config key '{key}'")),
    }
}
