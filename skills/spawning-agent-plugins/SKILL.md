---
name: spawning-agent-plugins
description: Turns an existing repository (skills, an MCP server, hooks, or all three) into a plugin that Claude Code, Codex and Agent Plugins 1.0 clients (Cursor, VS Code, Copilot, Kiro) can all install from GitHub, by generating the seven manifests plus the optional hooks and maintainer dev-plugin skeleton from one spec, then checking them for the drifts no loader reports. Use this whenever the task is to package, publish, expose or "make installable" a repo's skills or MCP server for coding agents, to add a `.claude-plugin/`, `.codex-plugin/`, `plugin.json` or marketplace to a repo, to add a second harness to a repo that already has one, to ship hooks with a plugin, or to add a maintainer-only skills plugin next to a product plugin — even when the request never says "plugin" and just says "let agents install this" or "put this on the marketplace". Also load it to audit an existing plugin tree that fails to connect or drifts on a version bump.
---

# spawning-agent-plugins

One repository, three plugin ecosystems, one spec. The layout below is the one
proven on `CrackingShells/colgrep-mcp`: Claude Code verified end to end
(remote install from GitHub, server connected, hooks firing); Codex and Agent
Plugins 1.0 written against their published specs and validated with the
tooling available, not exercised with a live client. Keep that distinction in
what you tell the user.

```
plugin.json                      Agent Plugins 1.0 manifest (whitelisted fields only)
mcp.json                         Agent Plugins 1.0 MCP manifest             [mcp]
.claude-plugin/plugin.json       Claude Code manifest
.claude-plugin/marketplace.json  Claude Code marketplace (product [+ dev] plugin)
.claude-plugin/mcp.json          Claude Code MCP manifest (the only one with placeholders)
.codex-plugin/plugin.json        Codex manifest (with the `interface` block)
.codex-plugin/mcp.json           Codex MCP manifest
.agents/plugins/marketplace.json Codex marketplace
hooks/hooks.json                 portable hook events; both loaders read it unasked [hooks]
hooks/<event>.json               one file per event not every harness knows          [hooks]
hooks/<name>_policy.py           one stdlib, fail-open hook script                  [hooks]
dev/.claude-plugin/plugin.json   maintainer skills plugin, versioned with the product [dev]
skills/<skill>/SKILL.md          the product skills, shared by every ecosystem
```

Every manifest describes the same plugin: same `name`, same `version`, same
launch command. Nothing enforces that but `scripts/check_plugin.py`, which is
why step 4 is not optional.

## Workflow

### 1. Decide the components

Ask, or read off the repo, four yes/no answers. Each one adds files; none is
required.

| Component | Include when | Spec section |
|:--|:--|:--|
| skills | the repo has `skills/<name>/SKILL.md` for agents to load | `"skills": "./skills/"` |
| MCP server | there is a server to launch (`uvx <pkg>==<version>`, `npx`, an installed executable) | `"mcp": {...}` |
| hooks | the plugin must inject context at session start or gate a tool (deny `grep -r`, for instance) | `"hooks": {...}` |
| dev plugin | the repo's *maintenance* knowledge lives in skills end users must never receive | `"dev": {...}` |

Two defaults worth stating to the user rather than deciding silently. The
marketplace is named after the GitHub *organization* (`init` derives
`cracking-shells` from `CrackingShells/<repo>`), the same in both ecosystems,
because a marketplace lists many plugins and an organization has many repos;
colgrep-mcp first named its Claude marketplace after the plugin, which made
the install string read `colgrep-mcp@colgrep-mcp`, and renamed it later.
Codex hooks are opt-in (`"codex": false`) because Codex's own reference both
lists the `hooks` field and says its validator rejects it. See
`references/traps.md`.

### 2. Write the spec

```bash
python3 <skill>/scripts/spawn_plugin.py --root <repo> init
```

writes `plugin.spec.json` prefilled from `git remote`, `git config`, the
`LICENSE` header and the version source it finds (a `[project]` pyproject at
the root or one level down, `package.json`, `Cargo.toml`). Fill the `TODO`s,
move the sections you want out of `_optional_sections`, delete the rest.
`{version}` inside `mcp.args` is replaced by the resolved version, which is
how the launch pin tracks the release (`references/versioning.md`).
`assets/examples/colgrep-mcp.spec.json` is a complete spec with every section
on; it regenerates that repository's manifests and hook files with identical
JSON content (checked against `main` after its PR #14, 2026-09-15).

Hook events go in two spec lists. `hooks.portable` takes the events every
hook-capable harness knows; they land in `hooks/hooks.json`. `hooks.extra`
takes every other event, one entry each with its own `description`; each
lands in its own file named after the event (`WorktreeRemove` →
`hooks/worktree-remove.json`). The generator refuses a spec that puts an
event on the wrong side.

Keep the spec in the repo (it is the record of the decisions), or delete it
after step 7 if the user prefers a smaller root. The checker does not need it.

### 3. Spawn

```bash
python3 <skill>/scripts/spawn_plugin.py --root <repo> spawn --spec plugin.spec.json --dry-run
python3 <skill>/scripts/spawn_plugin.py --root <repo> spawn --spec plugin.spec.json
```

The generator never overwrites: a file whose JSON content differs from the
spec is listed as `kept` with the top-level keys that would change, and only
`--force` rewrites it. Marketplaces are merged, so a repo that already has a
`.claude-plugin/marketplace.json` (a Rust conventions plugin, a `dist/`
source) gains the new entries and loses nothing. `.agents/` is touched only
at `.agents/plugins/marketplace.json`; Codex keeps its skills under
`.agents/skills/` and some repos hold agent definitions there.

With `hooks` on, a missing script is seeded from
`assets/hooks/policy_template.py`: a dispatcher on `hook_event_name` that
injects `POLICY` at session and subagent start, has a `PreToolUse` stub, and
fails open on every path. Fill the `TODO`s; the shape is the part that took
a cycle to get right (`references/hooks.md`).

### 4. Verify

```bash
python3 <skill>/scripts/check_plugin.py --root <repo>     # drift guards; exit 1 lists each problem
claude plugin validate <repo>                              # the marketplace, when one exists
claude plugin validate <repo>/.claude-plugin/plugin.json   # the Claude manifest
claude plugin validate <repo>/dev                          # the dev plugin, if any
```

Do not point `claude plugin validate` at `.codex-plugin/plugin.json`: it
rejects Codex's `interface` block as an unknown field, which is noise, not a
finding. None of these commands, nor `claude --plugin-dir` or `claude plugin
details`, runs the check a marketplace install runs on the `hooks` field
(`references/hooks.md#one-file-per-event-class`); `check_plugin.py` does, and
the install-path oracle is a scratch marketplace: a directory holding a
`.claude-plugin/marketplace.json` with a throwaway `name` whose plugin
`source` is a copy of the tree without `.git` and `.venv`, then
`claude plugin marketplace add <dir>`, `claude plugin install
<plugin>@<that-name>`, `claude plugin list`, and remove both afterwards.
With an MCP server, `claude --plugin-dir <repo> mcp list` must show
`✔ Connected`; that launches whatever the manifest pins (a PyPI release, not
the tree), so a pin that is not published yet fails here by design. With
hooks, pipe an event to the script and read the JSON back:

```bash
echo '{"hook_event_name":"SessionStart","cwd":"/"}' | python3 hooks/<name>_policy.py
```

Codex cannot be verified without a Codex CLI on the machine; say so in the
README rather than implying parity.

### 5. Wire the version

The plugin version appears in three or four manifests and in every MCP pin.
One bump must move all of them, or the next release ships a plugin that
launches the previous server. `references/versioning.md` gives the
commitizen `version_files` list, the semantic-release `prepareCmd`, and the
bare-`sed` fallback when the repo has no release tooling; pick the one the
repo already uses. `check_plugin.py` catches the drift when nothing else does.

### 6. Tell installers and maintainers

```bash
python3 <skill>/scripts/spawn_plugin.py install-snippet --spec plugin.spec.json   # prints the README section
```

Paste it into the README's Install section and edit the prose; it names the
right `<plugin>@<marketplace>` for each ecosystem, which is the part people
get wrong (`references/traps.md#namespaces`). If the repo has an `AGENTS.md`
or `CLAUDE.md`, add one line saying the repo is a plugin, where the manifests
are, and how to load the dev plugin (`claude --plugin-dir ./dev`).

### 7. Keep it from drifting

Copy the checker into the test suite; it is pytest-collectable as is:

```bash
cp <skill>/scripts/check_plugin.py <repo>/tests/test_plugin_structure.py
```

It walks up from its own location to the first directory holding a plugin
manifest, so it needs no configuration; set `REPO_ROOT` at the top if the
tests live somewhere unusual. Commit the manifests with the vocabulary the
repo uses; where the repo has none, `build(plugin): package the repo as a
Claude Code, Codex and Agent Plugins 1.0 plugin` is the shape colgrep-mcp used.

## Rules that survived a cycle

- **No root `.mcp.json`, ever.** Claude Code reads it as project-scope config
  with no notion of `${CLAUDE_PLUGIN_ROOT}`; a placeholder there is spawned
  literally. The Claude MCP manifest lives at `.claude-plugin/mcp.json`.
- **Placeholders only in the Claude Code manifest, only in `env`.** Codex and
  Agent Plugins 1.0 are not documented to expand them; a literal `${...}` in
  `args` means the server silently never started. Pin a published artifact
  instead (`uvx pkg==<version>`), identical in all three MCP manifests.
- **`hooks/hooks.json` holds only events every hook-capable harness knows;
  every other event gets its own file named after it.** A parser that meets
  an unknown event may drop every hook, and `hooks.json` is the one name both
  loaders claim unasked, so no second generic name (`claude-code.json`) sits
  beside it.
- **The Claude Code manifest's `hooks` names exactly the per-event files and
  never `./hooks/hooks.json`; the Codex manifest names `./hooks/hooks.json`
  and nothing else.** Claude Code always loads `hooks/hooks.json` and reads
  the field as additional files, so naming it again fails every marketplace
  install with "Duplicate hooks file detected" while `--plugin-dir`,
  `plugin details` and `plugin validate` stay silent (Claude Code 2.1.270).
  Codex discovers `hooks/hooks.json` only when the manifest defines no
  `hooks`; an explicit value replaces that discovery. The generator omits the
  Claude field when every event is portable; the checker pins both manifests.
- **The hook script is stdlib-only and exits 0 on every path.** It runs under
  `uv run --no-project python` on each matched tool call; a crash or a
  non-stdlib import makes the session unusable.
- **The dev plugin carries knowledge, never a server, and its skills tree is
  disjoint from the product's.** The marketplace lists both from `./` and
  `./dev`; the product manifest's `skills` never reaches into `dev/`.
- **`plugin.json` (Agent Plugins 1.0) takes whitelisted fields only.** No
  `hooks`, `skills` or `mcpServers` keys; the spec defines skills and MCP
  through its own files and says extra component types are ignored.

## References

| Read | When |
|:--|:--|
| `references/manifests.md` | filling a spec field you are unsure of; what each manifest may and may not contain; the placeholder rules table |
| `references/hooks.md` | designing the hook script: the per-event file rule and what each manifest may name, events, matcher names per harness, I/O contract, context cap, Cursor and Codex limits |
| `references/versioning.md` | wiring the version bump: commitizen, semantic-release, npm, or none |
| `references/traps.md` | a plugin lists but does not connect, "Duplicate hooks file detected" at install, hooks do not fire, `x@x` install strings, stale uvx cache, Codex hooks |
