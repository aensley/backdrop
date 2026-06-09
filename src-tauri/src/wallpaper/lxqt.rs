use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// WallpaperMode values used by pcmanfm-qt
fn wallpaper_mode(option: &str) -> &'static str {
    match option {
        "scaled" => "Fit",
        _ => "Zoom",
    }
}

fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("pcmanfm-qt/lxqt/settings.conf"))
}

pub fn set(file: &Path, option: &str) -> Result<()> {
    let path_str = file.to_string_lossy();
    let mode = wallpaper_mode(option);

    // Modern pcmanfm-qt (≥ 1.2) accepts --set-wallpaper / --wallpaper-mode directly
    let ok = Command::new("pcmanfm-qt")
        .args([
            &format!("--set-wallpaper={path_str}"),
            &format!("--wallpaper-mode={mode}"),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if ok {
        return Ok(());
    }

    // Fallback: write the INI config file and reload
    let config_file = config_path().ok_or_else(|| anyhow::anyhow!("cannot find config directory"))?;
    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let existing = fs::read_to_string(&config_file).unwrap_or_default();
    fs::write(&config_file, update_config(&existing, &path_str, mode))?;

    Command::new("pcmanfm-qt").arg("--desktop").status().ok();

    Ok(())
}

/// Update all [Desktop*] sections with new Wallpaper/WallpaperMode values.
/// Appends a [Desktop] section if none exists.
fn update_config(content: &str, path: &str, mode: &str) -> String {
    let mut out = String::new();
    let mut in_desktop = false;
    let mut found_any = false;
    let mut wallpaper_set = false;
    let mut mode_set = false;

    for line in content.lines() {
        if line.starts_with('[') {
            if in_desktop {
                if !wallpaper_set {
                    out.push_str(&format!("Wallpaper={path}\n"));
                }
                if !mode_set {
                    out.push_str(&format!("WallpaperMode={mode}\n"));
                }
            }
            in_desktop = line.starts_with("[Desktop");
            if in_desktop {
                found_any = true;
                wallpaper_set = false;
                mode_set = false;
            }
            out.push_str(line);
            out.push('\n');
        } else if in_desktop && line.starts_with("Wallpaper=") {
            out.push_str(&format!("Wallpaper={path}\n"));
            wallpaper_set = true;
        } else if in_desktop && line.starts_with("WallpaperMode=") {
            out.push_str(&format!("WallpaperMode={mode}\n"));
            mode_set = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    if in_desktop {
        if !wallpaper_set {
            out.push_str(&format!("Wallpaper={path}\n"));
        }
        if !mode_set {
            out.push_str(&format!("WallpaperMode={mode}\n"));
        }
    }

    if !found_any {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("\n[Desktop]\n");
        out.push_str(&format!("Wallpaper={path}\n"));
        out.push_str(&format!("WallpaperMode={mode}\n"));
    }

    out
}

pub fn current_option() -> Option<String> {
    let content = fs::read_to_string(config_path()?).ok()?;
    let mut in_desktop = false;
    for line in content.lines() {
        if line.starts_with("[Desktop") {
            in_desktop = true;
        } else if line.starts_with('[') {
            in_desktop = false;
        } else if in_desktop {
            if let Some(val) = line.strip_prefix("WallpaperMode=") {
                return match val.trim() {
                    "Fit" => Some("scaled".to_string()),
                    _ => Some("zoom".to_string()),
                };
            }
        }
    }
    None
}
