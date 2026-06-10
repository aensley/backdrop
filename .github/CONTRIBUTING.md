# Contributing

## Requirements

- **Node.js** (LTS) - dev tooling and git hooks
- **Rust** (stable via [rustup](https://rustup.rs/)) - building the app and running lint

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Linux system dependencies

`npm run dev` and `npm run build` require these system packages (Ubuntu/Debian):

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
  patchelf build-essential libxdo-dev libssl-dev
```

For RPM-based distros (Fedora, RHEL): `dnf install` the equivalent packages.

## Setup

After cloning, run:

```bash
npm install
```

This installs dev dependencies and registers two git hooks via `simple-git-hooks`:

- **pre-commit** - runs the full lint suite (Prettier + rustfmt + Clippy)
- **prepare-commit-msg** - launches the interactive conventional commit prompt (`czg`)

## Development

```bash
npm run dev      # Tauri dev build with hot reload
npm run build    # production build
```

Rust only:

```bash
cargo build --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## Commits

Commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) spec. The `prepare-commit-msg` hook will prompt you interactively. You can also invoke it directly:

```bash
npm run commit
```

## Linting

```bash
npm run lint        # check: Prettier + rustfmt + Clippy
npm run lint:fix    # auto-fix: Prettier + rustfmt + Clippy
```

Both commands cover the full stack. There is no need to run `cargo fmt` or `cargo clippy` separately.

## Adding a new image source

1. Create `src-tauri/src/sources/<key>.rs` and export:

   ```rust
   pub async fn resolve(client: &Client) -> Result<ImageInfo>
   ```

   `ImageInfo` (defined in `sources/mod.rs`) carries candidate image URLs plus optional metadata:

   ```rust
   pub struct ImageInfo {
       pub urls: Vec<String>,        // preference order; caller tries each until one downloads
       pub title: Option<String>,    // image title from the source site
       pub description: Option<String>, // caption / copyright / explanation
       pub page_url: Option<String>, // link back to the source page
   }
   ```

   Extract metadata from whatever the source API provides (RSS `<title>`, JSON fields, HTML `<title>` tag, etc.). Use the `clean_text` and `extract_tag` helpers in `sources/mod.rs` for stripping CDATA and HTML from RSS content.

2. In `src-tauri/src/sources/mod.rs`:
   - Add `pub mod <key>;`
   - Add `"<key>"` to `VALID_SOURCES`
   - Add a match arm in `resolve()`

3. Document the new source in `README.md` (Sources table).

Metadata is automatically saved alongside each downloaded image as a `.json` sidecar and surfaced in `backdrop status`, `backdrop update`, and the `get_image_meta` Tauri command.

## Adding a new wallpaper backend (desktop environment)

1. Create `src-tauri/src/wallpaper/<de>.rs` and export:

   ```rust
   pub fn set(file: &Path, option: &str) -> Result<()>
   pub fn current_option() -> Option<String>
   ```

2. In `src-tauri/src/wallpaper/mod.rs`:
   - Add `pub mod <de>;`
   - Add a variant to `DesktopEnv`
   - Add detection logic in `detect_de()`
   - Wire up `set()` and `current_option()` in their respective dispatch blocks

3. Add the DE to the Supported Platforms list in `README.md`.

## Architecture constraints

- **No new dependencies** unless they are pervasive (99%+ ecosystem adoption) and unavoidably large. Keep backdrop self-contained.
- **No image-processing libraries.** The custom header parser in `image.rs` is intentional.
- **No async in `timer.rs`.** The blocking `Command` calls there are intentional.
- **Config format must stay flat `key = value`.** It must remain hand-editable.

## Release process

Releases are fully automated via [release-please](https://github.com/googleapis/release-please):

1. Every merge to `main` runs the `release-please` job, which maintains a version bump PR.
2. Merging that PR creates a **draft** GitHub release (no git tag yet) and triggers CI builds for all platforms (native packages + Flatpak + Snap).
3. Once every build leg succeeds, the draft is published, which creates the `v*` tag.

Release notes are generated automatically from commit history and published directly to the GitHub release. No `CHANGELOG.md` is written. Conventional Commit types (`feat`, `fix`, `chore`, etc.) control what appears in each release entry, which is another reason commits must follow the spec.
