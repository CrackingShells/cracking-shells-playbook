# Stand-Alone Pass

**Goal**: Make every shipped file stand alone for a future agent in an arbitrary repo, by removing meta-discourse, references to individual repos, projects or campaigns, and advice overfitted to one data point (C5).
**Pre-conditions**:
- [ ] `best_practices.md` is merged, so this pass runs last and nothing re-contaminates the files after it
- [ ] `chores.md` is merged (its writing-history and `plugin.spec.json` edits are in scope)
**Success Gates**:
- ⬜ The leaf report lists every removed or generalised passage as `file:line → before (short) → after or "cut" → why`
- ⬜ The residue grep is empty for every shipped file, and the pass also caught what the grep cannot: e.g. "the old templates", "we", "our Org", project-specific numbers, a single repo's quirks presented as a rule
- ⬜ No rule, threshold or factual claim changed; any change of that kind was flagged instead
**References**: [R01](../../../../../__reports__/writing-release-rework/00-architecture_analysis_v0.md) — the names and data points that must NOT appear (use it as a blocklist, not as content)

## Step 1: writing-release files
**Goal**: Clean `SKILL.md` and all six references.
**Implementation Logic**:
Read each file as a cold agent working in a repo you have never seen. Remove or generalise:
1. anything that tells the story of this refactor or its reasoning;
2. names of repos, projects, people, PRs or campaigns, and their numbers;
3. advice that only makes sense for one data point, replaced with the pattern or cut;
4. first-person-plural, Org-insider phrasing.
Generic placeholders are fine (`<scope>`, `<unit>@<version>`, "a Rust workspace"). Tool names are fine: they are the patterns' vocabulary.
**Deliverables**: `skills/writing-release/SKILL.md`, `skills/writing-release/references/*.md` — edited in place; report rows
**Consistency Checks**: `for f in skills/writing-release/SKILL.md skills/writing-release/references/*.md; do ! grep -nE '[Cc]andler|[Hh]atch|colgrep|[Pp]inion|[Pp]lumage|[Mm]urmuration|\bNest\b|py-repo-template|[Dd]iscord|[Rr]ework|[Pp]reviously|[Aa]s decided|old template|CrackingShells|[Oo]ur [Oo]rg' "$f" || exit 1; done && uv run tools/package_skill.py skills/writing-release dist/` (expected: PASS)
**Commit**: `refactor(writing-release): keep the manual free of project-specific context`

## Step 2: Edited writing-history sections and plugin metadata
**Goal**: Apply the same rule to what `chores.md` changed outside writing-release.
**Implementation Logic**:
In scope: the new subsection in `skills/writing-history/references/branches.md`, the revised §Recommending a Stack in `convention-setup.md`, the new Navigation row in `skills/writing-history/SKILL.md`, and the writing-release `description` and `keywords` in `plugin.spec.json`. Leave everything else in writing-history untouched. If `plugin.spec.json` changes, say so in the report: the coordinator re-runs `spawn_plugin.py` at the close (C6 forbids staging `plugins/`).
**Deliverables**: those four locations, edited in place if needed; report rows
**Consistency Checks**: `for f in skills/writing-history/references/branches.md skills/writing-history/references/convention-setup.md skills/writing-history/SKILL.md; do ! grep -nE '[Cc]andler|[Hh]atch|colgrep|[Pp]inion|[Pp]lumage|[Mm]urmuration|\bNest\b|py-repo-template|[Dd]iscord|[Rr]ework|CrackingShells' "$f" || exit 1; done && uv run --with pyyaml tools/quick_validate.py skills/writing-history` (expected: PASS)
**Commit**: `refactor(writing-history): keep the release handoff free of project-specific context`
