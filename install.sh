#!/usr/bin/env bash
# backdrop installer - detects your platform, downloads the latest release from
# GitHub, and installs the appropriate package.
#
# Usage (pipe from curl):
#   curl -fsSL https://raw.githubusercontent.com/aensley/backdrop/main/install.sh | bash
#
# Usage (run locally):
#   bash install.sh
#
# Snap / Flatpak:
#   These are distributed through their own stores and are not handled here.
#   See https://github.com/aensley/backdrop for store links.
#
# Windows:
#   Download the .msi or setup .exe directly from the releases page:
#   https://github.com/aensley/backdrop/releases
set -euo pipefail

REPO="aensley/backdrop"

die()  { echo "error: $*" >&2; exit 1; }
info() { printf '==> %s\n' "$*"; }

command -v curl &>/dev/null || die "'curl' is required. Install it and re-run."

# ── platform detection ────────────────────────────────────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

# Tauri uses different arch labels per package format.
case "$ARCH" in
  x86_64)
    ARCH_DEB="amd64"; ARCH_APPIMAGE="amd64"; ARCH_RPM="x86_64"; ARCH_MAC="x64"
    ;;
  aarch64|arm64)
    ARCH_DEB="arm64"; ARCH_APPIMAGE="aarch64"; ARCH_RPM="aarch64"; ARCH_MAC="aarch64"
    ;;
  *)
    die "Unsupported architecture: $ARCH"
    ;;
esac

# ── fetch latest release ──────────────────────────────────────────────────────

info "Fetching latest release info from github.com/${REPO}..."
RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")"

# Return the browser_download_url for the first asset whose name contains $1.
asset_url() {
  echo "$RELEASE_JSON" \
    | grep '"browser_download_url"' \
    | grep -F "$1" \
    | head -1 \
    | sed 's/.*"browser_download_url": *"\([^"]*\)".*/\1/'
}

# ── download helper ───────────────────────────────────────────────────────────

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

download() {
  local url="$1" dest="$2"
  [ -n "$url" ] || die "No matching package found for this platform. See https://github.com/${REPO}/releases"
  info "Downloading $(basename "$dest")..."
  curl -fsSL --location "$url" -o "$dest"
}

# ── systemd user unit files (Linux) ──────────────────────────────────────────

install_systemd_units() {
  local dir
  # Snap redirects XDG_CONFIG_HOME into its container; bypass it so unit files
  # land in the real ~/.config/systemd/user where systemd expects them.
  if [ -n "${SNAP:-}" ]; then
    dir="$HOME/.config/systemd/user"
  else
    dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
  fi
  mkdir -p "$dir"

  if [ ! -f "$dir/backdrop.service" ]; then
    cat > "$dir/backdrop.service" <<'EOF'
[Unit]
Description=Backdrop wallpaper updater
After=network.target

[Service]
Type=oneshot
ExecStart=backdrop update
EOF
  fi

  if [ ! -f "$dir/backdrop.timer" ]; then
    cat > "$dir/backdrop.timer" <<'EOF'
[Unit]
Description=Daily backdrop wallpaper update

[Timer]
OnCalendar=*-*-* 08:00:00
Persistent=true

[Install]
WantedBy=timers.target
EOF
  fi
}

# ── install ───────────────────────────────────────────────────────────────────

case "$OS" in

  Linux)
    if command -v apt-get &>/dev/null; then
      url="$(asset_url "_${ARCH_DEB}.deb")"
      download "$url" "$tmp/backdrop.deb"
      info "Installing .deb package..."
      sudo apt-get install -y "$tmp/backdrop.deb"

    elif command -v dnf &>/dev/null; then
      url="$(asset_url ".${ARCH_RPM}.rpm")"
      download "$url" "$tmp/backdrop.rpm"
      info "Installing .rpm package..."
      sudo dnf install -y "$tmp/backdrop.rpm"

    elif command -v yum &>/dev/null; then
      url="$(asset_url ".${ARCH_RPM}.rpm")"
      download "$url" "$tmp/backdrop.rpm"
      info "Installing .rpm package..."
      sudo yum localinstall -y "$tmp/backdrop.rpm"

    elif command -v zypper &>/dev/null; then
      url="$(asset_url ".${ARCH_RPM}.rpm")"
      download "$url" "$tmp/backdrop.rpm"
      info "Installing .rpm package..."
      sudo zypper --non-interactive install --allow-unsigned-rpm "$tmp/backdrop.rpm"

    else
      # Fallback: install AppImage as /usr/local/bin/backdrop
      url="$(asset_url "_${ARCH_APPIMAGE}.AppImage")"
      download "$url" "$tmp/backdrop.AppImage"
      chmod +x "$tmp/backdrop.AppImage"
      info "Installing AppImage to /usr/local/bin/backdrop..."
      sudo install -m 755 "$tmp/backdrop.AppImage" /usr/local/bin/backdrop
    fi

    # Ensure unit files exist; deb/rpm packages may already ship them.
    install_systemd_units

    # Rehash so the shell finds the newly installed binary without restarting.
    hash -r 2>/dev/null || true

    info "Enabling the daily wallpaper timer..."
    backdrop enable
    ;;

  Darwin)
    url="$(asset_url "_${ARCH_MAC}.dmg")"
    download "$url" "$tmp/backdrop.dmg"

    info "Mounting disk image..."
    hdiutil attach "$tmp/backdrop.dmg" -mountpoint "$tmp/mnt" -quiet -nobrowse

    app="$(find "$tmp/mnt" -maxdepth 1 -name "*.app" | head -1)"
    if [ -z "$app" ]; then
      hdiutil detach "$tmp/mnt" -quiet
      die "No .app bundle found inside the disk image."
    fi

    info "Installing $(basename "$app") to /Applications..."
    rm -rf "/Applications/$(basename "$app")"
    cp -R "$app" /Applications/
    hdiutil detach "$tmp/mnt" -quiet

    # Symlink the CLI binary so 'backdrop' works in Terminal.
    app_binary="/Applications/$(basename "$app")/Contents/MacOS/$(basename "$app" .app)"
    if [ -f "$app_binary" ]; then
      sudo mkdir -p /usr/local/bin
      sudo ln -sf "$app_binary" /usr/local/bin/backdrop
      info "CLI linked at /usr/local/bin/backdrop"
    fi

    info "Run 'backdrop enable' to start the daily wallpaper timer."
    ;;

  *)
    die "Unsupported OS: $OS. Download the Windows installer from https://github.com/${REPO}/releases"
    ;;
esac

echo "Done."
