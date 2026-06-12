# backdrop-cli

<p align="center">
  <img src="icon.svg" alt="backdrop" width="128"/><br/>
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

backdrop supports the following Desktop Environments.

| Desktop                   | Method                        | Notes                                                                           |
| ------------------------- | ----------------------------- | ------------------------------------------------------------------------------- |
| GNOME                     | `gsettings`                   |                                                                                 |
| KDE Plasma                | `qdbus6` or `qdbus`           | Sets the wallpaper and fill mode on all desktops                                |
| KDE Plasma (fallback)     | `plasma-apply-wallpaperimage` | Used if qdbus is unavailable; requires Plasma 5.21+, fill mode not configurable |
| Other (Cinnamon, MATE...) | `gsettings` or `qdbus`        | Tries gsettings first, then qdbus; set `XDG_CURRENT_DESKTOP` if detection fails |

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

| Command            | Description                                                    |
| ------------------ | -------------------------------------------------------------- |
| `update`           | Refresh wallpaper from the active source (default)             |
| `set <source>`     | Switch active source and refresh now                           |
| `set-time <HH:MM>` | Set the daily run time (24-hour); restarts timer if active     |
| `status`           | Show the active source, last image, and image metadata         |
| `random`           | Refresh from a randomly chosen source (does not change active) |
| `enable`           | Enable the daily systemd --user timer                          |
| `uninstall`        | Remove backdrop from this system                               |
| `help`             | Show help                                                      |

## Sources

| Key      | Name                                                                                                          |
| -------- | ------------------------------------------------------------------------------------------------------------- |
| `bing`   | Bing Image of the Day<br>https://www.bing.com/                                                                |
| `earth`  | Earth.com Image of the Day<br>https://www.earth.com/gallery/images-of-the-day/                                |
| `apod`   | NASA Astronomy Picture of the Day<br>https://apod.nasa.gov/apod/                                              |
| `eo`     | NASA Earth Observatory Image of the Day<br>https://science.nasa.gov/earth/earth-observatory/image-of-the-day/ |
| `iotd`   | NASA Image of the Day _(default)_<br>https://www.nasa.gov/image-of-the-day/                                   |
| `natgeo` | National Geographic Photo of the Day<br>https://www.nationalgeographic.com/photo-of-the-day/                  |
| `wmc`    | Wikimedia Commons Picture of the Day<br>https://commons.wikimedia.org/wiki/Commons:Picture_of_the_day         |

Switch sources at any time:

```bash
backdrop set apod
```

This will also immediately update the wallpaper from the new source.

## Configuration

The config file lives at `~/.config/backdrop/config` and is created on first run. You can edit it directly or use the `set` / `set-time` commands.

| Key                   | Default              | Description                                                                                                      |
| --------------------- | -------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `source`              | `iotd`               | Active wallpaper source                                                                                          |
| `screen_aspect_ratio` | `1.7778`             | Fallback aspect ratio if auto-detect fails (16:9=1.7778, 16:10=1.6, 21:9=2.3333, 4:3=1.3333)                     |
| `zoom_min_coverage`   | `0.55`               | Crop tolerance: if zoom-filling would keep less than this fraction of the image visible, use scaled mode instead |
| `user_agent`          | `backdrop/1.0 (...)` | HTTP User-Agent sent with all requests                                                                           |
| `timer_time`          | `08:00`              | Time of day to run the daily update (24-hour HH:MM)                                                              |

## Uninstallation

```bash
backdrop uninstall
```

To also remove your config and cached wallpapers:

```bash
backdrop uninstall --purge
```
