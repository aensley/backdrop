# backdrop

<p align="center">
  <img src="src-tauri/icons/icon.svg" alt="backdrop" width="128"/><br/>
  <a href="https://github.com/aensley/backdrop/releases"><img src="https://img.shields.io/github/v/release/aensley/backdrop.svg?logo=gnubash&label=backdrop&logoColor=fff" alt="Version"/></a>
  <a href="https://github.com/aensley/backdrop/blob/main/LICENSE"><img src="https://img.shields.io/github/license/aensley/backdrop.svg" alt="License"/></a>
</p>

Set a new desktop wallpaper every day from various sources.

## What it does

backdrop fetches a daily image from one of several curated sources and sets it as your desktop wallpaper. It automatically picks the best display mode based on the image dimensions and your screen aspect ratio.

Run it once manually or let backdrop schedule a daily update automatically.

## Supported Platforms

- Linux (debian, arch, fedora, etc.)
  - Gnome, KDE, Cinnamon, XFCE, Mate, Cosmic, LXQt
- Windows
- Mac OS

## Installation

Download the latest release for your platform from the [releases page](https://github.com/aensley/backdrop/releases).

### Linux

Install with a single command:

```bash
curl -fsSL https://ensl.ee/backdrop | bash
```

Or clone the repo and run the installer locally:

```bash
git clone https://github.com/aensley/backdrop.git \
  && cd backdrop && ./install.sh
```

The installer detects your distro and installs the best available package format (`.deb`, `.rpm`, or AppImage), ensures the systemd unit files are in place, and runs `backdrop enable` to start the daily timer.

### Windows

Run the `.msi` or setup `.exe` installer. Windows SmartScreen may warn about an unknown publisher; click **More info → Run anyway**.

### macOS

Install with a single command:

```bash
curl -fsSL https://ensl.ee/backdrop | bash
```

Or download the `.dmg` from the [releases page](https://github.com/aensley/backdrop/releases), open it, and drag backdrop to your Applications folder. Then symlink the CLI:

```bash
sudo ln -sf "/Applications/backdrop.app/Contents/MacOS/backdrop" /usr/local/bin/backdrop
```

After either method, run `backdrop enable` to start the daily timer.

> **Note:** backdrop is not code-signed, so macOS Gatekeeper may block it on first launch.
> To open it anyway:
>
> 1. Right-click the app → **Open**
> 2. Click **Open** in the dialog that appears
>
> You only need to do this once. Alternatively, go to **System Settings → Privacy & Security** and click **Open Anyway** after the blocked launch attempt.

## Usage

```
backdrop <command>
```

| Command                | Description                                                               |
| ---------------------- | ------------------------------------------------------------------------- |
| `update` / `refresh`   | Refresh wallpaper from the active source (default)                        |
| `set <source>` / `use` | Switch active source and refresh now                                      |
| `set-time <HH:MM>`     | Set the daily run time (24-hour); restarts timer if active                |
| `status`               | Show active source, last image, display method, screen aspect, and config |
| `random`               | Refresh from a randomly chosen source (does not change active source)     |
| `enable`               | Enable the daily timer (see [Scheduling](#scheduling))                    |
| `uninstall`            | Remove backdrop; `--purge` also deletes config and cached images          |
| `help`                 | Show help                                                                 |

## Scheduling

`backdrop enable` registers a daily timer using the platform's native scheduler:

| Platform | Mechanism                                                                                   |
| -------- | ------------------------------------------------------------------------------------------- |
| Linux    | systemd `--user` timer (`backdrop.timer` / `backdrop.service` in `~/.config/systemd/user/`) |
| macOS    | launchd agent (`~/Library/LaunchAgents/com.andrewensley.backdrop.plist`)                    |
| Windows  | Task Scheduler task named **backdrop**                                                      |

Use `backdrop set-time HH:MM` to change when it fires. The timer is disabled as part of `backdrop uninstall`; there is no standalone `disable` command. To disable manually, use your platform's scheduler directly (e.g. `systemctl --user disable --now backdrop.timer` on Linux).

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

| Key                   | Default                                       | Description                                                                                                    |
| --------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `source`              | `iotd`                                        | Active wallpaper source                                                                                        |
| `screen_aspect_ratio` | `1.7778`                                      | Fallback aspect ratio if auto-detect fails (16:9=1.7778, 16:10=1.6, 21:9=2.3333, 4:3=1.3333)                   |
| `zoom_min_coverage`   | `0.55`                                        | Crop tolerance: if filling the screen would crop more than this fraction of the image, use scaled mode instead |
| `user_agent`          | `backdrop/2.0 (personal daily wallpaper app)` | HTTP User-Agent sent with all requests                                                                         |
| `timer_time`          | `08:00`                                       | Time of day to run the daily update (24-hour HH:MM)                                                            |

## File locations

| Platform | Config file                                     | Cached images                             |
| -------- | ----------------------------------------------- | ----------------------------------------- |
| Linux    | `~/.config/backdrop/config`                     | `~/.local/share/backdrop/`                |
| macOS    | `~/Library/Application Support/backdrop/config` | `~/Library/Application Support/backdrop/` |
| Windows  | `%APPDATA%\Roaming\backdrop\config`             | `%LOCALAPPDATA%\backdrop\`                |

Cached images older than 14 days are pruned automatically on each update.

## Uninstallation

```bash
backdrop uninstall
```

To also remove your config and cached wallpapers:

```bash
backdrop uninstall --purge
```
