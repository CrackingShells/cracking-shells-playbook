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

## A catalogue is missing plugins a user expects, with no error {#marketplace-name-collision}

**Symptom.** A user who added two CrackingShells repositories as
marketplaces sees only some of the org's plugins under `cracking-shells`,
or the wrong repository's plugins entirely; `claude plugin marketplace list`
shows one entry named `cracking-shells`, not two; nothing in Claude Code's
output says a marketplace was replaced. In Codex the same setup instead
refuses outright: `already added from a different source`.

**Cause.** A marketplace's identity is its `name` field, not the repository
it came from. If two repositories each declare a marketplace named
`cracking-shells`, Claude Code's documented behaviour is a **silent
replace** — "when they add a second marketplace with the same name, Claude
Code replaces the first" — while Codex hard-errors. Nothing in either tool's
output before that point distinguishes "one org, one catalogue" from
"two unrelated repos that happen to agree on a string". This is the reason
this campaign's generator gained a hub mode at all: a second product
repository writing its own `.claude-plugin/marketplace.json` and
`.agents/plugins/marketplace.json` under the same name as an existing one
is not a merge, it is a replacement nobody asked for.

**Do.** One hub repository owns the marketplace name for the organization
(`CrackingShells/Nest` owns `cracking-shells`); every other repository's
spec sets the top-level `marketplace` key — `{"hub": "<repository URL>"}` —
so `spawn_plugin.py` writes **no** local marketplace file for it at all
(`manifests.md#codex` describes the suppression). Record the hub's
repository in `marketplace.hub`; `load_spec` refuses a spec that sets
`marketplace` without it, rather than guessing it from `repository` (this
repo's own, not the hub's). A client that already
registered the name from the wrong source must
`claude plugin marketplace remove cracking-shells` (and the Codex
equivalent) before adding the hub — re-adding under the same name does not
retroactively fix which source it points at. When auditing an existing
install, check the registration's `source` field
(`known_marketplaces.json` for Claude Code), not just which plugins
`claude plugin list` shows: a truncated-but-present catalogue and a
wrong-source catalogue look identical from the plugin list alone.

## A generator fix never reaches a repo that already declared hub mode {#hub-mode-no-retro-repair}

**Symptom.** A bug in a marketplace-file builder (`build_claude_marketplace`,
`build_codex_marketplace`) gets fixed in `spawn_plugin.py`, `spawn` is
re-run against a repo that already set `marketplace.hub`, and the broken
value is still live in the hub's own marketplace file — nothing about
running `spawn` touched it, and nothing warned that it wouldn't. The live
instance: a Codex marketplace entry hardcoding `"authentication": "NONE"`
(an invalid enum value, `#codex-auth-enum`) shipped before that bug was
found; colgrep-mcp's own entry in the hub's marketplace had to be repaired
by hand, and any other repo generated before the fix carries the same
latent breakage with no self-healing path.

**Cause.** Hub mode means `spawn()` never calls `merge_marketplace` for
this repo at all (`spawn_plugin.py:612`) — that is the entire point of the
mode. A generator fix to marketplace-file *content* only ever reaches a
marketplace file the generator itself writes or merges. Once a repo
declares `marketplace.hub`, its plugin entries live in the hub's own
`.claude-plugin/marketplace.json` and `.agents/plugins/marketplace.json`,
files this repo's `spawn` invocation never opens, so a fix here is
invisible to it forever.

**Do.** Treat a marketplace-file builder bug as two fixes, not one, the
moment any repo is in hub mode: the generator fix, and a hand (or scripted)
correction of every affected entry already sitting in the hub's
marketplace files. `spawn --dry-run` against a hub-mode repo cannot surface
this — it never reads the hub's marketplace at all — so check the hub
repository directly, entry by entry, rather than trusting a clean dry run
from the product side.

## The assembled plugin tree drifts from its source skill {#assembled-tree-drift}

**Symptom.** `check_plugin.py` and `claude plugin validate` both pass, the
plugin installs, but it ships stale skill content — a fix landed in
`skills/<name>/` never reaches users of `plugins/<name>/skills/<name>/`, or
`git status --porcelain plugins/` is unexpectedly non-empty (or, worse,
unexpectedly empty right after an editing session).

**Cause.** `plugins/<name>/` is **assembled, committed content**, not a
symlink and not gitignored build output like `dist/`: Agent Plugins 1.0
requires a plugin to be a directory rooted at a single filesystem location
with every path inside it and forbids symlink escapes, so
`plugins/<name>/skills/<name>/` must be a real copy of
`skills/<name>/`. `spawn_plugin.py` never makes that copy — it only writes
manifests — so nothing about running `spawn` catches a stale copy. `git`
tracks the assembled tree beside its own source, which is a drift risk
`dist/*.skill` (gitignored) never had: someone can edit `skills/<name>/`,
commit, and never re-run the assembly step, and every static check still
passes because it only checks manifest *shape*, never content parity with
the source skill.

**Do.** Never hand-edit anything under `plugins/`; `skills/<name>/` is the
only editable source of truth. Regenerating `plugins/` must be wired into
three places, not one: the `.githooks/pre-commit` hook (regenerate on
staged skill changes, the same pattern it already uses for `dist/`), the
release tooling's asset list (so a release commits the tree it actually
ships), and CI (fail the build on any diff after a clean regenerate). A
`check_plugin.py` pass is necessary but not sufficient evidence the
assembled tree is current — it says the shape is valid, not that the
content matches; the only sufficient check is "regenerate and diff".

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

**Symptom.** Uncertainty whether `"hooks"` belongs in the manifest Codex
reads.

**Cause.** Codex's own `plugin-json-spec.md` lists `hooks` (a path) among the
top-level fields and, under validation notes, says the validator "rejects
unsupported manifest fields such as `hooks`". Codex was never run against
colgrep-mcp, which ships the field — now under
`extensions["com.openai"].hooks` on the shared root `plugin.json`, not a
manifest of its own (`manifests.md#codex`).

**Do.** Opt in with `hooks.codex: true` when the user wants Codex hooks and
can test them; leave it off otherwise. Either way the key names
only `hooks/hooks.json`, never a per-event file: Codex's plugin docs ("Build
a plugin") say an explicit `hooks` replaces the default discovery of
`hooks/hooks.json`, so naming the per-event file would drop the portable
events and hand Codex an event it may not know.

## `claude plugin validate` passed, and proved less than you think {#validate-picks-one}

**Symptom.** `claude plugin validate <dir>` reports `✔ Validation passed`, and a manifest error
later turns out to have been there the whole time. Or: deleting a marketplace file makes the same
command start reporting problems it never mentioned before.

**Cause.** It validates **one** manifest per invocation and prefers
`.claude-plugin/marketplace.json`. It announces which file it chose on its first line — read that
line, not just the tick. A repository that ships both a marketplace and a plugin manifest has only
ever had its *catalogue* validated; `plugin.json` is never reached. Observed on a repo carrying
both: the command named only the marketplace, though a `plugin.json` sat beside it.

**Do.** Point it at what you mean. For a hub-and-spoke org this falls out naturally — the hub holds
only a marketplace, and each plugin root holds only a plugin manifest, so validating each separately
covers everything. For a repo that ships both, validate the plugin manifest by its own path as well.
Treat "validation passed" as a claim about one named file, and let `check_plugin.py` carry the
cross-manifest invariants it cannot see.

## `claude plugin validate` fails on the shared manifest {#validate-codex}

**Symptom.** `extensions: Unknown field 'extensions'` (or, on an older tree,
`interface: Unknown field 'interface'`) and `✘ Validation failed`.

**Cause.** You pointed Claude Code's validator at the root `plugin.json` —
the Agent Plugins 1.0 / Codex manifest, which carries `extensions` and,
inside it, `interface`. Claude Code's validator knows only its own
manifest shape (`.claude-plugin/plugin.json`), its marketplace, and skill
directories; there is no Codex-specific manifest to point it at instead,
because none exists.

**Do.** Validate `<repo>` (its marketplace), `<repo>/.claude-plugin/plugin.json`
and `<repo>/dev`; run `check_plugin.py` for the cross-ecosystem invariants,
including everything under `extensions["com.openai"]`. Add `--strict` in CI
so unknown fields in the Claude manifest fail.

## A Codex marketplace entry makes every plugin in the file fail to parse {#codex-auth-enum}

**Symptom.** Adding a marketplace under Codex reports the whole marketplace
as unreadable or empty, not just one plugin — even though only one entry's
`policy` looks unusual, and the same file's `.claude-plugin/marketplace.json`
counterpart loads fine in Claude Code.

**Cause.** Codex's `policy.authentication` enum is exactly `ON_INSTALL |
ON_USE`, with **no `"NONE"` value and no serde catch-all** — and Codex
parses a marketplace file in **one pass**, so one entry with an unparseable
enum value fails the deserialisation of the entire file, not just that
entry. An earlier version of this generator wrote `"authentication":
"NONE"` for a plugin with no auth step, and `references/manifests.md`
documented that value as unverified; it is now confirmed invalid by the
enum definition itself, not merely untested.

**Do.** Never write `"NONE"`, `null`, or omit `authentication` and hope for
a default. `build_codex_marketplace` now hardcodes
`{"installation": "AVAILABLE", "authentication": "ON_INSTALL"}` for every
entry it writes, and `check_plugin.py` rejects any other value in either
field (`policy.installation` ∉ `{NOT_AVAILABLE, AVAILABLE,
INSTALLED_BY_DEFAULT}` or `policy.authentication` ∉ `{ON_INSTALL, ON_USE}`).
If a plugin genuinely needs `ON_USE` semantics, set it explicitly per entry
rather than relying on the generator's default.

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
