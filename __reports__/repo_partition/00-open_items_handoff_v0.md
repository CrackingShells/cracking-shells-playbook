# Repo Partition — Open Items Hand-off (v0)

Date: 2026-10-06

---
type: observation
topic: repo_partition
spotted-during: reorganising the CrackingShells playbook into playbook, Plumage and Pinion; the session closed with 19 items still open
date: 2026-10-06
domain: code
confidence: confirmed
urgency: medium
deferred-because: the remaining work is either gated on the maintainer's rulings, on a new repo reaching a release path, or belongs in a session started inside another repo
---

> **Report type.** None of the seven report types is a backlog. This is one consolidated `observation`, because its job is the one that type exists for: orient a cold agent on peripheral material the originating session did not finish. It is not split into per-item `open-question` reports: the UNRULED items are already framed as proposals, the RULED-TODO items need no decision, and 19 separate files would bury the sequencing. An item that grows into a real decision should be spun out into its own `open-question` report (next round).

## What Was Noticed

The reorganisation split the org's engineering practice across repositories. It left 19 open items, summarised below. Status values:

- **RULED-TODO**: the maintainer approved it; it is not yet done.
- **UNRULED**: a proposal awaiting the maintainer's decision. Do not execute it without a ruling.
- **WATCH**: nothing to do now; a condition to keep an eye on.

| # | Area | Item | Status | Gated on |
|--:|:---|:---|:---|:---|
| 1 | playbook | `code-change-phases` instruction becomes skill `leading-campaigns` | UNRULED | ruling |
| 2 | playbook | fold `debugging/<issue>` branch pattern into `branches.md`, delete file | UNRULED | ruling |
| 3 | playbook | deprecate `analytic-behavior` and `work-ethics`; maybe an org AGENTS.md template | UNRULED | ruling |
| 4 | playbook | 9 documentation/docstring instruction files become skill `documenting-python-projects` | UNRULED | ruling |
| 5 | playbook | delete `archive/committing-changes/` | UNRULED | ruling |
| 6 | playbook, Pinion, Plumage | move plugin/skill packaging scripts into `spawning-agent-plugins` | UNRULED | ruling; best done with 11 |
| 7 | GitHub | update the playbook repo description | UNRULED | ruling |
| 8 | `~/.claude/skills` | dangling symlink and stray zip | UNRULED | ruling |
| 9 | Plumage, Pinion | plugin + marketplace wiring, versioning and release framework | RULED-TODO | session inside each repo |
| 10 | Nest | register both repos' plugins; write a "how to add a plugin" section | RULED-TODO | 9 |
| 11 | playbook, Nest, colgrep-mcp | hand `spawning-agent-plugins` over to Pinion | RULED-TODO | Pinion plugin live (9, 10) |
| 12 | playbook | remove the three Plumage-owned skills from the playbook | RULED-TODO | Plumage release path (9) |
| 13 | `~/.claude/skills` | repoint symlinks to the new repos | RULED-TODO | 9, 12 |
| 14 | playbook | README "Using it in a project" ("Sister repositories" done) | RULED-TODO | maintainer test of the settings path |
| 15 | umbod (future repo) | replace `committing-changes` with `writing-history` | RULED-TODO | umbod repo exists |
| 16 | 4 repos | push and merge state, all local only | RULED-TODO | maintainer action |
| 17 | playbook | `dirtree-rdm` plugin ships darwin-only binaries | WATCH | |
| 18 | py-repo-template | residual Wobble references | WATCH | another session on it |
| 19 | Hatch-Validator, Hatchling | hook hygiene | WATCH | |

**Order of play.** The critical path is 9, then 10, then 11 and 12, then 13, then 14. Items 1 to 8 are independent of that path except 6, which pairs with 11. Item 16 can happen at any time, but see its note on release side effects.

## Context

### Decisions already made (background, not open items)

| Decision | Detail |
|:---|:---|
| Playbook keeps the "substance" practice | `managing-roadmaps`, `writing-reports`, `writing-history`, `writing-release` |
| **Plumage** (new, public, AGPL-3.0) | form and communication skills: `writing-prose`, `writing-design-canons`, `importing-design-figures`, `coining-neologisms`, `pdf-to-md` |
| **Pinion** (new, public, AGPL-3.0) | agent mechanics: `spawning-agent-plugins`, `waiting-on-processes` |
| Seeding | Skill directories only, plus README and LICENSE. Playbook skills were extracted with their git history and release tags; explore skills were plain-copied from `~/Documents/explore/skills`. The migrated skills' `.releaserc.js` files were dropped because they point at the playbook's `../../release-config.js`. |
| `umbod` / `umbod-setup` | get their own repo (maintainer's plan) |
| Private skills | `scheduling-work`, `japanese-tutor`, `logging-agentic-usage`, `serving-qwen-tts` stay private |
| `__roadmap__/` campaigns | not pruned; they are spec sources for blame |
| Done this session | Deleted 12 superseded instruction files (the `reporting*`, `roadmap*`, `git-workflow`, `git-workflow-milestone` families) and `testing.instructions.md` (Wobble abandoned). Fixed the `writing-reports` test-definition save path. Rewrote `README.md`. Added "Building a skill locally" to `CONTRIBUTING.md`. Removed the playbook git submodule from Hatchling, Hatch-Validator and py-repo-template. Deleted the stale `~/.claude/skills/writing-design-canons` symlink. |

Verified at write time: Plumage and Pinion exist locally at `~/Documents/src/CrackingShells/<Repo>` with their skill directories, README and LICENSE, and both are public on GitHub (`CrackingShells/Plumage`, `CrackingShells/Pinion`) with `main` and the carried release tags pushed. The playbook GitHub description is still "Engineering instructions for all repositories on Cracking Shells."

## Backlog

Paths are relative to the playbook worktree unless prefixed with `~` or an absolute path. Line numbers were checked at write time (2026-10-06) against branch `claude/repo-organization-review-c1adbd`.

### A. Playbook: instruction files still in `instructions/` (items 1 to 5)

Remaining files, with sizes: `analytic-behavior` (62 lines), `code-change-phases` (127), `documentation*` (8 files, 2,328 lines total, including the 119-line `documentation.instructions.md` index), `git-workflow-debugging` (81), `python_docstrings` (88), `work-ethics` (47). Total 2,733 lines.

#### 1. `code-change-phases` becomes the skill `leading-campaigns` (UNRULED)

- **What.** Convert `instructions/code-change-phases.instructions.md` into a playbook skill (working name `leading-campaigns`), generalised from colgrep-mcp's maintainer skill `campaign-lead`. Then delete the instruction file.
- **Why.** The instruction states a 3-stage workflow (Analysis, Roadmap, Execution; headings at lines 12, 50, 76). The `campaign-lead` skill already runs the lived version: architecture report, roadmap, hand-made worktrees, dispatch, merge per level, read-only reviewer, knowledge-transfer report. The instruction is the stale, unexecutable ancestor. Its closing section "What This Replaces" (line 115) is itself a sign that it was meant to be absorbed.
- **Where.** Source: `instructions/code-change-phases.instructions.md`. Model: `/Users/hacker/Documents/src/CrackingShells/colgrep-mcp/dev/skills/campaign-lead/SKILL.md` (103 lines) and its `references/` (`dirtree-gotchas.md`, `dispatch-prompt.md`, `reviewer-brief.md`). The "Rules Specific to This Repository" section (line 53) is colgrep-mcp-specific and must be stripped or parameterised; "What This Skill Does Not Restate" (line 91) shows which neighbours it composes with.
- **Dependencies.** It would compose `writing-reports`, `managing-roadmaps` and `writing-history`, all in the playbook, so no cross-repo dependency. If the ruling is yes, decide whether colgrep-mcp's `campaign-lead` then becomes a thin specialisation of the generic one or stays as is.
- **Done when.** New skill under `skills/leading-campaigns/` (with `package.json`, entry in `plugin.spec.json`, README table row, CONTRIBUTING count updated), then the instruction file deleted.

#### 2. Fold the `debugging/<issue>` branch pattern into `branches.md` (UNRULED)

- **What.** Add the `debugging/<issue>` pattern to `skills/writing-history/references/branches.md` as an optional pattern, then delete `instructions/git-workflow-debugging.instructions.md`.
- **Why.** The file is a 81-line standalone with no frontmatter. It carries one idea worth keeping: commit every hypothesis on a throwaway branch, then land either one clean commit on the original branch (its "Option A", recommended for simple fixes) or the squashed branch. `branches.md` is where branch discipline lives now.
- **Where.** Source `instructions/git-workflow-debugging.instructions.md`. Target `skills/writing-history/references/branches.md`; its sections are Integration Contract (line 15), Rationale (49), Prohibitions (60), Worktrees (68, explicitly conditional) and Parallel Sibling Work (82). Add the pattern as a conditional section modelled on Worktrees, and check it against the Prohibitions: a squash-style "Option A" landing sits awkwardly beside the rebase-then-`--no-ff` contract and needs one explicit sentence of reconciliation. Its `debug:` commit prefix is not in writing-history's derived-vocabulary model, so say that it is throwaway-branch-only.
- **Dependencies.** Editing the skill triggers a `writing-history` release when merged to main. Batch it with other writing-history edits.

#### 3. Deprecate `analytic-behavior` and `work-ethics` (UNRULED)

- **What.** Delete `instructions/analytic-behavior.instructions.md` (62 lines) and `instructions/work-ethics.instructions.md` (47 lines). Optionally distil a few lines into an org-level `AGENTS.md` template.
- **Why.**
  - `analytic-behavior` asks for "code snippets" in reports, which contradicts the `writing-reports` no-dump contract.
  - It has a dead link: line 11 points to `.github/instructions/research-analysis.md`, which does not exist in the repo.
  - Its frontmatter is `type: "always_apply"`, a style no other file uses.
  - `work-ethics` carries `applyTo: '**/*'` generalities (commit discipline, perseverance) that `writing-history` and the harness defaults now cover.
- **Where.** The two files above. A template, if wanted, has no home yet; decide where it lives (playbook root or a `templates/` directory) before writing it.
- **Done when.** Both files deleted; the optional template either written or explicitly declined.

#### 4. Documentation and docstring instructions become `documenting-python-projects` (UNRULED)

- **What.** Merge nine path-scoped instruction files into one org skill (working name `documenting-python-projects`), one reference file per source file. Then delete the sources.
- **Why.** About 2,400 lines of documentation and docstring guidance are unreachable by any harness that does not read `applyTo` globs, and the files mix three frontmatter styles (`applyTo` plus `description`; `applyTo` only; none or `type:`). A skill with a trigger description and one-level-deep references is loadable everywhere.
- **Where.**

  | Source file | Lines | `applyTo` scope |
  |:---|--:|:---|
  | `documentation.instructions.md` (master index) | 119 | `docs/**/*.md` |
  | `documentation-api` | 412 | `docs/articles/api/**/*.md, **/*.py` |
  | `documentation-mkdocs-setup` | 421 | `mkdocs.yml`, `.readthedocs.yaml`, brand CSS, overrides |
  | `documentation-readme` | 415 | `**/README.md` |
  | `documentation-resources` | 426 | `docs/resources/**/*` |
  | `documentation-structure` | 215 | `docs/**/*` |
  | `documentation-style-guide` | 114 | `docs/**/*.md` |
  | `documentation-tutorials` | 206 | `**/docs/**/tutorials/*.md` |
  | `python_docstrings` | 88 | `**/*.py` |

- **Known defect.** `documentation-structure.instructions.md` line 82 links `./tutorials.instructions.md`, which does not exist; it should be `documentation-tutorials.instructions.md`. Fix it in transit, or note that the move makes it moot.
- **Option, not decided.** Express the MkDocs brand theme in `documentation-mkdocs-setup` as a design canon made with `writing-design-canons`, consumed by the docs skill. That skill is now owned by Plumage, so this introduces a playbook-to-Plumage dependency. If chosen, it should wait for item 12 (Plumage release path), or it must tolerate a missing dependency.
- **Dependencies.** Largest item in the backlog. The path scoping the files rely on is lost in the move; the skill description must carry the triggers instead (README work, docs tree work, docstrings, `mkdocs.yml`). Consider whether this belongs in the playbook at all versus Plumage (form) since it is about form of documentation; the maintainer's partition rule decides.

#### 5. Delete `archive/committing-changes/` (UNRULED)

- **What.** Remove `archive/committing-changes/`.
- **Why.** Superseded by `writing-history`; git history keeps it. It holds `CHANGELOG.md`, `SKILL.md`, `package.json` and `references/`.
- **Where.** `archive/committing-changes/`. After deletion the `archive/` directory is empty and goes with it. `__reports__/create-committing-changes-skill/` is history about that skill's creation and stays.
- **Dependencies.** Item 15 (umbod still names `committing-changes`) is the last live reference known; resolve or accept it first so no reader is left chasing a deleted name. The old GitHub Releases for `committing-changes` stay.

### B. Packaging and release plumbing (items 6, 9, 11, 12)

#### 6. Move generic packaging scripts into `spawning-agent-plugins` (UNRULED)

- **What.** Move `tools/assemble_plugin.py`, `tools/package_skill.py` and `tools/set_plugin_version.py` into `skills/spawning-agent-plugins/scripts/` so Plumage, Pinion and the playbook consume one copy instead of three.
- **Why.** These are generic multi-skill-repo plumbing. Duplicating them into two new repos creates drift on day one. `spawning-agent-plugins/scripts/` today holds only `check_plugin.py` and `spawn_plugin.py`.
- **Trap the original session did not list.** `tools/package_skill.py` imports `quick_validate` (line 20), so `tools/quick_validate.py` is a fourth file that must move with them, or `package_skill.py` breaks.
- **Where, consumers to repoint.**
  - `release-config.js`: line 17 (`prepareCmd` running `../../tools/package_skill.py`) and line 20 (`../../tools/set_plugin_version.py`).
  - `.github/workflows/release.yml`: line 74 (`tools/assemble_plugin.py`) and the error message on line 78.
  - `CONTRIBUTING.md`: line 40 (`tools/set_plugin_version.py`) and line 52 (`tools/package_skill.py`).
- **Dependencies.** Since the script's new home is a skill that item 11 moves to Pinion, doing 6 before 11 means moving the scripts twice, and doing it after means Plumage and Pinion cannot share a copy until Pinion is live. Recommended order: 11 first, then 6 into Pinion, with Plumage and the playbook fetching the scripts from Pinion's release. How a consumer fetches them (vendor, submodule, release asset, installed plugin) is itself undecided and is the real question inside this item.

#### 9. Plumage and Pinion plugin and marketplace wiring, versioning and releases (RULED-TODO)

- **What.** In each new repo: generate the plugin manifests with `spawning-agent-plugins` in Nest hub mode, and install the versioning/release framework (semantic-release config, the `.releaserc.js` replacement for the dropped ones, CI, commit conventions via `writing-history`).
- **Why.** The seeds contain only skill directories, README and LICENSE. Without a release path nothing downstream (items 10 to 13) can proceed.
- **Where.** `~/Documents/src/CrackingShells/Plumage` (exists); Pinion (to be created by the coordinator). Use the playbook's `plugin.spec.json` (hub mode, `marketplace.hub` = `https://github.com/CrackingShells/Nest`) and `release-config.js` as the working reference.
- **How to run.** Start a session inside each repo, so its own AGENTS.md, hooks and worktree conventions apply. Do not run it from the playbook worktree.
- **Dependencies.** Item 6 decides whether each repo ships its own copy of the scripts; settle it first or accept a temporary copy.

#### 11. Hand `spawning-agent-plugins` over to Pinion (RULED-TODO)

- **Gate.** Do not start until Pinion's plugin is live and installable from Nest.
- **Steps, in order.**
  1. Repoint Nest's `spawning-agent-plugins` entry to Pinion (`.claude-plugin/marketplace.json`, `.agents/plugins/marketplace.json`, README table; see item 10).
  2. Remove `skills/spawning-agent-plugins/` and `plugins/spawning-agent-plugins/` from the playbook.
  3. Update `plugin.spec.json`: the `spawning-agent-plugins` entry begins at line 57, and the top-level `description` ("Five playbook skills, packaged as five installable plugins ...") becomes four, minus any additions from items 1 and 4.
  4. Update the CI drift job in `.github/workflows/release.yml`: line 68 runs `skills/spawning-agent-plugins/scripts/check_plugin.py`. That script leaves with the skill, so the job must obtain it from Pinion (or the job must be replaced).
  5. Update playbook `README.md` (skills table row at line 18) and `CONTRIBUTING.md` (line 35 lists five skills by name).
  6. Update colgrep-mcp: `AGENTS.md` line 59 requires `colgrep-mcp.spec.json` to stay byte-identical to the playbook's `skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json`. That path moves to Pinion. Also check `CLAUDE.md` in the same repo for the same statement (a search for "playbook" found it only in `AGENTS.md` line 59 at write time; confirm that `CLAUDE.md` merely defers to `AGENTS.md`).
- **Why the colgrep-mcp step matters.** The regeneration guard measures against the example spec; a divergence "leaves the guard green while measuring a spec nobody uses" (AGENTS.md wording). Moving the example without updating that sentence silently breaks the rule.
- **Cost of the transition.** Old GitHub Releases and `.skill` assets for `spawning-agent-plugins` stay on the playbook.

#### 12. Remove the three Plumage-owned skills from the playbook (RULED-TODO)

- **Gate.** Plumage has a working release path (item 9).
- **What.** Delete `skills/writing-prose/`, `skills/writing-design-canons/`, `skills/importing-design-figures/`. Remove their README table rows (`README.md` lines 19, 20, 21).
- **Drift risk until then.** The playbook copy and the Plumage copy coexist. **Edit Plumage only**; any playbook-side edit is lost and may re-trigger a playbook release for a skill about to leave.
- **Check before deleting.** None of the three appear in `plugin.spec.json`, `release-config.js`, `release.yml` or `package.json` at write time (searched), so removal should touch only the directories and the README. Confirm `CONTRIBUTING.md` is clean too.
- **Cost of the transition.** Old GitHub Releases and `.skill` assets for those skills remain on the playbook.

### C. Nest (item 10)

#### 10. Register both repos' plugins in Nest; document how (RULED-TODO)

- **What.** Add Plumage's and Pinion's plugins to `/Users/hacker/Documents/src/CrackingShells/Nest/.claude-plugin/marketplace.json` and `.agents/plugins/marketplace.json`, and add rows to the table under "Plugins" in `README.md` (heading at line 50). Then write a short "how to add a plugin" section, since none exists.
- **Why.** Nest is the hub (`marketplace.hub` in each repo's spec). Currently listed: `managing-roadmaps`, `writing-history`, `writing-release`, `writing-reports`, `spawning-agent-plugins`, `colgrep-mcp`, `colgrep-mcp-dev`. Nest's README states no entry carries a `version`, `ref` or `sha`, so entries need no release-time edit; follow that rule.
- **Dependencies.** Needs the plugins to exist (item 9). Item 11 step 1 reuses the same procedure, so write the "how to add a plugin" section the first time and cite it afterwards.

### D. Local environment (items 8, 13)

#### 8. `~/.claude/skills` housekeeping (UNRULED)

- **What.** (a) `~/.claude/skills/serving-qwen-tts` is a symlink to a worktree path under `~/Documents/tmp/claude-worktrees/ai_agent_academy/tts-hackathon-narration-timing-d59c85/skills/serving-qwen-tts`; the skill is private and stays, so decide whether to repoint it at a durable home or remove the link. (b) Stray `~/.claude/skills/logging-agentic-usage.zip` (19 KB, next to the real `logging-agentic-usage/` directory).
- **Why.** A dangling symlink makes a skill listing error or silently shows nothing; the zip is an unclaimed duplicate of a private skill.
- **Note.** Verify the symlink target is actually gone before calling it dangling: worktrees are recycled, so it may have been valid when checked and since changed. Deleting is permanent for the zip; confirm with the maintainer.

#### 13. Repoint `~/.claude/skills` symlinks once the new repos are the source (RULED-TODO)

| Skill | Symlink target today | Target after |
|:---|:---|:---|
| `writing-prose` | playbook main checkout `skills/writing-prose` | Plumage `skills/writing-prose` |
| `importing-design-figures` | playbook main checkout `skills/importing-design-figures` | Plumage |
| `coining-neologisms` | `~/Documents/explore/skills/coining-neologisms` | Plumage |
| `pdf-to-md` | `~/Documents/explore/skills/pdf-to-md` | Plumage |
| `waiting-on-processes` | `~/Documents/explore/skills/waiting-on-processes` | Pinion |
| `writing-design-canons` | none (symlink was deleted) | Plumage, if it should be loaded locally at all |

- **Gate.** The new repos are the source of truth (items 9 and 12). Repointing earlier risks loading a checkout that is mid-wiring.
- **Consequence.** The `~/Documents/explore/skills` copies of `coining-neologisms`, `pdf-to-md` and `waiting-on-processes` become stale duplicates. Delete or mark them once repointed. Do not delete `scheduling-work` or `japanese-tutor` there; they stay private and are still symlinked from `~/.claude/skills`.
- **Note.** The `umbod` and `umbod-setup` symlinks point into `~/Documents/explore/async_work_style/skills/`; they move with item 15's repo.

### E. Playbook README (item 14)

#### 14. README "Sister repos" and "Using it in a project" (RULED-TODO)

- **What.** The "Sister repositories" section linking Plumage and Pinion was added once both were public. Still open: add a "Using it in a project" section (enabling plugins through project settings) **only after the maintainer has tested that path**. Do not document an untested install flow.
- **Where.** `README.md` (rewritten this session; see commit `45617e0`). The skills table rows move with items 11 and 12, so do those first to avoid rewriting the table twice.

### F. umbod (item 15)

#### 15. `umbod` still names `committing-changes` (RULED-TODO)

- **What.** In umbod's SKILL.md replace `committing-changes` (archived) with `writing-history`.
- **Where.** `/Users/hacker/Documents/explore/async_work_style/skills/umbod/SKILL.md`: line 13 (description frontmatter: "composes with ... `committing-changes` for commit-message discipline") and line 143 ("`committing-changes` governs commit-message discipline ..."). Both verified. `umbod-setup` should be checked for the same name.
- **Gate.** Do it in umbod's future repo, not in `explore/`. Skill descriptions drive triggering, so the edit to line 13 changes behaviour and deserves a trigger check.
- **Note.** The statement that `writing-history` "governs commit-message discipline" is accurate only in part; it also covers integration and release conventions. Phrase it as composition, not as a one-for-one rename.

### G. Push and merge state (item 16)

#### 16. Everything is local only (RULED-TODO)

| Repo | Branch | State | Note |
|:---|:---|:---|:---|
| playbook | `claude/repo-organization-review-c1adbd` | 4 commits ahead of `main` (`fa2b6fa`, `26b40fd`, `55397aa`, `45617e0`); a 5th when this report is committed | Merging to `main` triggers a `writing-reports@1.1.3` release (the `fix(writing-reports)` commit). Merge deliberately: it publishes a release. |
| Hatchling | `dev` | merge `ec693ea` ("Merge chore/remove-playbook-submodule into dev") | `dev` was already 117 commits ahead of `origin/dev` before this merge, so pushing publishes far more than the submodule removal. |
| Hatch-Validator | `dev` | merge `3872faf` | |
| py-repo-template | `main` | merge `b0e993a` | |
| Plumage | `main` | pushed to `origin/main` with 4 tags | Not local-only: listed for completeness. |
| Pinion | `main` | pushed to `origin/main` with 2 tags | Not local-only: listed for completeness. |

- **Order.** No ordering constraint among the four. For the playbook, merge using the `writing-history` contract (rebase onto `main`, then `--no-ff`).
- **Do not** force-push any of these.

### H. Watch items (17 to 19)

| # | Watch | Why nothing to do now | Trigger to act |
|--:|:---|:---|:---|
| 17 | The `managing-roadmaps` plugin ships `dirtree-rdm` binaries for darwin only (`__reports__/dirtree_rdm_plugin_binaries/`, observation v0, status "open and deferred") | Release-pipeline change with its own open report | Anyone installing the plugin on Linux or Windows, or a managing-roadmaps release that touches the pipeline |
| 18 | `py-repo-template` still references Wobble: confirmed in `pyproject.toml`, `TEMPLATE_USAGE.md`, `CONTRIBUTING.md`, and others (about 7 files in total per the originating session) | A separate session already handles it | That session finishing; then check nothing is left after the submodule removal |
| 19 | Hook hygiene: Hatch-Validator has a pre-commit hook installed but no `.pre-commit-config.yaml`; Hatchling's commitlint hook runs `npx commitlint@21.2.3`, which cannot install offline although `node_modules` already holds commitlint; Hatchling git config keeps a harmless `submodule.active .` | Not blocking; offline commits can fail until the hook is bypassed or pointed at the local binary | A failed commit in either repo, or the next housekeeping pass |

## Evidence

- Playbook worktree contents at write time: `instructions/` holds 13 files (list in section A); `archive/committing-changes/`; `tools/` holds four scripts (`assemble_plugin.py`, `package_skill.py`, `quick_validate.py`, `set_plugin_version.py`); `skills/` holds eight skills; `plugins/` holds five.
- Broken link confirmed: `instructions/documentation-structure.instructions.md` line 82 points to `./tutorials.instructions.md`; the real file is `documentation-tutorials.instructions.md`.
- Dead link confirmed: `instructions/analytic-behavior.instructions.md` line 11 points to `.github/instructions/research-analysis.md`.
- Frontmatter styles confirmed: `type: "always_apply"` (analytic-behavior), `applyTo` plus `description` (most), none (git-workflow-debugging).
- `release.yml` line 68 runs `skills/spawning-agent-plugins/scripts/check_plugin.py`; `release-config.js` lines 17 and 20 and `release.yml` line 74 call the `tools/` scripts.
- `colgrep-mcp/AGENTS.md` line 59 carries the byte-identity requirement for `colgrep-mcp.spec.json`.
- umbod SKILL.md lines 13 and 143 name `committing-changes`.
- Playbook GitHub description read live: "Engineering instructions for all repositories on Cracking Shells."
- Not verified: Nest line numbers beyond the README heading at line 50; the exact Wobble file count in py-repo-template; whether `CLAUDE.md` in colgrep-mcp restates the byte-identity rule.

## Hand-off Questions

Working theory, if any: the partition rule is "substance stays in the playbook; form goes to Plumage; agent mechanics go to Pinion". Items 1, 2, 4 and 6 are only UNRULED because the maintainer has not confirmed where each lands under that rule.

- Item 7: is the new description a one-liner on "substance practice for the org" (roadmaps, reports, history, release), and should it name the sister repos?
- Item 4: does documentation-form guidance belong in the playbook (substance) or in Plumage (form)? Does the brand-theme canon option create a playbook-to-Plumage dependency the maintainer accepts?
- Item 6: how does a consuming repo fetch the shared scripts (vendor, release asset, installed plugin)?
- Item 1: should colgrep-mcp's `campaign-lead` become a thin specialisation of the generic skill?
- Item 16: merge the playbook branch before or after items 11 and 12? Merging first publishes `writing-reports@1.1.3` now; merging last bundles everything into one history.
- Item 14: what is the tested path for enabling plugins through project settings, and who tests it?

## Scope Boundary

This report authorises no deletion, move, push, merge, release or edit in any repository: UNRULED items need the maintainer's ruling first, RULED-TODO items run only in their stated sessions and order, and WATCH items need no action.
