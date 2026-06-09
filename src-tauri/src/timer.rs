use anyhow::Result;
use std::process::Command;

// ── macOS / launchd ──────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
const LABEL: &str = "com.andrewensley.backdrop";

#[cfg(target_os = "macos")]
fn plist_path() -> Option<std::path::PathBuf> {
    Some(dirs::home_dir()?.join(format!("Library/LaunchAgents/{LABEL}.plist")))
}

#[cfg(target_os = "macos")]
fn gui_domain() -> String {
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    format!("gui/{uid}")
}

#[cfg(target_os = "macos")]
fn write_plist(time: &str) -> Result<()> {
    let path = plist_path().ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let exe = std::env::current_exe()?.to_string_lossy().into_owned();
    let mut parts = time.splitn(2, ':');
    let hour: u32 = parts.next().unwrap_or("8").parse().unwrap_or(8);
    let minute: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{LABEL}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{exe}</string>
		<string>update</string>
	</array>
	<key>StartCalendarInterval</key>
	<dict>
		<key>Hour</key>
		<integer>{hour}</integer>
		<key>Minute</key>
		<integer>{minute}</integer>
	</dict>
</dict>
</plist>
"#
    );

    std::fs::write(&path, plist)?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn apply_timer_time(time: &str) -> Result<()> {
    let active = is_active();
    if active {
        let path = plist_path().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
        Command::new("launchctl")
            .args(["bootout", &gui_domain(), &path.to_string_lossy()])
            .status()
            .ok();
    }
    write_plist(time)?;
    if active {
        let path = plist_path().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
        Command::new("launchctl")
            .args(["bootstrap", &gui_domain(), &path.to_string_lossy()])
            .status()?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn enable(time: &str) -> Result<()> {
    write_plist(time)?;
    let path = plist_path().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
    Command::new("launchctl")
        .args(["bootstrap", &gui_domain(), &path.to_string_lossy()])
        .status()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn is_active() -> bool {
    Command::new("launchctl")
        .args(["list", LABEL])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
pub fn restart() -> Result<()> {
    let path = plist_path().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
    let path_str = path.to_string_lossy().into_owned();
    let domain = gui_domain();
    Command::new("launchctl")
        .args(["bootout", &domain, &path_str])
        .status()
        .ok();
    Command::new("launchctl")
        .args(["bootstrap", &domain, &path_str])
        .status()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn disable() -> Result<()> {
    if let Some(path) = plist_path() {
        Command::new("launchctl")
            .args(["bootout", &gui_domain(), &path.to_string_lossy()])
            .status()
            .ok();
        std::fs::remove_file(&path).ok();
    }
    Ok(())
}

// ── Linux / systemd ──────────────────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
pub fn apply_timer_time(time: &str) -> Result<()> {
    let dropin_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("no config dir"))?
        .join("systemd/user/backdrop.timer.d");

    std::fs::create_dir_all(&dropin_dir)?;

    // Empty OnCalendar= clears the inherited value before setting the new one.
    let content = format!("[Timer]\nOnCalendar=\nOnCalendar=*-*-* {time}:00\n");
    std::fs::write(dropin_dir.join("time.conf"), content)?;

    Command::new("systemctl").args(["--user", "daemon-reload"]).status()?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn enable(time: &str) -> Result<()> {
    apply_timer_time(time)?;
    Command::new("systemctl")
        .args(["--user", "enable", "--now", "backdrop.timer"])
        .status()?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn is_active() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "backdrop.timer"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
pub fn restart() -> Result<()> {
    Command::new("systemctl")
        .args(["--user", "restart", "backdrop.timer"])
        .status()?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn disable() -> Result<()> {
    Command::new("systemctl")
        .args(["--user", "disable", "--now", "backdrop.timer"])
        .status()
        .ok();
    Command::new("systemctl").args(["--user", "daemon-reload"]).status()?;
    Ok(())
}
