#!/usr/bin/env bats

# Live integration tests for the wallpaper sources.
#
# These hit the real source websites, so they need network access and can fail
# when a site is down or changes its markup. They are not part of `npm test`;
# run them with `npm run test:integration`.

# Smallest image width accepted as a usable wallpaper.
MIN_WIDTH=800

setup() {
  export XDG_DATA_HOME XDG_CONFIG_HOME
  XDG_DATA_HOME="$(mktemp -d)"
  XDG_CONFIG_HOME="$(mktemp -d)"
  # shellcheck source=src/backdrop.sh
  source "$BATS_TEST_DIRNAME/../../src/backdrop.sh"
}

teardown() {
  rm -rf "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"
}

# Resolve a source for real, then download its image the same way
# apply_wallpaper does (first candidate URL that downloads wins) and check that
# the result is a real image of a usable size.
_check_source() {
  local src="$1" output candidates url dest dims width
  output="$("resolve_$src")" || {
    echo "resolve_$src failed to reach the source" >&2
    return 1
  }

  grep -q '^META_TITLE:.' <<<"$output" || {
    printf 'resolve_%s returned no META_TITLE. Output:\n%s\n' "$src" "$output" >&2
    return 1
  }
  grep -q '^META_URL:.' <<<"$output" || {
    printf 'resolve_%s returned no META_URL. Output:\n%s\n' "$src" "$output" >&2
    return 1
  }

  candidates="$(grep -v '^META_' <<<"$output" || true)"
  if [ -z "$candidates" ] && [ "$src" = "apod" ]; then
    skip "APOD has no image today (probably a video day)"
  fi
  [ -n "$candidates" ] || {
    printf 'resolve_%s returned no image URLs. Output:\n%s\n' "$src" "$output" >&2
    return 1
  }

  dest="$XDG_DATA_HOME/$src.img"
  while IFS= read -r url; do
    [ -n "$url" ] || continue
    if curl -fsSL --max-time 120 -A "$USER_AGENT" "$url" -o "$dest"; then
      break
    fi
    echo "download failed, trying next candidate: $url" >&2
    rm -f "$dest"
  done <<<"$candidates"
  [ -s "$dest" ] || {
    printf 'could not download any candidate for %s:\n%s\n' "$src" "$candidates" >&2
    return 1
  }

  dims="$(image_dims "$dest")"
  [ -n "$dims" ] || {
    echo "downloaded file for $src is not a JPEG/PNG/GIF image ($url)" >&2
    return 1
  }
  width="${dims% *}"
  [ "$width" -ge "$MIN_WIDTH" ] || {
    echo "image for $src is only ${dims/ /x}, expected at least ${MIN_WIDTH}px wide ($url)" >&2
    return 1
  }
  echo "# $src: ${dims/ /x} <- $url" >&3
}

@test "every valid source has an integration test" {
  local src
  for src in "${VALID_SOURCES[@]}"; do
    grep -q "^@test \"source: $src\"" "$BATS_TEST_FILENAME" || {
      echo "missing integration test for source: $src" >&2
      return 1
    }
  done
}

@test "source: apod" {
  _check_source apod
}

@test "source: bing" {
  _check_source bing
}

@test "source: earth" {
  _check_source earth
}

@test "source: eo" {
  _check_source eo
}

@test "source: iotd" {
  _check_source iotd
}

@test "source: natgeo" {
  _check_source natgeo
}

@test "source: wmc" {
  _check_source wmc
}
