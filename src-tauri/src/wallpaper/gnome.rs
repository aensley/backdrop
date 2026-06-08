use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn set(file: &Path, option: &str) -> Result<()> {
    let uri = format!("file://{}", file.display());
    Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri", &uri])
        .status()?;
    Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri-dark", &uri])
        .status()?;
    Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-options", option])
        .status()?;
    Ok(())
}

pub fn current_option() -> Option<String> {
    let out = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.background", "picture-options"])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().replace('\'', "");
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
