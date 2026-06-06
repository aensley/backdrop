#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}"
SYSTEMD_USER_DIR="$BASE_CONFIG_DIR/systemd/user"
REPO_RAW="https://raw.githubusercontent.com/aensley/backdrop/main/src"

fetch() {
  local src="$1" dest="$2"
  if [ -f "$SCRIPT_DIR/src/$(basename "$src")" ]; then
    cp "$SCRIPT_DIR/src/$(basename "$src")" "$dest"
  else
    curl -fsSL "$REPO_RAW/$(basename "$src")" -o "$dest"
  fi
}

echo "Installing backdrop..."

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fetch backdrop.sh "$tmp/backdrop"
fetch backdrop.service "$tmp/backdrop.service"
fetch backdrop.timer "$tmp/backdrop.timer"

sudo install -m 755 "$tmp/backdrop" /usr/local/bin/backdrop

mkdir -p "$SYSTEMD_USER_DIR"
install -m 644 "$tmp/backdrop.service" "$SYSTEMD_USER_DIR/backdrop.service"
install -m 644 "$tmp/backdrop.timer" "$SYSTEMD_USER_DIR/backdrop.timer"

backdrop enable

echo "Done."
