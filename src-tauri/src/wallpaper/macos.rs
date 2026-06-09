use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// AppleScript picture scaling values for System Events Desktop
fn scaling_option(option: &str) -> &'static str {
    match option {
        "scaled" => "fit to screen",
        _ => "fill screen",
    }
}

pub fn set(file: &Path, option: &str) -> Result<()> {
    let path = file.to_string_lossy().replace('"', "\\\"");
    let scaling = scaling_option(option);
    let script = format!(
        r#"tell application "System Events"
    repeat with d in (get every desktop)
        tell d
            set picture to "{path}"
            set picture scaling to {scaling}
        end tell
    end repeat
end tell"#
    );

    let status = Command::new("osascript").args(["-e", &script]).status()?;

    if !status.success() {
        // Fallback: Finder can set the image without fill-mode control
        let script2 = format!(r#"tell application "Finder" to set desktop picture to POSIX file "{path}""#);
        Command::new("osascript").args(["-e", &script2]).status()?;
    }

    Ok(())
}

pub fn current_option() -> Option<String> {
    let script = r#"tell application "System Events"
    tell first desktop
        get picture scaling
    end tell
end tell"#;

    let out = Command::new("osascript").args(["-e", script]).output().ok()?;

    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    match s.as_str() {
        "fit to screen" => Some("scaled".to_string()),
        _ => Some("zoom".to_string()),
    }
}
