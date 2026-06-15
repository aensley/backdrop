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
  for src in iotd apod bing wmc eo earth natgeo; do
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
# xfce_imagestyle
# ---------------------------------------------------------------------------

@test "xfce_imagestyle: zoom -> 5" {
  run xfce_imagestyle "zoom"
  [ "$output" = "5" ]
}

@test "xfce_imagestyle: scaled -> 4" {
  run xfce_imagestyle "scaled"
  [ "$output" = "4" ]
}

@test "xfce_imagestyle: unknown input defaults to 5" {
  run xfce_imagestyle "other"
  [ "$output" = "5" ]
}

# ---------------------------------------------------------------------------
# lxqt_wallpapermode
# ---------------------------------------------------------------------------

@test "lxqt_wallpapermode: zoom -> zoom" {
  run lxqt_wallpapermode "zoom"
  [ "$output" = "zoom" ]
}

@test "lxqt_wallpapermode: scaled -> fit" {
  run lxqt_wallpapermode "scaled"
  [ "$output" = "fit" ]
}

@test "lxqt_wallpapermode: unknown input defaults to zoom" {
  run lxqt_wallpapermode "other"
  [ "$output" = "zoom" ]
}

# ---------------------------------------------------------------------------
# cosmic_scalingmode
# ---------------------------------------------------------------------------

@test "cosmic_scalingmode: zoom -> Zoom" {
  run cosmic_scalingmode "zoom"
  [ "$output" = "Zoom" ]
}

@test "cosmic_scalingmode: scaled -> Fit" {
  run cosmic_scalingmode "scaled"
  [ "$output" = "Fit([0.0, 0.0, 0.0])" ]
}

@test "cosmic_scalingmode: unknown input defaults to Zoom" {
  run cosmic_scalingmode "other"
  [ "$output" = "Zoom" ]
}

# ---------------------------------------------------------------------------
# detect_de
# ---------------------------------------------------------------------------

@test "detect_de: detects GNOME from XDG_CURRENT_DESKTOP" {
  XDG_CURRENT_DESKTOP="GNOME" DESKTOP_SESSION="" run detect_de
  [ "$output" = "gnome" ]
}

@test "detect_de: detects Cinnamon from XDG_CURRENT_DESKTOP" {
  XDG_CURRENT_DESKTOP="X-Cinnamon" DESKTOP_SESSION="" run detect_de
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

@test "detect_de: detects XFCE from XDG_CURRENT_DESKTOP" {
  XDG_CURRENT_DESKTOP="XFCE" DESKTOP_SESSION="" run detect_de
  [ "$output" = "xfce" ]
}

@test "detect_de: detects MATE from XDG_CURRENT_DESKTOP" {
  XDG_CURRENT_DESKTOP="MATE" DESKTOP_SESSION="" run detect_de
  [ "$output" = "mate" ]
}

@test "detect_de: detects COSMIC from XDG_CURRENT_DESKTOP" {
  XDG_CURRENT_DESKTOP="COSMIC" DESKTOP_SESSION="" run detect_de
  [ "$output" = "cosmic" ]
}

@test "detect_de: detects LXQt from XDG_CURRENT_DESKTOP" {
  XDG_CURRENT_DESKTOP="LXQt" DESKTOP_SESSION="" run detect_de
  [ "$output" = "lxqt" ]
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
# load_config
# ---------------------------------------------------------------------------

@test "load_config: reads config values into globals" {
  mkdir -p "$CONFIG_DIR"
  cat >"$CONFIG_FILE" <<'EOF'
screen_aspect_ratio = 2.3333
zoom_min_coverage = 0.75
timer_time = 10:30
rotate_interval = 15
user_agent = test-agent/1.0
EOF
  load_config
  [ "$SCREEN_ASPECT_RATIO" = "2.3333" ]
  [ "$ZOOM_MIN_COVERAGE" = "0.75" ]
  [ "$TIMER_TIME" = "10:30" ]
  [ "$ROTATE_INTERVAL" = "15" ]
  [ "$USER_AGENT" = "test-agent/1.0" ]
}

@test "load_config: leaves globals at defaults when config is missing" {
  load_config
  [ "$SCREEN_ASPECT_RATIO" = "1.7778" ]
  [ "$ZOOM_MIN_COVERAGE" = "0.55" ]
  [ "$TIMER_TIME" = "08:00" ]
  [ "$ROTATE_INTERVAL" = "0" ]
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
# screen_ar
# ---------------------------------------------------------------------------

@test "screen_ar: returns a numeric aspect ratio" {
  run screen_ar
  [ "$status" -eq 0 ]
  [[ "$output" =~ ^[0-9]+\.[0-9]+$ ]]
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

# ---------------------------------------------------------------------------
# resolve_earth
# ---------------------------------------------------------------------------

@test "resolve_earth: returns URL and metadata from article page" {
  local stubdir
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
echo '<a href="https://www.earth.com/image/test-image/"></a>'
echo '<meta property="og:title" content="Test Title"/>'
echo '<meta property="og:description" content="Test description."/>'
echo '<meta property="og:url" content="https://www.earth.com/image/test-image/"/>'
echo '"https://cff2.earth.com/uploads/2025/10/01/photo.jpg"'
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_earth
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "https://cff2.earth.com/uploads/2025/10/01/photo.jpg" ]
  rm -rf "$stubdir"
}

@test "resolve_earth: returns exit code 1 on curl failure" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\nexit 1\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_earth
  [ "$status" -eq 1 ]
  rm -rf "$stubdir"
}

@test "resolve_earth: returns empty image output when no article link found" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\necho "<html>no image here</html>"\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_earth
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "" ]
  rm -rf "$stubdir"
}

# ---------------------------------------------------------------------------
# resolve_natgeo
# ---------------------------------------------------------------------------

@test "resolve_natgeo: returns high-res and base URLs from og:image" {
  local stubdir url_lines
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
echo '<meta property="og:image" content="https://i.natgeofe.com/n/abc123/photo.jpg"/>'
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_natgeo
  [ "$status" -eq 0 ]
  url_lines="$(grep -v '^META_' <<<"$output")"
  [ "$(sed -n '1p' <<<"$url_lines")" = "https://i.natgeofe.com/n/abc123/photo.jpg?w=5120" ]
  [ "$(sed -n '2p' <<<"$url_lines")" = "https://i.natgeofe.com/n/abc123/photo.jpg" ]
  rm -rf "$stubdir"
}

@test "resolve_natgeo: returns exit code 1 on curl failure" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\nexit 1\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_natgeo
  [ "$status" -eq 1 ]
  rm -rf "$stubdir"
}

@test "resolve_natgeo: returns empty image output when no natgeofe og:image found" {
  local stubdir
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
echo '<meta property="og:image" content="https://www.nationalgeographic.com/logo.jpg"/>'
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_natgeo
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "" ]
  rm -rf "$stubdir"
}

# ---------------------------------------------------------------------------
# resolve_natgeo (additional)
# ---------------------------------------------------------------------------

@test "resolve_natgeo: strips site suffix from og:title" {
  local stubdir
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
echo '<meta property="og:image" content="https://i.natgeofe.com/n/abc123/photo.jpg"/>'
echo '<meta property="og:title" content="Forever in Motion | National Geographic"/>'
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_natgeo
  [ "$status" -eq 0 ]
  [ "$(grep '^META_TITLE:' <<<"$output")" = "META_TITLE:Forever in Motion" ]
  rm -rf "$stubdir"
}

# ---------------------------------------------------------------------------
# resolve_iotd
# ---------------------------------------------------------------------------

@test "resolve_iotd: returns URL and metadata from RSS feed" {
  local stubdir
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
cat <<'RSS'
<rss><channel>
<item>
  <title><![CDATA[Test IOTD Image]]></title>
  <description>A test NASA image.</description>
  <link>https://www.nasa.gov/image-of-the-day/test/</link>
  <enclosure url="https://www.nasa.gov/wp-content/uploads/2025/01/test.jpg" type="image/jpeg"/>
</item>
</channel></rss>
RSS
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_iotd
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "https://www.nasa.gov/wp-content/uploads/2025/01/test.jpg" ]
  [ "$(grep '^META_TITLE:' <<<"$output")" = "META_TITLE:Test IOTD Image" ]
  [ "$(grep '^META_URL:' <<<"$output")" = "META_URL:https://www.nasa.gov/image-of-the-day/test/" ]
  rm -rf "$stubdir"
}

@test "resolve_iotd: returns exit code 1 on curl failure" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\nexit 1\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_iotd
  [ "$status" -eq 1 ]
  rm -rf "$stubdir"
}

@test "resolve_iotd: returns empty image output when no enclosure URL found" {
  local stubdir
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
cat <<'RSS'
<rss><channel>
<item>
  <title><![CDATA[No Image Today]]></title>
  <link>https://www.nasa.gov/image-of-the-day/</link>
</item>
</channel></rss>
RSS
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_iotd
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "" ]
  rm -rf "$stubdir"
}

# ---------------------------------------------------------------------------
# resolve_apod
# ---------------------------------------------------------------------------

@test "resolve_apod: returns URL and metadata from HTML page" {
  local stubdir
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
cat <<'HTML'
<html><body>
<center><b>Starry Night</b><br</center>
<a href="image/2025/starry_night.jpg">image</a>
<b>Explanation:</b> A wonderful view of stars.
</body></html>
HTML
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_apod
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "https://apod.nasa.gov/apod/image/2025/starry_night.jpg" ]
  [ "$(grep '^META_TITLE:' <<<"$output")" = "META_TITLE:Starry Night" ]
  rm -rf "$stubdir"
}

@test "resolve_apod: returns exit code 1 on curl failure" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\nexit 1\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_apod
  [ "$status" -eq 1 ]
  rm -rf "$stubdir"
}

@test "resolve_apod: returns empty image output when no image link found" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\necho "<html>No image today.</html>"\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_apod
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "" ]
  rm -rf "$stubdir"
}

# ---------------------------------------------------------------------------
# resolve_bing
# ---------------------------------------------------------------------------

@test "resolve_bing: returns 4K and fallback URLs with metadata" {
  local stubdir url_lines
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
echo '{"images":[{"urlbase":"/th/id/OHR.TestImage","url":"/th/id/OHR.TestImage_1920x1080.jpg","title":"Test Bing Image","copyright":"Test copyright 2025"}]}'
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_bing
  [ "$status" -eq 0 ]
  url_lines="$(grep -v '^META_' <<<"$output")"
  [ "$(sed -n '1p' <<<"$url_lines")" = "https://www.bing.com/th/id/OHR.TestImage_UHD.jpg" ]
  [ "$(sed -n '2p' <<<"$url_lines")" = "https://www.bing.com/th/id/OHR.TestImage_1920x1080.jpg" ]
  [ "$(grep '^META_TITLE:' <<<"$output")" = "META_TITLE:Test Bing Image" ]
  rm -rf "$stubdir"
}

@test "resolve_bing: returns exit code 1 on curl failure" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\nexit 1\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_bing
  [ "$status" -eq 1 ]
  rm -rf "$stubdir"
}

# ---------------------------------------------------------------------------
# resolve_eo
# ---------------------------------------------------------------------------

@test "resolve_eo: returns 4K and base URLs with metadata from RSS feed" {
  local stubdir url_lines
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
cat <<'RSS'
<rss><channel>
<item>
  <title><![CDATA[Earth at Night]]></title>
  <description><![CDATA[A stunning view.]]></description>
  <link>https://earthobservatory.nasa.gov/images/12345/earth-at-night</link>
  <p>https://assets.science.nasa.gov/dynamicimage/eo/2025/01/photo.jpg</p>
</item>
</channel></rss>
RSS
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_eo
  [ "$status" -eq 0 ]
  url_lines="$(grep -v '^META_' <<<"$output")"
  [ "$(sed -n '1p' <<<"$url_lines")" = "https://assets.science.nasa.gov/dynamicimage/eo/2025/01/photo.jpg?w=3840" ]
  [ "$(sed -n '2p' <<<"$url_lines")" = "https://assets.science.nasa.gov/dynamicimage/eo/2025/01/photo.jpg" ]
  [ "$(grep '^META_TITLE:' <<<"$output")" = "META_TITLE:Earth at Night" ]
  rm -rf "$stubdir"
}

@test "resolve_eo: returns exit code 1 on curl failure" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\nexit 1\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_eo
  [ "$status" -eq 1 ]
  rm -rf "$stubdir"
}

@test "resolve_eo: returns empty image output when no asset URL found" {
  local stubdir
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
cat <<'RSS'
<rss><channel>
<item>
  <title><![CDATA[No Image]]></title>
  <link>https://earthobservatory.nasa.gov/images/12345/no-image</link>
</item>
</channel></rss>
RSS
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_eo
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "" ]
  rm -rf "$stubdir"
}

# ---------------------------------------------------------------------------
# resolve_wmc
# ---------------------------------------------------------------------------

@test "resolve_wmc: returns thumbnail and base URLs with metadata" {
  local stubdir url_lines
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
if [[ "$*" == *"imageinfo"* ]]; then
  echo '{"query":{"pages":{"-1":{"imageinfo":[{"thumburl":"https://upload.wikimedia.org/thumb/photo.jpg","url":"https://upload.wikimedia.org/photo.jpg"}]}}}}'
elif [[ "$*" == *"(en)"* ]]; then
  echo '{"expandtemplates":{"wikitext":"A beautiful photograph of the day."}}'
else
  echo '{"expandtemplates":{"wikitext":"File:Test Photo.jpg"}}'
fi
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_wmc
  [ "$status" -eq 0 ]
  url_lines="$(grep -v '^META_' <<<"$output")"
  [ "$(sed -n '1p' <<<"$url_lines")" = "https://upload.wikimedia.org/thumb/photo.jpg" ]
  [ "$(sed -n '2p' <<<"$url_lines")" = "https://upload.wikimedia.org/photo.jpg" ]
  [ "$(grep '^META_TITLE:' <<<"$output")" = "META_TITLE:Test Photo" ]
  rm -rf "$stubdir"
}

@test "resolve_wmc: returns exit code 1 on curl failure" {
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\nexit 1\n' >"$stubdir/curl"
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_wmc
  [ "$status" -eq 1 ]
  rm -rf "$stubdir"
}

@test "resolve_wmc: returns empty image output when no file found" {
  local stubdir
  stubdir="$(mktemp -d)"
  cat >"$stubdir/curl" <<'STUB'
#!/usr/bin/env bash
echo '{"expandtemplates":{"wikitext":""}}'
STUB
  chmod +x "$stubdir/curl"
  PATH="$stubdir:$PATH" run resolve_wmc
  [ "$status" -eq 0 ]
  [ "$(grep -v '^META_' <<<"$output")" = "" ]
  rm -rf "$stubdir"
}

# ---------------------------------------------------------------------------
# _strip_html
# ---------------------------------------------------------------------------

@test "_strip_html: removes HTML tags" {
  run _strip_html "<b>Hello</b> <i>World</i>"
  [ "$output" = "Hello World" ]
}

@test "_strip_html: decodes common HTML entities" {
  run _strip_html "&amp; &lt; &gt; &quot; &#39;"
  [ "$output" = "& < > \" '" ]
}

@test "_strip_html: collapses whitespace" {
  run _strip_html "  foo   bar  "
  [ "$output" = "foo bar" ]
}

# ---------------------------------------------------------------------------
# _meta_get / _write_meta
# ---------------------------------------------------------------------------

@test "_meta_get: reads a key from a meta file" {
  mkdir -p "$STATE_DIR"
  printf 'title = My Image\ndesc = Some desc\nurl = https://example.com/\n' >"$STATE_DIR/test.meta"
  run _meta_get "$STATE_DIR/test.meta" "title"
  [ "$output" = "My Image" ]
}

@test "_meta_get: returns empty for a missing file" {
  run _meta_get "$STATE_DIR/nonexistent.meta" "title"
  [ "$status" -eq 0 ]
  [ "$output" = "" ]
}

@test "_meta_get: returns empty for a missing key" {
  mkdir -p "$STATE_DIR"
  printf 'url = https://example.com/\n' >"$STATE_DIR/test.meta"
  run _meta_get "$STATE_DIR/test.meta" "title"
  [ "$output" = "" ]
}

@test "_write_meta: writes non-empty fields to a .meta file" {
  mkdir -p "$STATE_DIR"
  local dest="$STATE_DIR/src-2025-01-01.jpg"
  touch "$dest"
  # shellcheck disable=SC2034
  META_TITLE="Test Image"
  # shellcheck disable=SC2034
  META_DESC="A test description"
  # shellcheck disable=SC2034
  META_URL="https://example.com/"
  _write_meta "$dest"
  run _meta_get "${dest%.jpg}.meta" "title"
  [ "$output" = "Test Image" ]
  run _meta_get "${dest%.jpg}.meta" "desc"
  [ "$output" = "A test description" ]
  run _meta_get "${dest%.jpg}.meta" "url"
  [ "$output" = "https://example.com/" ]
}

# ---------------------------------------------------------------------------
# _version_gt
# ---------------------------------------------------------------------------

@test "_version_gt: newer major returns true" {
  run _version_gt "2.0.0" "1.9.9"
  [ "$status" -eq 0 ]
}

@test "_version_gt: newer minor returns true" {
  run _version_gt "1.2.0" "1.1.9"
  [ "$status" -eq 0 ]
}

@test "_version_gt: newer patch returns true" {
  run _version_gt "1.1.1" "1.1.0"
  [ "$status" -eq 0 ]
}

@test "_version_gt: equal versions returns false" {
  run _version_gt "1.1.0" "1.1.0"
  [ "$status" -ne 0 ]
}

@test "_version_gt: older version returns false" {
  run _version_gt "1.0.0" "1.1.0"
  [ "$status" -ne 0 ]
}

@test "_write_meta: omits empty fields from the .meta file" {
  mkdir -p "$STATE_DIR"
  local dest="$STATE_DIR/src-2025-01-01.jpg"
  touch "$dest"
  # shellcheck disable=SC2034
  META_TITLE=""
  # shellcheck disable=SC2034
  META_DESC=""
  # shellcheck disable=SC2034
  META_URL="https://example.com/"
  _write_meta "$dest"
  run _meta_get "${dest%.jpg}.meta" "title"
  [ "$output" = "" ]
  run _meta_get "${dest%.jpg}.meta" "url"
  [ "$output" = "https://example.com/" ]
}

# ---------------------------------------------------------------------------
# get_sources
# ---------------------------------------------------------------------------

@test "get_sources: returns default when config does not exist" {
  run get_sources
  [ "$output" = "iotd" ]
}

@test "get_sources: returns single configured source" {
  mkdir -p "$CONFIG_DIR"
  echo "source = bing" >"$CONFIG_FILE"
  run get_sources
  [ "$output" = "bing" ]
}

@test "get_sources: returns space-separated list for multiple sources" {
  mkdir -p "$CONFIG_DIR"
  echo "source = iotd apod bing" >"$CONFIG_FILE"
  run get_sources
  [ "$output" = "iotd apod bing" ]
}

@test "get_sources: expands 'all' to every valid source" {
  mkdir -p "$CONFIG_DIR"
  echo "source = all" >"$CONFIG_FILE"
  run get_sources
  [ "$output" = "apod bing earth iotd natgeo eo wmc" ]
}

# ---------------------------------------------------------------------------
# _rotation_index
# ---------------------------------------------------------------------------

@test "_rotation_index: returns 0 at epoch with any interval" {
  run _rotation_index 0 60 3
  [ "$output" = "0" ]
}

@test "_rotation_index: advances to next source after one interval" {
  # 60 minutes elapsed, interval=60, 3 sources -> index 1
  run _rotation_index $((60 * 60)) 60 3
  [ "$output" = "1" ]
}

@test "_rotation_index: wraps around to 0 after all sources used" {
  # 180 minutes elapsed, interval=60, 3 sources -> index 0
  run _rotation_index $((180 * 60)) 60 3
  [ "$output" = "0" ]
}

@test "_rotation_index: floors partial intervals" {
  # 90 minutes elapsed, interval=60, 3 sources -> floor(90/60)=1 -> index 1
  run _rotation_index $((90 * 60)) 60 3
  [ "$output" = "1" ]
}

@test "_rotation_index: works with large intervals" {
  # 2880 minutes (2 days) elapsed, interval=1440 (daily), 3 sources -> index 2
  run _rotation_index $((2880 * 60)) 1440 3
  [ "$output" = "2" ]
}

# ---------------------------------------------------------------------------
# get_active_source
# ---------------------------------------------------------------------------

@test "get_active_source: returns single source when only one configured" {
  mkdir -p "$CONFIG_DIR"
  echo "source = apod" >"$CONFIG_FILE"
  # shellcheck disable=SC2034
  ROTATE_INTERVAL=0
  run get_active_source
  [ "$output" = "apod" ]
}

@test "get_active_source: returns first source when rotation is disabled" {
  mkdir -p "$CONFIG_DIR"
  echo "source = iotd apod bing" >"$CONFIG_FILE"
  # shellcheck disable=SC2034
  ROTATE_INTERVAL=0
  run get_active_source
  [ "$output" = "iotd" ]
}

@test "get_active_source: returns correct source based on rotation index" {
  mkdir -p "$CONFIG_DIR"
  echo "source = iotd apod bing" >"$CONFIG_FILE"
  # Stub date to return a fixed timestamp: 60 minutes elapsed -> index 1 -> apod
  local stubdir
  stubdir="$(mktemp -d)"
  printf '#!/usr/bin/env bash\necho %s\n' $((60 * 60)) >"$stubdir/date"
  chmod +x "$stubdir/date"
  # shellcheck disable=SC2034
  ROTATE_INTERVAL=60
  PATH="$stubdir:$PATH" run get_active_source
  [ "$output" = "apod" ]
  rm -rf "$stubdir"
}
