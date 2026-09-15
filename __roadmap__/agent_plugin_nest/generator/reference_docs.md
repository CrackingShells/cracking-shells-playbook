# Reference Documentation

**Goal**: Document the multi-plugin layout, the hub marketplace, the Codex extensions shape and the cross-format equivalence limits, so the next reader does not re-derive any of it.
**Pre-conditions**:
- [ ] The generator's final shape is settled (this leaf may be written in parallel with `generator_reshape`, but must be reconciled against what actually shipped before it merges)
**Success Gates**:
- ⬜ `references/manifests.md` carries an equivalence matrix naming every field that cannot be held equivalent across the three formats [static]
- ⬜ `references/versioning.md` describes sibling plugins with independent version lines [static]
- ⬜ `references/traps.md` carries a trap for the same-name marketplace collision, with its symptom, cause and fix [static]
- ⬜ Every anchor referenced from `SKILL.md` or another reference resolves to a heading that exists [run]
- ⬜ No document still claims `.codex-plugin/plugin.json` is written [run]
**References**: [R01 §Where equivalence is impossible](~/.claude/plans/good-news-overall-it-s-gleaming-wreath.md) — the irreducible divergences to document

## Step 1: Document the layout and the equivalence limits

**Goal**: Say what the manifests now are, and be honest about where the three ecosystems cannot agree.

**Implementation Logic**:
In `references/manifests.md`, add a section after `## Codex` covering multi-plugin repositories: a plugin root is a directory containing its own `skills/`, Agent Plugins 1.0 requires containment and forbids symlink escapes, so a plugin root is assembled rather than aliased. Rewrite the Codex section around `extensions["com.openai"]` and delete the `.codex-plugin/plugin.json` material. Add a cross-format equivalence matrix listing, per field, whether it is expressible in all three formats: `hooks` has no Agent Plugins concept at all; `displayName` and every `interface.*` presentation field has no root slot in a closed schema; `commands`/`agents`/`lspServers` are Claude-only and `apps` Codex-only; `author`/`homepage`/`repository`/`license` are absent from Codex's general manifest struct; and `.mcp.json` versus a dot-less root `mcp.json` is a silent naming trap.
**Deliverables**: `skills/spawning-agent-plugins/references/manifests.md` — new sections `## Multi-plugin repositories` and `## Cross-format equivalence`, `## Codex` rewritten around the extensions namespace
**Consistency Checks**: `grep -q "Cross-format equivalence" skills/spawning-agent-plugins/references/manifests.md` (expected: PASS)
**Commit**: `docs(spawning-agent-plugins): document multi-plugin layout and equivalence limits`

## Step 2: Document versioning and the new traps

**Goal**: Record the rules whose violation fails silently.

**Implementation Logic**:
In `references/versioning.md`, add a section on sibling plugins holding independent version lines, and state the rule that a plugin's own `plugin.json` version is authoritative and must bump on every release — Claude Code resolves it first, and Codex's reinstall gate compares version strings, so a static version means Codex silently never reinstalls. Note that setting `version` in both the manifest and a marketplace entry is a trap, because the manifest wins without warning.

In `references/traps.md`, add a trap anchored `{#marketplace-name-collision}` after `{#namespaces}`: symptom — a catalogue is missing plugins a user expects, with no error; cause — two repositories declare the same marketplace `name`, and Claude Code replaces the first while Codex refuses the second; fix — one hub repository owns the name, and existing clients must `marketplace remove` before re-adding. Add a second trap for the assembled plugin tree drifting from its source skill.
**Deliverables**: `skills/spawning-agent-plugins/references/versioning.md` — section `## Sibling plugins, independent versions`; `skills/spawning-agent-plugins/references/traps.md` — traps `{#marketplace-name-collision}` and `{#assembled-tree-drift}`
**Consistency Checks**: `grep -q "marketplace-name-collision" skills/spawning-agent-plugins/references/traps.md` (expected: PASS)
**Commit**: `docs(spawning-agent-plugins): add versioning and marketplace-collision traps`

## Step 3: Reconcile SKILL.md and check every anchor

**Goal**: Leave no document describing a generator that no longer exists.

**Implementation Logic**:
Update `SKILL.md`'s workflow and its `## References` table for the new spec keys (`plugins[]`, the hub marketplace mode), the two-manifest-set output, and the multi-plugin path. Update the layout diagram at the top, which still shows `.codex-plugin/`. Then sweep all five documents for stale claims and broken anchors: every `{#anchor}` referenced must exist as a heading, and no file may still assert that `.codex-plugin/plugin.json` is generated. Write the sweep as a runnable command, not a reading pass — an anchor check done by eye is the kind that silently rots.
**Deliverables**: `skills/spawning-agent-plugins/SKILL.md` — layout block, workflow steps and References table reconciled
**Consistency Checks**: `test $(grep -rl "codex-plugin/plugin.json" skills/spawning-agent-plugins/ | wc -l) -eq 0` (expected: PASS)
**Commit**: `docs(spawning-agent-plugins): reconcile SKILL.md with the reshaped generator`
