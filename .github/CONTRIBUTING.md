# Contributing

## Setup

After cloning, install the Node dev dependencies:

```bash
npm install
```

This registers the git hooks via `simple-git-hooks`:

- **pre-commit** - runs Prettier, shfmt, ShellCheck, and the BATS test suite
- **prepare-commit-msg** - launches an interactive conventional commit prompt (`czg`)

You also need [**shellcheck**](https://github.com/koalaman/shellcheck) and [**shfmt**](https://github.com/mvdan/sh) installed locally for the hooks and `npm run lint` / `npm test` to work:

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

To run the full test suite (ShellCheck + BATS):

```bash
npm test
```

Tests live in `test/backdrop.bats` and cover the pure and file-I/O functions in `src/backdrop.sh`; things like config read/write, source validation, image dimension detection, wallpaper option selection, and metadata read/write. Source resolver functions are tested using a stub `curl` script injected via `PATH`.

## Image metadata

Each `resolve_<source>()` function emits `META_TITLE:`, `META_DESC:`, and `META_URL:` prefixed lines before the image URLs it returns. `apply_wallpaper` strips these out to get the candidate URL list, then stores the values in the `META_TITLE`/`META_DESC`/`META_URL` globals. After a successful download, `_write_meta` saves those values to a `<source>-<date>.meta` file alongside the `.jpg`. The `status` command reads this file via `_meta_get` to display image title, description, and URL. Old `.meta` files are pruned on the same 14-day schedule as wallpaper images.

## Linting

To check formatting:

```bash
npm run lint
```

To auto-fix formatting:

```bash
npm run lint:fix
```
