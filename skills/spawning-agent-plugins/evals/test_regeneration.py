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
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SKILL_ROOT = HERE.parent

sys.path.insert(0, str(SKILL_ROOT / "scripts"))
from spawn_plugin import load_spec, spawn  # noqa: E402  (needs the sys.path.insert above)

SPEC_PATH = SKILL_ROOT / "assets" / "examples" / "colgrep-mcp.spec.json"

# Measured 2026-09-16 against colgrep-mcp ef54b8f (v0.5.1): the invariant holds
# for every manifest and hook file. The ONLY divergence is this hand-maintained
# prose file. SKILL.md's claim is specifically about manifests and hook files,
# so this is exactly what the guard allows — asserting `not w.skipped` outright
# would fail today, before any generator change, training its readers to
# ignore it.
ALLOWED_DIVERGENCE = {"dev/README.md"}

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

    skipped_names = {rel.split("  [keys:", 1)[0].strip() for rel in w.skipped}
    unexpected = skipped_names - ALLOWED_DIVERGENCE
    assert not unexpected, f"unexpected divergence from the spec: {sorted(unexpected)} (full detail: {w.skipped})"


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
