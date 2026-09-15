# The manifests, field by field

Seven JSON files describe one plugin to three ecosystems. This file says what
each may contain and why the generator writes it the way it does. The spec
keys it cites are those of `plugin.spec.json` (`scripts/spawn_plugin.py init`
writes an example; `assets/examples/colgrep-mcp.spec.json` is a full one).

## Contents

- [Shared identity](#shared-identity)
- [Agent Plugins 1.0: `plugin.json`, `mcp.json`](#agent-plugins-10)
- [Claude Code: `.claude-plugin/`](#claude-code)
- [Codex: `.codex-plugin/`, `.agents/plugins/marketplace.json`](#codex)
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

`.codex-plugin/plugin.json`: identity (`author.name` required), `skills`
(`"./skills/"`), `mcpServers` (a path, `"./.codex-plugin/mcp.json"`, or an
inline object), optionally `hooks` (a single path, `"./hooks/hooks.json"`
and nothing else; see below), and the required `interface` block:

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
`logoDark`, `screenshots` (PNG under `./assets/`), `apps`.

`hooks`: Codex discovers `hooks/hooks.json` only when the manifest defines
no `hooks`, and an explicit value *replaces* that discovery (Codex plugin
docs, "Build a plugin"): the opposite of Claude Code, which always loads
the file and treats the field as additional. So the field, when present,
is `"./hooks/hooks.json"` and never a per-event file whose event Codex may
not know. The `plugin-creator` sample reference both lists `"hooks":
"./hooks.json"` among the top-level fields and says under validation notes
that "validation rejects unsupported manifest fields such as `hooks`"; the
two statements may describe two products (the CLI and the ChatGPT app
ingestion). colgrep-mcp ships `"hooks": "./hooks/hooks.json"`; the generator
writes it only with `hooks.codex: true`.

`.codex-plugin/mcp.json`: the launch and nothing else. It exists as a
separate file so that the Claude placeholder never reaches a client not
documented to expand it.

`.agents/plugins/marketplace.json`: `name`, `interface {displayName}`,
`plugins[{name, source {source: "local", path}, policy {installation,
authentication}, category}]`. `policy.installation` ∈ `NOT_AVAILABLE |
AVAILABLE | INSTALLED_BY_DEFAULT`; `policy.authentication` ∈ `ON_INSTALL |
ON_USE` per the reference, `NONE` in colgrep-mcp (unverified either way;
the generator keeps `NONE` for a plugin with no auth to perform, change it
if a Codex run rejects it). The reference's convention is
`path: "./plugins/<name>"`; a root-level plugin uses `"./"`. Install:
`codex plugin marketplace add <owner/repo>` then
`codex plugin add <plugin>@<marketplace-name>`. `.agents/` is also where
Codex keeps repo skills (`.agents/skills/`), so the generator merges into
this one file and touches nothing else there.

## Placeholder rules

| Ecosystem, file | `command` | `args` / `env` | `${VAR:-default}` |
|:--|:--|:--|:--|
| Agent Plugins 1.0 `mcp.json` | never | allowed by spec (generator writes none) | forbidden, stays literal |
| Claude Code `.claude-plugin/mcp.json` | never | bare `${CLAUDE_PLUGIN_ROOT}` / `${CLAUDE_PROJECT_DIR}` | undocumented for plugins |
| Claude Code project `.mcp.json` (do not use) | expands | expands | documented, but the plugin root is unknown here |
| Codex `.codex-plugin/mcp.json` | never | not documented; generator writes none | not documented |
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
| Codex: an explicit manifest `hooks` replaces the `hooks/hooks.json` discovery | Codex plugin docs, "Build a plugin"; no CLI run |
| Codex manifests, marketplace, hooks | written to the `openai/codex` reference; no CLI run |
| Agent Plugins 1.0 manifests | written to the spec; VS Code install path documented from its docs |
| Cursor loads Claude Code *plugin* hooks | no: it imports hooks from `settings.json` files only |
