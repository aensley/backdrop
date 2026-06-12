# backdrop-cli

<p align="center">
  <img src="icon/icon.svg" alt="backdrop" width="128"/><br/>
  <a href="https://github.com/aensley/backdrop-cli/releases"><img src="https://img.shields.io/github/v/release/aensley/backdrop-cli.svg?logo=gnubash&label=backdrop-cli&logoColor=fff" alt="Version"/></a>
  <a href="https://github.com/aensley/backdrop-cli/blob/main/LICENSE"><img src="https://img.shields.io/github/license/aensley/backdrop-cli.svg" alt="License"/></a>
</p>

Set a new desktop wallpaper every day from various sources.

## What it does

backdrop-cli fetches a daily image from one of several curated sources and sets it as your desktop wallpaper. It automatically picks the best display mode based on the image dimensions and your screen aspect ratio.

Run it once manually or let a systemd timer handle it on a schedule.

## Requirements

- A supported desktop environment (see below)
- `curl`, `python3` (standard on most distros)
- systemd (for the daily timer)

## Desktop Environments

backdrop-cli supports the following Desktop Environments.

| Desktop               | Method                        | Notes                                                                           |
| --------------------- | ----------------------------- | ------------------------------------------------------------------------------- |
| GNOME                 | `gsettings`                   |                                                                                 |
| Cinnamon              | `gsettings`                   |                                                                                 |
| KDE Plasma            | `qdbus6` or `qdbus`           | Sets the wallpaper and fill mode on all desktops                                |
| KDE Plasma (fallback) | `plasma-apply-wallpaperimage` | Used if qdbus is unavailable; requires Plasma 5.21+, fill mode not configurable |
| XFCE                  | `xfconf-query`                | Sets wallpaper and fill mode on all monitors; open Display settings once first  |
| MATE                  | `gsettings`                   |                                                                                 |
| COSMIC                | config file                   | Writes `~/.config/cosmic/com.system76.CosmicBackground/v1/all` (RON format)     |
| LXQt                  | `pcmanfm-qt`                  |                                                                                 |
| Other                 | `gsettings` or `qdbus`        | Tries gsettings first, then qdbus; set `XDG_CURRENT_DESKTOP` if detection fails |

## Installation

Install with a single command:

```bash
curl -fsSL https://ensl.ee/backdrop-cli | bash
```

Or clone the repo and run the installer locally:

```bash
git clone https://github.com/aensley/backdrop-cli.git \
  && cd backdrop-cli && ./install.sh
```

The installer:

1. Downloads (or uses local) `backdrop.sh`, `backdrop.service`, and `backdrop.timer`
2. Copies `backdrop` to `/usr/local/bin/`
3. Installs `backdrop.service` and `backdrop.timer` to `~/.config/systemd/user/`
4. Runs `backdrop enable` to start the daily timer

## Usage

```
backdrop <command>
```

| Command                         | Description                                                        |
| ------------------------------- | ------------------------------------------------------------------ |
| `update [--force]`              | Refresh wallpaper from the active source (default)                 |
| `set <source...> [--force]`     | Switch active source(s) and refresh now; use `all` for all sources |
| `set-time <HH:MM>`              | Set the daily run time (24-hour); restarts timer if active         |
| `set-rotate-interval <minutes>` | Set rotation interval in minutes; 0 to disable                     |
| `status`                        | Show the active source, last image, and image metadata             |
| `random [--force]`              | Refresh from a randomly chosen source (does not change active)     |
| `enable`                        | Enable the systemd --user timer                                    |
| `disable`                       | Disable the systemd --user timer                                   |
| `upgrade`                       | Check for and install the latest version from GitHub               |
| `uninstall`                     | Remove backdrop from this system                                   |
| `help`                          | Show help                                                          |

## Sources

One, multiple, or all sources can be active simultaneously.

| Key      | Name                                                                                                          |
| -------- | ------------------------------------------------------------------------------------------------------------- |
| `bing`   | Bing Image of the Day<br>https://www.bing.com/                                                                |
| `earth`  | Earth.com Image of the Day<br>https://www.earth.com/gallery/images-of-the-day/                                |
| `apod`   | NASA Astronomy Picture of the Day<br>https://apod.nasa.gov/apod/                                              |
| `eo`     | NASA Earth Observatory Image of the Day<br>https://science.nasa.gov/earth/earth-observatory/image-of-the-day/ |
| `iotd`   | NASA Image of the Day _(default)_<br>https://www.nasa.gov/image-of-the-day/                                   |
| `natgeo` | National Geographic Photo of the Day<br>https://www.nationalgeographic.com/photo-of-the-day/                  |
| `wmc`    | Wikimedia Commons Picture of the Day<br>https://commons.wikimedia.org/wiki/Commons:Picture_of_the_day         |

Switch to a single source at any time:

```bash
backdrop set apod
```

Or enable multiple sources to rotate between them:

```bash
# Enable two or more specific sources
backdrop set iotd apod bing
```

To enable all sources:

```bash
# Enable all sources
backdrop set all
```

The wallpaper updates immediately when you run `set`, and the active source is reflected in `backdrop status`.

## Rotation

With multiple sources enabled, you can rotate between them at a fixed interval. When you set multiple sources, rotation is automatically enabled at 30 minutes:

```bash
# Rotate between three sources every 30 minutes (auto-set default)
backdrop set iotd apod bing

# Rotate through all sources
backdrop set all

# Change the rotation interval (e.g. every 2 hours)
backdrop set-rotate-interval 120

# Disable rotation and go back to a single source
backdrop set iotd
```

When rotation is active, the systemd timer fires at the rotation interval instead of the daily time set by `set-time`. The active source is determined by a time-slot calculation: `slot = (current_minute / interval) % num_sources`, so the same source is always shown for the full duration of its slot.

## Configuration

The config file lives at `~/.config/backdrop/config` and is created on first run. You can edit it directly or use the `set` / `set-time` commands.

| Key                   | Default              | Description                                                                                                      |
| --------------------- | -------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `source`              | `iotd`               | Active wallpaper source(s); space-separated list or `all`                                                        |
| `rotate_interval`     | `0`                  | Minutes between source rotations; 0 to disable (uses `timer_time` for daily updates instead)                     |
| `screen_aspect_ratio` | `1.7778`             | Fallback aspect ratio if auto-detect fails (16:9=1.7778, 16:10=1.6, 21:9=2.3333, 4:3=1.3333)                     |
| `zoom_min_coverage`   | `0.55`               | Crop tolerance: if zoom-filling would keep less than this fraction of the image visible, use scaled mode instead |
| `user_agent`          | `backdrop/1.0 (...)` | HTTP User-Agent sent with all requests                                                                           |
| `timer_time`          | `08:00`              | Time of day to run the daily update (24-hour HH:MM); only applies when `rotate_interval = 0`                     |

## Uninstallation

```bash
backdrop uninstall
```

To also remove your config and cached wallpapers:

```bash
backdrop uninstall --purge
```
