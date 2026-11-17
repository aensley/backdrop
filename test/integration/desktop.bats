#!/usr/bin/env bats

# Integration tests for the desktop environment wallpaper setters, using the
# real gsettings and xfconf tools against the real setting schemas.
#
# Everything is isolated from your own desktop:
#   - gsettings uses the keyfile backend inside a temporary XDG_CONFIG_HOME
#   - xfconf runs on a private D-Bus session (dbus-run-session)
#
# These confirm that the correct settings get written. They can't confirm that
# the image actually appears on screen.
#
# Tests skip when a tool or schema is missing. Set BACKDROP_REQUIRE_DE_TOOLS=1
# (as CI does) to make a missing tool a failure instead.

bats_require_minimum_version 1.5.0

setup() {
  export XDG_DATA_HOME XDG_CONFIG_HOME GSETTINGS_BACKEND=keyfile
  XDG_DATA_HOME="$(mktemp -d)"
  XDG_CONFIG_HOME="$(mktemp -d)"
  # shellcheck source=src/backdrop.sh
  source "$BATS_TEST_DIRNAME/../../src/backdrop.sh"
  WALL="$XDG_DATA_HOME/wall.jpg"
  : >"$WALL"
}

teardown() {
  rm -rf "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"
}

_missing() {
  if [ "${BACKDROP_REQUIRE_DE_TOOLS:-0}" = "1" ]; then
    echo "required for this test: $*" >&2
    return 1
  fi
  skip "$* not available"
}

_need_schema() {
  command -v gsettings >/dev/null 2>&1 || {
    _missing gsettings
    return
  }
  # Capture first: `grep -q` exiting early would SIGPIPE gsettings under pipefail.
  local schemas
  schemas="$(gsettings list-schemas)"
  grep -qx "$1" <<<"$schemas" || _missing "gsettings schema $1"
}

# --- GNOME ------------------------------------------------------------------

@test "gnome: sets picture-uri, picture-uri-dark and zoom option" {
  _need_schema org.gnome.desktop.background
  run set_wallpaper_gnome "$WALL" zoom
  [ "$status" -eq 0 ]
  [ "$(gsettings get org.gnome.desktop.background picture-uri)" = "'file://$WALL'" ]
  [ "$(gsettings get org.gnome.desktop.background picture-uri-dark)" = "'file://$WALL'" ]
  [ "$(gsettings get org.gnome.desktop.background picture-options)" = "'zoom'" ]
}

@test "gnome: accepts scaled option" {
  _need_schema org.gnome.desktop.background
  run set_wallpaper_gnome "$WALL" scaled
  [ "$status" -eq 0 ]
  [ "$(gsettings get org.gnome.desktop.background picture-options)" = "'scaled'" ]
}

@test "gnome: set_wallpaper dispatches on XDG_CURRENT_DESKTOP=GNOME" {
  _need_schema org.gnome.desktop.background
  XDG_CURRENT_DESKTOP=ubuntu:GNOME DESKTOP_SESSION="" run set_wallpaper "$WALL" zoom
  [ "$status" -eq 0 ]
  [ "$(gsettings get org.gnome.desktop.background picture-uri)" = "'file://$WALL'" ]
}

# --- Cinnamon ---------------------------------------------------------------

@test "cinnamon: sets picture-uri and scaled option" {
  _need_schema org.cinnamon.desktop.background
  run set_wallpaper_cinnamon "$WALL" scaled
  [ "$status" -eq 0 ]
  [ "$(gsettings get org.cinnamon.desktop.background picture-uri)" = "'file://$WALL'" ]
  [ "$(gsettings get org.cinnamon.desktop.background picture-options)" = "'scaled'" ]
}

@test "cinnamon: set_wallpaper dispatches on XDG_CURRENT_DESKTOP=X-Cinnamon" {
  _need_schema org.cinnamon.desktop.background
  XDG_CURRENT_DESKTOP=X-Cinnamon DESKTOP_SESSION="" run set_wallpaper "$WALL" zoom
  [ "$status" -eq 0 ]
  [ "$(gsettings get org.cinnamon.desktop.background picture-uri)" = "'file://$WALL'" ]
  [ "$(gsettings get org.cinnamon.desktop.background picture-options)" = "'zoom'" ]
}

# --- MATE -------------------------------------------------------------------

@test "mate: sets picture-filename and zoom option" {
  _need_schema org.mate.background
  run set_wallpaper_mate "$WALL" zoom
  [ "$status" -eq 0 ]
  [ "$(gsettings get org.mate.background picture-filename)" = "'$WALL'" ]
  [ "$(gsettings get org.mate.background picture-options)" = "'zoom'" ]
}

@test "mate: set_wallpaper dispatches on XDG_CURRENT_DESKTOP=MATE" {
  _need_schema org.mate.background
  XDG_CURRENT_DESKTOP=MATE DESKTOP_SESSION="" run set_wallpaper "$WALL" scaled
  [ "$status" -eq 0 ]
  [ "$(gsettings get org.mate.background picture-filename)" = "'$WALL'" ]
  [ "$(gsettings get org.mate.background picture-options)" = "'scaled'" ]
}

# --- Xfce (xfconf on a private D-Bus session) ---------------------------------

# Run a bash snippet with backdrop.sh sourced, on its own D-Bus session so
# xfconfd starts fresh and stores its data under the temporary XDG_CONFIG_HOME.
# stderr is kept separate ($stderr) because dbus-daemon logs activations there.
_xfconf_session() {
  command -v dbus-run-session >/dev/null 2>&1 || {
    _missing dbus-run-session
    return
  }
  command -v xfconf-query >/dev/null 2>&1 || {
    _missing xfconf-query
    return
  }
  run --separate-stderr dbus-run-session -- bash -c "source '$BATS_TEST_DIRNAME/../../src/backdrop.sh'; $1"
}

@test "xfce: sets last-image and image-style on every monitor" {
  local base=/backdrop/screen0
  _xfconf_session "
    set -e
    for m in monitorHDMI-1 monitoreDP-1; do
      xfconf-query -c xfce4-desktop -p $base/\$m/workspace0/last-image --create -t string -s /old.jpg
    done
    set_wallpaper_xfce '$WALL' scaled
    for m in monitorHDMI-1 monitoreDP-1; do
      echo \"\$m \$(xfconf-query -c xfce4-desktop -p $base/\$m/workspace0/last-image) \$(xfconf-query -c xfce4-desktop -p $base/\$m/workspace0/image-style)\"
    done
  "
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "monitorHDMI-1 $WALL 4" ]
  [ "${lines[1]}" = "monitoreDP-1 $WALL 4" ]
}

@test "xfce: set_wallpaper dispatches on XDG_CURRENT_DESKTOP=XFCE" {
  local prop=/backdrop/screen0/monitor0/workspace0
  _xfconf_session "
    set -e
    xfconf-query -c xfce4-desktop -p $prop/last-image --create -t string -s /old.jpg
    XDG_CURRENT_DESKTOP=XFCE DESKTOP_SESSION='' set_wallpaper '$WALL' zoom
    xfconf-query -c xfce4-desktop -p $prop/last-image
    xfconf-query -c xfce4-desktop -p $prop/image-style
  "
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "$WALL" ]
  [ "${lines[1]}" = "5" ]
}

@test "xfce: fails with a helpful message when no backdrop properties exist" {
  _xfconf_session "xfconf-query -c xfce4-desktop -p /desktop-icons/style --create -t int -s 0; set_wallpaper_xfce '$WALL' zoom"
  [ "$status" -ne 0 ]
  [[ "$stderr" == *"no backdrop properties found"* ]]
}
