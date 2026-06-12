use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

/// ScalingMode values used by cosmic-bg: "Zoom" fills the screen,
/// {"Fit":[r,g,b]} fits the image with a solid background color.
fn scaling_mode(option: &str) -> &'static str {
    match option {
        "scaled" => r#"{"Fit":[0.0,0.0,0.0]}"#,
        _ => r#""Zoom""#,
    }
}

pub fn set(file: &Path, option: &str) -> Result<()> {
    let path_str = file.to_string_lossy().replace('"', "\\\"");
    let scaling = scaling_mode(option);

    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("cannot find config directory"))?
        .join("cosmic/com.system76.CosmicBackground/v1");

    fs::create_dir_all(&config_dir)?;

    let content = format!(
        r#"[{{"output":null,"source":{{"path":"{path_str}"}},"scaling_mode":{scaling},"filter_method":"Lanczos","filter_by_theme":false}}]"#
    );

    fs::write(config_dir.join("backgrounds"), content)?;

    // Signal cosmic-bg to reload its config
    Command::new("pkill").args(["-HUP", "cosmic-bg"]).status().ok();

    Ok(())
}

pub fn current_option() -> Option<String> {
    let config_file = dirs::config_dir()?.join("cosmic/com.system76.CosmicBackground/v1/backgrounds");

    let content = fs::read_to_string(config_file).ok()?;

    if content.contains("Fit") {
        Some("scaled".to_string())
    } else {
        Some("zoom".to_string())
    }
}
