# The manifests, field by field

Seven JSON files describe one plugin to three ecosystems. This file says what
each may contain and why the generator writes it the way it does. The spec
keys it cites are those of `plugin.spec.json` (`scripts/spawn_plugin.py init`
writes an example; `assets/examples/colgrep-mcp.spec.json` is a full one).

## Contents

- [Shared identity](#shared-identity)
- [Agent Plugins 1.0: `plugin.json`, `mcp.json`](#agent-plugins-10)
- [Claude Code: `.claude-plugin/`](#claude-code)
- [Codex: `extensions["com.openai"]`, `.agents/plugins/marketplace.json`](#codex)
- [Multi-plugin repositories](#multi-plugin-repositories)
- [Cross-format equivalence](#cross-format-equivalence)
- [Placeholder rules](#placeholder-rules)
- [Verification status](#verification-status)

## Shared identity

`name` (kebab-case; Codex and Claude Code both namespace components by it),
`version` (strict semver; Codex validates it), `description`, `author {name,
email}`, `homepage`, `repository`, `license` (SPDX id), `keywords`. The same
values go into every manifest; `check_plugin.py` fails when `name` or
`version` differ between them. `displayName` is Claude Code's top-level
field and Codex's `interface.displayName`; Agent Plugins 1.0 has no such
field.

## Agent Plugins 1.0

Spec: <https://agent-plugins.org/specification>. Clients: Cursor, GitHub
Copilot, VS Code, Kiro. The spec defines the *package* only; installation
and marketplaces are each client's own (VS Code: **Chat: Install Plugin from
Source** with the GitHub slug).

`plugin.json` at the repository root, with `"$schema":
"https://agent-plugins.org/schemas/1.0.0/plugin.schema.json"`. Permitted
fields: `$schema, name, version, description, author, homepage, repository,
license, keywords, extensions`. Nothing else: the spec knows skills (a
`skills/` directory it discovers) and MCP servers (`mcp.json`) and says other
component types "do not affect conformance", so a `hooks` or `skills` key here
is at best ignored. The checker enforces the whitelist.

`mcp.json` at the root, `"$schema": ".../1.0.0/mcp.schema.json"`, same
schema version as `plugin.json`, `"type": "stdio"` on each server. Env keys
`PLUGIN_ROOT` and `PLUGIN_DATA` are reserved by the spec. Placeholder
expansion is allowed in `args`, `env` values and `cwd`, never in `command`,
and `${VAR:-default}` fallback syntax "MUST remain literal", so the generator
writes none: the launch is the bare pinned artifact.

## Claude Code

Docs: <https://code.claude.com/docs/en/plugins-reference>. Verified on this
machine end to end: `claude plugin marketplace add <owner/repo>` clones the
repo and resolves relative `source` paths against the clone;
`claude plugin install <plugin>@<marketplace>` connects the server.

`.claude-plugin/plugin.json`: identity plus `displayName`, `mcpServers`
(a path, `"./.claude-plugin/mcp.json"`), `hooks` (a path or an array of
paths naming only the per-event files, `"./hooks/worktree-remove.json"` in
colgrep-mcp; omitted when every event is portable), `skills`
(`"./skills/"`). Paths are relative to the plugin root and begin with `./`.
Claude Code loads `hooks/hooks.json` on its own and reads `hooks` as
*additional* files; a manifest that names `./hooks/hooks.json` fails every
marketplace install with "Duplicate hooks file detected" (Claude Code
2.1.270), and neither `--plugin-dir`, `plugin details` nor `plugin validate`
runs that check (`hooks.md#one-file-per-event-class`). The generator writes
one string for one file and an array for several; the checker fails on the
default file and on a one-element array.

`.claude-plugin/mcp.json`: the same launch as the other two MCP manifests,
plus the one placeholder the whole tree carries, in `env`:
`"COLGREP_MCP_ROOT": "${CLAUDE_PROJECT_DIR}"` in colgrep-mcp's case
(spec key `mcp.claude_env`). Claude Code substitutes `${CLAUDE_PLUGIN_ROOT}`
and `${CLAUDE_PROJECT_DIR}` in `command`, `args` and `env` of plugin MCP
servers; the fallback syntax is documented for project-scope `.mcp.json`
only, so do not rely on it here. The file is *not* at the root as
`.mcp.json`: that path is project-scope auto-discovery, a different loader
with no plugin root (`traps.md#root-mcp-json`).

`.claude-plugin/marketplace.json`: `name`, `description`, `owner {name,
email}`, `plugins[{name, source, description}]`. `source: "./"` means the
repository root is the plugin; `"./dev"` lists the maintainer plugin from
the same clone. Marketplace names and plugin names are separate namespaces
(`traps.md#namespaces`). `claude plugin validate <repo>` validates this file
when it exists; validate `.claude-plugin/plugin.json` and `dev/` separately.

## Codex

Reference: the `plugin-creator` sample skill in `openai/codex`
(`codex-rs/skills/src/assets/samples/plugin-creator/references/plugin-json-spec.md`).
No Codex CLI was available when colgrep-mcp was built, so these manifests
are written to that reference and never exercised. Say so in the README.

**Codex has no manifest of its own.** It parses the same root, Agent
Plugins 1.0-conformant `plugin.json` natively (`agent_plugin_manifest.rs`)
and reads its own extras from `extensions["com.openai"]`, the namespace
Agent Plugins 1.0 reserves for client-specific data in an otherwise closed
schema. Codex then auto-wires `skills` to `./skills` and `mcp_servers` to
the root `mcp.json` *by convention* — neither needs a field anywhere. There
is no second plugin folder, no second `mcp.json`, and no code path in
`spawn_plugin.py` that writes one: `.codex-plugin/plugin.json` and
`.codex-plugin/mcp.json` do not exist in the generator's output, for any
spec. (An older layout wrote both; if you are reading a plugin tree or a
doc that still has them, it predates this shape and should be regenerated.)

`build_codex_extensions(spec)` produces the `extensions["com.openai"]`
payload, attached under `build_agent_plugin` whenever `"codex"` is one of
the spec's `ecosystems`:

```json
{
  "extensions": {
    "com.openai": {
      "interface": { "...": "see table below" },
      "hooks": "./hooks/hooks.json"
    }
  }
}
```

`interface` (required; `check_plugin.py` flags any missing key):

| `interface.` | Meaning | Spec key |
|:--|:--|:--|
| `displayName` | title | `displayName` |
| `shortDescription` | subtitle in compact views | `codex.shortDescription` |
| `longDescription` | details page paragraph | `codex.longDescription` |
| `developerName` | publisher | `author.name` |
| `category` | bucket, e.g. `Developer Tools` | `codex.category` |
| `capabilities` | e.g. `["Read"]`, `["Read", "Write"]`, `["Interactive"]` | `codex.capabilities` |
| `defaultPrompt` | at most 3 starter prompts, ≤128 chars, ~50 reads best | `codex.defaultPrompt` |

Optional and unused by the generator: `websiteURL`, `privacyPolicyURL`,
`termsOfServiceURL` (absolute https), `brandColor`, `composerIcon`, `logo`,
`logoDark`, `screenshots` (PNG under `./assets/`), `apps`. These, like
`interface`, have no root slot in Agent Plugins 1.0 either, so a client
that wanted them would also read them from `extensions["com.openai"]` —
the generator just doesn't populate them.

`extensions["com.openai"].hooks`: Codex discovers `hooks/hooks.json` only
when this key is absent, and an explicit value *replaces* that discovery
(Codex plugin docs, "Build a plugin"): the opposite of Claude Code, which
always loads the file and treats the manifest field as additional. So the
key, when present, is `"./hooks/hooks.json"` and never a per-event file
whose event Codex may not know. The `plugin-creator` sample reference both
lists `"hooks": "./hooks.json"` among the top-level fields and says under
validation notes that "validation rejects unsupported manifest fields such
as `hooks`"; the two statements may describe two products (the CLI and the
ChatGPT app ingestion). The generator writes the key only with
`hooks.codex: true` in the spec (`_check_hooks_spec` / `build_codex_extensions`).

There is no separate Codex `mcp.json` any more either: Codex reads the same
root `mcp.json` that Agent Plugins 1.0 clients read (`build_mcp(spec,
version, "agent-plugins")`), by the `mcp_servers -> ./mcp.json` auto-wire
above — one file, one launch, checked once in `check_plugin.py` rather than
under a separate Codex key.

`.agents/plugins/marketplace.json`: `name`, `interface {displayName}`,
`plugins[{name, source, policy {installation, authentication}, category}]`.
`source` is `{"source": "local", "path": "./"}` for a root plugin and
`{"source": "git-subdir", "url": "<repo>.git", "path": "./<dir>"}` for a
subdirectory sibling — Codex has no `github`-style shorthand, so a
subdirectory source is the identical `git-subdir`/`url`/`path` shape Claude
Code uses (`_codex_source`). `policy.installation` ∈ `NOT_AVAILABLE |
AVAILABLE | INSTALLED_BY_DEFAULT`; `policy.authentication` ∈ `ON_INSTALL |
ON_USE`, with **no third value and no catch-all** — the generator hardcodes
`{"installation": "AVAILABLE", "authentication": "ON_INSTALL"}` for every
entry (`build_codex_marketplace`), never `"NONE"` (`traps.md#codex-auth-enum`
says why that matters). `category` is set per entry from
`entry.codex.category`, falling back to the spec-level `codex.category`
(default `"Developer Tools"`); once a marketplace entry sets `category`, it
overrides the plugin's own manifest permanently in Codex's browse view — a
real plugin manifest wins on every other field post-install, but not this
one. The reference's convention is `path: "./plugins/<name>"` for a
non-root plugin; the generator instead uses the plugin's own `dir`
verbatim, which is equivalent when `dir` is `plugins/<name>`. Install:
`codex plugin marketplace add <owner/repo>` then
`codex plugin add <plugin>@<marketplace-name>`. `.agents/` is also where
Codex keeps repo skills (`.agents/skills/`), so the generator merges into
this one file and touches nothing else there.

A `marketplace` key at the top of the spec (any truthy value, e.g.
`"marketplace": "hub"`) suppresses **both** local marketplace files for
every plugin the spec declares — `spawn()` never calls `merge_marketplace`
at all when it is set. Use it when another repository (a hub, e.g.
`CrackingShells/Nest`) owns the marketplace naming these plugins: two
repositories that each write a marketplace entry under the same
`name` collide (`traps.md#marketplace-name-collision`), so a spec bound for
a hub carries no `claude_marketplace` or `codex.marketplace_name` section at
all and this repo's own manifests are the only files it produces.

## Multi-plugin repositories

One spec can declare several sibling plugins with a top-level `plugins[]`
array; `plugin_entries(spec)` normalises both shapes so every `build_*`
function is unchanged either way. A single-plugin spec (no `plugins[]`)
yields one entry with `dir: ""`; a multi-plugin spec yields one entry per
array item, each with a kebab-case `name`, a `dir` (no two entries may
share either), and its own `description`. Fields the entry omits —
`author`, `homepage`, `repository`, `license`, `ecosystems`, `keywords` —
fall back to the spec's top-level value.

`spawn()` loops over `plugin_entries(spec)` and prefixes every path it
writes with the entry's `dir` (`_prefixed`), so a root plugin's output is
byte-for-byte what a single-plugin spec always wrote, and a sibling's is
the same tree rooted at `<dir>/`:

```
plugins/writing-history/
├── plugin.json                       Agent Plugins 1.0 manifest + extensions["com.openai"]
├── mcp.json                          [mcp]
├── .claude-plugin/plugin.json        Claude Code manifest
└── skills/writing-history/           assembled from skills/writing-history/
```

**Why the plugin root is assembled, not aliased.** Agent Plugins 1.0
requires a plugin to be "a directory rooted at a single filesystem
location" with every path inside it, and prohibits symlink escapes. A
skill directory cannot be a plugin root by itself, because `skills` must
point at a directory whose *children* are skill folders — so
`plugins/<name>/skills/<name>/` is a real, committed copy of
`skills/<name>/`, assembled by a separate tool (`tools/assemble_plugin.py`,
owned by `plugin_assembly_tool`), not a symlink into it. `spawn_plugin.py`
itself only writes manifests; it never copies a skill tree.

`resolve_version(entry, root)` runs once per entry (not once per spec), so
siblings hold **independent version lines** — one plugin can be at `2.3.0`
while another is at `0.9.1`, each resolved from its own `version_from`
relative path. `versioning.md#sibling-plugins-independent-versions` has the
detail.

**Marketplace entries accumulate across every plugin**, and are written
once per ecosystem after the loop, never once per plugin
(`claude_entries` / `codex_entries` lists, then one `merge_marketplace`
call each) — `Writer.merge_marketplace` only ever appends entries a
marketplace lacks, and multi-plugin mode never passes `force=True` to it,
so re-running `spawn` after adding a third sibling adds only that
sibling's entry.

**Marketplace `source` shapes, root vs. subdirectory** (`_claude_source`,
`_codex_source`): a root plugin (`dir == ""`) stays the simple form both
ecosystems always used — `"./"` for Claude Code, `{"source": "local",
"path": "./"}` for Codex. A subdirectory sibling gets the `git-subdir`
shape in **both** ecosystems: `{"source": "git-subdir", "url":
"<repository>.git", "path": "./<dir>"}`. A sibling therefore needs its own
`repository` (or the spec's top-level one) — `_git_clone_url` refuses to
build a marketplace source without it.

**Known gap: `install-snippet` and `init_spec` were not extended for
multi-plugin specs.** `install_snippet(spec)` reads only `spec['name']` and
the spec's own top-level marketplace names; run on a multi-plugin spec it
prints install instructions for the top-level identity only; run it once
per entry (`plugin_entries(spec)`) by hand for now if every sibling needs
its own README section. `init_spec(root)` still derives `name` from the
repository directory name, so `init` is only a starting point for a
multi-plugin repo — the `plugins[]` array itself must be hand-written or
copied from `evals/fixtures/two_plugins.spec.json`.

## Cross-format equivalence

Held equivalent across all three formats when the spec sets them: `name`,
`version`, `description`, `keywords`, skills location, and the MCP launch
command. Everything below is an irreducible divergence — expressible in
some formats and not others, or expressible only through an
extension namespace with no defined semantics:

| Field / concept | Claude Code | Codex | Agent Plugins 1.0 | Note |
|:--|:--|:--|:--|:--|
| `hooks` | native (`hooks/hooks.json` + per-event files, manifest `hooks` field) | native, via `extensions["com.openai"].hooks` | **no concept at all** | AP 1.0's schema is closed (`additionalProperties: false`); a `hooks` key in `plugin.json` is a whitelist violation, not an ignored extra |
| `displayName` and every `interface.*` field (`shortDescription`, `category`, `capabilities`, `defaultPrompt`, …) | `displayName` is a top-level manifest field | only inside `extensions["com.openai"].interface` | **no root slot** | reachable in Codex only via the semantics-free `extensions` namespace; AP 1.0 defines no presentation metadata at all |
| `commands`, `agents`, `lspServers` | native | not defined | not defined | Claude Code-only component types |
| `apps` | not defined | native | not defined | Codex-only component type |
| `author`, `homepage`, `repository`, `license` | native | absent from Codex's own general manifest struct | native | valid in the Claude Code and AP 1.0 schemas; harmless-but-inert JSON if present in the fields Codex's struct doesn't declare (identity fields Codex actually reads — `author.name`, `version` — come from the shared root AP manifest, not a Codex-specific struct) |
| `.mcp.json` (project scope) vs. `mcp.json` (AP 1.0 plugin root, dot-less) | reads `.mcp.json` at project scope, `.claude-plugin/mcp.json` as its own plugin manifest | reads the dot-less root `mcp.json` | reads the dot-less root `mcp.json` | a silent naming trap if the two get confused — see `traps.md#root-mcp-json` |

## Placeholder rules

| Ecosystem, file | `command` | `args` / `env` | `${VAR:-default}` |
|:--|:--|:--|:--|
| Agent Plugins 1.0 `mcp.json` (also Codex's, auto-wired) | never | allowed by spec (generator writes none) | forbidden, stays literal |
| Claude Code `.claude-plugin/mcp.json` | never | bare `${CLAUDE_PLUGIN_ROOT}` / `${CLAUDE_PROJECT_DIR}` | undocumented for plugins |
| Claude Code project `.mcp.json` (do not use) | expands | expands | documented, but the plugin root is unknown here |
| Hook commands (every hook file) | n/a | `${CLAUDE_PLUGIN_ROOT}` once, quoted; Codex sets the same variable | n/a |

The rule that falls out: the server launch is a published artifact pinned to
the plugin version (`uvx pkg==<version>`, `npx -y pkg@<version>`), identical
in all three MCP manifests; the only placeholder is a Claude-side `env`
value, and it is optional.

## Verification status

| Claim | Status |
|:--|:--|
| Claude Code remote install, server connect, hooks fire, `WorktreeRemove` | verified (colgrep-mcp, this machine, 2026-09) |
| `claude plugin validate` accepts the generated marketplace, plugin and dev manifests | verified (generator test run) |
| A Claude manifest naming `./hooks/hooks.json` fails a marketplace install ("Duplicate hooks file detected"); `--plugin-dir`, `plugin details` and `plugin validate` accept it | verified (colgrep-mcp 0.4.0 and 0.5.0 installs, Claude Code 2.1.270; fixed in PR #14) |
| Codex: an explicit `extensions["com.openai"].hooks` replaces the `hooks/hooks.json` discovery | Codex plugin docs, "Build a plugin"; no CLI run |
| Codex reads a root, Agent Plugins 1.0-conformant `plugin.json` natively and takes its extras from `extensions["com.openai"]` (no `.codex-plugin/` folder) | written to Codex's `agent_plugin_manifest.rs` behaviour as documented; no CLI run |
| Codex marketplace `policy.authentication` accepts only `ON_INSTALL \| ON_USE` (no `"NONE"`, no catch-all), and a marketplace file parses in one pass — one bad enum value fails every entry | Codex marketplace schema (`policy.authentication` enum); no CLI run; `traps.md#codex-auth-enum` |
| Agent Plugins 1.0 manifests | written to the spec; VS Code install path documented from its docs |
| Cursor loads Claude Code *plugin* hooks | no: it imports hooks from `settings.json` files only |
