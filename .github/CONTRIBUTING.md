# Contributing

## Setup

After cloning, install the Node dev dependencies:

```bash
npm install
```

This registers the git hooks via `simple-git-hooks`:

- **pre-commit** - runs Prettier, shfmt, ShellCheck, and BATS
- **prepare-commit-msg** - launches an interactive conventional commit prompt (`czg`)

You also need [**shellcheck**](https://github.com/koalaman/shellcheck) and [**shfmt**](https://github.com/mvdan/sh) installed locally for `npm run lint` / `npm test` to work:

```bash
# Ubuntu/Debian
sudo apt install shellcheck shfmt

# macOS
brew install shellcheck shfmt
```

## Commits

Commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) spec. The pre-commit hook will guide you through it. You can also run the prompt manually:

```bash
npm run commit
```

## Testing

To run the test suite (ShellCheck + BATS):

```bash
npm test
```

Tests live in `test/backdrop.bats` and cover the pure and file-I/O functions in `src/backdrop.sh`; things like config read/write, source validation, image dimension detection, wallpaper option selection, metadata read/write, version comparison, and source rotation. Source resolver functions are tested using a stub `curl` script injected via `PATH`. The `_rotation_index` helper takes a unix timestamp as a parameter rather than calling `date`, so rotation tests run without needing to stub `date`; `get_active_source` integration tests stub `date` via `PATH` in the same way as `curl`. The KDE and LXQt wallpaper setters are tested against stub `qdbus6`/`qdbus`/`plasma-apply-wallpaperimage`/`pcmanfm-qt` commands that record their arguments; the COSMIC setter and the Xfce XML fallback are tested against real files in a temporary directory.

### Integration tests

Integration tests live in `test/integration/` and are not part of `npm test`. Run them all with:

```bash
npm run test:integration
```

- **`sources.bats`** downloads today's image from every source and checks that it has a title, a URL, and is a real image at least 800px wide. It needs network access and fails when a site is down or changes its markup. CI runs it daily (`.github/workflows/integration.yml`), and you can also start it from the Actions tab. Each failing source gets its own GitHub issue, labelled `source-check` and titled e.g. "Wallpaper source failing: earth". If that source's issue is already open, CI comments on it instead, and closes it once the source passes again (`.github/scripts/report-sources.sh`). Adding a source without adding a test here fails the suite.
- **`desktop.bats`** runs the GNOME, MATE and Xfce setters against the real `gsettings` and `xfconf-query` tools and setting schemas, then reads the values back. It never touches your own desktop: `gsettings` writes to a temporary keyfile and `xfconf` runs on a private D-Bus session. Tests skip when a tool or schema isn't installed; CI installs them all and sets `BACKDROP_REQUIRE_DE_TOOLS=1` so nothing is skipped silently. To install the tools on Ubuntu/Debian:

  ```bash
  sudo apt install dbus libglib2.0-bin gsettings-desktop-schemas mate-desktop-common cinnamon-desktop-data xfconf
  ```

These tests confirm the right settings get written, not that the image shows up on screen. Before a release that changes a wallpaper setter, check it by hand on the affected desktop:

- [ ] `backdrop update --force` changes the visible wallpaper
- [ ] `backdrop status` shows the new image and its metadata
- [ ] A tall image (e.g. from `apod`) is letterboxed rather than cropped
- [ ] On multi-monitor setups, every monitor changes

## Image metadata

Each `resolve_<source>()` function emits `META_TITLE:`, `META_DESC:`, and `META_URL:` prefixed lines before the image URLs it returns. `apply_wallpaper` strips these out to get the candidate URL list, then stores the values in the `META_TITLE`/`META_DESC`/`META_URL` globals.

After a successful download the metadata is saved to a `<source>-<date>.meta` file (key = value format) alongside the `.jpg`. The `status` command reads this file to display image title, description, and URL. Old `.meta` files are pruned on the same 14-day schedule as wallpaper images.

## Linting

To check formatting:

```bash
npm run lint
```

To auto-fix formatting:

```bash
npm run lint:fix
```
