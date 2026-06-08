use anyhow::Result;
use std::process::Command;

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

pub fn enable(time: &str) -> Result<()> {
    apply_timer_time(time)?;
    Command::new("systemctl")
        .args(["--user", "enable", "--now", "backdrop.timer"])
        .status()?;
    Ok(())
}

pub fn is_active() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "backdrop.timer"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn restart() -> Result<()> {
    Command::new("systemctl")
        .args(["--user", "restart", "backdrop.timer"])
        .status()?;
    Ok(())
}

pub fn disable() -> Result<()> {
    Command::new("systemctl")
        .args(["--user", "disable", "--now", "backdrop.timer"])
        .status()
        .ok();
    Command::new("systemctl").args(["--user", "daemon-reload"]).status()?;
    Ok(())
}
