#!/usr/bin/env bats

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_make_png() {
  local path="$1" w="$2" h="$3"
  python3 - "$path" "$w" "$h" <<'PY'
import sys, struct, zlib

path, w, h = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])

def chunk(tag, data):
    crc = zlib.crc32(tag + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

ihdr = struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)
scanline = b"\x00" + bytes(w * 3)
idat = zlib.compress(scanline * h)
data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")
with open(path, "wb") as f:
    f.write(data)
PY
}

# ---------------------------------------------------------------------------
# Setup / teardown
# ---------------------------------------------------------------------------

setup_file() {
  export FIXTURE_DIR
  FIXTURE_DIR="$(mktemp -d)"
  _make_png "$FIXTURE_DIR/wide.png" 1920 1080   # 16:9   ar≈1.778
  _make_png "$FIXTURE_DIR/tall.png" 1080 1920   # 9:16   ar≈0.563
  _make_png "$FIXTURE_DIR/square.png" 1000 1000 # 1:1   ar=1.0
}

teardown_file() {
  rm -rf "$FIXTURE_DIR"
}

setup() {
  export XDG_DATA_HOME XDG_CONFIG_HOME
  XDG_DATA_HOME="$(mktemp -d)"
  XDG_CONFIG_HOME="$(mktemp -d)"
  # shellcheck source=src/backdrop.sh
  source "$BATS_TEST_DIRNAME/../src/backdrop.sh"
}

teardown() {
  rm -rf "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"
}

# ---------------------------------------------------------------------------
# is_valid
# ---------------------------------------------------------------------------

@test "is_valid: accepts all built-in sources" {
  for src in iotd apod bing wmc eo; do
    run is_valid "$src"
    [ "$status" -eq 0 ]
  done
}

@test "is_valid: rejects unknown source" {
  run is_valid "unknown"
  [ "$status" -ne 0 ]
}

@test "is_valid: rejects empty string" {
  run is_valid ""
  [ "$status" -ne 0 ]
}

# ---------------------------------------------------------------------------
# kde_fillmode
# ---------------------------------------------------------------------------

@test "kde_fillmode: zoom -> 2" {
  run kde_fillmode "zoom"
  [ "$output" = "2" ]
}

@test "kde_fillmode: scaled -> 1" {
  run kde_fillmode "scaled"
  [ "$output" = "1" ]
}

@test "kde_fillmode: unknown input defaults to 2" {
  run kde_fillmode "other"
  [ "$output" = "2" ]
}

# ---------------------------------------------------------------------------
# detect_de
# ---------------------------------------------------------------------------

@test "detect_de: detects GNOME from XDG_CURRENT_DESKTOP" {
  XDG_CURRENT_DESKTOP="GNOME" DESKTOP_SESSION="" run detect_de
  [ "$output" = "gnome" ]
}

@test "detect_de: detects KDE from XDG_CURRENT_DESKTOP" {
  XDG_CURRENT_DESKTOP="KDE" DESKTOP_SESSION="" run detect_de
  [ "$output" = "kde" ]
}

@test "detect_de: detects KDE from DESKTOP_SESSION" {
  XDG_CURRENT_DESKTOP="" DESKTOP_SESSION="kde-plasma" run detect_de
  [ "$output" = "kde" ]
}

@test "detect_de: returns unknown for unrecognised desktop" {
  XDG_CURRENT_DESKTOP="" DESKTOP_SESSION="" run detect_de
  [ "$output" = "unknown" ]
}

# ---------------------------------------------------------------------------
# cfg_get
# ---------------------------------------------------------------------------

@test "cfg_get: returns empty when config does not exist" {
  run cfg_get "source"
  [ "$status" -eq 0 ]
  [ "$output" = "" ]
}

@test "cfg_get: reads a simple key = value" {
  mkdir -p "$CONFIG_DIR"
  echo "source = bing" >"$CONFIG_FILE"
  run cfg_get "source"
  [ "$output" = "bing" ]
}

@test "cfg_get: strips surrounding double quotes" {
  mkdir -p "$CONFIG_DIR"
  echo 'user_agent = "my agent"' >"$CONFIG_FILE"
  run cfg_get "user_agent"
  [ "$output" = "my agent" ]
}

@test "cfg_get: strips surrounding single quotes" {
  mkdir -p "$CONFIG_DIR"
  echo "user_agent = 'my agent'" >"$CONFIG_FILE"
  run cfg_get "user_agent"
  [ "$output" = "my agent" ]
}

@test "cfg_get: ignores comment lines" {
  mkdir -p "$CONFIG_DIR"
  printf '# source = apod\nsource = wmc\n' >"$CONFIG_FILE"
  run cfg_get "source"
  [ "$output" = "wmc" ]
}

@test "cfg_get: returns last value when key appears multiple times" {
  mkdir -p "$CONFIG_DIR"
  printf 'source = bing\nsource = eo\n' >"$CONFIG_FILE"
  run cfg_get "source"
  [ "$output" = "eo" ]
}

# ---------------------------------------------------------------------------
# cfg_set / ensure_config
# ---------------------------------------------------------------------------

@test "ensure_config: creates config file with built-in defaults" {
  ensure_config
  [ -f "$CONFIG_FILE" ]
  run cfg_get "source"
  [ "$output" = "iotd" ]
  run cfg_get "screen_aspect_ratio"
  [ "$output" = "1.7778" ]
}

@test "cfg_set: writes a new key" {
  ensure_config
  cfg_set "source" "apod"
  run cfg_get "source"
  [ "$output" = "apod" ]
}

@test "cfg_set: overwrites an existing key" {
  ensure_config
  cfg_set "source" "bing"
  cfg_set "source" "wmc"
  run cfg_get "source"
  [ "$output" = "wmc" ]
}

@test "cfg_set: preserves other keys when updating one" {
  ensure_config
  cfg_set "source" "eo"
  run cfg_get "screen_aspect_ratio"
  [ "$output" = "1.7778" ]
}

# ---------------------------------------------------------------------------
# image_dims
# ---------------------------------------------------------------------------

@test "image_dims: reads width and height from a PNG" {
  run image_dims "$FIXTURE_DIR/wide.png"
  [ "$output" = "1920 1080" ]
}

@test "image_dims: reads tall PNG correctly" {
  run image_dims "$FIXTURE_DIR/tall.png"
  [ "$output" = "1080 1920" ]
}

@test "image_dims: returns empty for a non-image file" {
  run image_dims "/etc/hostname"
  [ "$output" = "" ]
}

# ---------------------------------------------------------------------------
# pick_picture_option
# ---------------------------------------------------------------------------

@test "pick_picture_option: wide image on 16:9 screen -> zoom" {
  SCREEN_ASPECT_RATIO="1.7778"
  run pick_picture_option "$FIXTURE_DIR/wide.png"
  [ "$output" = "zoom" ]
}

@test "pick_picture_option: tall image on 16:9 screen -> scaled" {
  SCREEN_ASPECT_RATIO="1.7778"
  run pick_picture_option "$FIXTURE_DIR/tall.png"
  [ "$output" = "scaled" ]
}

@test "pick_picture_option: square image on 16:9 screen -> zoom (cov=0.5625 >= threshold 0.55)" {
  # shellcheck disable=SC2034
  SCREEN_ASPECT_RATIO="1.7778"
  run pick_picture_option "$FIXTURE_DIR/square.png"
  [ "$output" = "zoom" ]
}
