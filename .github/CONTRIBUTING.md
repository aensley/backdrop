# Contributing

## Requirements

- **Node.js** (LTS) — for dev tooling and git hooks
- **Rust** (stable) — required to build the app and run `npm run lint` (which includes Clippy)

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Setup

After cloning, run:

```bash
npm install
```

This installs dev dependencies and registers the git hooks via `simple-git-hooks`:

- **pre-commit** — runs Prettier to enforce consistent formatting
- **prepare-commit-msg** — launches an interactive conventional commit prompt (`czg`)

## Commits

Commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) spec — the pre-commit hook will guide you through it. You can also run the prompt manually:

```bash
npm run commit
```

## Linting

To check formatting without committing:

```bash
npm run lint
```

This runs Prettier (JS/JSON/Markdown), `rustfmt --check` (Rust), and Clippy in sequence.

To auto-fix formatting:

```bash
npm run lint:fix        # Prettier
cargo fmt --manifest-path src-tauri/Cargo.toml  # rustfmt
```
