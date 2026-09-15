# One version, everywhere it appears

The plugin version is written literally into `plugin.json`,
`.claude-plugin/plugin.json`, `.codex-plugin/plugin.json`,
`dev/.claude-plugin/plugin.json`, and into the pin of every MCP launch
(`uvx pkg==<version>` in three `mcp.json` files). A release that moves the
package version and not these ships a plugin whose manifest says the new
version and whose server is the old one; worse, `uvx` keys its cached
environment on the requirement string, so an unmoved pin reuses a stale
environment forever. The generator writes the resolved version once; keeping
it moving is the release tooling's job, and `check_plugin.py` is the alarm
when nothing else does it.

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
    "../.codex-plugin/plugin.json:\"version\"",
    "../dev/.claude-plugin/plugin.json:\"version\"",
    "../.claude-plugin/mcp.json:<pkg>==",
    "../.codex-plugin/mcp.json:<pkg>==",
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
           ".codex-plugin/plugin.json", "dev/.claude-plugin/plugin.json",
           "mcp.json", ".claude-plugin/mcp.json", ".codex-plugin/mcp.json"]
}]
```

where `set_plugin_version.py` is the twelve-line rewrite below. In a
`multi-semantic-release` monorepo of skills, the plugin version is the
*repository's* release, not one skill's: give the root its own release
config or bump the manifests from the aggregate step.

## npm alone

`"version": "python3 scripts/set_plugin_version.py $npm_package_version && git add plugin.json .claude-plugin .codex-plugin mcp.json dev"`
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
for rel in ("plugin.json", ".claude-plugin/plugin.json", ".codex-plugin/plugin.json", "dev/.claude-plugin/plugin.json"):
    p = Path(rel)
    if p.exists():
        data = json.loads(p.read_text()); data["version"] = version
        p.write_text(json.dumps(data, indent=2) + "\n")
for rel in ("mcp.json", ".claude-plugin/mcp.json", ".codex-plugin/mcp.json"):
    p = Path(rel)
    if p.exists():
        p.write_text(re.sub(r"(==)\d[^\"'\s]*", rf"\g<1>{version}", p.read_text()))
```

Then `python3 check_plugin.py` (or the copied pytest) confirms the tree
agrees with itself before the tag is pushed.

## The dev plugin's version

It is the product's version, not its own. The dev skills describe how to
maintain *this* repository at *this* version, so one bump moves both, and a
reader of `claude plugin list` sees the same number twice. The checker
flags a lagging dev version.
