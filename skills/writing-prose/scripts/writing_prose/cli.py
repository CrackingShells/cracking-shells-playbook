#!/usr/bin/env python3
"""writing-prose: deterministic support for the writing-prose skill.

Two subcommands:

  rules <document>    List the recorded prose-rule sources binding a document.
  edges [edges.json]  Compute reach distances for a causal-locality edge list.

Stdlib only, no dependencies. Installation is optional; uninstalled use:

    python <skill>/scripts/writing_prose/cli.py <subcommand> ...
"""

import argparse
import json
import sys
from fnmatch import fnmatch
from pathlib import Path

# ---------------------------------------------------------------- rules ----

# Conventional authoring formats, the DEFAULT target filter; anything
# else (source code, configs, data) is out of scope for prose rules
# even when its path matches a scope glob. The default is maskable per
# call: --allow globs admit files it would reject, --deny globs reject
# files it would admit, and deny wins over everything.
AUTHORING_EXTENSIONS = {
    ".md", ".markdown", ".txt", ".tex", ".rst", ".adoc",
    ".docx", ".odt", ".pptx", ".html",
}


def glob_hits(document: Path, patterns) -> str:
    """First pattern matching the document's path or bare name, else ''."""
    for pattern in patterns:
        if fnmatch(str(document), pattern) or fnmatch(document.name, pattern):
            return pattern
    return ""

# Author-level standing doctrine lives here; project-level config lives
# in the project's canon directory. Both are scanned; a file's own
# scope: frontmatter overrides the location default.
AUTHOR_ROOT = Path.home() / ".config" / "writing-prose"
PROJECT_CANON = Path("__canons__") / "prose_writing"


def read_scope(rule_file: Path, default: str) -> str:
    """Return the scope: glob from a file's frontmatter, or the default."""
    try:
        lines = rule_file.read_text(encoding="utf-8").splitlines()
    except OSError:
        return default
    if not lines or lines[0].strip() != "---":
        return default
    for line in lines[1:30]:
        if line.strip() == "---":
            break
        if line.startswith("scope:"):
            return line.split(":", 1)[1].strip().strip("'\"")
    return default


def scope_matches(document: Path, scope: str) -> bool:
    # ~ expands to home; fnmatch lets * cross directory separators,
    # so ** and * are equivalent here.
    pattern = str(Path(scope.replace("~", str(Path.home()), 1)))
    return fnmatch(str(document), pattern)


def cmd_rules(args: argparse.Namespace) -> int:
    document = args.document.expanduser().resolve()

    denied_by = glob_hits(document, args.deny)
    if denied_by:
        print(f"excluded by deny pattern '{denied_by}'; no prose rules apply")
        return 0
    if document.suffix.lower() not in AUTHORING_EXTENSIONS:
        allowed_by = glob_hits(document, args.allow)
        if not allowed_by:
            print(f"not an authoring document ({document.suffix or 'no extension'}); "
                  f"no prose rules apply (admit it with --allow if intended)")
            return 0

    roots = [
        (AUTHOR_ROOT, "~/**"),
        (args.project_root.resolve() / PROJECT_CANON,
         str(args.project_root.resolve() / "**")),
    ]

    bound = []
    for root, default_scope in roots:
        if not root.is_dir():
            continue
        for rule_file in sorted(root.rglob("*.md")):
            scope = read_scope(rule_file, default_scope)
            if scope_matches(document, scope):
                bound.append((scope, rule_file))

    if not bound:
        print("no recorded rules bind this document "
              "(the skill's universal register rules always apply)")
        return 0

    # Wider scopes first: author-level doctrine, then project, then
    # narrower document-family globs (longer pattern = more specific).
    bound.sort(key=lambda item: (len(item[0]), str(item[1])))
    print(f"rules binding {document}")
    print("(the skill's universal register rules always apply, and are not listed)")
    for scope, rule_file in bound:
        print(f"  [{scope}]  {rule_file}")
    return 0


# ---------------------------------------------------------------- edges ----

# A supplier within the previous two sentences reads as adjacent;
# beyond that, readers stop reconnecting the reference themselves.
ADJACENT_REACH = 2

# Worst first: no supplier at all, then a supplier the reader has not
# met yet, then a supplier too far back, then fine.
SEVERITY = {"dangling": 0, "forward": 1, "long": 2, "ok": 3}

EDGES_HELP = """\
Input: JSON with one key, "edges". Each edge records one reference in
the prose (a pronoun, definite reference, or presupposition), the
sentence holding it, and the sentence supplying it:

    {"edges": [
        {"ref": "that output",   "from": 4, "to": 3},
        {"ref": "the crossing",  "from": 7, "to": 2},
        {"ref": "prior consent", "from": 9, "to": null}
    ]}

Sentences are numbered in reading order (captions and pulled quotes
included where the reader meets them). "to": null records a premise
with no supplier. A reference with several suppliers is judged by its
nearest earlier one, because a re-anchoring mention keeps a chain
alive. The semantic work (finding the edges) belongs to the reviewer;
this command only does the arithmetic and ordering. Do not reuse a
previous round's edges: any edit renumbers sentences, so an edge list
describes exactly one draft state.
"""


def judge(suppliers):
    """Verdict for one reference from its supplier list."""
    backward = [f - t for f, t in suppliers if t is not None and t < f]
    forward = [t - f for f, t in suppliers if t is not None and t >= f]
    if backward:
        reach = min(backward)
        return ("ok" if reach <= ADJACENT_REACH else "long"), reach
    if forward:
        return "forward", min(forward)
    return "dangling", None


def cmd_edges(args: argparse.Namespace) -> int:
    source = open(args.file, encoding="utf-8") if args.file else sys.stdin
    with source:
        edges = json.load(source)["edges"]

    by_ref = {}
    for edge in edges:
        key = (edge["ref"], edge["from"])
        by_ref.setdefault(key, []).append((edge["from"], edge.get("to")))

    rows = []
    for (ref, holder), suppliers in by_ref.items():
        verdict, reach = judge(suppliers)
        rows.append((SEVERITY[verdict], holder, ref, verdict, reach))
    rows.sort()

    print("| reference | in sentence | verdict | reach |")
    print("|---|---|---|---|")
    for _, holder, ref, verdict, reach in rows:
        shown = "-" if reach is None else str(reach)
        print(f"| {ref} | {holder} | {verdict} | {shown} |")

    defects = sum(1 for row in rows if row[0] < SEVERITY["ok"])
    print(f"\n{defects} defect(s) of {len(rows)} reference(s); "
          f"adjacency threshold {ADJACENT_REACH}.")
    return 0


# ----------------------------------------------------------------- main ----

def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        prog="writing-prose",
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    rules = subparsers.add_parser(
        "rules",
        help="list the recorded prose-rule sources binding a document",
        description="Given a document path, list every recorded rule file "
                    "whose scope glob matches it, widest first. Filters to "
                    "conventional authoring formats by default so code and "
                    "config files never pick up prose rules; mask the "
                    "default per call with --allow and --deny.",
    )
    rules.add_argument("document", type=Path)
    rules.add_argument("-p", "--project-root", type=Path, default=Path.cwd(),
                       help="project to scan for a canon directory "
                            "(default: current directory)")
    rules.add_argument("-a", "--allow", action="append", default=[],
                       metavar="GLOB",
                       help="admit files the default authoring-format "
                            "filter would reject; repeatable")
    rules.add_argument("-d", "--deny", action="append", default=[],
                       metavar="GLOB",
                       help="reject files even when the default filter or "
                            "an allow pattern admits them; deny wins; "
                            "repeatable")
    rules.set_defaults(func=cmd_rules)

    edges = subparsers.add_parser(
        "edges",
        help="compute reach distances for a causal-locality edge list",
        description=EDGES_HELP,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    edges.add_argument("file", nargs="?", default=None,
                       help="edge list JSON (default: stdin)")
    edges.set_defaults(func=cmd_edges)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
