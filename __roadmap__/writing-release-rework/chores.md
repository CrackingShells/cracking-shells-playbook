# Coordinator Chores

**Goal**: Hand release concerns off from `writing-history`, refresh the writing-release metadata, and record the retired Discord material for a possible future skill. All three are small, mechanical and owned by the coordinator.
**Pre-conditions**:
- [ ] The campaign README contract is committed; C1 file names are final
- [ ] Run on the integration branch while the `author/` dispatches work; these touch no file an author leaf owns
**Success Gates**:
- ⬜ `skills/writing-history/references/branches.md` and `convention-setup.md` name the `writing-release` skill and its reference files, with no relative path across skills
- ⬜ `uv run tools/check_plugin.py --root . --spec plugin.spec.json` passes after the metadata step
- ⬜ `__reports__/writing-release-rework/` has a `notice` report citing commit `461036c` for the four Hatch-era files
**References**: [R04 §Integration Contract](../../skills/writing-history/references/branches.md) — where the main-integration note goes; [R05 §Recommending a Stack](../../skills/writing-history/references/convention-setup.md) — the rows to hand off

## Step 1: writing-history handoff
**Goal**: Make `writing-history` defer to `writing-release` for landing on `main` and for choosing release tooling, without restating either.
**Implementation Logic**:
1. `branches.md`: below the Integration Contract, add a short subsection. When the parent is the repo's release branch, the `writing-release` skill's `references/lifecycle.md` decides the lane: Reviewed and Self-merge PRs carry out the four steps through the PR, and the Direct lane uses this contract as written.
2. `convention-setup.md` §Recommending a Stack: keep the commit-convention and enforcement side of each row, and replace release-tool selection with a pointer to the `writing-release` skill's `references/setting-up.md`.
3. `SKILL.md`: add one Navigation row, "Land work on `main`, set up or debug releases → the `writing-release` skill".
This commit must be `feat`, so that `writing-history` releases: the hook regenerates `plugins/writing-history/`.
**Deliverables**: `skills/writing-history/references/branches.md` — new subsection under `Integration Contract`; `skills/writing-history/references/convention-setup.md` — revised `Recommending a Stack`; `skills/writing-history/SKILL.md` — one Navigation row
**Consistency Checks**: `grep -c 'writing-release' skills/writing-history/references/branches.md skills/writing-history/references/convention-setup.md skills/writing-history/SKILL.md && uv run --with pyyaml tools/quick_validate.py skills/writing-history` (expected: PASS)
**Commit**: `feat(writing-history): hand landing on main and release tooling to writing-release`

## Step 2: writing-release metadata
**Goal**: Replace the Discord-era description and keywords wherever the repo advertises the skill.
**Implementation Logic**:
1. `plugin.spec.json`: the writing-release `description` (one line: the release-cycle manual) and `keywords` (release, pull-request, ci, versioning, agents).
2. `uv run tools/spawn_plugin.py spawn --force` with the repo's spec, so that `plugins/writing-release/plugin.json` and `.claude-plugin/plugin.json` match. Check `git diff --stat plugins/` for unrelated churn first.
3. `README.md`: the writing-release row in the skills table, and the mermaid "Release" node label.
4. `CONTRIBUTING.md`: only if its plugin-skills line describes writing-release's content (it currently only lists names, so it most likely needs nothing).
**Deliverables**: `plugin.spec.json` — writing-release `description`, `keywords`; `plugins/writing-release/plugin.json`, `plugins/writing-release/.claude-plugin/plugin.json` — regenerated; `README.md` — skills-table row, mermaid node
**Consistency Checks**: `uv run tools/check_plugin.py --root . --spec plugin.spec.json && ! grep -n 'Discord' plugin.spec.json` (expected: PASS)
**Commit**: `docs(plugins): describe writing-release as the release-cycle manual`

## Step 3: Notice report for a future announcements skill
**Goal**: Keep the retired Discord material findable for a possible dedicated announcements skill.
**Implementation Logic**:
1. Write `__reports__/writing-release-rework/02-notice_announcements_skill_v0.md` (writing-reports `notice` type). It records what the two Discord references held (dev and stable announcement templates and custom-emoji conventions), that they live at commit `461036c` under `skills/writing-release/references/`, why they were retired from writing-release (announcing is a different job from running the cycle), and what a dedicated skill would need to decide (channels, cadence, who posts).
2. Add the row to `__reports__/writing-release-rework/README.md`.
**Deliverables**: `__reports__/writing-release-rework/02-notice_announcements_skill_v0.md`; `__reports__/writing-release-rework/README.md` — new row
**Consistency Checks**: `git cat-file -e 461036c:skills/writing-release/references/discord-main-release.md && grep -c '461036c' __reports__/writing-release-rework/02-notice_announcements_skill_v0.md` (expected: PASS)
**Commit**: `docs(reports): note the retired announcement templates for a future skill`
