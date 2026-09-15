# Shipping hooks with the plugin

Hooks are how a plugin changes what the harness *does* rather than what the
model knows: inject a policy at session start, deny a tool call with a reason
the model reads, clean up when a worktree goes. colgrep-mcp's hooks
(`hooks/colgrep_policy.py`) are the worked example; the template at
`assets/hooks/policy_template.py` is their shape with the domain removed.

## One file per event class, one script {#one-file-per-event-class}

`hooks/hooks.json` holds only events every hook-capable harness documents:

```
SessionStart  SessionEnd  SubagentStart  SubagentStop
PreToolUse  PostToolUse  UserPromptSubmit  PreCompact  Stop
```

Every other event (`WorktreeRemove`, and whatever Claude Code adds next)
gets its own file named after it in kebab-case: `hooks/worktree-remove.json`
holds `WorktreeRemove` and nothing else. Two reasons. It is unverified
whether a Codex or Cursor parser that meets an unknown event drops that
event or the whole file, so the unknown never reaches them. And
`hooks.json` is the one name both loaders claim unasked (below), so a second
generic name beside it, the former `claude-code.json`, is the kind a loader
could claim next; a name derived from the event is claimed by nothing.
`check_plugin.py` pins the split and the stems.

The two manifests then read the same `hooks` field with opposite semantics,
and no single value serves both:

| Loader | `hooks/hooks.json` | The manifest's `hooks` field | So the manifest says |
|:--|:--|:--|:--|
| Claude Code | always loaded | *additional* files; naming the default again fails the whole plugin at marketplace install ("Duplicate hooks file detected", Claude Code 2.1.270) | exactly the per-event files: `"./hooks/worktree-remove.json"` (a string for one, an array for several), omitted when every event is portable |
| Codex | discovered only when the manifest defines no `hooks` | *replaces* the discovery | `"./hooks/hooks.json"` and nothing else, or no field |

The Claude Code failure is invisible before the install: `claude
--plugin-dir`, `claude plugin details` and `claude plugin validate` all
accept a manifest naming `./hooks/hooks.json`; only a marketplace install
rejects it (colgrep-mcp shipped that manifest from 0.4.0 to 0.5.0 with every
tree-level gate green). `check_plugin.py` fails on it; to check the way an
install does, register a scratch directory marketplace whose plugin `source`
is a copy of the tree without `.git` and `.venv`, install from it, read
`claude plugin list`, and remove both afterwards.

Shape, identical in Claude Code, Codex and Cursor's Claude-Code import:

```json
{"hooks": {"<Event>": [{"matcher": "<regex>", "hooks": [{"type": "command", "command": "...", "timeout": 10}]}]}}
```

`matcher` is a regex on the tool name and is omitted for events without a
tool (`SessionStart`, `SubagentStart`). `statusMessage` is a Claude Code
nicety shown while the hook runs; harmless elsewhere. `timeout` is seconds;
size it to the work (10 for a policy echo, 90 for something that runs a
binary).

Every handler is the same command, differing only in nothing:

```
uv run --no-project --quiet python "${CLAUDE_PLUGIN_ROOT}/hooks/<name>_policy.py"
```

- `uv` because the plugin already requires it when it launches a server with
  `uvx`, and because `python3` is not a given on Windows. Measured: 0.09 s per
  call against 0.68 s for launching the packaged server (`uvx pkg==v hook`),
  which matters because a `PreToolUse` hook runs on every matched call.
- `--no-project` keeps `uv` from syncing whatever project the session's cwd is
  in; `--quiet` keeps its output off the JSON channel.
- `${CLAUDE_PLUGIN_ROOT}` is the one placeholder; Claude Code expands it, and
  Codex sets the same variable for plugin hooks. Quote the path.
- One script for every event, dispatched on `hook_event_name`, so a change to
  the policy is one file and one test.

## Matcher names per harness

Codex matches its shell as `Bash` and MCP tools as `mcp__<server>__<tool>`;
Cursor maps `Bash` to `Shell` and keeps `Grep`; both deliver the command in
`tool_input.command`, and Cursor may also carry `command` at the top level.
A matcher of `Grep|Bash` is therefore right for Claude Code, inert on the
`Grep` half in Codex (no such tool), and needs the script to accept
`tool_name == "Shell"` for Cursor. The template does.

## The script's I/O contract

One JSON object on stdin, JSON or nothing on stdout, exit 0. Fields read:
`hook_event_name`, `tool_name`, `tool_input.command`, `tool_input.path`,
`cwd`, and for `WorktreeRemove` `worktree_path`.

| Event | stdout | Why this form |
|:--|:--|:--|
| `SessionStart`, `SubagentStart` | `{"hookSpecificOutput": {"hookEventName": <echoed>, "additionalContext": "..."}}` | echo the event name; Codex checks it |
| `PreToolUse`, deny | `{"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny", "permissionDecisionReason": "..."}}` | the reason reaches the model on Claude Code, Codex and Cursor; exit code 2 with stderr reaches it on only two |
| `PreToolUse`, allow | nothing | silence is allow |
| anything else, bad stdin, own exception | nothing, exit 0 | a hook that crashes makes the session unusable; the only acceptable failure is "the block did not happen" |

Keep the injected context under about 2 000 characters. Codex truncates
`additionalContext` around 2 500 tokens per handler and several plugins'
hooks add up; carry the *rule* and the *tool names*, and leave the *how* to
the skill, the server's `instructions` or a resource. State the escape hatch
in the deny reason (colgrep-mcp: `COLGREP_BYPASS=1`), or the model has no
sanctioned way past a false positive.

## Constraints on the script

- Stdlib only, Python ≥ 3.8 syntax: `uv run --no-project python` installs
  nothing and may pick the system interpreter. `check_plugin.py` parses the
  imports.
- No subprocess on the common path. Regex-test the command first and spawn
  (`git rev-parse`, a binary) only after a pattern matched; a `PreToolUse`
  hook on `Bash` runs for every shell call in the session.
- Treat temp directories and platform state trees (`~/Library`, `AppData`,
  the system temp) as not-a-corpus if the gate is "inside a source tree".
  The first Windows CI run of colgrep-mcp's hook denied a grep inside a
  pytest `tmp_path` because Windows' temp lives under the home directory.
- Never name a harness-prefixed tool id in messages
  (`mcp__plugin_x_y__search`): the prefix differs between a plugin install
  and `claude mcp add`. Name the tool and the server.

## Testing without a harness

Pipe events in and read JSON out; that is exactly what the harness does.

```bash
echo '{"hook_event_name":"SessionStart","cwd":"/"}' | python3 hooks/<name>_policy.py
echo '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"grep -rn foo ."},"cwd":"'"$PWD"'"}' | python3 hooks/<name>_policy.py
echo garbage | python3 hooks/<name>_policy.py; echo $?     # nothing, 0
```

In pytest, run the script through `sys.executable` with `subprocess.run`, so
the Windows CI job is the portability oracle. colgrep-mcp's
`server/tests/test_hooks.py` is the pattern: behaviour tests through the
pipe, drift guards on the hook files and on what each manifest names
(`check_plugin.py` carries the generic ones).

## Loading and reloading, per harness

| Harness | Loads plugin hooks from | After an edit |
|:--|:--|:--|
| Claude Code | `hooks/hooks.json` always, plus the files the manifest's `hooks` names, at plugin load | `/reload-plugins` or a new session; a marketplace install runs the cached copy under `~/.claude/plugins/cache/`, `claude --plugin-dir <tree>` runs the tree |
| Codex | `hooks/hooks.json` when the manifest defines no `hooks`; otherwise the manifest's path and only that | skips plugin hooks until reviewed and trusted in `/hooks`; re-asks when the definition's hash changes |
| Cursor | `settings.json` files only, never a plugin | copy the portable entries into the project's `.claude/settings.json` |
