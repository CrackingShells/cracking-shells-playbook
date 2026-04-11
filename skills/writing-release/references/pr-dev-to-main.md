# PR Type B — Upstream `dev` → Upstream `main`

This PR marks a versioned release. It aggregates everything that landed in
`dev` since the last main-branch release and presents it as a coherent
changelog for the project's main history.

---

## When to use this template

- **Head branch:** upstream `dev` (e.g. `upstream-org/repo:dev`)
- **Base branch:** upstream `main` (e.g. `upstream-org/repo:main`)
- Typical trigger: a stable version is ready (e.g. `v1.2.0`) or a small
  feature/fix goes directly to main (e.g. `v1.2.1`)

---

## Output format

Always produce both a **title line** and a **body**:

```
**Title:** [vX.Y.Z] <release title — summarises the theme of this release>

**Body:**

## Overview
…
```

The title format is `[vX.Y.Z]` followed by a phrase describing the release
theme (e.g. "MCP Host Support Additions and CI Streamlining").

---

## Body structure

```
## Overview          ← 1–3 sentence paragraph summarising the release.
                        Mention which dev iterations contributed if applicable.

## [Major Theme 1]   ← group related changes under meaningful headings
### [Sub-item] (#PR-number)
…

## Bug Fixes         ← consolidated across all dev iterations
- **[Bug name]** (#PR-number) — [one-sentence description]

## Documentation Updates  ← if significant

## Developer Experience   ← CI, tooling, testing infra

## Breaking Changes  ← ONLY if there are any; omit section otherwise
**[What changed]**
Migration: [what the user needs to do]
```

**Notes:**

- **Overview** reads like a one-paragraph release summary — it tells the
  story of what this version is *about*.

- **Section headings** should reflect the product dimension, not PR
  mechanics (e.g. "MCP Host Support Additions" not "Merged PRs"). This makes
  the changelog readable without knowing the PR history.

- **Cross-reference dev PRs** with `(#N)` inline so anyone can trace a
  feature back to its detailed dev PR.

- **Breaking Changes** always include a migration path. If there are none,
  omit the section entirely — never write "No breaking changes."

- **Tone:** technical and precise but oriented toward the release record
  rather than an individual reviewer. Slightly more polished than Type A.

---

## Annotated example — Hatch PR #51 (`v0.8.1`)

*(Style reference from the CrackingShells/Hatch project.)*

```markdown
**Title:** [v0.8.1] MCP Host Support (3 new hosts), CLI improvements, CI streamlining

**Body:**

## Features

### MCP Host Support Additions
- **OpenCode MCP Host** (#47)
  Adds full adapter + strategy for OpenCode with JSONC read/write, OAuth
  field support, and canonical-form serialization. Includes new fields
  (`opencode_oauth_scope`, `opencode_oauth_disable`).
- **Augment Code MCP Host** (#48)
  Implements full adapter contract for Augment Code with field validation
  and integration across all host points.
- **Mistral Vibe MCP Host** (#49)
  Introduces complete MCP host support with new discriminator fields
  (`transport`, `prompt`, `sampling_enabled`, etc.) and CLI flag mappings.

---

## Bug Fixes

- **Claude URL transport serialization** (#49) — `ClaudeAdapter.serialize()`
  now explicitly includes `"type": "http"` for URL-based remote configs.
- **JSONC comment regex fix** (#47) — Anchored regex to line start to prevent
  stripping `#` characters from inline string values.

---

## Developer Experience

- **Adding MCP Hosts Agent Skill** (#47) — Packages a skill enabling an LLM
  agent to autonomously add support for new MCP host platforms in 5 steps:
  Discover, Declare, Adapt, Strategize, Wire & Test.
- **CI pipeline streamlining** (#49) — Consolidated release/publish/Discord
  notification workflows into a single orchestrated pipeline.
- **Global `--log-level` flag** (#48):
  ```bash
  hatch --log-level {DEBUG,INFO,WARNING,ERROR} <command>
  ```
```

Key things to notice:
- Features are grouped under a category heading, not one-per-section.
- Each bullet: bold name → PR number → concise description.
- Bug Fixes are a flat bullet list (not sub-sections) since individually small.

---

## Annotated example — Hatch PR #46 (`v0.8.0`, larger release)

```markdown
**Title:** [v0.8.0] CLI Architecture Refactoring, MCP Adapter System, and UX Normalization

**Body:**

## Overview

Major architectural evolution introducing modular CLI architecture, unified
MCP adapter system, and comprehensive UX improvements. Combines dev1
(architecture refactoring) and dev2 (validation fixes, tooling) while
maintaining backward compatibility.

## Major Changes

### 1. CLI Architecture Refactoring
Refactored monolithic `cli_hatch.py` (2000+ LOC) into modular handler-based
architecture with backward compat shim (removal in v0.9.0).

### 2. Unified MCP Adapter System
Pipeline: **filter → validate → transform** enforced across all adapters.
- **Breaking:** Removed legacy models (`MCPServerConfigGemini`, etc.)

---

## Breaking Changes

**Removed Models:**
- `MCPServerConfigBase`, `MCPServerConfigGemini`, `MCPServerConfigVSCode`, …

**Migration:** Use `MCPServerConfig` with adapters from
`your_package.adapters`
```

Key things to notice:
- The Overview paragraph sets context ("Combines dev1 and dev2").
- Numbered `### 1.` headings used when changes are large and architectural.
- Breaking Changes: what was removed AND what to use instead.

---

## Fill-in template

```markdown
**Title:** [vX.Y.Z] <release theme>

**Body:**

## Overview

[1–3 sentences. What is this release about? If it rolls up multiple dev
iterations, say which ones.]

---

## [Main Feature Category]

### [Sub-feature] (#PR-number)
[2–4 sentence description. Key technical details, new fields/commands/flags.]

### [Another sub-feature] (#PR-number)
…

---

## Bug Fixes

- **[Bug name]** (#PR-number) — [One sentence: what was wrong and the fix.]

---

## Documentation Updates

- **[Doc area]** (#PR-number) — [What was added or corrected.]

---

## Developer Experience

- **[DX item]** (#PR-number) — [Description.]

---

## Breaking Changes

**[What changed]**
- `OldClassName` — replaced by `NewClassName`

**Migration:** [What the user needs to change.]
```

*(Delete sections that have nothing to report. Overview + at least one
content section are always required.)*
