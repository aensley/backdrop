# Contributing

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

To auto-fix formatting:

```bash
npm run lint:fix
```
