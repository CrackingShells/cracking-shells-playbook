#!/usr/bin/env python3
"""
Set Plugin Version - Writes a version into a plugin's manifests

Used by release-config.js as a semantic-release `@semantic-release/exec`
prepareCmd, run once per skill release from that skill's own directory
(skills/<name>/) — which is why the repo root is resolved from this
script's own file location rather than from the working directory.

Writes the same version string into both plugins/<name>/plugin.json and
plugins/<name>/.claude-plugin/plugin.json, so a plugin never has one
manifest reporting the new version and the other lagging (the identity
check_plugin.py enforces at any other time).

`main()` also `git add`s the two manifests itself, from repo_root, right
after writing them. This is not cosmetic: `@semantic-release/git` decides
what to commit by running `git ls-files -m -o` with `cwd` set to the
*package* directory (skills/<name>/) and no pathspec — and plain git
restricts an unqualified `ls-files`/`status` to the invoking directory's
own subtree. `../../plugins/<name>/plugin.json` lives outside that
subtree, so it never appears in that modified-files list no matter how
the `git.assets` glob in release-config.js is spelled; the glob only
filters that list down, it cannot add to it. `git commit` (also run with
no pathspec) commits whatever is already staged, so pre-staging the
manifests here — with an explicit path, which is not subject to the same
default-scope restriction — is what actually gets them into the release
commit. Measured empirically: `git ls-files -m -o` from inside
skills/writing-history/ returns nothing for a modified
../../plugins/writing-history/plugin.json; `git add
../../plugins/writing-history/plugin.json` from the same cwd stages it
correctly (`git add`'s explicit path argument is not pathspec-limited the
way the default, argument-less form is).

Usage:
    python tools/set_plugin_version.py <plugin-name> <version>

Example:
    python tools/set_plugin_version.py writing-history 1.2.0
"""

import json
import subprocess
import sys
from pathlib import Path

# Manifests that carry an authoritative "version" field for a plugin root,
# relative to plugins/<plugin_name>/. Both must agree.
MANIFEST_RELPATHS = ("plugin.json", ".claude-plugin/plugin.json")


def set_version(plugin_name, version, repo_root):
    """
    Write `version` into plugin_name's plugin.json and .claude-plugin/plugin.json.

    Args:
        plugin_name: Name of the plugin folder under plugins/
        version: Semver string to write, e.g. "1.2.0"
        repo_root: Path to the repository root

    Returns:
        List of Paths written, in MANIFEST_RELPATHS order.

    Raises:
        FileNotFoundError: if plugins/<plugin_name>/ or a manifest is missing.
    """
    repo_root = Path(repo_root)
    plugin_dir = repo_root / "plugins" / plugin_name

    if not plugin_dir.is_dir():
        raise FileNotFoundError(f"Plugin root not found: {plugin_dir}")

    written = []
    for relpath in MANIFEST_RELPATHS:
        manifest_path = plugin_dir / relpath
        if not manifest_path.is_file():
            raise FileNotFoundError(f"Manifest not found: {manifest_path}")

        data = json.loads(manifest_path.read_text(encoding="utf-8"))
        data["version"] = version
        manifest_path.write_text(
            json.dumps(data, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        written.append(manifest_path)

    return written


def main():
    if len(sys.argv) != 3:
        print("Usage: python tools/set_plugin_version.py <plugin-name> <version>")
        sys.exit(1)

    plugin_name, version = sys.argv[1], sys.argv[2]
    repo_root = Path(__file__).parent.parent

    try:
        written = set_version(plugin_name, version, repo_root)
    except FileNotFoundError as e:
        print(f"Error: {e}")
        sys.exit(1)

    for path in written:
        print(f"wrote {path.relative_to(repo_root)} -> version {version}")

    # Stage explicitly (see module docstring): the @semantic-release/exec step
    # that runs this script has cwd=skills/<plugin_name>/, and
    # @semantic-release/git's own modified-file detection never looks outside
    # that directory. An explicit `git add` path is not subject to that
    # default-scope restriction, so this is what actually gets these two
    # files into the release commit.
    subprocess.run(
        ["git", "add", "--", *[str(p) for p in written]],
        cwd=repo_root,
        check=True,
    )
    for path in written:
        print(f"staged {path.relative_to(repo_root)}")


if __name__ == "__main__":
    main()
