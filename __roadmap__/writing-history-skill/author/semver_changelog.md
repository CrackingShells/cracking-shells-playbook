# Author semver-changelog Reference

**Goal**: Author `skills/writing-history/references/semver-changelog.md` — the tool-agnostic principles tying structured commit messages → automated semantic-version bumps → changelog, anchored by the "machinery/history is the authorized vocabulary" rule.
**Pre-conditions**:
- [ ] `scaffold_skill_package` merged (skill dir + `references/` exist)
**Success Gates**:
- ⬜ `references/semver-changelog.md` exists and states the ground-truth rule (the project's machinery — or, absent that, its history/docs — defines the authorized vocabulary and its bump mapping)
- ⬜ It explains the bump-mapping *concept* (which change classes imply major/minor/patch) as PROJECT-DEFINED, never a hardcoded universal table
- ⬜ It covers changelog automation tool-agnostically and frames both code and document repos
**References**: [R04 release-config.js](../../../release-config.js) — one concrete changelog/versioning automation example to cite

## Step 1: Write semver-changelog.md (principles)

**Goal**: Produce the tool-agnostic semver + changelog principles reference.

**Implementation Logic**:
Author `skills/writing-history/references/semver-changelog.md` with:
1. **Ground Truth: The Vocabulary Is Derived** — the authorized commit vocabulary and its version-bump semantics come from the project's machinery (commitlint/semantic-release config); absent that, from its history + CONTRIBUTING docs. The skill never imposes `feat/fix/...`. This is the same rule `commit-authoring.md` applies.
2. **From Commits to Versions** — how a structured message's type/class maps to a semver bump (major/minor/patch); present the mapping as a project-defined contract with a worked example, explicitly noting the classes and mapping differ per project and per material type.
3. **Changelog Automation** — how conventional-structured history feeds an automated changelog; list tool options (e.g. semantic-release + @semantic-release/changelog, or equivalents) without prescribing one; cite R04 as an example.
4. **Documents, Not Just Code** — what a "minor" vs "patch" means for scientific/application documents (e.g. a new section vs a correction) is defined by the project's own vocabulary; give a document-repo example.
5. **Relationship to Setup** — when no machinery exists, `convention-setup.md` stands it up; this file explains the principles that setup encodes.

**Deliverables**: `skills/writing-history/references/semver-changelog.md` (headings: `Ground Truth: The Vocabulary Is Derived`, `From Commits to Versions`, `Changelog Automation`, `Documents, Not Just Code`, `Relationship to Setup`)
**Consistency Checks**: `python3 -c "t=open('skills/writing-history/references/semver-changelog.md').read(); tl=t.lower(); assert 'ground truth' in tl and 'semver' in tl and 'changelog' in tl and 'convention-setup.md' in t and ('project-defined' in tl or 'per project' in tl or 'never' in tl); print('PASS')"` (expected: PASS)
**Commit**: `feat(writing-history): add semver-changelog principles reference`
