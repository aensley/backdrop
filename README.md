# backdrop

[![Version](https://img.shields.io/github/v/release/aensley/backdrop.svg?logo=gnubash&label=backdrop&logoColor=fff)](https://github.com/aensley/backdrop/releases)
[![License](https://img.shields.io/github/license/aensley/backdrop.svg)](https://github.com/aensley/backdrop/blob/main/LICENSE)
[![code style: prettier](https://img.shields.io/badge/code_style-prettier-ff69b4.svg?logo=prettier)](https://prettier.io)

Set a new desktop wallpaper every day from various sources.

## What it does

backdrop fetches a daily image from one of several curated sources and sets it as your desktop wallpaper. It automatically picks the best display mode based on the image dimensions and your screen aspect ratio.

Run it once manually or let a systemd timer handle it on a schedule.

## Requirements

- GNOME or KDE Plasma desktop
- `curl`, `python3` (standard on most distros)
- systemd (for the daily timer)

## Desktop Environments

backdrop supports the following Desktop Environments.

| Desktop               | Method                        | Notes                                                                           |
| --------------------- | ----------------------------- | ------------------------------------------------------------------------------- |
| GNOME                 | `gsettings`                   |                                                                                 |
| KDE Plasma            | `qdbus6` or `qdbus`           | Sets the wallpaper and fill mode on all desktops                                |
| KDE Plasma (fallback) | `plasma-apply-wallpaperimage` | Used if qdbus is unavailable; requires Plasma 5.21+, fill mode not configurable |

## Installation

Install with a single command:

```bash
curl -fsSL https://ensl.ee/backdrop | bash
```

Or clone the repo and run the installer locally:

```bash
git clone https://github.com/aensley/backdrop.git \
  && cd backdrop && ./install.sh
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

| Command            | Description                                                    |
| ------------------ | -------------------------------------------------------------- |
| `update`           | Refresh wallpaper from the active source (default)             |
| `set <source>`     | Switch active source and refresh now                           |
| `set-time <HH:MM>` | Set the daily run time (24-hour); restarts timer if active     |
| `status`           | Show the active source and last image                          |
| `random`           | Refresh from a randomly chosen source (does not change active) |
| `enable`           | Enable the daily systemd --user timer                          |
| `help`             | Show help                                                      |

## Sources

| Key    | Name                                                                                                          |
| ------ | ------------------------------------------------------------------------------------------------------------- |
| `iotd` | NASA Image of the Day _(default)_<br>https://www.nasa.gov/image-of-the-day/                                   |
| `apod` | Astronomy Picture of the Day<br>https://apod.nasa.gov/apod/                                                   |
| `bing` | Bing Image of the Day (4K)<br>https://www.bing.com/                                                           |
| `eo`   | NASA Earth Observatory Image of the Day<br>https://science.nasa.gov/earth/earth-observatory/image-of-the-day/ |
| `wmc`  | Wikimedia Commons Picture of the Day<br>https://commons.wikimedia.org/wiki/Commons:Picture_of_the_day         |

Switch sources at any time:

```bash
backdrop set apod
```

This will also immediately update the wallpaper from the new source.

## Configuration

The config file lives at `~/.config/backdrop/config` and is created on first run. You can edit it directly or use the `set` / `set-time` commands.

| Key                   | Default              | Description                                                                                                    |
| --------------------- | -------------------- | -------------------------------------------------------------------------------------------------------------- |
| `source`              | `iotd`               | Active wallpaper source                                                                                        |
| `screen_aspect_ratio` | `1.7778`             | Fallback aspect ratio if auto-detect fails (16:9=1.7778, 16:10=1.6, 21:9=2.3333, 4:3=1.3333)                   |
| `zoom_min_coverage`   | `0.55`               | Crop tolerance: if filling the screen would crop more than this fraction of the image, use scaled mode instead |
| `user_agent`          | `backdrop/1.0 (...)` | HTTP User-Agent sent with all requests                                                                         |
| `timer_time`          | `08:00`              | Time of day to run the daily update (24-hour HH:MM)                                                            |

## Uninstallation

```bash
backdrop uninstall
```

To also remove your config and cached wallpapers:

```bash
backdrop uninstall --purge
```
