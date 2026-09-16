# PR Type A — Fork `dev` → Upstream `dev`

This is the most common PR type: you've been working on your personal fork
and want to merge a batch of commits into the upstream project's dev branch
for review and integration.

---

## When to use this template

- **Head branch:** your fork's `dev` (e.g. `your-username/repo:dev`)
- **Base branch:** upstream `dev` (e.g. `upstream-org/repo:dev`)
- Typical trigger: a dev pre-release is ready (e.g. `v1.2.0-dev.2`)

---

## Output format

Always produce both a **title line** and a **body**, clearly separated:

```
**Title:** [vX.Y.Z-devN] <short imperative phrase summarising the main thrust>

**Body:**

## Summary
…
```

The title format is `[vX.Y.Z-devN]` followed by a concise phrase — one line,
imperative mood (e.g. "Add Zed MCP host support and fix cache refresh bug").

---

## Body structure

```
## Summary          ← 3–7 bullet points, one per top-level change
- …

---

## Features        ← omit section if no new features
### <Feature name>
…technical description, usage, implementation notes, code blocks…

## Bug Fixes       ← omit section if no bugs fixed
### <Bug name / area>
…root cause, solution, regression test coverage…

## Developer Experience  ← omit if no DX / infra / tooling changes
### <DX item>
…description…
```

**Notes on each section:**

- **Summary** is a scannable bullet list — one item per major change. Keep
  each bullet to one sentence. Reviewers grasp scope before reading details.

- **Features** sub-sections explain *what* was added and *why*, not just
  file names. Include CLI usage examples when user-visible, and implementation
  patterns when architectural.

- **Bug Fixes** sub-sections state: root cause, solution, and tests added.

- **Developer Experience** covers pre-commit hooks, CI changes, test
  infrastructure, documentation, and contributor-facing tooling.

- Omit sections entirely when empty — don't leave empty headers.

- Use a horizontal rule (`---`) to separate Summary from the detail sections.

---

## Tone

Technical and thorough. The audience is the maintainer reviewing the code.
Favour precision over brevity. Code blocks, bold emphasis on key terms, and
bullet sub-lists are all appropriate.

---

## Annotated example — Hatch PR #45 (`v0.8.0.dev2`)

*(This is from the CrackingShells/Hatch project — use as a style reference,
not a template to copy literally.)*

```markdown
**Title:** [v0.8.0.dev2] Adapter validation fix, pre-commit infrastructure, and test modernization

**Body:**

## Summary

- Fixes critical validation ordering bug in MCP adapters causing false
  rejections during cross-host sync
- Adds `--detailed` flag to `mcp sync` for field-level transparency
- Introduces pre-commit hooks for enforced code quality
- Completes CLI error message standardization
- Modernizes test suite to eliminate slow integration tests

---

## Bug Fixes

### MCP adapter validation ordering (CRITICAL)

**Root cause:** Adapters validated configuration fields **before** filtering
unsupported ones. Cross-host sync operations were rejected because target
host validators saw fields they didn't support — fields that would have been
filtered anyway.

**Example:** Claude config with `type` field → Gemini sync fails because
Gemini validator rejects `type` before filter runs.

**Solution:**
- Introduced `validate_filtered()` abstract method on `BaseAdapter`
- Enforces strict pipeline: **filter → validate → transform**
- Converted all 7 adapters

**Test coverage:**
- 64 cross-host sync pair tests
- 8 per-host configuration tests
- Field filtering and validation regression tests
```

Key things to notice:
- The title `[v0.8.0.dev2]` exactly matches the pre-release being shipped.
- Summary bullet 1 calls out severity ("CRITICAL") right up front.
- The Bug Fix section names the root cause explicitly, gives a concrete
  example, then lists the solution and test coverage.

---

## Annotated example — Hatch PR #47 (`v0.8.1.dev1`)

```markdown
**Title:** [v0.8.1.dev1] OpenCode MCP host, adding-mcp-hosts skill, and dev docs refresh

**Body:**

## Summary

- Adds full MCP host support for **OpenCode** (`~/.config/opencode/opencode.json`)
  with JSONC read/write, OAuth field support, and canonical-form serialization
- Packages the **`adding-mcp-hosts` skill** — a Claude Code agent skill that
  autonomously adds support for any new MCP host platform in 5 structured steps
- Refreshes **MCP host configuration developer docs** to match current codebase
  reality

---

## Features

### OpenCode MCP Host Support

Implements the full adapter + strategy stack for [OpenCode](https://opencode.ai).

**New fields on `MCPServerConfig`:**
- `opencode_oauth_scope` — OAuth scope string for OpenCode provider entries
- `opencode_oauth_disable` — boolean to disable OAuth for a provider entry

**New `OpenCodeAdapter`** (`hatch/mcp_host_config/adapters/opencode.py`):
- `serialize()` produces canonical OpenCode JSON
- `validate_filtered()` enforces the `filter → validate → transform` pipeline
```

Key things to notice:
- New fields and classes are named with inline code so they're greppable.
- External projects get hyperlinks.

---

## Fill-in template

```markdown
**Title:** [vX.Y.Z-devN] <short imperative phrase>

**Body:**

## Summary

- [Top-level change 1 — one sentence]
- [Top-level change 2]
- [Top-level change 3]

---

## Features

### [Feature / module / command name]

[What it does and why it matters. 1–3 sentences.]

[New fields / classes / methods — inline code + bullet list:]
- `field_name` — what it represents
- `ClassName` (`path/to/file.py`) — what it does

[Usage example if user-visible:]
```bash
[command] [--flag]
```

[Test coverage:]
- [Number and type of new tests]

---

## Bug Fixes

### [Bug area or descriptive name]

**Root cause:** [One sentence.]

**Example:** [what triggered the bug] → [what went wrong]

**Solution:**
- [Fix step 1]

**Test coverage:**
- [Regression test description]

---

## Developer Experience

### [DX item name]

[Description — what was added/changed and why it helps contributors.]
```
