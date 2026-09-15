# Regeneration Guard

**Goal**: Make the claim that `colgrep-mcp.spec.json` regenerates colgrep-mcp's manifests byte-identically into an executable tripwire, baselined before any generator change.
**Pre-conditions**:
- [ ] A colgrep-mcp checkout exists (`/Users/hacker/Documents/src/CrackingShells/colgrep-mcp`, currently at `v0.5.1`)
- [ ] `uv` resolves on PATH
**Success Gates**:
- ⬜ The guard exits 0 against the unmodified generator and the current colgrep-mcp checkout [run]
- ⬜ The guard exits non-zero if any manifest or hook file is altered [run]
- ⬜ The guard skips cleanly, without failing, when no colgrep-mcp checkout is present [run]
- ⬜ The guard lives under `evals/`, which `package_skill.py` excludes at the skill root, so it never ships to consumers [static]
**References**: [R01 §Verification](~/.claude/plans/good-news-overall-it-s-gleaming-wreath.md) — the invariant and its one known divergence

## Step 1: Write the guard and record the measured baseline

**Goal**: Turn a prose claim in `SKILL.md` into a command anyone can run.

**Implementation Logic**:
Add `evals/test_regeneration.py`, importing `load_spec` and `spawn` from `../scripts/spawn_plugin.py` and calling `spawn(spec, root, force=False, dry_run=True)` against a colgrep-mcp checkout located from the `COLGREP_MCP_ROOT` environment variable, defaulting to `../../../colgrep-mcp` relative to the repo. Skip — do not fail — when that path is absent, so the check is portable. Assert that `w.written` and `w.merged` are empty, and that `w.skipped` contains nothing beyond the one measured, allowed divergence.

That allowance is load-bearing and must be written as a named constant, not a loose filter: measured on 2026-09-16 against colgrep-mcp `ef54b8f`, the only divergence is `dev/README.md`, which is hand-maintained prose rather than a generated manifest. `SKILL.md`'s claim is specifically about manifests and hook files, so the guard encodes exactly that. Asserting `not w.skipped` outright would fail today, before any change — a false alarm that would train its readers to ignore it.
**Deliverables**: `skills/spawning-agent-plugins/evals/test_regeneration.py` — constants `ALLOWED_DIVERGENCE = {"dev/README.md"}` and `DEFAULT_CHECKOUT`, functions `checkout_root()` and `test_spec_regenerates_manifests()`
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py --root /Users/hacker/Documents/src/CrackingShells/colgrep-mcp spawn --spec skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json --dry-run 2>&1 | grep -c "^kept" | grep -qx 1` (expected: PASS)
**Commit**: `test(spawning-agent-plugins): guard the colgrep-mcp regeneration invariant`

## Step 2: Prove the guard actually bites

**Goal**: Confirm the tripwire fails when the thing it guards is broken — an always-passing guard is worse than none.

**Implementation Logic**:
Temporarily perturb one generated manifest in the colgrep-mcp checkout (for example, change the `version` string in `.claude-plugin/plugin.json`), re-run the guard, and confirm it now reports that file as a divergence and exits non-zero. Restore the checkout with `git checkout --` afterwards and confirm the guard returns to green. Record both observed outputs in the commit body so a later reader can tell a genuine regression from a changed baseline. Do not leave the perturbation behind, and do not commit anything inside the colgrep-mcp checkout.
**Deliverables**: no new files — evidence recorded in the commit message body of this step
**Consistency Checks**: `test -z "$(cd /Users/hacker/Documents/src/CrackingShells/colgrep-mcp && git status --porcelain)"` (expected: PASS)
**Commit**: `test(spawning-agent-plugins): verify the regeneration guard fails on a perturbed manifest`
