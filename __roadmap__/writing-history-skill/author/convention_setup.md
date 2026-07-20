# Author convention-setup Reference

**Goal**: Author `skills/writing-history/references/convention-setup.md` — the agent-run setup interview and tool-agnostic bootstrap flow for standing up commit-convention / semver / changelog machinery when a project has none.
**Pre-conditions**:
- [ ] `scaffold_skill_package` merged (skill dir + `references/` exist)
**Success Gates**:
- ⬜ `references/convention-setup.md` exists with a setup-interview question set and a bootstrap checklist
- ⬜ It is explicitly tool-agnostic — recommendations are derived from repo context (materials, languages, file types, existing tooling), not a single prescribed stack
- ⬜ It cross-links `commit-authoring.md` (vocabulary use) and `semver-changelog.md` (bump/changelog principles)
**References**: [R03 release pipeline architecture](../../../__reports__/skill_cicd/01-architecture_analysis_v0.md) — one concrete stack example (semantic-release monorepo) to cite as *an* option, not the mandate

## Step 1: Write convention-setup.md (interview + bootstrap)

**Goal**: Produce the setup-interview + recommendation + bootstrap reference.

**Implementation Logic**:
Author `skills/writing-history/references/convention-setup.md` with:
1. **When to Run Setup** — trigger: no machinery AND no established history/CONTRIBUTING pattern (the greenfield branch of `commit-authoring.md`'s precedence).
2. **Setup Interview** — a concrete question set the agent asks the user: what materials/deliverables does the repo hold (code? prose? data? mixed)? which languages / file types? is there existing tooling (hooks, CI, release config)? who commits (humans, agents, both)? what release cadence / does a changelog matter to stakeholders? Note that answers determine tooling — file types drive tool choice.
3. **Derive vs. Bootstrap** — first exhaust derivation (machinery → history+docs); only bootstrap when truly greenfield.
4. **Recommending a Stack (context-driven)** — guidance, not a fixed answer: map common contexts to sensible options (e.g. JS/TS repo → commitlint + semantic-release; Python → equivalent tools; prose/scientific docs → a lightweight convention + changelog generator), while stating there is no one-size-fits-all and the agent must justify its pick from repo context. Cite R03 as one worked example.
5. **Bootstrap Checklist** — the ordered steps to stand up the chosen stack (define the vocabulary, add config, wire changelog automation, decide local-hook vs CI-only, document in CONTRIBUTING).
6. **Delivering Recommendations** — the agent presents final recommendations as part of its setup-interview answer, for user approval, before creating any config.

**Deliverables**: `skills/writing-history/references/convention-setup.md` (headings: `When to Run Setup`, `Setup Interview`, `Derive vs. Bootstrap`, `Recommending a Stack`, `Bootstrap Checklist`, `Delivering Recommendations`)
**Consistency Checks**: `python3 -c "t=open('skills/writing-history/references/convention-setup.md').read(); assert 'Setup Interview' in t and 'semver-changelog.md' in t and 'commit-authoring.md' in t and ('one-size' in t.lower() or 'agnostic' in t.lower()); print('PASS')"` (expected: PASS)
**Commit**: `feat(writing-history): add convention-setup reference with setup interview`
