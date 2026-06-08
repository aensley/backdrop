/// Returns the primary display aspect ratio, or the config fallback if detection fails.
pub fn get_ar(fallback: f64) -> f64 {
    detect_ar().unwrap_or(fallback)
}

#[cfg(target_os = "linux")]
fn detect_ar() -> Option<f64> {
    let drm = std::path::Path::new("/sys/class/drm");
    if !drm.exists() {
        return None;
    }
    for entry in std::fs::read_dir(drm).ok()?.flatten() {
        let status_path = entry.path().join("status");
        if std::fs::read_to_string(&status_path).ok()?.trim() != "connected" {
            continue;
        }
        let modes_path = entry.path().join("modes");
        let modes = std::fs::read_to_string(&modes_path).ok()?;
        let first = modes.lines().next()?;
        if let Some((w_str, h_str)) = first.split_once('x') {
            let w: f64 = w_str.trim().parse().ok()?;
            let h: f64 = h_str.trim().parse().ok()?;
            if h > 0.0 {
                return Some(w / h);
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn detect_ar() -> Option<f64> {
    None
}
