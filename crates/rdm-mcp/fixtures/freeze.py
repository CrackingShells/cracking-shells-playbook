"""Freeze the meta-roadmap at a commit as the spike fixture (stdlib only).

Usage: uv run --no-project crates/rdm-mcp/fixtures/freeze.py <commit>
"""
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT_DIR = "__roadmap__/roadmap-mcp-rework"
OUT_DIR = Path(__file__).resolve().parent

STATUS_WORDS = {
    "planned": "planned",
    "in progress": "inprogress",
    "done": "done",
    "amendment": "amendment",
    "blocked": "blocked",
}


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], check=True, capture_output=True, text=True
    ).stdout


def read_tree(commit: str) -> dict[str, str]:
    """Return {path relative to the campaign root: file text} for every file."""
    names = git("ls-tree", "-r", "--name-only", commit, ROOT_DIR + "/").splitlines()
    return {
        n[len(ROOT_DIR) + 1 :]: git("show", f"{commit}:{n}") for n in names
    }


def parse_nodes_table(readme: str) -> list[tuple[str, str, str]]:
    """Return [(name, kind, status)] from a README's Nodes table, in table order."""
    rows = []
    in_nodes = False
    for line in readme.splitlines():
        if line.startswith("## "):
            in_nodes = line.strip() == "## Nodes"
            continue
        if not in_nodes or not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        m = re.fullmatch(r"`([^`]+)`", cells[0]) if len(cells) == 3 else None
        if not m:
            continue
        name = m.group(1)
        label = re.sub(r"^[^\w]+", "", cells[2]).strip().lower()
        rows.append(
            ("dir" if name.endswith("/") else "leaf", name.rstrip("/"), STATUS_WORDS[label])
        )
    return [(name, kind, status) for kind, name, status in rows]


def title_of(text: str, fallback: str) -> str:
    for line in text.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return fallback


def main() -> None:
    commit = sys.argv[1]
    sha = git("rev-parse", commit).strip()
    tree = read_tree(sha)
    nodes = []
    queue = [""]  # directories, breadth first
    while queue:
        d = queue.pop(0)
        readme = tree[(d + "/" if d else "") + "README.md"]
        for name, kind, status in parse_nodes_table(readme):
            path = f"{d}/{name}" if d else name
            text = tree[path + "/README.md"] if kind == "dir" else tree[path]
            nodes.append(
                {
                    "path": path,
                    "parent": d or None,
                    "kind": kind,
                    "title": title_of(text, name),
                    "status": status,
                    "depth": path.count("/"),
                }
            )
            if kind == "dir":
                queue.append(path)
    out = {"campaign": "roadmap-mcp-rework", "frozen_at": sha, "nodes": nodes}
    (OUT_DIR / "roadmap.json").write_text(json.dumps(out, indent=2) + "\n")
    leaves = OUT_DIR / "leaves"
    shutil.rmtree(leaves, ignore_errors=True)
    for n in nodes:
        if n["kind"] == "leaf":
            dest = leaves / n["path"]
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text(tree[n["path"]])


if __name__ == "__main__":
    main()
