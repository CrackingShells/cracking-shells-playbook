# Write CONTRIBUTING.md and README Installation Guide

**Goal**: Document developer setup and user installation so the CI pipeline is usable without tribal knowledge.
**Pre-conditions**:
- [ ] CI pipeline verified working (dry-run passed, `actionlint` clean)
- [ ] Confirmed: correct skill install path for Claude Code CLI (`~/.claude/skills/` or `.agent/skills/` — check Claude Code docs or test)
- [ ] Confirmed: whether `.skill` double-click install works in Claude Desktop
**Success Gates**:
- ✅ `CONTRIBUTING.md` exists and covers all 6 required topics (see Step 1) [static]
- ✅ `README.md` installation section covers 3 install options with correct platform-specific paths [static]
- ✅ No placeholder text remains in either file [static]
**References**: [R01 §Phase 3](~/.claude/plans/binary-jumping-trinket.md) — content outline; [R02 open questions](~/.claude/plans/binary-jumping-trinket.md#things-to-verify) — paths to verify

---

## Step 1: Write CONTRIBUTING.md

**Goal**: Give contributors everything needed to set up their environment and understand the release process, with no guessing required.

**Implementation Logic**:
Create `CONTRIBUTING.md` at the repo root. Cover exactly 6 topics as `##` sections: (1) Prerequisites — `git`, `node ≥20`, `uv`, `rustup` (managing-roadmaps only); (2) First-time setup — `make dev-setup` and what it does; (3) Pre-commit hook — what it builds, what it does NOT do (no git staging, no Rust recompile); (4) Rust binary rebuild — when and how to use `make build-rust-local`; (5) Conventional commit format — `feat:` minor, `fix:` patch, `BREAKING CHANGE:` major; (6) First-run release note — all skills release at once on first pipeline run, this is expected behavior. Verify all content is accurate against the actual implementation before writing.
**References**: [R01 §Phase 3 §10](~/.claude/plans/binary-jumping-trinket.md) — CONTRIBUTING.md content spec
**Deliverables**: `CONTRIBUTING.md` — 6 `##` sections covering prerequisites, dev-setup, hook behavior, build-rust-local, commit format, first-run note
**Consistency Checks**: `[ $(grep -c '^##' CONTRIBUTING.md) -ge 6 ]` (expected: PASS)
**Commit**: `docs(contributing): add CONTRIBUTING.md with dev setup and release process guide`

---

## Step 2: Write README.md installation guide

**Goal**: Give skill users a clear, actionable path to install skills whether they prefer downloading, cloning, or building from source.

**Implementation Logic**:
Before writing, verify two facts: (a) the correct Claude Code CLI skill install path (check `~/.claude/skills/` by reading Claude Code docs or testing with a real `.skill` file); (b) whether Claude Desktop supports `.skill` double-click install (check the Claude Desktop UI or official docs). Then add (or create) `README.md` with an Installation section containing 3 subsections: Option A — download `.skill` from GitHub Releases and install to the verified path; Option B — clone repo, run `make dev-setup`, then `uv run tools/package_skill.py skills/<name> dist/` and install the output; Option C (managing-roadmaps only) — build Rust from source with `cargo build --release`, copy binary to `bin/`, then follow Option B. Fill in verified paths and Claude Desktop instructions — no placeholder text.
**References**: [R01 §Phase 3 §11](~/.claude/plans/binary-jumping-trinket.md) — README install guide spec
**Deliverables**: `README.md` — Installation section with Option A, B, C subsections containing verified install paths
**Consistency Checks**: `[ $(grep -c '^### Option' README.md) -ge 3 ]` (expected: PASS)
**Commit**: `docs(readme): add skill installation guide to README`
