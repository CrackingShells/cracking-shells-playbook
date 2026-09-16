#!/usr/bin/env python3
"""
Set Plugin Version - Writes a release's version where the pipeline needs it

Used by release-config.js as a semantic-release `@semantic-release/exec`
prepareCmd, run once per *skill* release (every skill in skills/*, plugin
or not — release-config.js is the shared factory) from that skill's own
directory (skills/<name>/) — which is why the repo root is resolved from
this script's own file location rather than from the working directory.

A skill with no plugins/<name>/ root (writing-prose, writing-design-canons,
importing-design-figures at the time of writing) is a legitimate,
deliberate state, not an error — the release for that skill must still
succeed, and this script writes nothing and exits 0 for it. A typo'd
plugin name is `check_plugin.py --spec`'s job to catch, not this script's.

For a skill that *does* have a plugin root, two things get written:

1. plugins/<name>/plugin.json and plugins/<name>/.claude-plugin/plugin.json.
   `main()` also `git add`s these two manifests itself, from repo_root,
   right after writing them. This is not cosmetic: `@semantic-release/git`
   decides what to commit by running `git ls-files -m -o` with `cwd` set
   to the *package* directory (skills/<name>/) and no pathspec — and
   plain git restricts an unqualified `ls-files`/`status` to the invoking
   directory's own subtree. `../../plugins/<name>/plugin.json` lives
   outside that subtree, so it never appears in that modified-files list
   no matter how the `git.assets` glob in release-config.js is spelled;
   the glob only filters that list down, it cannot add to it. `git
   commit` (also run with no pathspec) commits whatever is already
   staged, so pre-staging the manifests here — with an explicit path,
   which is not subject to the same default-scope restriction — is what
   actually gets them into the release commit. Measured empirically:
   `git ls-files -m -o` from inside skills/writing-history/ returns
   nothing for a modified ../../plugins/writing-history/plugin.json;
   `git add ../../plugins/writing-history/plugin.json` from the same cwd
   stages it correctly (`git add`'s explicit path argument is not
   pathspec-limited the way the default, argument-less form is).

2. skills/<name>/package.json's "version" field. Nothing else in this
   pipeline ever writes it: there is no `@semantic-release/npm`, and
   multi-semantic-release's own version bookkeeping is internal to its
   run, not persisted back to the workspace member's package.json.
   Without this write, package.json stays frozen at whatever it was last
   hand-set to while git tags move on with every release (measured on
   this repo: managing-roadmaps has tags through 1.1.1 while its
   package.json still read 1.0.0). That frozen file is also
   `plugin.spec.json`'s `version_from` source for this plugin — so
   leaving it frozen doesn't just make the "manifest version equals
   package.json version" Success Gate go stale the moment the first real
   release lands, it means a future `spawn --force` (run for an unrelated
   spec edit) would re-resolve the old version and silently downgrade an
   already-released plugin manifest. The rewrite touches only the
   "version" value and leaves the rest of the file's exact formatting
   (compact, no whitespace, trailing newline) alone. It needs no staging
   trick of its own — package.json lives inside skills/<name>/, i.e.
   inside cwd, so plain `git ls-files -m -o` (and the existing
   "package.json" entry already in `git.assets`) sees it; verified
   empirically, not assumed. This write is scoped to plugin-bearing
   skills only, alongside the manifests, because it exists to keep
   `version_from` truthful for a plugin — a skill with no plugin makes no
   such promise, and nothing else needs its package.json to move.

Usage:
    python tools/set_plugin_version.py <skill-name> <version>

Example:
    python tools/set_plugin_version.py writing-history 1.2.0   # writes manifests + package.json
    python tools/set_plugin_version.py writing-prose 1.2.0     # no plugin root: exits 0, writes nothing
"""

import json
import re
import subprocess
import sys
from pathlib import Path

# Manifests that carry an authoritative "version" field for a plugin root,
# relative to plugins/<plugin_name>/. Both must agree.
MANIFEST_RELPATHS = ("plugin.json", ".claude-plugin/plugin.json")

_VERSION_FIELD_RE = re.compile(r'("version"\s*:\s*")([^"]*)(")')


def set_skill_package_version(skill_name, version, repo_root):
    """
    Rewrite the "version" field in skills/<skill_name>/package.json in place.

    Only the value of the existing "version" field is changed; every other
    byte of the file (key order, spacing, trailing newline) is left as-is.

    Args:
        skill_name: Name of the skill folder under skills/
        version: Semver string to write, e.g. "1.2.0"
        repo_root: Path to the repository root

    Returns:
        Path to the package.json written.

    Raises:
        FileNotFoundError: if skills/<skill_name>/package.json is missing.
        ValueError: if the file has no "version" field to rewrite.
    """
    repo_root = Path(repo_root)
    package_json_path = repo_root / "skills" / skill_name / "package.json"

    if not package_json_path.is_file():
        raise FileNotFoundError(f"package.json not found: {package_json_path}")

    text = package_json_path.read_text(encoding="utf-8")
    new_text, count = _VERSION_FIELD_RE.subn(
        lambda m: f"{m.group(1)}{version}{m.group(3)}", text, count=1
    )
    if count == 0:
        raise ValueError(f'no "version" field found in {package_json_path}')

    package_json_path.write_text(new_text, encoding="utf-8")
    return package_json_path


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
        print("Usage: python tools/set_plugin_version.py <skill-name> <version>")
        sys.exit(1)

    skill_name, version = sys.argv[1], sys.argv[2]
    repo_root = Path(__file__).parent.parent

    # A skill with no plugin root is a legitimate, deliberate state (see
    # module docstring) — not an error, and must not fail the release.
    # Say so rather than exiting silently, and write nothing: there is
    # nothing here for a non-plugin skill to disagree about.
    plugin_dir = repo_root / "plugins" / skill_name
    if not plugin_dir.is_dir():
        print(f"no plugin root at plugins/{skill_name}/; this skill has no plugin, nothing to write")
        sys.exit(0)

    try:
        written = set_version(skill_name, version, repo_root)
    except FileNotFoundError as e:
        print(f"Error: {e}")
        sys.exit(1)

    for path in written:
        print(f"wrote {path.relative_to(repo_root)} -> version {version}")

    # Stage explicitly (see module docstring): the @semantic-release/exec step
    # that runs this script has cwd=skills/<skill_name>/, and
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

    # Keep skills/<name>/package.json's version_from source truthful for
    # this plugin (see module docstring). No staging trick needed: it
    # lives inside cwd, so plain `git ls-files -m -o` already sees it.
    try:
        package_json_path = set_skill_package_version(skill_name, version, repo_root)
    except (FileNotFoundError, ValueError) as e:
        print(f"Error: {e}")
        sys.exit(1)
    print(f"wrote {package_json_path.relative_to(repo_root)} -> version {version}")


if __name__ == "__main__":
    main()
