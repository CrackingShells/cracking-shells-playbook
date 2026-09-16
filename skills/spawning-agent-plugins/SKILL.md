---
name: spawning-agent-plugins
description: Turns an existing repository (skills, an MCP server, hooks, or all three) into one or several plugins that Claude Code, Codex and Agent Plugins 1.0 clients (Cursor, VS Code, Copilot, Kiro) can install from GitHub, generating the manifests plus optional hooks and dev-plugin skeleton from one spec, then checking them for drifts no loader reports. Handles one plugin rooted at the repo, several sibling plugins from a spec's `plugins[]` array, and a hub-owned marketplace that suppresses this repo's own marketplace files. Use this to package, publish or "make installable" a repo's skills or MCP server, to add a `.claude-plugin/`, `plugin.json` or marketplace, to add a second harness to a repo that has one, to ship hooks, to add a maintainer-only skills plugin, or to split a repo into several plugins — even when the request just says "let agents install this". Also load it to audit a plugin tree that fails to connect or drifts on a version bump.
---

# spawning-agent-plugins

One repository, three plugin ecosystems, one spec — one plugin by default, or
several sibling plugins from a `plugins[]` array
(`references/manifests.md#multi-plugin-repositories`). The layout below is the one proven on
`CrackingShells/colgrep-mcp`: Claude Code verified end to end (remote install
from GitHub, server connected, hooks firing); Codex and Agent Plugins 1.0
written against their published specs and validated with the tooling
available, not exercised with a live client. Keep that distinction in what
you tell the user.

```
plugin.json                      Agent Plugins 1.0 manifest, also Codex's (extras
                                  under extensions["com.openai"]; no separate
                                  Codex plugin folder is ever written)
mcp.json                         Agent Plugins 1.0 MCP manifest, also Codex's
                                  (auto-wired to ./mcp.json by convention)      [mcp]
.claude-plugin/plugin.json       Claude Code manifest
.claude-plugin/marketplace.json  Claude Code marketplace (product [+ dev] plugin) [not in hub mode]
.claude-plugin/mcp.json          Claude Code MCP manifest (the only one with placeholders)
.agents/plugins/marketplace.json Codex marketplace                             [not in hub mode]
hooks/hooks.json                 portable hook events; both loaders read it unasked [hooks]
hooks/<event>.json               one file per event not every harness knows          [hooks]
hooks/<name>_policy.py           one stdlib, fail-open hook script                  [hooks]
dev/.claude-plugin/plugin.json   maintainer skills plugin, versioned with the product [dev]
skills/<skill>/SKILL.md          the product skills, shared by every ecosystem
```

A multi-plugin spec repeats every path above once per `plugins[]` entry,
prefixed by that entry's `dir` (`plugins/<name>/plugin.json`, and so on); a
spec that sets the top-level `marketplace` key writes neither marketplace
file at all, because another repository (a hub) owns that name
(`references/traps.md#marketplace-name-collision`).

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

Two more yes/no answers, orthogonal to the four above:

| Question | When yes | Spec section |
|:--|:--|:--|
| Several plugins from one repo? | the repo holds more than one installable unit (e.g. several skills, each its own release line) | top-level `"plugins": [{"name", "dir", "description", ...}]` |
| Does another repo own the marketplace name? | an organization already has (or is about to have) a hub repository listing every plugin under one name | top-level `"marketplace": {"hub": "<repository URL>"}` (suppresses both local marketplace files for every plugin this spec declares; `claude_marketplace.name` and `codex.marketplace_name` stay — they still name the hub's marketplace) |

A spec can set both: several sibling plugins, none of which write a local
marketplace, because the hub lists all of them
(`references/manifests.md#multi-plugin-repositories`).

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
`assets/examples/colgrep-mcp.spec.json` is a complete, single-plugin spec
with every section on; `evals/test_regeneration.py` drives it against a real
colgrep-mcp checkout in `--dry-run` and asserts nothing would be written or
merged. As of the `extensions["com.openai"]` reshape (2026-09-16) that guard
allows exactly one deliberate divergence — `plugin.json`'s `extensions`
key — because colgrep-mcp's own manifest on disk still predates this shape;
a later leaf regenerates it for real. Any other divergence the guard reports
is a real defect, not an artifact of the reshape.
`evals/fixtures/two_plugins.spec.json` is the minimal multi-plugin
counterpart, two siblings with no marketplace-suppressing key.

**Writing a hub-mode spec: ask, don't infer.** Before setting the top-level
`marketplace` key, ask the human three things — who owns the marketplace,
what it is called, and which repository hosts it — and record every
answer: the repository goes in `marketplace.hub`, the name stays in
`claude_marketplace.name` and `codex.marketplace_name` exactly as it would
for a repo that owned its own marketplace. This is the general principle
behind every field in this section, not a special case for hub mode: the
generator reads recorded answers and never infers human-facing identity
(a marketplace name, a hub repository, a display name), because the two
things that consume that identity — `dev_readme()` and `install_snippet()`
— are documentation, and wrong documentation fails silently in a reader's
hands. It is also what keeps `spawn` reproducible: an interactively
supplied answer would make `evals/test_regeneration.py` meaningless, since
it re-runs `spawn` and diffs the output — a spec that cannot answer this
question on its own is a question for a human, not a prompt the generator
should ask at run time. `load_spec` enforces the hub half of this: a
hub-mode spec recording no `marketplace.hub` is rejected before anything
is written, naming the exact key to add.

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
`--force` rewrites it — except `dev/README.md`, which is seeded once and
never regenerated, `--force` included, once a maintainer starts editing it
by hand; a `kept` listing for it is not a signal `--force` will clear.
Marketplaces are merged, so a repo that already has a
`.claude-plugin/marketplace.json` (a Rust conventions plugin, a `dist/`
source) gains the new entries and loses nothing. `.agents/` is touched only
at `.agents/plugins/marketplace.json`; Codex keeps its skills under
`.agents/skills/` and some repos hold agent definitions there.

With `plugins[]` set, `spawn` loops the whole procedure once per entry,
prefixing every path by that entry's `dir` and resolving each entry's
version independently — but merges marketplace entries once per ecosystem
after the loop, not once per plugin, so a rerun after adding a fourth
sibling adds exactly that sibling's entry and touches nothing else
(`references/manifests.md#multi-plugin-repositories`). With the spec's
top-level `marketplace` key set, neither marketplace file is written at
all; a plugin only gets listed once whoever owns the hub adds it there by
hand.

With `hooks` on, a missing script is seeded from
`assets/hooks/policy_template.py`: a dispatcher on `hook_event_name` that
injects `POLICY` at session and subagent start, has a `PreToolUse` stub, and
fails open on every path. Fill the `TODO`s; the shape is the part that took
a cycle to get right (`references/hooks.md`).

### 4. Verify

```bash
python3 <skill>/scripts/check_plugin.py --root <repo> --spec plugin.spec.json  # drift guards; exit 1 lists each problem
claude plugin validate <repo>                              # the marketplace, when one exists
claude plugin validate <repo>/.claude-plugin/plugin.json   # the Claude manifest
claude plugin validate <repo>/dev                          # the dev plugin, if any
```

`check_plugin.py` scans for every plugin root under `--root` (a
`plugins[]` sibling, an assembled `plugins/<name>/` tree, or a maintainer
`dev/`) and checks each independently, so one sibling's identity is never
compared against another's. Pass `--spec` so dev-plugin checks (version lag,
skills disjointness) read which directories actually *are* dev plugins from
the spec's own `dev` declarations — tree shape alone cannot tell a dev
plugin nested under its product apart from an ordinary sibling product
nested the same way. **Without `--spec`, dev-plugin checks run against
nothing** in the common case: the fallback path only warns when the repo
*also* has no local `.claude-plugin/marketplace.json` (hub mode); a
single-repo tree with a local marketplace and no `--spec` silently skips
dev-plugin validation with no message at all. This is exactly the case for
the copied-into-`tests/` pytest entry point in step 7, which never passes
`--spec` — if a repo relies on that copy for its only structural check, its
dev plugin (if it has one) is unguarded; pass `--spec` there too, or accept
the gap knowingly.

Do not point `claude plugin validate` at the root `plugin.json`: it rejects
`extensions` (and, inside it, `interface`) as unknown fields, which is
noise, not a finding — there is no separate Codex manifest to validate
instead, because Codex reads this same file
(`references/traps.md#validate-codex`). None of these commands, nor
`claude --plugin-dir` or `claude plugin details`, runs the check a
marketplace install runs on the `hooks` field
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

The plugin version appears in two or three manifests (`plugin.json`,
`.claude-plugin/plugin.json`, and `dev/.claude-plugin/plugin.json` when a dev
plugin exists — Codex reads the first of these, no separate file of its own)
and in every MCP pin. One bump must move all of them, or the next release
ships a plugin that launches the previous server; a static `plugin.json`
version also means Codex's reinstall gate never fires
(`references/versioning.md`). `references/versioning.md` gives the
commitizen `version_files` list, the semantic-release `prepareCmd`, and the
bare-`sed` fallback when the repo has no release tooling; pick the one the
repo already uses. With `plugins[]`, each sibling bumps independently — wire
each one's own release config to its own `dir`, never a single bumper
walking fixed top-level paths. `check_plugin.py` catches the drift when
nothing else does.

### 6. Tell installers and maintainers

```bash
python3 <skill>/scripts/spawn_plugin.py install-snippet --spec plugin.spec.json   # prints the README section
```

Paste it into the README's Install section and edit the prose; it names the
right `<plugin>@<marketplace>` for each ecosystem, which is the part people
get wrong (`references/traps.md#namespaces`). If the repo has an `AGENTS.md`
or `CLAUDE.md`, add one line saying the repo is a plugin, where the manifests
are, and how to load the dev plugin (`claude --plugin-dir ./dev`).

**`install-snippet` was not extended for `plugins[]`:** it reads only the
spec's top-level `name` and marketplace names, so on a multi-plugin spec it
prints one section for the top-level identity, not one per sibling. Write
each sibling's README section by hand, or by editing the printed snippet's
plugin name and `dir` for each one
(`references/manifests.md#multi-plugin-repositories`).

### 7. Keep it from drifting

Copy the checker into the test suite; it is pytest-collectable as is:

```bash
cp <skill>/scripts/check_plugin.py <repo>/tests/test_plugin_structure.py
```

It walks up from its own location to the first directory holding a plugin
manifest, so it needs no configuration; set `REPO_ROOT` at the top if the
tests live somewhere unusual. This entry point calls `collect_problems`
with no `--spec`, so the dev-plugin gap in step 4 applies here by
default — a repo with both a local marketplace and a `dev/` plugin gets no
dev-plugin validation from this copy unless it is edited to pass one.
Commit the manifests with the vocabulary the repo uses; where the repo has
none, `build(plugin): package the repo as a Claude Code, Codex and Agent
Plugins 1.0 plugin` is the shape colgrep-mcp used.

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
  never `./hooks/hooks.json`; Codex's `extensions["com.openai"].hooks` names
  `./hooks/hooks.json` and nothing else.** Claude Code always loads
  `hooks/hooks.json` and reads the field as additional files, so naming it
  again fails every marketplace install with "Duplicate hooks file detected"
  while `--plugin-dir`, `plugin details` and `plugin validate` stay silent
  (Claude Code 2.1.270). Codex discovers `hooks/hooks.json` only when its
  extensions carry no `hooks` key; an explicit value replaces that discovery.
  The generator omits the Claude field when every event is portable; the
  checker pins both.
- **The hook script is stdlib-only and exits 0 on every path.** It runs under
  `uv run --no-project python` on each matched tool call; a crash or a
  non-stdlib import makes the session unusable.
- **The dev plugin carries knowledge, never a server, and its skills tree is
  disjoint from the product's.** The marketplace lists both from `./` and
  `./dev`; the product manifest's `skills` never reaches into `dev/`.
- **`plugin.json` (Agent Plugins 1.0, also Codex's) takes whitelisted fields
  only, `extensions` included.** No `hooks`, `skills` or `mcpServers` keys at
  the top level; the spec defines skills and MCP through its own files and
  says extra component types are ignored. Codex's presentation metadata and
  hooks pointer live inside `extensions["com.openai"]`, the one namespace the
  whitelist permits for client-specific data.
- **No repo but one hub owns a marketplace `name`.** Two repositories that
  each write a marketplace under the same name collide — silently in Claude
  Code, hard-erroring in Codex
  (`references/traps.md#marketplace-name-collision`). A spec whose plugins
  belong to someone else's catalogue sets the top-level `marketplace` key so
  the generator writes neither local marketplace file.
- **A `plugins[]` sibling's `dir` is committed, assembled content — never a
  symlink into `skills/`.** Agent Plugins 1.0 requires a plugin to be a
  single self-contained directory tree; `references/traps.md#assembled-tree-drift`
  is what happens when the assembled copy and its source skill disagree.

## References

| Read | When |
|:--|:--|
| `references/manifests.md` | filling a spec field you are unsure of; what each manifest may and may not contain; the placeholder rules table; multi-plugin repositories and hub marketplaces; the cross-format equivalence matrix |
| `references/hooks.md` | designing the hook script: the per-event file rule and what each manifest may name, events, matcher names per harness, I/O contract, context cap, Cursor and Codex limits |
| `references/versioning.md` | wiring the version bump: commitizen, semantic-release, npm, or none; independent sibling version lines; why tags need no migration |
| `references/traps.md` | a plugin lists but does not connect, "Duplicate hooks file detected" at install, hooks do not fire, `x@x` install strings, stale uvx cache, Codex hooks, the marketplace-name collision, an assembled plugin tree drifting from its source, a Codex marketplace entry with an invalid auth enum, a generator fix that never reaches an already hub-mode repo |
