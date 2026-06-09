use anyhow::{bail, Result};
use std::process::Command;

use crate::{config, sources, timer, wallpaper};

pub async fn dispatch(args: Vec<String>) -> Result<()> {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("update");

    match cmd {
        "update" | "refresh" => {
            let cfg = config::load()?;
            let msg = wallpaper::apply(&cfg.source, &cfg).await?;
            println!("{msg}");
        }

        "set" | "use" => {
            let src = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("set: choose a source ({})", sources::VALID_SOURCES.join(", ")))?;
            if !sources::is_valid(src) {
                bail!(
                    "set: unknown source '{src}' (valid: {})",
                    sources::VALID_SOURCES.join(", ")
                );
            }
            config::cfg_set("source", src)?;
            println!("backdrop: active source is now '{src}'");
            let cfg = config::load()?;
            let msg = wallpaper::apply(src, &cfg).await?;
            println!("{msg}");
        }

        "set-time" => {
            let t = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("set-time: expected HH:MM (24-hour), e.g. 08:00"))?;
            let re = regex::Regex::new(r"^([01][0-9]|2[0-3]):[0-5][0-9]$").unwrap();
            if !re.is_match(t) {
                bail!("set-time: expected HH:MM (24-hour), e.g. 08:00");
            }
            config::cfg_set("timer_time", t)?;
            timer::apply_timer_time(t)?;
            if timer::is_active() {
                timer::restart()?;
                println!("backdrop: timer time set to {t} and timer restarted.");
            } else {
                println!("backdrop: timer time set to {t} (run 'backdrop enable' to start the timer).");
            }
        }

        "status" => {
            let cfg = config::load()?;
            println!("Active source:     {}", cfg.source);
            if let Some(img) = wallpaper::latest_image() {
                println!("Last image:        {}", img.display());
            }
            let method = wallpaper::current_option().unwrap_or_else(|| "unknown".to_string());
            println!("Display method:    {method}");
            println!("Zoom min coverage: {}", cfg.zoom_min_coverage);
            let sar = crate::screen::get_ar(cfg.screen_aspect_ratio);
            println!(
                "Screen aspect:     {sar:.5} (config fallback: {})",
                cfg.screen_aspect_ratio
            );
            println!("Config file:       {}", config::config_file().display());
        }

        "random" => {
            let cfg = config::load()?;
            let idx = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos() as usize)
                % sources::VALID_SOURCES.len();
            let src = sources::VALID_SOURCES[idx];
            let msg = wallpaper::apply(src, &cfg).await?;
            println!("{msg}");
        }

        "enable" => {
            let cfg = config::load()?;
            timer::enable(&cfg.timer_time)?;
            println!("backdrop: daily timer enabled (runs at {}).", cfg.timer_time);
        }

        "uninstall" => {
            let purge = args.get(1).map(|s| s == "--purge").unwrap_or(false);
            timer::disable()?;

            let systemd_user = dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("no config dir"))?
                .join("systemd/user");
            std::fs::remove_file(systemd_user.join("backdrop.timer")).ok();
            std::fs::remove_file(systemd_user.join("backdrop.service")).ok();
            std::fs::remove_dir_all(systemd_user.join("backdrop.timer.d")).ok();

            // Remove binary (may need sudo)
            Command::new("sudo")
                .args(["rm", "-f", "/usr/local/bin/backdrop"])
                .status()
                .ok();

            if purge {
                std::fs::remove_dir_all(config::config_dir()).ok();
                std::fs::remove_dir_all(config::state_dir()).ok();
                println!("backdrop: uninstalled. Config and cached wallpapers removed.");
            } else {
                println!("backdrop: uninstalled.");
                println!("Note: config and cached wallpapers were not removed. Run 'backdrop uninstall --purge' to delete them.");
            }
        }

        "-h" | "--help" | "help" => print_usage(),

        other => bail!("unknown command '{other}' (try: backdrop help)"),
    }

    Ok(())
}

fn print_usage() {
    println!(
        "Usage: backdrop <command>

  update            Refresh wallpaper from the active source (default command)
  set <source>      Switch active source and refresh now
  set-time <HH:MM>  Set the daily run time (24-hour); restarts timer if active
  status            Show the active source and last image
  random            Refresh from a randomly chosen source (does not change active)
  enable            Enable the daily systemd --user timer (backdrop.timer)
  uninstall         Remove backdrop from this system
  uninstall --purge Remove backdrop and delete config and cached wallpapers
  help              Show this help

Sources:
  iotd   NASA Image of the Day (default)
  apod   Astronomy Picture of the Day
  bing   Bing image of the day (4K)
  eo     NASA Earth Observatory Image of the Day
  wmc    Wikimedia Commons Picture of the Day"
    );
}
