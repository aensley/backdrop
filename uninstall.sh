#!/usr/bin/env bash
set -euo pipefail

PURGE=false
for arg in "$@"; do
  case "$arg" in
    --purge) PURGE=true ;;
    *)
      echo "Usage: uninstall.sh [--purge]" >&2
      exit 1
      ;;
  esac
done

BASE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}"
SYSTEMD_USER_DIR="$BASE_CONFIG_DIR/systemd/user"

echo "Uninstalling backdrop..."

systemctl --user disable --now backdrop.timer 2> /dev/null || true
systemctl --user daemon-reload

rm -f "$SYSTEMD_USER_DIR/backdrop.timer"
rm -f "$SYSTEMD_USER_DIR/backdrop.service"
rm -rf "$SYSTEMD_USER_DIR/backdrop.timer.d"

sudo rm -f /usr/local/bin/backdrop

if $PURGE; then
  rm -rf "$BASE_CONFIG_DIR/backdrop"
  rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/backdrop"
  echo "Done. Config and cached wallpapers removed."
else
  echo "Done."
  echo "Note: config and cached wallpapers in ~/.config/backdrop and ~/.local/share/backdrop were not removed."
  echo "      Run with --purge to remove them."
fi
