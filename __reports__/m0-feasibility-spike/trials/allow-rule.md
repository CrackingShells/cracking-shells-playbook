# Allow-rule string

Read from real `claude -p` sessions on 2026-11-16, CLI version 2.1.316.

| How the server is loaded | Tool name, and therefore the allow-rule string |
|:--|:--|
| Project `.mcp.json`, server `rdm` | `mcp__rdm__roadmap_view` |
| Plugin `managing-roadmaps` (local dev copy through `--plugin-dir`), server `rdm` | `mcp__plugin_managing-roadmaps_rdm__roadmap_view` |

- **The hyphen survives:** the plugin form is `mcp__plugin_<plugin>_<server>__<tool>`, and the hyphen in `managing-roadmaps` is kept as is.
- **A same-named project server wins:** when a project `.mcp.json` also defines `rdm`, that project server takes over and the plugin name never appears. The first plugin check, run in the trial folder, showed `mcp__rdm__roadmap_view` for that reason.
