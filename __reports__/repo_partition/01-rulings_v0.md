# Repo Partition — Rulings on Items 1 to 8 (v0)

Date: 2026-10-18

The maintainer's rulings on the eight UNRULED proposals in [00-open_items_handoff_v0.md](00-open_items_handoff_v0.md). These replace "UNRULED" for items 1 to 8; the item numbers and their details are the hand-off's.

| # | Item | Ruling | State |
|--:|:---|:---|:---|
| 1 | `code-change-phases` becomes skill `leading-campaigns` | **Deferred** to its own task. The instruction file stays until then. | open, own reminder |
| 2 | fold `debugging/<issue>` into `branches.md` | **Approved** as written | done on `chore/partition-rulings` |
| 3 | delete `analytic-behavior` and `work-ethics` | **Approved**, no org AGENTS.md template | done on `chore/partition-rulings` |
| 4 | documentation + docstring instructions become one skill | **Deferred** to its own task, including where it lives (playbook or Plumage) | open, own reminder |
| 5 | delete `archive/committing-changes/` | **Approved** | done on `chore/partition-rulings` |
| 6 | move the packaging scripts into `spawning-agent-plugins` | **Rejected: copy, don't move.** Plumage and Pinion each get their own copy of `tools/` (`assemble_plugin.py`, `package_skill.py`, `quick_validate.py`, `set_plugin_version.py`) when their versioning is set up. This matches `spawning-agent-plugins/references/versioning.md`, where each repo owns its `set_plugin_version.py`. The playbook keeps its `tools/`; item 11 no longer moves them. | applied in RP-05 / RP-07 |
| 7 | new playbook GitHub description | **Approved:** "Cracking Shells engineering practice as agent skills: roadmaps, reports, commit history, releases. Sister repos: Plumage (form), Pinion (agent mechanics)." | done (live on GitHub) |
| 8 | `~/.claude/skills` dangling symlink and stray zip | Symlink `serving-qwen-tts` was already gone. **Zip moved to the Trash.** | done |

## Consequences for the RULED-TODO items

- Item 9 (RP-05, RP-07): the versioning setup copies the playbook's `tools/` and `release-config.js` pattern into each repo; nothing is fetched cross-repo.
- Item 11 (RP-09): its last step ("if ruling 6 = yes, move `tools/`") falls away.
- Item 16 (RP-02): the maintainer holds the Hatchling and Hatch-Validator pushes for now (2026-10-18). py-repo-template was already pushed.
