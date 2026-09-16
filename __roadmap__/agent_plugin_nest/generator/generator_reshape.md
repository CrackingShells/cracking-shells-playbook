# Generator Reshape

**Goal**: Teach `spawn_plugin.py` to emit several sibling plugins per repository, target a hub-owned marketplace, and express Codex through `extensions["com.openai"]`; teach `check_plugin.py` to validate that shape.
**Pre-conditions**:
- [ ] `regeneration_guard` is green on the unmodified generator
- [ ] `plugin_assembly_tool` is done — the per-plugin loop calls it
- [ ] `skills/spawning-agent-plugins/` is committed, so a worktree can check it out
**Success Gates**:
- ⬜ A spec declaring two plugins writes two complete manifest sets under their own directories [run]
- ⬜ Each plugin resolves its own version from its own `version_from` [run]
- ⬜ No `.codex-plugin/` directory is written by any code path [run]
- ⬜ `check_plugin.py` passes on a repo with two sibling plugins at different versions [run]
- ⬜ The regeneration guard passes against its re-baselined colgrep-mcp expectation [run]
- ⬜ No call site passes `force=True` to `Writer.merge_marketplace` [static]
- ⬜ The generator emits no `authentication: "NONE"`, and the checker rejects it [run]
**References**: [R02 §Codex](../../../skills/spawning-agent-plugins/references/manifests.md) — the fields each ecosystem permits

## Step 1: Accept several plugins in one spec

**Goal**: Let a spec describe N plugins without breaking the single-plugin specs that exist.

**Implementation Logic**:
Add an optional top-level `plugins` array to the spec. Each entry carries `name`, `dir`, `description`, its own optional `version`/`version_from`, and the per-ecosystem blocks a top-level spec already accepts, inheriting `author`, `homepage`, `repository` and `license` from the top level via `_identity`. When `plugins` is absent, behaviour is byte-for-byte what it is today — this is what keeps `colgrep-mcp.spec.json` regenerating unchanged, and the regeneration guard is the proof. Extend `load_spec`'s validation to the new shape: each entry needs a kebab-case `name` and a `dir`, and no two entries may share either.
**Deliverables**: `skills/spawning-agent-plugins/scripts/spawn_plugin.py` — `load_spec` extended, new helper `plugin_entries(spec)` returning a list of normalised per-plugin dicts (a one-element list for a single-plugin spec)
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py --root /Users/hacker/Documents/src/CrackingShells/colgrep-mcp spawn --spec skills/spawning-agent-plugins/assets/examples/colgrep-mcp.spec.json --dry-run 2>&1 | grep -c "^kept" | grep -qx 1` (expected: PASS)
**Commit**: `feat(spawning-agent-plugins): accept several plugins in one spec`

## Step 2: Write one manifest set per plugin

**Goal**: Turn `spawn()`'s fixed path table into a per-plugin loop.

**Implementation Logic**:
Wrap `spawn()`'s ecosystem branches in a loop over `plugin_entries(spec)`, prefixing every written path with the entry's `dir` (empty for a root plugin, so single-plugin output is unchanged). Call `resolve_version` once per entry rather than once per `spawn()`, so siblings hold independent version lines — `resolve_version` already accepts an arbitrary `kind:relpath`, so only the call site moves. Accumulate all entries into a single `merge_marketplace` call per ecosystem rather than one call per plugin.

`Writer` itself must not change. `merge_marketplace` appends only the entries a marketplace lacks and never rewrites existing ones, but under `force=True` it replaces the whole `plugins` array and silently discards siblings this spawn does not know about. No call path added here may pass `force` to it.
**Deliverables**: `skills/spawning-agent-plugins/scripts/spawn_plugin.py` — `spawn()` looping over entries with a `dir` prefix; `build_claude_marketplace`/`build_codex_marketplace` taking the entry list and emitting one entry each. `skills/spawning-agent-plugins/evals/fixtures/two_plugins.spec.json` — a minimal two-plugin spec (`alpha`, `beta`) used as the fixture for this step's check and by later regression runs
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/spawn_plugin.py --root "$(mktemp -d)" spawn --spec skills/spawning-agent-plugins/evals/fixtures/two_plugins.spec.json --dry-run 2>&1 | grep -c "plugin.json" | grep -qx 4` (expected: PASS)
**Commit**: `feat(spawning-agent-plugins): write one manifest set per declared plugin`

## Step 3: Express Codex through the Agent Plugins manifest

**Goal**: Collapse three manifest sets into two by using the interop path Codex actually implements.

**Implementation Logic**:
Codex parses a root Agent-Plugins-conformant `plugin.json` whose `$schema` names the agent-plugins.org schema, reads its Codex-specific data from `extensions["com.openai"]`, and auto-wires `skills` to `./skills` and `mcp_servers` to `./mcp.json`. Move the `interface` block and any Codex `hooks` value into that namespace, and stop writing `.codex-plugin/plugin.json` and `.codex-plugin/mcp.json` entirely. `extensions` is already inside `AGENT_PLUGIN_FIELDS`, so the closed Agent Plugins schema still validates.

This deliberately changes colgrep-mcp's generated output, so the regeneration guard's baseline must be re-recorded **in this commit**, with the diff shown in the commit body. Re-baselining silently is the one failure mode that makes the guard worthless.
**Deliverables**: `skills/spawning-agent-plugins/scripts/spawn_plugin.py` — `build_agent_plugin` emitting `extensions["com.openai"]`, `build_codex_plugin` removed or folded into it, `.codex-plugin/*` dropped from the path table; `skills/spawning-agent-plugins/evals/test_regeneration.py` re-baselined
**Consistency Checks**: `grep -rq "codex-plugin" skills/spawning-agent-plugins/scripts/spawn_plugin.py; test $? -ne 0` (expected: PASS)
**Commit**: `feat(spawning-agent-plugins): express Codex through the com.openai extensions namespace`

## Step 4: Point marketplaces at a hub

**Goal**: Let a repo contribute entries to a marketplace owned elsewhere instead of shipping its own.

**Implementation Logic**:
Add a spec key — `marketplace: "hub"` or an explicit hub repository — that suppresses writing `.claude-plugin/marketplace.json` and `.agents/plugins/marketplace.json` into the product repo, since two repos declaring one marketplace name silently replace each other in Claude Code and hard-error in Codex. When a marketplace *is* written, parameterise the `source` instead of hardcoding `"./"`: a plugin in a subdirectory uses `git-subdir` with its `path` for Claude Code, and Codex takes the same shape via `url` + `path` because it has no `github` shorthand. Emit no `version`, `ref` or `sha` on any generated entry.

Fix a shipped bug while here: `spawn_plugin.py:273` hardcodes `"authentication": "NONE"` in the Codex marketplace entry. That value is not in Codex's auth-policy enum (`ON_INSTALL` | `ON_USE`, no catch-all), and Codex parses the marketplace in one pass, so it makes every entry in the file unparseable. Every plugin this generator has produced carries it, colgrep-mcp's live marketplace included. Emit `ON_INSTALL`.
**Deliverables**: `skills/spawning-agent-plugins/scripts/spawn_plugin.py` — `build_claude_marketplace`/`build_codex_marketplace` emitting parameterised sources and a valid `authentication` value, a hub mode suppressing both files
**Consistency Checks**: `uv run python3 -c "src=open('skills/spawning-agent-plugins/scripts/spawn_plugin.py').read();assert 'git-subdir' in src"` (expected: PASS)
**Commit**: `feat(spawning-agent-plugins): support marketplaces owned by a hub repository`

## Step 5: Validate the new shape

**Goal**: Make the checker agree with what the generator now writes, instead of failing every sibling plugin.

**Implementation Logic**:
Scope `collect_problems` to one plugin root at a time and iterate roots, so the cross-manifest name/version identity check compares a plugin against itself rather than against its siblings. Three existing rules currently misfire and must change together: the entry-source assertion that demands `"./"`, the dev-plugin rule that reports any non-root marketplace entry as "lagging the product version", and the Codex `interface` checks, which must now read `extensions["com.openai"]`. Keep the dev-plugin version rule alive but scope it to entries a spec actually declares as `dev` — a maintainer plugin should still track its product's version; a sibling product plugin should not.

Add one rule the checker lacks: it currently asserts only that `policy` and `category` are *present*, never their values, which is why the invalid `authentication: "NONE"` survived every check. Validate `policy.installation` against `NOT_AVAILABLE | AVAILABLE | INSTALLED_BY_DEFAULT` and `policy.authentication` against `ON_INSTALL | ON_USE`.
**Deliverables**: `skills/spawning-agent-plugins/scripts/check_plugin.py` — `collect_problems(root)` iterating plugin roots, `_check_codex_interface(extensions)` replacing the `.codex-plugin` reads, the `"./"` source assertion and dev-lag rule scoped
**Consistency Checks**: `uv run skills/spawning-agent-plugins/scripts/check_plugin.py --root /Users/hacker/Documents/src/CrackingShells/colgrep-mcp` (expected: PASS)
**Commit**: `feat(spawning-agent-plugins): validate sibling plugins and the extensions shape`
