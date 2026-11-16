#!/usr/bin/env python3
"""Check that every production of the legacy rdm grammars is traced.

Prints each grammar with `dirtree-rdm.sh grammar leaf|readme`, extracts the name of each
production, and checks that `__design__/rdm/traceability.md` has one row for it, in the table of
the same grammar, with a disposition of `stored`, `derived` or `dropped` and a non-empty target.
A row that names no production fails too, so a renamed production cannot leave a stale row.
Beyond presence: a `stored` target must name, in backticks, fields that exist in
`crates/rdm-schema/src/stored.rs` (its last path segment, or a type name), a `dropped` row needs a
reason of at least three words, and a target of `-` or nothing is refused.

Usage: uv run --no-project tools/check_rdm_traceability.py
Exit status: 0 when every production is traced, 1 otherwise, 2 when the grammar printer fails.
Standard library only.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PRINTER = "skills/managing-roadmaps/scripts/dirtree-rdm.sh"
TABLE = ROOT / "__design__/rdm/traceability.md"
GRAMMARS = {"leaf": "## Leaf grammar", "readme": "## README grammar"}
DISPOSITIONS = {"stored", "derived", "dropped"}
STORED_RS = ROOT / "crates/rdm-schema/src/stored.rs"
MIN_REASON_WORDS = 3
# Backticked tokens that are file names or values, not fields.
LITERALS = {"md", "json", "true", "false"}
# A backticked token that is a field path: steps[].title, Leaf.progress.branch, notes.
FIELD_PATH = re.compile(r"`([A-Za-z_][A-Za-z0-9_]*(?:\[\])?(?:\.[A-Za-z_][A-Za-z0-9_]*(?:\[\])?)*)`")

# A production starts a line: <name> ::= or TOKEN ::=. Comment lines start with '#'.
PRODUCTION = re.compile(r"^(<[a-z0-9-]+>|[A-Z][A-Z_]*)\s*::=")
# A table row: | `name` | disposition | target |
ROW = re.compile(r"^\|\s*`(<[a-z0-9-]+>|[A-Z][A-Z_]*)`\s*\|\s*(.*?)\s*\|\s*(.*?)\s*\|\s*$")


def production_names(grammar_text: str) -> list[str]:
    """Return the production names of a grammar, in order, comments excluded."""
    names = []
    for line in grammar_text.splitlines():
        if line.lstrip().startswith("#"):
            continue
        match = PRODUCTION.match(line)
        if match:
            names.append(match.group(1))
    return names


def table_rows(markdown: str, heading: str) -> list[tuple[str, str, str]]:
    """Return (name, disposition, target) for each production row under `heading`.

    The section runs to the next line starting with `## `. The disposition has its
    backticks removed. A row whose first cell is not a production name is ignored.
    """
    rows: list[tuple[str, str, str]] = []
    inside = False
    for line in markdown.splitlines():
        if line.startswith("## "):
            inside = line.strip() == heading
            continue
        if not inside:
            continue
        match = ROW.match(line)
        if match:
            name, disposition, target = match.groups()
            rows.append((name, disposition.strip("` "), target))
    return rows


def stored_names() -> set[str]:
    """Field names and type names declared in stored.rs (`pub name:`, `struct`, `enum`)."""
    text = STORED_RS.read_text(encoding="utf-8")
    names = set(re.findall(r"pub\s+(?:r#)?([a-z_][a-z0-9_]*)\s*:", text))
    names |= set(re.findall(r"pub\s+(?:struct|enum|type)\s+([A-Za-z0-9_]+)", text))
    return names


def target_problems(disposition: str, target: str, known: set[str]) -> list[str]:
    """Return what is wrong with a row's target, or nothing."""
    if target in {"", "-", "--", "—"}:
        return ["has an empty target or reason"]
    if disposition == "dropped" and len(target.split()) < MIN_REASON_WORDS:
        return [f"has a dropped reason of fewer than {MIN_REASON_WORDS} words: '{target}'"]
    if disposition == "stored":
        paths = [
            p for p in FIELD_PATH.findall(target) if p.split(".")[-1] not in LITERALS
        ]
        if not paths:
            return [f"stored target names no field in backticks: '{target}'"]
        bad = [p for p in paths if p.replace("[]", "").split(".")[-1] not in known]
        if bad:
            return [f"stored target names no field of stored.rs: {', '.join(bad)}"]
    return []


def print_grammar(name: str) -> str:
    """Run the grammar printer from the repository root and return its output."""
    result = subprocess.run(
        ["bash", PRINTER, "grammar", name],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0 or not result.stdout.strip():
        sys.stderr.write(f"grammar printer failed for '{name}': {result.stderr.strip()}\n")
        raise SystemExit(2)
    return result.stdout


def main() -> int:
    if not TABLE.is_file():
        print(f"FAIL: {TABLE.relative_to(ROOT)} does not exist")
        return 1
    markdown = TABLE.read_text(encoding="utf-8")
    problems: list[str] = []
    known = stored_names()
    for grammar, heading in GRAMMARS.items():
        names = production_names(print_grammar(grammar))
        rows = table_rows(markdown, heading)
        by_name: dict[str, list[tuple[str, str]]] = {}
        for name, disposition, target in rows:
            by_name.setdefault(name, []).append((disposition, target))
        for name in names:
            entries = by_name.get(name)
            if not entries:
                problems.append(f"{grammar}: {name} has no row under '{heading}'")
                continue
            if len(entries) > 1:
                problems.append(f"{grammar}: {name} has {len(entries)} rows")
            disposition, target = entries[0]
            if disposition not in DISPOSITIONS:
                problems.append(f"{grammar}: {name} has disposition '{disposition}'")
            for issue in target_problems(disposition, target, known):
                problems.append(f"{grammar}: {name} {issue}")
        for name in by_name:
            if name not in names:
                problems.append(f"{grammar}: row {name} matches no production")
        print(f"{grammar}: {len(names)} productions, {len(rows)} rows")
    if problems:
        print("FAIL")
        for problem in problems:
            print("  " + problem)
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
