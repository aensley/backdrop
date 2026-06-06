#!/usr/bin/env bash
set -euo pipefail

BASE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}"
SYSTEMD_USER_DIR="$BASE_CONFIG_DIR/systemd/user"
REPO_RAW="https://raw.githubusercontent.com/aensley/backdrop/main/src"

# Resolve the local src/ directory only when running from a real file, not piped stdin.
if [ -n "${BASH_SOURCE[0]:-}" ] && [ "${BASH_SOURCE[0]}" != "bash" ]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
else
  SCRIPT_DIR=""
fi

fetch() {
  local name dest="$2"
  name="$(basename "$1")"
  if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/src/$name" ]; then
    cp "$SCRIPT_DIR/src/$name" "$dest"
  else
    curl -fsSL "$REPO_RAW/$name" -o "$dest"
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
