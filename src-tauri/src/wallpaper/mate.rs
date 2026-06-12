use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn set(file: &Path, option: &str) -> Result<()> {
    let path = file.to_string_lossy();
    Command::new("gsettings")
        .args(["set", "org.mate.background", "picture-filename", &path])
        .status()?;
    Command::new("gsettings")
        .args(["set", "org.mate.background", "picture-options", option])
        .status()?;
    Ok(())
}

pub fn current_option() -> Option<String> {
    let out = Command::new("gsettings")
        .args(["get", "org.mate.background", "picture-options"])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().replace('\'', "");
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
