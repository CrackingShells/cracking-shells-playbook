#!/usr/bin/env python3
"""Guard the regeneration claim in SKILL.md as an executable tripwire.

SKILL.md claims that `assets/examples/colgrep-mcp.spec.json` regenerates
colgrep-mcp's manifests and hook files with identical content. This turns
that prose claim into a command anyone can run: it drives `spawn_plugin.py`
in `--dry-run` mode against a real colgrep-mcp checkout and asserts that
nothing would be written or merged, and that nothing is skipped beyond the
one measured, allowed divergence.

Run directly (no pytest required — `spawn_plugin.py` and `check_plugin.py`
are both stdlib-only, so importing them adds no extra dependency):

    uv run skills/spawning-agent-plugins/evals/test_regeneration.py

Or let pytest collect it (`test_spec_regenerates_manifests` is a plain
function using bare `assert` and `unittest.SkipTest`, both of which pytest
understands natively):

    uv run --with pytest pytest skills/spawning-agent-plugins/evals/test_regeneration.py

`evals/` sits at the skill root, which `tools/package_skill.py`
(`ROOT_EXCLUDE_DIRS = {"evals"}`) excludes when packaging — this guard is a
maintainer check and never ships to consumers.
"""

from __future__ import annotations

import os
import re
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SKILL_ROOT = HERE.parent

sys.path.insert(0, str(SKILL_ROOT / "scripts"))
from spawn_plugin import load_spec, spawn  # noqa: E402  (needs the sys.path.insert above)

SPEC_PATH = SKILL_ROOT / "assets" / "examples" / "colgrep-mcp.spec.json"

# Re-baselined 2026-09-16 against colgrep-mcp ef54b8f (v0.5.1), inside the
# generator_reshape leaf's step 3 (com.openai extensions namespace) commit.
#
# Before step 3: the invariant held for every manifest and hook file, with the
# ONLY divergence being this hand-maintained prose file.
#
# Step 3 moves the `interface` block and any Codex `hooks` value out of
# `.codex-plugin/plugin.json` (no longer written by any code path) and into
# `extensions["com.openai"]` of the root `plugin.json` — deliberately, per the
# leaf spec ("Codex parses a root Agent-Plugins-conformant plugin.json ... reads
# its Codex-specific data from extensions[\"com.openai\"]"). colgrep-mcp's own
# `plugin.json` on disk still holds the pre-reshape shape (no `extensions` key)
# until a later leaf (`nest_migration/regenerate_manifests` in colgrep-mcp's own
# roadmap) regenerates it with the extended generator — that leaf is explicitly
# out of scope here, so `plugin.json` is added to ALLOWED_DIVERGENCE rather than
# silently dropped or the guard loosened wholesale. Confirmed empirically: dry
# run now reports exactly `kept plugin.json [keys: extensions]` plus the
# pre-existing `dev/README.md`, and nothing else — no .codex-plugin/* write or
# skip appears at all, because the generator no longer references that path.
#
# Re-baselined again 2026-09-16, hub_mode_reconciliation leaf's step 1: the
# example spec now declares hub mode (`"marketplace": {"hub": "..."}`), so
# `spawn()` never calls `merge_marketplace` and the guard no longer depends on
# colgrep-mcp keeping the two local marketplace files its own
# `relinquish_marketplace` leaf is about to delete — verified by simulating
# that deletion against a scratch copy of the checkout: without hub mode the
# dry run reports `would write .claude-plugin/marketplace.json` and
# `would write .agents/plugins/marketplace.json`; with it, neither line
# appears. That leaf's spec text also instructed dropping the `plugin.json`
# entry below as "now inert", on the claim that colgrep-mcp's manifest
# already carries `extensions`. That claim is false at this same commit
# (ef54b8f): `plugin.json` on disk still has no `extensions` key (see the
# paragraph above, written for the exact same commit), and dropping the
# entry was confirmed empirically to turn this guard red with
# `unexpected divergence from the spec: ['plugin.json  [keys: extensions]']`.
# The entry stays until `nest_migration/regenerate_manifests` actually lands
# in colgrep-mcp — dropping it now would be the silent re-baseline this
# guard exists to prevent, not a cleanup.
#
# Key-scoped, not file-scoped: `None` is a whole-file exemption (dev/README.md
# is hand-maintained prose with no JSON keys to compare); a set restricts the
# exemption to those top-level keys. A file-scoped allowlist (bare
# `{"plugin.json"}`) would silently absorb ANY future divergence in
# plugin.json — a corrupted `name`, `version` or `license` included — as "the
# expected extensions divergence". `_parse_skipped` below reads the `[keys:
# ...]` suffix `Writer._key_diff` reports and this is enforced as a subset
# check, not just logged in the failure message.
ALLOWED_DIVERGENCE: dict[str, set[str] | None] = {
    "dev/README.md": None,
    "plugin.json": {"extensions"},
}

_SKIPPED_RE = re.compile(r"^(?P<rel>.*?)(?:  \[keys: (?P<keys>.*)\])?$")


def _parse_skipped(entry: str) -> tuple[str, set[str] | None]:
    """Split one `Writer.skipped` entry into its path and the differing top-level
    keys (`None` when the file carries no `[keys: ...]` suffix at all, i.e. a
    non-JSON file or one whose parsed content is actually identical)."""
    match = _SKIPPED_RE.match(entry)
    rel = match.group("rel")
    keys = match.group("keys")
    return rel, ({k.strip() for k in keys.split(",")} if keys else None)

# Where to find the colgrep-mcp checkout when COLGREP_MCP_ROOT is unset.
# Resolved against this skill's root (spawning-agent-plugins/), not against
# this file's own directory: colgrep-mcp is a sibling of the playbook repo
# under CrackingShells/, and three levels up from the skill root lands there.
DEFAULT_CHECKOUT = "../../../colgrep-mcp"


def checkout_root() -> Path | None:
    """The colgrep-mcp checkout to test against, or None if none is present."""
    override = os.environ.get("COLGREP_MCP_ROOT")
    candidate = Path(override) if override else (SKILL_ROOT / DEFAULT_CHECKOUT)
    candidate = candidate.resolve()
    return candidate if candidate.is_dir() else None


def test_spec_regenerates_manifests() -> None:
    """`spawn --dry-run` against colgrep-mcp writes nothing, merges nothing,
    and skips nothing beyond the one measured, allowed divergence."""
    root = checkout_root()
    if root is None:
        location = os.environ.get("COLGREP_MCP_ROOT") or str((SKILL_ROOT / DEFAULT_CHECKOUT).resolve())
        raise unittest.SkipTest(f"no colgrep-mcp checkout at {location}; set COLGREP_MCP_ROOT to test against one")

    spec = load_spec(SPEC_PATH)
    w = spawn(spec, root, force=False, dry_run=True)

    assert w.written == [], f"spec would write new files — generator and checkout have drifted: {w.written}"
    assert w.merged == [], f"spec would merge marketplace entries — generator and checkout have drifted: {w.merged}"

    unexpected = []
    for entry in w.skipped:
        rel, keys = _parse_skipped(entry)
        if rel not in ALLOWED_DIVERGENCE:
            unexpected.append(entry)
            continue
        allowed_keys = ALLOWED_DIVERGENCE[rel]
        if allowed_keys is None:
            continue  # whole-file exemption (e.g. hand-maintained prose)
        if keys is None or not keys <= allowed_keys:
            unexpected.append(entry)
    assert not unexpected, f"unexpected divergence from the spec: {unexpected} (full detail: {w.skipped})"


if __name__ == "__main__":
    try:
        test_spec_regenerates_manifests()
    except unittest.SkipTest as exc:
        print(f"SKIPPED: {exc}")
        sys.exit(0)
    except AssertionError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        sys.exit(1)
    else:
        print("PASS: the spec regenerates colgrep-mcp's manifests and hook files")
        sys.exit(0)
