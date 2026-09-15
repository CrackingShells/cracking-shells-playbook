# Traps: symptom, cause, what to do

Each of these cost at least one cycle on colgrep-mcp. Recognise the symptom;
do not re-derive the cause.

## A freshly opened repo shows a pending server entry or `spawn ENOENT` {#root-mcp-json}

**Symptom.** Opening the repo as a plain project in Claude Code shows a
second, pending MCP entry named like the plugin's server, or an error such as
`posix_spawn '${CLAUDE_PLUGIN_ROOT}/...' ENOENT`, although the plugin
connects when loaded as a plugin.

**Cause.** A root-level `.mcp.json` is Claude Code's *project-scope* MCP
config, a different loader with no notion of `CLAUDE_PLUGIN_ROOT`; the
placeholder is spawned literally as a path.

**Do.** Keep the Claude MCP manifest at `.claude-plugin/mcp.json` and point
the manifest's `mcpServers` there. Never add a root `.mcp.json`; the generator
warns and the checker fails when one exists.

## The plugin connects but runs the published release, not your tree {#uvx-pin}

**Symptom.** `claude --plugin-dir . mcp list` says `✔ Connected` while an edit
you just made is absent; or right after a bump the plugin fails to start with
a resolution error naming the new version; or minutes after a publish went
green the plugin shows `CONNECTION_CLOSED` although
`uvx --no-cache pkg==<version> --version` works.

**Cause.** Every MCP manifest launches the pinned published artifact; the
loader never runs the checkout. Between bump and upload the pin names a
version the registry lacks, by design (`versioning.md`). The third symptom
is `uv`'s HTTP cache holding the registry's index page past the upload.

**Do.** To exercise the tree, register it directly:
`claude mcp add <server>-dev -- uv run --quiet --directory ./server <entry>`.
After a release, `uvx --refresh pkg==<version> --version` once, then re-run
the gate. Never edit the pin by hand and never re-tag.

## The install string reads `x@x`, or the Codex command does not mirror Claude's {#namespaces}

**Symptom.** `claude plugin install colgrep-mcp@colgrep-mcp` looks like a
typo; the Codex equivalent has a different right-hand side; a marketplace
added under an old name keeps failing after the manifest was renamed.

**Cause.** Marketplace and plugin names are separate namespaces:
`<plugin>@<marketplace>`. A marketplace named after one plugin reads oddly
and cannot grow; colgrep-mcp started that way (Claude `colgrep-mcp`, Codex
`colgrep-mcp-marketplace`) and later renamed both to the organization,
`cracking-shells`. A client that added a marketplace keeps the name it had at
add time.

**Do.** Name the marketplace after the organization, identically in both
files; `init` derives it from the GitHub owner and `install-snippet` prints
the matching pair. After a rename, tell users to
`claude plugin marketplace remove <old>` and add again. A `source` of `"./"`
resolves against a clone of the marketplace, so it works for
`marketplace add <owner/repo>` and for a local path, and not for a direct
URL to the `marketplace.json` file itself.

## `claude plugin list` says "failed to load: Duplicate hooks file detected" {#hooks-manifest-duplicate}

**Symptom.** After `claude plugin install` or `update` the plugin shows
`✘ failed to load` with `Hook load failed: Duplicate hooks file detected:
./hooks/hooks.json resolves to already-loaded file .../hooks/hooks.json`,
and nothing of it (hooks, server, skills) is available; yet
`claude --plugin-dir <tree> plugin details <name>` lists every hook and
`claude plugin validate` passes.

**Cause.** Claude Code loads `hooks/hooks.json` automatically and reads the
manifest's `hooks` field as *additional* files; a manifest naming
`./hooks/hooks.json` names the default twice and the marketplace loader
refuses the plugin (Claude Code 2.1.270; colgrep-mcp's 0.4.0 and 0.5.0
installs). No tree-level command runs that check. Codex reads the field the
other way round: it discovers `hooks/hooks.json` only when the manifest
defines no `hooks`, and an explicit value replaces that discovery.

**Do.** Claude manifest: exactly the per-event files, never
`./hooks/hooks.json`, no field when every event is portable. Codex manifest:
`./hooks/hooks.json` or no field. The generator writes both; `check_plugin.py`
fails on either mistake. To check the way an install does, register a
scratch directory marketplace whose plugin `source` is a copy of the tree,
install from it, read `claude plugin list`, then remove both
(`hooks.md#one-file-per-event-class`).

## Hooks do not fire, or run the old text {#plugin-hooks}

**Symptom.** You edited `hooks/hooks.json` or the script and the old
behaviour persists; `/hooks` lists nothing under plugin hooks; in Codex the
hooks are listed but never run; in Cursor nothing happens at all.

**Cause.** Plugin hooks are read at plugin load. A marketplace install runs
the cached copy under `~/.claude/plugins/cache/`, not your tree. Codex skips
plugin-bundled hooks until you trust them in `/hooks`, and asks again when
the definition's hash changes. Cursor imports Claude Code hooks from
`settings.json` files only, never from a plugin.

**Do.** `/reload-plugins` or a new session in Claude Code; load the tree with
`claude --plugin-dir .` to run the tree's hooks (they are scripts, so unlike
the MCP pin they really do run from the tree). Trust in Codex's `/hooks`. For
Cursor, copy the portable entries into the project's `.claude/settings.json`.
Test the script by piping JSON (`hooks.md#testing-without-a-harness`).

## Codex and the `hooks` field {#codex-hooks}

**Symptom.** Uncertainty whether `"hooks"` belongs in
`.codex-plugin/plugin.json`.

**Cause.** Codex's own `plugin-json-spec.md` lists `hooks` (a path) among the
top-level fields and, under validation notes, says the validator "rejects
unsupported manifest fields such as `hooks`". Codex was never run against
colgrep-mcp, which ships the field.

**Do.** Opt in with `hooks.codex: true` when the user wants Codex hooks and
can test them; leave it off otherwise. Either way the Codex manifest names
only `hooks/hooks.json`, never a per-event file: Codex's plugin docs ("Build
a plugin") say an explicit `hooks` replaces the default discovery of
`hooks/hooks.json`, so naming the per-event file would drop the portable
events and hand Codex an event it may not know.

## `claude plugin validate` fails on the Codex manifest {#validate-codex}

**Symptom.** `interface: Unknown field 'interface'` and `✘ Validation failed`.

**Cause.** You pointed Claude Code's validator at `.codex-plugin/plugin.json`.
It validates Claude manifests, marketplaces and skill directories.

**Do.** Validate `<repo>` (its marketplace), `<repo>/.claude-plugin/plugin.json`
and `<repo>/dev`; run `check_plugin.py` for the cross-ecosystem invariants.
Add `--strict` in CI so unknown fields in the Claude manifest fail.

## An Agent Plugins 1.0 client ignores the skills or the server {#agent-plugins-fields}

**Symptom.** A Cursor or VS Code install shows the plugin but no skills or no
server, or a validator rejects `plugin.json`.

**Cause.** The spec discovers `skills/` and `mcp.json` by *location*, not by
manifest keys, and permits a fixed field set in `plugin.json`. A `skills`,
`hooks` or `mcpServers` key there is foreign. A `${VAR:-default}` in
`mcp.json` stays literal by spec.

**Do.** Keep `plugin.json` to the whitelist (the checker enforces it), keep
skills at `./skills/`, keep `mcp.json` at the root with `"type": "stdio"` and
a bare pinned launch.

## `.agents/` already exists {#agents-dir}

**Symptom.** The target repo has `.agents/skills/` (Codex's repo skills) or
`.agents/<something>/` holding agent definitions.

**Cause.** `.agents/` is Codex's per-repo home, and some repos use it for
their own agent files.

**Do.** Nothing special: the generator writes or merges only
`.agents/plugins/marketplace.json`. Do not move the other contents.

## The dev plugin's skills reach end users {#dev-leak}

**Symptom.** `claude plugin install <name>@...` shows maintainer skills in the
skill list.

**Cause.** The product manifest's `skills` path points at or above `dev/`, or
the dev skills were placed under `skills/`.

**Do.** Product skills under `./skills/`, maintainer skills under
`./dev/skills/`, the marketplace lists `./` and `./dev` as two plugins. The
checker fails when the two trees overlap.

## A worker's "I am watching CI" never happens {#worker-turn}

Not a plugin trap but one that bites the same campaign: a subagent's turn
ends with its report, so any promise about later in that report is never
kept. Make it block (`gh run watch --exit-status <id>`) or read the run
yourself.
