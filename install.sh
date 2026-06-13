#!/usr/bin/env bash
set -euo pipefail

BASE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}"
SYSTEMD_USER_DIR="$BASE_CONFIG_DIR/systemd/user"
REPO="aensley/backdrop-cli"

# Resolve the local src/ directory only when running from a real file, not piped stdin.
if [ -n "${BASH_SOURCE[0]:-}" ] && [ "${BASH_SOURCE[0]}" != "bash" ]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
else
  SCRIPT_DIR=""
fi

# When running remotely, resolve the latest release tag from the GitHub API.
REPO_RAW=""
if [ -z "$SCRIPT_DIR" ]; then
  api_response="$(curl -fsSL --max-time 15 "https://api.github.com/repos/${REPO}/releases/latest")" ||
    {
      echo "error: could not reach GitHub API" >&2
      exit 1
    }
  latest_tag="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["tag_name"])' \
    <<<"$api_response" 2>/dev/null)" ||
    {
      echo "error: could not parse release info" >&2
      exit 1
    }
  REPO_RAW="https://raw.githubusercontent.com/${REPO}/${latest_tag}/src"
  echo "Installing backdrop ${latest_tag}..."
else
  echo "Installing backdrop..."
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
