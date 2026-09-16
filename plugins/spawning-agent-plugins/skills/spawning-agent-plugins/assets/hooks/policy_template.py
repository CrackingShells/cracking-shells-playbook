#!/usr/bin/env python3
"""This plugin's hooks, in one script dispatched on `hook_event_name`.

Runs under whatever interpreter `uv run --no-project python` resolves, so it
is stdlib-only and Python >= 3.8 syntax, and it must cost a bare interpreter
start and nothing more: a `PreToolUse` branch runs on every tool call it
matches.

Events:
  SessionStart, SubagentStart  -> a short policy as `additionalContext`
  PreToolUse <matcher>         -> allow silently, or deny with a reason the model reads
  <non-portable events>        -> one file each, hooks/<event-kebab>.json, named from the Claude manifest only

Every branch fails open: unparsable stdin, an unknown event or any exception
exits 0 with no output. A hook that crashes must never make a session
unusable, so the only observable failure is "the block did not happen".
The deny is the JSON `permissionDecision` form because Claude Code, Codex and
Cursor all carry its reason to the model; exit code 2 reaches the model on
only two of the three.
"""

from __future__ import annotations

import json
import sys

#: Keep this under ~2 000 characters: Codex truncates `additionalContext` at
#: roughly 2 500 tokens per handler, and context from several hooks adds up.
#: Carry the rule and the tool names; the *how* belongs in the skill or the
#: server's instructions.
POLICY = """\
POLICY (<plugin-name> plugin hook)

TODO: state in a few lines what this session should and should not do,
naming the tools to reach for instead.
"""


def emit(payload: dict) -> None:
    sys.stdout.write(json.dumps(payload))


def context(event: str) -> None:
    emit({"hookSpecificOutput": {"hookEventName": event, "additionalContext": POLICY}})


def deny(reason: str) -> None:
    emit(
        {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        }
    )


def decide_pre_tool_use(data: dict) -> None:
    tool = data.get("tool_name", "")
    tool_input = data.get("tool_input") or {}
    # Cursor's Claude-Code import maps Bash to Shell and may carry `command` at the top level.
    command = tool_input.get("command") or data.get("command") or ""
    if tool in ("Bash", "Shell") and command:
        # TODO: pattern-match the command; call deny(...) with the alternative, or return.
        return
    if tool == "Grep":
        # TODO: same for the built-in Grep tool.
        return


def handle(data: dict) -> None:
    event = data.get("hook_event_name", "")
    if event in ("SessionStart", "SubagentStart"):
        context(event)
    elif event == "PreToolUse":
        decide_pre_tool_use(data)
    # elif event == "WorktreeRemove":   # not every harness knows it; it lives in hooks/worktree-remove.json
    #     ...


def main() -> None:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return
    if not isinstance(data, dict):
        return
    try:
        handle(data)
    except Exception:  # noqa: BLE001 - a hook must never take the session down with it
        return


if __name__ == "__main__":
    main()
    sys.exit(0)
