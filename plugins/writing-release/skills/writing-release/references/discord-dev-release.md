# Discord — Dev Pre-release Announcement

These messages are posted to your project's Discord server when a new dev
pre-release (e.g. `v1.2.0-dev.3`) is published to the package registry.

---

## When to use this template

- A new `vX.Y.Z-devN` version is published
- Post to the project's releases/announcements channel

---

## Two sub-types

### Sub-type D1 — First dev of a new minor version (e.g. `v1.2.0-dev1`)

Debut of a new development cycle. Can be slightly longer and more
enthusiastic because it introduces a whole set of new capabilities.

**Structure:**
```
# <emoji> <ProjectName> vX.Y.Z-devN is Here!    ← H1 header (dark-bg emoji)

[Optional greeting — "Hey [community name]!" etc.]

[1–2 sentence context: what this release is and why it's still a dev release]

Install with:
```bash
pip install <package-name>==vX.Y.Z-dev.N
```

Here's what's new:

## <emoji> [Feature Section Title]    ← H2 per major feature

[2–4 sentences of user-friendly description]

```bash
[Code example if applicable]
```

[Optional bullet sub-list]

[Friendly closing line]
```

### Sub-type D2 — Subsequent devs of the same minor (e.g. `v1.2.0-dev.3`)

Shorter and more direct. No greeting needed — get to the point fast.

**Structure:**
```
## <emoji> *<ProjectName>!* vX.Y.Z-devN          ← H2 header

[Optional 1-sentence context]

### [Feature / Fix title]
[2–3 sentence description. Bold the change name, explain what it does
and why it matters.]

[Optional code block]

[Short friendly closing with emoji]
```

---

## Emoji guide

Discord custom emoji look like `<:emoji_name:numeric_id>`. The examples
below are from the Hatch project — **replace with your own server's custom
emoji codes**. If you don't have custom emoji, use standard Unicode (✨, 🚀,
🐛, 🔧, etc.).

**Hatch project emoji (for reference only):**

| Emoji | Code | Typical use |
|---|---|---|
| Hatch icon (dark bg) | `<:hatch_icon_dark_bg_transparent:1412018750739451974>` | Most dev announcements |
| Hatch icon (light bg) | `<:hatch_icon_light_bg_transparent:1412018696561623120>` | Occasional variation |
| Hatchling (dark bg) | `<:hatchling_core_dark_bg:1412018635081252884>` | Closings, happy moments |
| Hatchling (light bg) | `<:hatchling_core_light_bg:1412018539379949589>` | Closings, good nights |

Section headers use standard Unicode emoji (✨, 📊, 🔍, 🛡️, 📚, 🔧, etc.)
chosen to match the content.

---

## Tone

- Warm and personal — write in first person ("I've been working on…")
- Accessible — assume readers use the tool but don't read every PR
- Technical enough to be useful — show commands and output snippets
- Honest — it's fine to note limitations or explain why something is a dev
  release
- Focused — don't list every commit, just the things users will notice

---

## Annotated example — Hatch `v0.8.0-dev1` (Sub-type D1)

```markdown
# <:hatch_icon_dark_bg_transparent:1412018750739451974> Hatch v0.8.0-dev1 is Here!

Hey Hatchers!

(this was supposed to be `v0.7.2-dev1` but a `BREAKING CHANGE` flag eluded
me... oh well, `v0.8.0-dev1` it is!)

I've been working on making *Hatch!* easier and more pleasant to use.
This is a dev release because I still want to refresh the automated testing
infrastructure; but the user-side features are stable. Install with:
```bash
pip install hatch-xclam==v0.8.0-dev.1
```

## ✨ Colorful, Consistent Output

All commands now have a unified look with color-coded output:
- 🟢 Green for creating things (CREATE, ADD, INSTALL)
- 🔴 Red for removing things (DELETE, REMOVE)
…

Happy hatching! 🐣
```

Key things to notice:
- The aside about the version number is personal and honest.
- Each feature section: `## <emoji> Title` — emoji chosen to match content.
- Code blocks show real terminal output, not just commands.
- Install command is given early, before the feature list.

---

## Annotated example — Hatch `v0.8.1-dev.1` (Sub-type D2, compact)

```markdown
## <:hatch_icon_dark_bg_transparent:1412018750739451974> *Hatch!* v0.8.1-dev.1 is out!
Three things landed in:
### [OpenCode](https://opencode.ai) support
`hatch mcp` can now read and write `~/.config/opencode/opencode.json`.
### `adding-mcp-hosts` skill
An agent skill that walks through adding any new MCP host in 5 steps.
### Dev docs refresh
The architecture doc and extension guide are back in sync with the codebase.

Happy hatching <:hatchling_core_dark_bg:1412018635081252884>
```

Key things to notice:
- Ultra-compact: "Three things landed in:" then `###` sections immediately.
- External products get hyperlinks.
- Very short closing with emoji.

---

## Fill-in template — Sub-type D1

```markdown
# <emoji> <ProjectName> vX.Y.Z-devN is Here!

[Optional greeting]

[1–2 sentences: what this release is about and why it's still a dev release]

Install with:
```bash
pip install <package-name>==vX.Y.Z-dev.N
```

Here's what's new:

## <emoji> [Feature 1 Title]

[2–4 sentence user-friendly description]

[Optional code block]

[Friendly closing]
```

---

## Fill-in template — Sub-type D2

```markdown
## <emoji> *<ProjectName>!* vX.Y.Z-devN

[Optional: "N things in this one:" or a one-line teaser]

### [Feature / Fix 1]
[2–3 sentence description.]

### [Feature / Fix 2]
[Description]

[Short friendly closing with emoji]
```
