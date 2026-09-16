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

The hook at `.githooks/pre-commit` runs automatically on every `git commit`. It inspects the staged file list and, for every skill directory that has staged changes:

- re-packages it, writing the updated `.skill` file to `dist/` (gitignored — never committed, and not staged by the hook), and
- if that skill also ships a plugin root (`plugins/<name>/`, see below), re-assembles `plugins/<name>/skills/<name>/` from the skill source and **does** `git add plugins/<name>/` so the refreshed tree rides along in the same commit as the skill change that produced it.

The hook does not recompile Rust binaries (that step is too slow for a commit hook; see below).

If you stage any `.rs` or `Cargo.toml` file under `skills/managing-roadmaps/scripts/dirtree-rdm/`, the hook prints a reminder to rebuild the local binary — but it does not block the commit.

## Plugin distribution (`plugins/`)

Five skills — `managing-roadmaps`, `writing-history`, `writing-release`, `writing-reports`, `spawning-agent-plugins` — are also distributed as installable agent plugins, in addition to their `.skill` files. Each has a plugin root at `plugins/<name>/`: a Claude Code manifest (`.claude-plugin/plugin.json`), an Agent Plugins 1.0 / Codex manifest (`plugin.json`), and an assembled copy of the skill (`skills/<name>/`).

Unlike `dist/`, **`plugins/` is committed.** These trees are installed by git source (a marketplace entry pointing at this repo and a path inside it), so the assembled output has to exist in the repository at the resolved ref — it cannot be gitignored build output the way `.skill` files are. That makes `plugins/<name>/` generated content that lives in git beside the source it's generated from, which is a drift risk `dist/` never had. Three things keep it honest:

- the pre-commit hook above regenerates `plugins/<name>/` whenever `skills/<name>/` has staged changes,
- `release-config.js`'s `@semantic-release/exec` step writes each release's version into `plugins/<name>/plugin.json` and `.claude-plugin/plugin.json` (via `tools/set_plugin_version.py`) and the `@semantic-release/git` step commits it, and
- CI fails the build if regenerating `plugins/` from `skills/` would produce a diff (drift check).

`skills/<name>/` is the only editable source. **Nobody hand-edits anything under `plugins/`** — a manual edit there is indistinguishable from drift and will be reverted by the next regeneration or flagged by CI.

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
