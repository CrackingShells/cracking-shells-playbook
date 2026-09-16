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

## Validated only, not exercised

**Codex.** No Codex CLI is available on this machine, so every Codex claim rests on its published
manifest format and its loader source, never on a live run. The Codex catalogue in Nest, the
`extensions["com.openai"]` manifests and the `ON_INSTALL` policy values are written to that reference
and statically checked. This is the same distinction `references/manifests.md` keeps in its
verification-status table, and it must not blur now that the catalogue has moved.

**Agent Plugins 1.0 clients.** Conformance is asserted against the spec's closed schema, with no
Cursor, VS Code or Kiro install performed.

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
