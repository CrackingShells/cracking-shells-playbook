# dirtree-rdm Plugin Binaries — Observation (v0)

Date: 2026-10-05

---
type: observation
topic: dirtree_rdm_plugin_binaries
spotted-during: relaxing the dirtree-rdm `**Commit**:` grammar and tracing how the change reaches Nest plugin users
date: 2026-10-05
domain: code
confidence: confirmed
urgency: medium
deferred-because: release-pipeline change, outside the grammar fix's scope; it predates that change
---

## What Was Noticed

- The installable `managing-roadmaps` plugin ships `dirtree-rdm` binaries for **macOS only** (`darwin-arm64`, `darwin-x64`). Those are hand-built and committed.
- CI builds Linux (x64, arm64) and Windows (x64) binaries on every managing-roadmaps release. They reach only the `managing-roadmaps.skill` GitHub release asset, never `plugins/managing-roadmaps/`.
- So a Claude Code or Codex user on Linux or Windows who installs the plugin from Nest gets the error "binary not found" from `dirtree-rdm.sh` / `dirtree-rdm.ps1`. Every structural roadmap operation depends on that binary, so the skill's One Rule ("use `dirtree-rdm` for all structural mutations") cannot be followed.

## Context

This came up while confirming the release path for the `feat(dirtree-rdm)` grammar relaxation (merge `83a7e49`). The aim was to confirm that a merge to `main` produces a `managing-roadmaps@1.2.0` tag, updated plugin manifests, and an update served through Nest. All three hold. Tracing which files the release commit carries showed the binary gap. Fixing it means changing the release pipeline and deciding how binaries should be distributed, which is a separate decision from the grammar fix.

## Location Map

- `.github/workflows/release.yml` — job `build-managing-roadmaps-binaries` (matrix: darwin / linux / windows); `release` job downloads the artifacts into `skills/managing-roadmaps/scripts/dirtree-rdm/bin/`
- `release-config.js` — `@semantic-release/git` `assets` list; `prepareCmd` entries (`package_skill.py`, `set_plugin_version.py`)
- `tools/set_plugin_version.py` — module docstring explains why `@semantic-release/git` only sees files under `skills/<name>/`
- `tools/assemble_plugin.py` — copies `skills/<name>/` into `plugins/<name>/skills/<name>/`; it is not called during release
- `skills/managing-roadmaps/scripts/dirtree-rdm/build.sh` — target triple → output name map
- `skills/managing-roadmaps/scripts/dirtree-rdm.sh` — Linux dispatch to `dirtree-rdm-linux-{x64,arm64}`
- `skills/managing-roadmaps/scripts/dirtree-rdm.ps1` — Windows dispatch to `dirtree-rdm-windows-x64.exe`
- `plugins/managing-roadmaps/skills/managing-roadmaps/scripts/dirtree-rdm/bin/` — currently only the two darwin files
- `CrackingShells/Nest` `.claude-plugin/marketplace.json`, `.agents/plugins/marketplace.json` — unpinned `git-subdir` on `plugins/managing-roadmaps`

## Evidence

The pipeline, as currently wired:

```mermaid
flowchart LR
  B[CI build matrix<br/>darwin · linux · windows] --> D[release job:<br/>download into skills/.../bin/]
  D --> P[package_skill.py<br/>→ dist/managing-roadmaps.skill]
  P --> GH[GitHub release asset ✅ all OSes]
  D -. not assembled .-> PL[plugins/managing-roadmaps/.../bin/]
  D -. not in git.assets .-> RC[release commit]
  PL --> N[Nest git-subdir → users ❌ darwin only]
```

- `git ls-files` under either `bin/` lists only `dirtree-rdm-darwin-arm64` and `dirtree-rdm-darwin-x64`.
- In `release-config.js`, `@semantic-release/git` `assets` covers `package.json`, `CHANGELOG.md`, the two plugin manifests and `plugins/<name>/skills/<name>/**`. None of these paths matches the CI binaries, which land in `skills/.../bin/`, and nothing runs `assemble_plugin.py` during release.
- The `set_plugin_version.py` docstring records, as measured on this repo, that `@semantic-release/git` cannot see files outside `skills/<name>/` unless something stages them explicitly.
- Darwin binaries reach users only because maintainers rebuild and commit them by hand (`71efb50`, and `chore(dirtree-rdm): rebuild darwin binaries …` in merge `83a7e49`).

## Re-observation Steps

1. On a Linux machine, install `managing-roadmaps` from the Nest marketplace.
2. Run `bash <plugin>/skills/managing-roadmaps/scripts/dirtree-rdm.sh ls`.
3. Expect the error `dirtree-rdm: binary not found at …/dirtree-rdm-linux-x64`.

## Hand-off Questions

Working theory, if any: the smallest fix is for the release job to run `uv run tools/assemble_plugin.py managing-roadmaps` after downloading the artifacts. It would then `git add` the `plugins/managing-roadmaps/.../bin/*` files explicitly, the same staging technique `set_plugin_version.py` uses, so the release commit carries all five binaries. This also takes away the manual darwin rebuild step.

- Is committing about 15 MB of binaries per release into `main` acceptable, or should the plugin fetch the binary from the GitHub release asset on first use (a download or verify step in `dirtree-rdm.sh` / `.ps1`)?
- If they are committed, should the darwin binaries also come from CI only, so they stop drifting with each maintainer's local toolchain (the arm64 and x64 sizes moved in opposite directions in `83a7e49`)?
- Is the `drift` job affected? It re-assembles `plugins/` from `skills/` and fails on difference. Binaries that exist only in a release commit would make the next push look like drift unless `skills/.../bin/` carries them too.
- Does `dirtree-rdm.ps1` resolve correctly from a Codex or Agent Plugins install on Windows, and does any client run `.sh` there instead?
- Should Linux `aarch64` stay a target? `build.sh` lists it, but whether it is worth shipping is unverified.

## Scope Boundary

This report does not authorize changes to the `dirtree-rdm` validator, the grammar, or any skill content; it covers only the release pipeline and how binaries are distributed.
