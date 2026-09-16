# One version, everywhere it appears

The plugin version is written literally into `plugin.json`,
`.claude-plugin/plugin.json`, `dev/.claude-plugin/plugin.json`, and into the
pin of every MCP launch (`uvx pkg==<version>` in both `mcp.json` files —
Codex reads the same root `mcp.json` as Agent Plugins 1.0, so there is no
third pin to move). A release that moves the package version and not these
ships a plugin whose manifest says the new version and whose server is the
old one; worse, `uvx` keys its cached environment on the requirement string,
so an unmoved pin reuses a stale environment forever. The generator writes
the resolved version once; keeping it moving is the release tooling's job,
and `check_plugin.py` is the alarm when nothing else does it.

**A plugin's own `plugin.json` version is authoritative, and it is the
*only* one that must move.** Claude Code's resolution precedence reads it
first — a stale value anywhere else (a marketplace entry, most importantly)
never wins once `plugin.json` disagrees, and Claude Code substitutes the
manifest value **without warning**, so a version you set in
`.claude-plugin/marketplace.json` can silently mask the truth. Codex's own
reinstall gate compares manifest **version strings**: if `plugin.json`'s
`version` does not change between releases, Codex has no signal that
anything changed and silently never reinstalls, even though the underlying
git ref (or a re-tagged release) moved. **Never set `version` on a
marketplace entry at all** — the generator doesn't (`build_claude_marketplace`
and `build_codex_marketplace` write `name`, `source`, `description`/`category`,
never `version`), and hand-editing one in is the trap this paragraph exists
to name.

## Sibling plugins, independent versions

A multi-plugin spec's siblings are versioned independently:
`resolve_version(entry, root)` runs once **per entry** inside `spawn()`'s
loop, not once per spec, so `plugins[0]`'s `version_from` can point at
`skills/writing-history/package.json` while `plugins[1]`'s points at
`skills/writing-release/package.json`, and each plugin's own release moves
only its own `plugin.json`, `.claude-plugin/plugin.json` and any pin — never
a sibling's. Nothing links the two: a repository with five plugins at five
different version numbers is the expected steady state, not drift. Wire
each sibling's own release tooling (its own `commitizen`/`semantic-release`
config, or its own `set_plugin_version.py` invocation scoped to
`<dir>/plugin.json` and `<dir>/.claude-plugin/plugin.json`) to bump only
that sibling's manifests; a bumper written for the single-plugin layout that
walks fixed top-level paths will silently miss every sibling but the root
one.

## Which source

`spawn_plugin.py init` picks the first it finds and records it as
`version_from`:

| Tree has | `version_from` | Bumped by |
|:--|:--|:--|
| `pyproject.toml` with `[project] version` (root or one level down) | `pyproject:<path>` | `cz bump`, `uv version`, `hatch version` |
| `package.json` | `package.json:package.json` | `npm version`, semantic-release |
| `Cargo.toml` | `cargo:Cargo.toml` | `cargo release`, `cargo set-version` |
| none | literal `"version": "0.1.0"` | you, then the checker |

Re-running `spawn` after a bump regenerates nothing by default (the manifests
differ from the spec only in `version`, and it never overwrites); wire the
bump instead.

## commitizen (`cz bump`), pyproject as source

colgrep-mcp's `[tool.commitizen]` in `server/pyproject.toml`; paths are
relative to the pyproject's directory. The `key:pattern` form rewrites the
version on the first line matching the pattern in that file, so
`"version"` hits the manifest's version line and `pkg==` hits the pin.

```toml
[tool.commitizen]
version_provider = "pep621"
tag_format = "v$version"
version_files = [
    "../plugin.json:\"version\"",
    "../.claude-plugin/plugin.json:\"version\"",
    "../dev/.claude-plugin/plugin.json:\"version\"",
    "../.claude-plugin/mcp.json:<pkg>==",
    "../mcp.json:<pkg>==",
]
```

Between the bump commit and the publish run, the pin names a version the
registry does not have yet; `claude --plugin-dir . mcp list` fails to connect
until the upload lands, by design (`traps.md#uvx-pin`).

## semantic-release, package.json as source

Add an `exec` step that rewrites the manifests before the git step commits
them; the `git` plugin's `assets` must list them or the bump never lands.

```js
["@semantic-release/exec", {
  prepareCmd: "python3 scripts/set_plugin_version.py ${nextRelease.version}"
}],
["@semantic-release/git", {
  assets: ["package.json", "CHANGELOG.md", "plugin.json", ".claude-plugin/plugin.json",
           "dev/.claude-plugin/plugin.json", "mcp.json", ".claude-plugin/mcp.json"]
}]
```

where `set_plugin_version.py` is the twelve-line rewrite below. In a
`multi-semantic-release` monorepo of skills — the shape this campaign uses
for the five playbook plugins — each sibling has its own release line and
its own `plugin.json`, so the version this step writes is *that skill's*
release, scoped to `<dir>/plugin.json` (and its siblings), never the
repository's aggregate version and never another sibling's files. Give each
plugin's release config its own `prepareCmd` invocation naming only its own
`<dir>`.

## npm alone

`"version": "python3 scripts/set_plugin_version.py $npm_package_version && git add plugin.json .claude-plugin mcp.json dev"`
in `package.json`'s `scripts` runs on `npm version` after the bump and
before its commit.

## No release tooling

The rewrite that every option above calls; stdlib, idempotent, safe to run
by hand:

```python
#!/usr/bin/env python3
"""set_plugin_version.py <version>: write one version into every plugin manifest and MCP pin."""
import json, re, sys
from pathlib import Path
version = sys.argv[1]
for rel in ("plugin.json", ".claude-plugin/plugin.json", "dev/.claude-plugin/plugin.json"):
    p = Path(rel)
    if p.exists():
        data = json.loads(p.read_text()); data["version"] = version
        p.write_text(json.dumps(data, indent=2) + "\n")
for rel in ("mcp.json", ".claude-plugin/mcp.json"):
    p = Path(rel)
    if p.exists():
        p.write_text(re.sub(r"(==)\d[^\"'\s]*", rf"\g<1>{version}", p.read_text()))
```

Run it with `--root <dir>` (or `cd` into the sibling first) to bump one
plugin among several; it never touches a path outside the directory it is
run from.

Then `python3 check_plugin.py` (or the copied pytest) confirms the tree
agrees with itself before the tag is pushed.

## The dev plugin's version

It is the product's version, not its own. The dev skills describe how to
maintain *this* repository at *this* version, so one bump moves both, and a
reader of `claude plugin list` sees the same number twice. The checker
flags a lagging dev version.

## Tags: no migration needed

`{plugin-name}--v{version}` git tags are required only when a `plugin.json`
declares version-constrained `dependencies` on another plugin — nothing this
generator writes does, so a repository using its own tag scheme (or none)
needs no migration to adopt any of the above. Tags matter for *resolving* a
version-constrained dependency at install time, not for a plain
`marketplace add` + `install`.
