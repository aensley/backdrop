#!/usr/bin/env bash

# Open, update, and close one tracking issue per failing wallpaper source, based
# on the TAP results of test/integration/sources.bats.
#
# Usage: report-sources.sh <tap-results-file>
#
#   - A failing test gets its own issue ("Wallpaper source failing: <source>"),
#     or a comment on its issue if one is already open.
#   - An open issue is closed once its test passes. A skipped test (e.g. an
#     APOD video day) leaves its issue open.
#   - A missing or empty results file means the run broke before the tests
#     finished; that gets its own issue, closed on the next run with results.
#
# Requires the gh CLI with GH_TOKEN and GH_REPO set, plus RUN_URL (link to the
# workflow run).

set -euo pipefail

LABEL="source-check"
NO_RESULTS_TITLE="Source check failing: run did not complete"

tap="${1:?usage: report-sources.sh <tap-results-file>}"
: "${RUN_URL:?RUN_URL must be set}"

# Issue title for a test name: "source: earth" -> "Wallpaper source failing: earth".
title_for() {
  case "$1" in
    "source: "*) printf 'Wallpaper source failing: %s' "${1#source: }" ;;
    *) printf 'Source check failing: %s' "$1" ;;
  esac
}

# Print "<status>\t<test name>" for every test, status being pass, fail or skip.
test_results() {
  sed -nE \
    -e 's/^ok [0-9]+ (.*) # skip.*$/skip\t\1/p; t' \
    -e 's/^ok [0-9]+ (.*)$/pass\t\1/p; t' \
    -e 's/^not ok [0-9]+ (.*)$/fail\t\1/p' \
    "$tap"
}

# Print the diagnostic lines (TAP "# ..." comments) under one failing test.
failure_details() {
  awk -v name="$1" '
    /^ok / { show = 0; next }
    /^not ok / { line = $0; sub(/^not ok [0-9]+ /, "", line); show = (line == name); next }
    show && /^#/ { print }
  ' "$tap" | head -n 200
}

declare -A open_issues=() # title -> issue number

load_open_issues() {
  local listing number title
  listing="$(gh issue list --label "$LABEL" --state open --limit 200 \
    --json number,title --jq '.[] | "\(.number)\t\(.title)"')"
  while IFS=$'\t' read -r number title; do
    [ -n "$number" ] && open_issues["$title"]="$number"
  done <<<"$listing"
  return 0
}

report_failure() {
  local title="$1" body="$2" number="${open_issues[$1]:-}"
  if [ -n "$number" ]; then
    gh issue comment "$number" --body "$body"
    echo "Commented on #$number: $title"
  else
    gh issue create --title "$title" --label "$LABEL" --body "$body"
    echo "Opened: $title"
  fi
}

close_if_open() {
  local title="$1" message="$2" number="${open_issues[$1]:-}"
  [ -n "$number" ] || return 0
  gh issue close "$number" --comment "$message"
  echo "Closed #$number: $title"
}

failure_body() {
  local name="$1" details
  details="$(failure_details "$name")"
  # shellcheck disable=SC2016 # literal markdown backticks
  printf 'The daily wallpaper source check failed for `%s`: %s\n' "$name" "$RUN_URL"
  if [ -n "$details" ]; then
    # shellcheck disable=SC2016 # literal markdown code fence
    printf '\n<details><summary>Test output</summary>\n\n```\n%s\n```\n\n</details>\n' "$details"
  fi
}

gh label create "$LABEL" --color d93f0b \
  --description "Opened automatically by the daily source check" --force >/dev/null
load_open_issues

results=""
[ -s "$tap" ] && results="$(test_results)"

if [ -z "$results" ]; then
  report_failure "$NO_RESULTS_TITLE" \
    "The daily wallpaper source check failed before any test results were recorded: $RUN_URL"$'\n\n'"Check the run log."
  exit 0
fi

close_if_open "$NO_RESULTS_TITLE" "The source check completed again: $RUN_URL"

while IFS=$'\t' read -r status name; do
  title="$(title_for "$name")"
  case "$status" in
    fail) report_failure "$title" "$(failure_body "$name")" ;;
    pass) close_if_open "$title" "Passing again: $RUN_URL" ;;
    skip) echo "Skipped, leaving any issue as is: $name" ;;
  esac
done <<<"$results"
