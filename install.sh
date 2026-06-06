#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}"
SYSTEMD_USER_DIR="$BASE_CONFIG_DIR/systemd/user"

echo "Installing backdrop..."

sudo install -m 755 "$SCRIPT_DIR/backdrop.sh" /usr/local/bin/backdrop

mkdir -p "$SYSTEMD_USER_DIR"
install -m 644 "$SCRIPT_DIR/backdrop.service" "$SYSTEMD_USER_DIR/backdrop.service"
install -m 644 "$SCRIPT_DIR/backdrop.timer" "$SYSTEMD_USER_DIR/backdrop.timer"

backdrop enable

echo "Done."
