use anyhow::{bail, Result};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

#[link(name = "user32")]
extern "system" {
    fn SystemParametersInfoW(uiAction: u32, uiParam: u32, pvParam: *mut u16, fWinIni: u32) -> i32;
}

const SPI_SETDESKWALLPAPER: u32 = 0x0014;
const SPIF_UPDATEINIFILE: u32 = 0x0001;
const SPIF_SENDCHANGE: u32 = 0x0002;

/// WallpaperStyle registry values: 6 = Fit (scaled), 10 = Fill (zoom)
fn style_value(option: &str) -> &'static str {
    match option {
        "scaled" => "6",
        _ => "10",
    }
}

pub fn set(file: &Path, option: &str) -> Result<()> {
    reg_set("WallpaperStyle", style_value(option))?;
    reg_set("TileWallpaper", "0")?;

    let mut wide: Vec<u16> = file.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

    let ok = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            wide.as_mut_ptr(),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    };

    if ok == 0 {
        bail!("Windows: SystemParametersInfoW failed");
    }
    Ok(())
}

fn reg_set(name: &str, value: &str) -> Result<()> {
    let status = Command::new("reg")
        .args([
            "add",
            r"HKCU\Control Panel\Desktop",
            "/v",
            name,
            "/t",
            "REG_SZ",
            "/d",
            value,
            "/f",
        ])
        .status()?;
    if !status.success() {
        bail!("Windows: reg failed to set {name}");
    }
    Ok(())
}

pub fn current_option() -> Option<String> {
    let out = Command::new("reg")
        .args(["query", r"HKCU\Control Panel\Desktop", "/v", "WallpaperStyle"])
        .output()
        .ok()?;

    let s = String::from_utf8_lossy(&out.stdout);
    for line in s.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "WallpaperStyle" {
            return match parts[2] {
                "6" | "22" => Some("scaled".to_string()),
                _ => Some("zoom".to_string()),
            };
        }
    }
    None
}
