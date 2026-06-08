# Contributing

## Setup

After cloning, install the Node dev dependencies:

```bash
npm install
```

This registers the git hooks via `simple-git-hooks`:

- **pre-commit** — runs Prettier, shfmt, and ShellCheck
- **prepare-commit-msg** — launches an interactive conventional commit prompt (`czg`)

You also need **shellcheck** and **shfmt** installed locally for the hooks and `npm run lint` / `npm test` to work:

```bash
# Ubuntu/Debian
sudo apt install shellcheck shfmt

# macOS
brew install shellcheck shfmt
```

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

To auto-fix formatting:

```bash
npm run lint:fix
```
