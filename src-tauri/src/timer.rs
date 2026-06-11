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
fn write_plist(timer_time: &str, rotate_interval: u32) -> Result<()> {
    let path = plist_path().ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let exe = std::env::current_exe()?.to_string_lossy().into_owned();

    let schedule_xml = if rotate_interval > 0 {
        format!(
            "\t<key>StartInterval</key>\n\t<integer>{}</integer>",
            rotate_interval * 60
        )
    } else {
        let mut parts = timer_time.splitn(2, ':');
        let hour: u32 = parts.next().unwrap_or("8").parse().unwrap_or(8);
        let minute: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
        format!(
            "\t<key>StartCalendarInterval</key>\n\t<dict>\n\t\t<key>Hour</key>\n\t\t<integer>{hour}</integer>\n\t\t<key>Minute</key>\n\t\t<integer>{minute}</integer>\n\t</dict>"
        )
    };

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
{schedule_xml}
</dict>
</plist>
"#
    );

    std::fs::write(&path, plist)?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn apply_timer_schedule(timer_time: &str, rotate_interval: u32) -> Result<()> {
    let active = is_active();
    if active {
        let path = plist_path().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
        Command::new("launchctl")
            .args(["bootout", &gui_domain(), &path.to_string_lossy()])
            .status()
            .ok();
    }
    write_plist(timer_time, rotate_interval)?;
    if active {
        let path = plist_path().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
        Command::new("launchctl")
            .args(["bootstrap", &gui_domain(), &path.to_string_lossy()])
            .status()?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn enable(timer_time: &str, rotate_interval: u32) -> Result<()> {
    write_plist(timer_time, rotate_interval)?;
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

// ── Windows / Task Scheduler ─────────────────────────────────────────────────

#[cfg(target_os = "windows")]
const TASK_NAME: &str = "backdrop";

#[cfg(target_os = "windows")]
pub fn apply_timer_schedule(timer_time: &str, rotate_interval: u32) -> Result<()> {
    // Only update if the task is already registered; callers use enable() for first-time setup.
    if !is_active() {
        return Ok(());
    }
    let exe = std::env::current_exe()?.to_string_lossy().into_owned();
    let tr = format!("\"{exe}\" update");
    if rotate_interval > 0 {
        Command::new("schtasks")
            .args([
                "/create",
                "/f",
                "/tn",
                TASK_NAME,
                "/tr",
                &tr,
                "/sc",
                "minute",
                "/mo",
                &rotate_interval.to_string(),
            ])
            .status()?;
    } else {
        Command::new("schtasks")
            .args([
                "/create", "/f", "/tn", TASK_NAME, "/tr", &tr, "/sc", "daily", "/st", timer_time,
            ])
            .status()?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn enable(timer_time: &str, rotate_interval: u32) -> Result<()> {
    let exe = std::env::current_exe()?.to_string_lossy().into_owned();
    let tr = format!("\"{exe}\" update");
    if rotate_interval > 0 {
        Command::new("schtasks")
            .args([
                "/create",
                "/f",
                "/tn",
                TASK_NAME,
                "/tr",
                &tr,
                "/sc",
                "minute",
                "/mo",
                &rotate_interval.to_string(),
            ])
            .status()?;
    } else {
        Command::new("schtasks")
            .args([
                "/create", "/f", "/tn", TASK_NAME, "/tr", &tr, "/sc", "daily", "/st", timer_time,
            ])
            .status()?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn is_active() -> bool {
    Command::new("schtasks")
        .args(["/query", "/tn", TASK_NAME])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
pub fn restart() -> Result<()> {
    // apply_timer_schedule recreates the task in-place; no separate restart step needed.
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn disable() -> Result<()> {
    Command::new("schtasks")
        .args(["/delete", "/tn", TASK_NAME, "/f"])
        .status()
        .ok();
    Ok(())
}

// ── Linux / systemd ──────────────────────────────────────────────────────────

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn host_cmd(cmd: &str) -> Command {
    if std::path::Path::new("/.flatpak-info").exists() {
        let mut c = Command::new("flatpak-spawn");
        c.args(["--host", cmd]);
        c
    } else {
        Command::new(cmd)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn systemd_dropin_dir() -> Option<std::path::PathBuf> {
    // Snap redirects XDG_CONFIG_HOME into the snap container dir, so dirs::config_dir()
    // returns the wrong path. Systemd user timers must live under the real ~/.config.
    let base = if std::env::var_os("SNAP").is_some() {
        dirs::home_dir()?.join(".config")
    } else {
        dirs::config_dir()?
    };
    Some(base.join("systemd/user/backdrop.timer.d"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn apply_timer_schedule(timer_time: &str, rotate_interval: u32) -> Result<()> {
    let dropin_dir = systemd_dropin_dir().ok_or_else(|| anyhow::anyhow!("no config dir"))?;
    std::fs::create_dir_all(&dropin_dir)?;

    // Empty OnCalendar= clears the inherited value before setting the new one.
    let content = if rotate_interval > 0 {
        format!("[Timer]\nOnCalendar=\nOnBootSec={rotate_interval}min\nOnUnitActiveSec={rotate_interval}min\n")
    } else {
        format!("[Timer]\nOnCalendar=\nOnCalendar=*-*-* {timer_time}:00\n")
    };
    std::fs::write(dropin_dir.join("time.conf"), content)?;

    host_cmd("systemctl").args(["--user", "daemon-reload"]).status()?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn enable(timer_time: &str, rotate_interval: u32) -> Result<()> {
    apply_timer_schedule(timer_time, rotate_interval)?;
    host_cmd("systemctl")
        .args(["--user", "enable", "--now", "backdrop.timer"])
        .status()?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn is_active() -> bool {
    host_cmd("systemctl")
        .args(["--user", "is-active", "--quiet", "backdrop.timer"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn restart() -> Result<()> {
    host_cmd("systemctl")
        .args(["--user", "restart", "backdrop.timer"])
        .status()?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn disable() -> Result<()> {
    host_cmd("systemctl")
        .args(["--user", "disable", "--now", "backdrop.timer"])
        .status()
        .ok();
    host_cmd("systemctl").args(["--user", "daemon-reload"]).status()?;
    Ok(())
}
