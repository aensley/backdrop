use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// Maps zoom/scaled to KDE FillMode integers.
/// 2 = PreserveAspectCrop (zoom), 1 = PreserveAspectFit (scaled)
fn fill_mode(option: &str) -> u8 {
    match option {
        "zoom" => 2,
        "scaled" => 1,
        _ => 2,
    }
}

fn find_qdbus() -> Option<String> {
    for cmd in ["qdbus6", "qdbus"] {
        if Command::new("which").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false) {
            return Some(cmd.to_string());
        }
    }
    None
}

pub fn set(file: &Path, option: &str) -> Result<()> {
    let fm = fill_mode(option);
    let uri = format!("file://{}", file.display());

    if let Some(qdbus) = find_qdbus() {
        let script = format!(
            "var a=desktops();\
             for(var i=0;i<a.length;i++){{\
               var d=a[i];\
               d.wallpaperPlugin='org.kde.image';\
               d.currentConfigGroup=['Wallpaper','org.kde.image','General'];\
               d.writeConfig('Image','{uri}');\
               d.writeConfig('FillMode',{fm});\
             }}"
        );
        let status = Command::new(&qdbus)
            .args(["org.kde.plasmashell", "/PlasmaShell", "org.kde.PlasmaShell.evaluateScript", &script])
            .status()?;
        if status.success() {
            return Ok(());
        }
    }

    // Fallback: plasma-apply-wallpaperimage (Plasma 5.21+, no FillMode control)
    if Command::new("which")
        .arg("plasma-apply-wallpaperimage")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Command::new("plasma-apply-wallpaperimage")
            .arg(file)
            .status()?;
        return Ok(());
    }

    bail!("KDE: qdbus and plasma-apply-wallpaperimage are both unavailable")
}

pub fn current_option() -> Option<String> {
    let qdbus = find_qdbus()?;
    let script = "var d=desktops()[0];\
                  d.currentConfigGroup=['Wallpaper','org.kde.image','General'];\
                  print(d.readConfig('FillMode'));";
    let out = Command::new(&qdbus)
        .args(["org.kde.plasmashell", "/PlasmaShell", "org.kde.PlasmaShell.evaluateScript", script])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    match s.as_str() {
        "2" => Some("zoom".to_string()),
        "1" => Some("scaled".to_string()),
        other if !other.is_empty() => Some(format!("fillmode={other}")),
        _ => None,
    }
}
