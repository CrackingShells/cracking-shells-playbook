# Discord — Stable Release Announcement

These messages are posted when a stable version (e.g. `v1.2.0`) merges to
`main` and is published to the package registry.

---

## When to use this template

- The dev-to-main PR has been merged
- The release is live on PyPI / npm / etc.
- Post to the project's releases/announcements channel

---

## Structure

```
## <emoji> *<ProjectName>!* vX.Y.Z is now live!    ← H2 header
                                                       (light-bg or org emoji)
[1–2 sentence intro. Reference the dev releases that led here, with Discord
links if you have them.]

### ✨ [Summary heading]
- [User-facing bullet 1]
- [User-facing bullet 2]
- …

### ⚠️ Heads up    ← ONLY if there are breaking changes
- [Breaking change — state impact and migration in one sentence]
- [Backward compat note if applicable]
```

---

## Key differences from dev announcements

| | Dev announcement | Stable announcement |
|---|---|---|
| Header level | `#` (D1) or `##` (D2) | `##` |
| Length | Medium–long (D1) or short (D2) | Short–medium |
| Tone | Personal, can be chatty | Slightly more polished, still warm |
| Audience | Early adopters / devs | All users |
| Technical detail | High | Low — translate to user impact |
| Install command | Always | Optional (users know how) |

The stable announcement is for someone who didn't follow the dev iterations.
Translate technical changes into user-visible benefits:
- PR: "Fixed validation ordering bug in MCP adapters"
- Discord: "Fixed critical bug where cross-host MCP sync would incorrectly reject valid configs"

---

## Tone

- Welcoming and inclusive — "Here it is" / "Fresh from dev to main"
- Focus on *what users can now do* rather than *what code changed*
- Link back to dev announcements for readers who want details
- If there's a breaking change: honest and reassuring at the same time
- Keep it short — users who want details have the dev announcements

---

## Annotated example — Hatch `v0.8.0`

```markdown
## <:hatch_icon_light_bg_transparent:1412018696561623120> *Hatch!* v0.8.0 is now live!

Here it is, as announced 🙂
We've merged dev to main, bringing together all the improvements from
[dev1](https://discord.com/channels/…) and [dev2](https://discord.com/…)
into a stable release:

### ✨ Summary of what's new for you
- New commands for better visibility: `env show`, `mcp show hosts/servers`
- `mcp sync --detailed` shows exactly what changed during sync operations
- Improved output formatting with clearer colors and tables
- Fixed critical bug where cross-host MCP sync would incorrectly reject valid configs
- Better error messages that actually help you fix issues
- Updated documentation with correct command examples

### ⚠️ Heads up
- A few commands are deprecated (you'll see warnings with replacement commands)
- Your existing workflows still work — backward compatibility maintained
```

Key things to notice:
- "Here it is, as announced 🙂" — callbacks to the "What's Next" in the
  last dev announcement create continuity.
- Bullets are user-benefit statements, not implementation details.
- `⚠️ Heads up` softens the breaking change and adds immediate reassurance.

---

## Annotated example — Hatch `v0.8.1` (no breaking changes)

```markdown
## <:cs_icon_light_bg:1412018838911844362> *Hatch!* v0.8.1 is now live!
Fresh from `dev` to `main`, this update brings together all the latest
improvements:

### ✨ What's new for you
- Meet 3 new MCP hosts: OpenCode, Augment Code, and Mistral Vibe
- Refreshed documentation — new style at [hatch.readthedocs.io](https://hatch.readthedocs.io/en/latest/)
- Smoother workflows: a global `--log-level` flag makes debugging easier
- Fix: enabled configuration of Claude Code and Claude Desktop `http` transport
```

Key things to notice:
- No `⚠️ Heads up` section — omit entirely when no breaking changes.
- "Meet 3 new MCP hosts" is punchy and user-facing.
- Documentation refresh is linked so users can explore it.

---

## Annotated example — Hatch `v0.8.2` (small single-feature release)

```markdown
## <:hatch_icon_light_bg_transparent:1412018696561623120> *Hatch!* v0.8.2 is now live!

Yes, would you believe it, two releases in two days 🐣

### Easier Filtering in listing commands

We've just added a `filter_name` positional argument — simpler to use than
flag-based filtering:

**Before:** `hatch mcp list servers --host claude-desktop`
**After:** `hatch mcp list servers weather-server`
```

Key things to notice:
- Small releases can describe a single feature directly without a `### ✨`
  summary heading — adapt the format to the content.
- A before/after comparison makes UX improvements immediately obvious.
- Personal voice ("would you believe it") is welcome even in stable releases.

---

## Fill-in template

```markdown
## <emoji> *<ProjectName>!* vX.Y.Z is now live!

[1–2 sentence intro — reference dev releases with links if applicable]

### ✨ [Summary heading, e.g. "What's new for you"]
- [User-facing benefit 1]
- [User-facing benefit 2]
- [Bug fix translated to user impact, if notable]

### ⚠️ Heads up    ← DELETE if no breaking changes
- [Breaking change — impact and mitigation in one sentence]
- [Backward compat note]
```
