# Findings: Nest install verification

## Exercised, end to end, in Claude Code

| Step | Result |
|:--|:--|
| Playbook `main` pushed | `4362960..ec5b723`, `plugins/` present on the remote default branch with all five roots |
| Nest published | new branch `main`, 7 catalogue entries fetchable |
| Old registration removed first | `cracking-shells` dropped while still pointing at `CrackingShells/colgrep-mcp` |
| Nest added | clone validated; `known_marketplaces.json` now records `repo: CrackingShells/Nest` — the **source** was checked, not just the name |
| Five plugins installed | all at `1.0.0`, scope user |
| Load state | `claude plugin list` shows all five, **0** `failed to load` |

The removal-before-add order mattered: adding over an existing registration replaces it silently, so
verifying the plugin list without verifying the recorded `source` would have measured whichever
catalogue was added last.

## Two fixes validated in production, not just in fixtures

**The shared-factory defect.** `writing-prose@1.1.0` released in the same run. It is one of the three
skills with no plugin root, so `set_plugin_version.py`'s skip path ran for real — the path that,
before it was fixed, raised and aborted the whole `prepare` step. Had that defect shipped, this push
would have failed the release for `writing-prose` rather than publishing it.

**The CI gate's skipped-job case.** `build-managing-roadmaps-binaries` was skipped by its own `if`,
and `release` still ran, confirming `always() && !contains(needs.*.result, 'failure') && !cancelled()`
treats a skipped ancestor as passing while still blocking on a failed one.

The `drift` job also ran green, which is its first real execution against a populated `plugins/`.

## The original question, answered

`spawning-agent-plugins@1.0.0` is tagged and released. Before this campaign the skill was untracked
and had no `package.json`, so `multi-semantic-release` could not see it: pushing it published nothing,
silently, with a green pipeline.

## All three ecosystems exercised — reported by the maintainer

Superseding this report's original "validated only" section: the maintainer installed the plugins
from **Claude Code, Codex and VS Code**, all pointing at Nest. Reported by him rather than measured
here, so attributed as such — but it retires the largest caveat this campaign carried, and confirms
several decisions that had rested on documentation alone:

- **The `extensions["com.openai"]` shape works in a real Codex.** It was written to the
  `openai/codex` reference and never exercised; the whole `.codex-plugin/` removal rested on that
  reading.
- **The `authentication` fix was load-bearing, not cosmetic.** Codex deserialises a marketplace file
  in one pass, so the `"NONE"` value that the generator hardcoded would have made *every* entry in
  Nest's Codex catalogue unparseable. A successful Codex install is only possible because that
  defect was caught. Its enum (`ON_INSTALL` | `ON_USE`) had been established by reading Rust source,
  with no live run to confirm it.
- **Codex's `git-subdir` / `url` source forms resolve**, including the absence of a `github`
  shorthand, which is why the Codex catalogue differs in shape from the Claude one.
- **Agent Plugins 1.0 conformance holds in a real client.** VS Code discovered the package by
  location — root `plugin.json` and `skills/` — which is what the assembled `plugins/<name>/` layout
  exists to satisfy, given the spec's containment rule and its symlink prohibition.

What remains genuinely untested: Cursor and Kiro, and Codex's `hooks` field, which the generator
emits only on an explicit opt-in that this repo's plugins do not set.

`references/manifests.md`'s verification-status table should be updated from this, since it still
records the Codex rows as written-to-spec rather than exercised.

## Outstanding

- Four `~/.claude/skills/` symlinks (`managing-roadmaps`, `writing-history`, `writing-release`,
  `writing-reports`) now duplicate installed plugins. The symlinks track the working tree live; the
  plugin copies are commit snapshots, so the same skill name can resolve to two different versions.
- Existing clients that added `cracking-shells` from colgrep-mcp keep that catalogue until they remove
  and re-add it. Nest's README carries the migration note.
- `ALLOWED_DIVERGENCE["plugin.json"]` stays until colgrep-mcp's `regenerate_manifests` reaches its
  default branch.

## Gate status

**Open.** `nest_migration`'s `relinquish_marketplace` and `verify/install_check` in colgrep-mcp were
blocked on this leaf. Nest demonstrably installs playbook plugins, and it already lists both
`colgrep-mcp` and `colgrep-mcp-dev` with valid `ON_INSTALL` policies, so nothing is dropped when that
repo relinquishes.
