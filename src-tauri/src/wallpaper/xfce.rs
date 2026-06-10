use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// image-style values: 4 = Scaled (fit to screen), 5 = Zoomed (fill screen)
fn style_value(option: &str) -> &'static str {
    match option {
        "scaled" => "4",
        _ => "5",
    }
}

pub fn set(file: &Path, option: &str) -> Result<()> {
    let path_str = file.to_string_lossy();
    let style = style_value(option);

    // Enumerate all existing last-image properties to cover every monitor/workspace
    let list_out = Command::new("xfconf-query")
        .args(["-c", "xfce4-desktop", "-l"])
        .output()?;

    let props_text = String::from_utf8_lossy(&list_out.stdout);
    let image_props: Vec<&str> = props_text.lines().filter(|l| l.ends_with("/last-image")).collect();

    if image_props.is_empty() {
        // No backdrop properties exist yet. Create defaults for screen0/monitor0/workspace0
        let base = "/backdrop/screen0/monitor0/workspace0";
        xfconf_set(&format!("{base}/last-image"), &path_str, Some("string"))?;
        xfconf_set(&format!("{base}/image-style"), style, Some("int"))?;
    } else {
        for prop in &image_props {
            xfconf_set(prop, &path_str, None)?;
            let style_prop = prop.replace("/last-image", "/image-style");
            xfconf_set(&style_prop, style, None)?;
        }
    }

    // Signal xfdesktop to reload. Ignore errors (e.g. running headless)
    Command::new("xfdesktop").args(["--reload"]).status().ok();

    Ok(())
}

fn xfconf_set(prop: &str, value: &str, type_hint: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("xfconf-query");
    cmd.args(["-c", "xfce4-desktop", "-p", prop, "-s", value]);
    if let Some(t) = type_hint {
        cmd.args(["--create", "-t", t]);
    }
    cmd.status()?;
    Ok(())
}

pub fn current_option() -> Option<String> {
    let list_out = Command::new("xfconf-query")
        .args(["-c", "xfce4-desktop", "-l"])
        .output()
        .ok()?;

    let props_text = String::from_utf8_lossy(&list_out.stdout);
    let style_prop = props_text.lines().find(|l| l.ends_with("/image-style"))?;

    let out = Command::new("xfconf-query")
        .args(["-c", "xfce4-desktop", "-p", style_prop])
        .output()
        .ok()?;

    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    match s.as_str() {
        "4" => Some("scaled".to_string()),
        _ => Some("zoom".to_string()),
    }
}
