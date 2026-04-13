# Contributing

## Prerequisites

Install these tools before setting up the development environment:

- **git** — version control
- **Node.js ≥ 20** — required for `npm ci` and semantic-release
- **uv** — Python runner used by the packager (`curl -LsSf https://astral.sh/uv/install.sh | sh`)
- **rustup** — Rust toolchain manager, required only if contributing to `managing-roadmaps` (`curl https://sh.rustup.rs -sSf | sh`)

## First-time setup

Run once after cloning the repository:

```bash
make dev-setup
```

This command does three things: registers `.githooks/` as the active git hooks directory (`git config core.hooksPath .githooks`), ensures `.githooks/pre-commit` is executable, and runs `npm install` to install semantic-release dev dependencies.

## Pre-commit hook

The hook at `.githooks/pre-commit` runs automatically on every `git commit`. It inspects the staged file list and re-packages any skill directory that has staged changes, writing the updated `.skill` file to `dist/` (gitignored — never committed). The hook does **not** stage anything into git and does **not** recompile Rust binaries (that step is too slow for a commit hook; see below).

If you stage any `.rs` or `Cargo.toml` file under `skills/managing-roadmaps/scripts/dirtree-rdm/`, the hook prints a reminder to rebuild the local binary — but it does not block the commit.

## Rust binary rebuild

The `managing-roadmaps` skill ships a pre-compiled Rust CLI (`dirtree-rdm`). The hook skips Rust recompilation. To rebuild the binary for your local architecture after changing Rust source:

```bash
make build-rust-local
```

This runs `bash build.sh local` inside `skills/managing-roadmaps/scripts/dirtree-rdm/`, detecting your OS and architecture automatically. CI handles the full cross-compilation matrix (macOS arm64, macOS x64, Linux x64, Linux arm64, Windows x64) via the `release-managing-roadmaps` GitHub Actions workflow.

## Conventional commit format

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/). The CI pipeline uses these prefixes to compute per-skill version bumps:

| Prefix | Version bump |
|--------|-------------|
| `fix(scope):` | patch (1.0.0 → 1.0.1) |
| `feat(scope):` | minor (1.0.0 → 1.1.0) |
| `BREAKING CHANGE:` footer | major (1.0.0 → 2.0.0) |
| `chore:`, `docs:`, `refactor:` | no release |

The scope should be the skill directory name (e.g., `feat(writing-reports): ...`). Only commits that touch files under `skills/<name>/` will trigger a release for that skill, regardless of the commit scope.

## First-run release note

When the CI pipeline runs for the first time against a skill — or when no prior git tag exists for that skill — semantic-release-monorepo treats all commits as new and publishes a release for every skill simultaneously. This is expected behavior on first run, not a bug. Subsequent releases are scoped to only the skills with changes since the last tag.
