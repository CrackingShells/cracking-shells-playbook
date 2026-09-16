#!/usr/bin/env python3
# /// script
# dependencies = ["pyyaml"]
# ///
"""
Plugin Assembly Tool - Materialises a plugin's skills subtree from its canonical skill source

Copies skills/<name>/ into plugins/<name>/skills/<name>/, reusing the exclusion
rules from package_skill.py so a distributed .skill file and an assembled
plugin tree never disagree about what belongs in a skill.

Usage:
    python tools/assemble_plugin.py <skill-name>

Example:
    python tools/assemble_plugin.py writing-history
"""

import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from package_skill import should_exclude

# package_skill.py's EXCLUDE_FILES doesn't list .releaserc.js because a
# .skill zip never contained one; the assembled plugin tree still needs it
# excluded since the canonical skill source may carry one.
EXTRA_EXCLUDE_FILES = {".releaserc.js"}


def assemble(skill_name: str, repo_root) -> Path:
    """
    Materialise plugins/<skill_name>/skills/<skill_name>/ from skills/<skill_name>/.

    The destination subtree is removed before being rewritten, so the copy is
    idempotent: a file deleted from the source disappears from the assembled
    copy instead of lingering from a previous run.

    Args:
        skill_name: Name of the skill folder under skills/
        repo_root: Path to the repository root

    Returns:
        Path to the assembled skill directory (plugins/<skill_name>/skills/<skill_name>/).
    """
    repo_root = Path(repo_root)
    src = repo_root / "skills" / skill_name
    dest = repo_root / "plugins" / skill_name / "skills" / skill_name

    if not src.is_dir():
        raise FileNotFoundError(f"Skill folder not found: {src}")

    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True, exist_ok=True)

    for file_path in src.rglob("*"):
        if not file_path.is_file():
            continue
        # should_exclude expects a path relative to the skill's *parent* dir
        # (src.parent == repo_root/skills), matching package_skill.py's own
        # zipf.write(file_path, arcname) call.
        rel_path = file_path.relative_to(src.parent)
        if should_exclude(rel_path) or rel_path.name in EXTRA_EXCLUDE_FILES:
            continue
        dest_file = dest / file_path.relative_to(src)
        dest_file.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(file_path, dest_file)

    return dest


def main():
    if len(sys.argv) != 2:
        print("Usage: python tools/assemble_plugin.py <skill-name>")
        sys.exit(1)

    skill_name = sys.argv[1]
    repo_root = Path(__file__).parent.parent

    try:
        dest = assemble(skill_name, repo_root)
    except FileNotFoundError as e:
        print(f"Error: {e}")
        sys.exit(1)

    print(f"Assembled {dest}")


if __name__ == "__main__":
    main()
