# Workspace skeleton

**Goal**: The repository has a Cargo workspace under `crates/` and a Rust CI job, committed on the shared base so that neither M0 nor M0b creates them.
**Pre-conditions**:
- [x] Plan approved by Eliott (2026-11-16): the skeleton goes on the base branch before `feat/m0-feasibility-spike` and `feat/m0b-domain-model-spec` are cut
**Success Gates**:
- ⬜ `cargo metadata --format-version 1 --no-deps` succeeds at the repository root
- ⬜ `cargo metadata --format-version 1 --no-deps --manifest-path skills/managing-roadmaps/scripts/dirtree-rdm/Cargo.toml` still succeeds, so the legacy crate is not captured by the new workspace
**References**: [R01 §Proposed State](../../__design__/rdm/architecture.md) — "Home: a Cargo workspace in this repository under `crates/`, outside `skills/`"

## Step 1: Add the workspace root
**Goal**: The root manifest, the target ignore entry and the CI job exist.
**Implementation Logic**:
1. Root `Cargo.toml`:
   - `[workspace]` with `resolver = "3"` and `members = ["crates/*"]`.
   - `exclude` lists both copies of the legacy crate: `skills/managing-roadmaps/scripts/dirtree-rdm` and `plugins/managing-roadmaps/skills/managing-roadmaps/scripts/dirtree-rdm`. Without it, cargo walks up from those crates, finds the new root and refuses to build them.
   - `[workspace.package]` with `edition = "2024"`, `rust-version = "1.88"` (the MSRV of `rmcp`) and `license = "MIT"`. The license is checked against the repository's LICENSE.
   - `crates/.gitkeep` is added so the glob has a directory to resolve against.
2. Root `.gitignore` gains `/target`.
3. `.github/workflows/rust.yml` runs `cargo test --workspace --locked` on `ubuntu-latest` with `dtolnay/rust-toolchain@stable`.
   - It triggers on pull requests and on pushes to `main` that touch `crates/**`, `Cargo.toml`, `Cargo.lock` or the workflow itself.
   - It does not touch `skills/**`, so it never triggers a skill release.
4. Record the crates.io check of `rdm-schema`, `rdm-engine`, `rdm-mcp` and `rdm-cli` (all returned 404, which means free, on 2026-11-16) in the commit body, for the spike report.
**Deliverables**: `Cargo.toml`: `[workspace]` with `members` and `exclude`, and `[workspace.package]`; `crates/.gitkeep`; `.gitignore`: the `/target` entry; `.github/workflows/rust.yml`: job `test`
**Consistency Checks**: `cargo metadata --format-version 1 --no-deps >/dev/null && cargo metadata --format-version 1 --no-deps --manifest-path skills/managing-roadmaps/scripts/dirtree-rdm/Cargo.toml >/dev/null` (expected: PASS)
**Commit**: `build(workspace): add the cargo workspace skeleton`
