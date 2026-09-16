#!/usr/bin/env python3
"""Drift guards for a three-ecosystem plugin tree. CLI and pytest in one file.

    python3 check_plugin.py --root <repo>      # exit 1 and one line per problem
    cp check_plugin.py <repo>/tests/test_plugin_structure.py   # pytest collects test_*

The checks are the invariants no loader verifies for you, distilled from
colgrep-mcp's `test_manifests.py`, `test_dev_plugin.py` and `test_hooks.py`
(references/manifests.md, references/hooks.md). Each one exists because
breaking it produced a silent failure once: a server that never started, a
hook that never fired, a version that lagged behind a release.

Stdlib only, Python >= 3.9. When copied into a test suite, set REPO_ROOT to
the repository root (the default walks up from this file until it finds a
`.claude-plugin/` or `plugin.json`).

A repo may hold more than one plugin (a multi-plugin spec's siblings, or a
maintainer `dev/` plugin next to the product): `collect_problems` scans for
every plugin root under `--root` and checks each independently, so one
sibling's identity (name, version) is never compared against another's.
Codex has no manifest of its own since it reads the shared root
`plugin.json`, taking its extras from `extensions["com.openai"]`.
"""

from __future__ import annotations

import ast
import json
import os
import re
import sys
from pathlib import Path

#: Events documented by Claude Code, Codex and Cursor's Claude-Code hook import alike.
PORTABLE_EVENTS = {
    "SessionStart",
    "SessionEnd",
    "SubagentStart",
    "SubagentStop",
    "PreToolUse",
    "PostToolUse",
    "UserPromptSubmit",
    "PreCompact",
    "Stop",
}
AGENT_PLUGIN_FIELDS = {"$schema", "name", "version", "description", "author", "homepage", "repository", "license", "keywords", "extensions"}
FORBIDDEN_MCP_ENV_KEYS = {"PLUGIN_ROOT", "PLUGIN_DATA"}
HOOK_LAUNCHER = 'uv run --no-project --quiet python "${CLAUDE_PLUGIN_ROOT}/'
PORTABLE_HOOKS_FILE = "hooks/hooks.json"
_FRONTMATTER = re.compile(r"\A---\n(.*?)\n---\n", re.S)


def event_file_stem(event: str) -> str:
    """The file that holds one non-portable event is named after it: WorktreeRemove -> worktree-remove."""
    return re.sub(r"(?<!^)(?=[A-Z])", "-", event).lower()


def find_root(start: Path) -> Path:
    for candidate in (start, *start.parents):
        if any((candidate / m).exists() for m in (".claude-plugin", "plugin.json")):
            return candidate
    return start


REPO_ROOT = find_root(Path(__file__).resolve().parent)


def _load(root: Path, rel: str) -> dict | None:
    path = root / rel
    if not path.exists():
        return None
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        raise ValueError(f"{rel}: invalid JSON ({exc})") from exc


_SKIP_DIR_NAMES = {
    ".git",
    "node_modules",
    "dist",
    "__pycache__",
    ".venv",
    "venv",
    "site-packages",
    ".tox",
    ".pytest_cache",
    ".ruff_cache",
    "target",
}


def _looks_like_plugin_root(candidate: Path) -> bool:
    return (candidate / ".claude-plugin" / "plugin.json").exists() or (candidate / "plugin.json").exists()


def find_plugin_roots(root: Path) -> list[Path]:
    """Every plugin root under `root`: the repo root itself when it looks like a plugin,
    plus every directory anywhere under it with its own manifest (a `plugins[]` sibling,
    an assembled `plugins/<name>/` tree, or a maintainer `dev/` plugin). One entry per
    plugin, so a sibling's identity check never runs against another sibling's.

    `_SKIP_DIR_NAMES` is pruned DURING the walk (`os.walk`'s in-place `dirnames`
    trick), not filtered out of the result afterward: a vendored tree like
    `server/.venv` (colgrep-mcp ships one with 3339 files) is never descended into
    at all, so the exclusion also bounds the walk's cost, not just its correctness
    — and a stray `plugin.json` dropped by some unrelated package under
    `site-packages/` can never be mistaken for a plugin root.
    """
    roots: list[Path] = [root] if _looks_like_plugin_root(root) else []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in _SKIP_DIR_NAMES]
        candidate = Path(dirpath)
        if candidate == root or candidate.name.startswith("."):
            continue  # repo root already handled above; a dotfile dir (.claude-plugin
            # itself, .git, ...) is never a plugin root in its own right
        has_claude_manifest = (candidate / ".claude-plugin" / "plugin.json").exists()
        has_root_manifest = "plugin.json" in filenames
        if (has_claude_manifest or has_root_manifest) and candidate not in roots:
            roots.append(candidate)
    return sorted(roots, key=lambda p: p.as_posix())


def _resolve_local_source_path(source) -> str | None:
    """The relative path a marketplace `source` points at, when it names a location in
    THIS repo: a plain string, or a dict carrying `path` (`local` or `git-subdir`). None
    for a source this checker cannot resolve locally (e.g. a bare `github`/`url` source
    with no `path`, naming a different repo entirely)."""
    if isinstance(source, str):
        return source
    if isinstance(source, dict) and "path" in source:
        return source["path"]
    return None


def _source_matches(source, expected_rel: str) -> bool:
    """`expected_rel` is `"./"` for the repo root or `"./<subdir>"` for a sibling."""
    path = _resolve_local_source_path(source)
    return path is not None and path.rstrip("/") == expected_rel.rstrip("/")


def _root_version(root: Path) -> str | None:
    for rel in (".claude-plugin/plugin.json", "plugin.json"):
        data = _load(root, rel)
        if data and data.get("version"):
            return data["version"]
    return None


def _check_codex_interface(extensions: dict | None, agent: dict | None, version: str | None) -> list[str]:
    """Codex's extras, now under `plugin.json`'s `extensions["com.openai"]` rather than a
    manifest of its own: identity (`author.name`, strict-semver `version`) lives on the
    shared root manifest; only `interface` is Codex-specific."""
    out: list[str] = []
    if not extensions:
        return out
    interface = extensions.get("interface", {})
    for key in ("displayName", "shortDescription", "longDescription", "developerName", "category", "capabilities"):
        if not interface.get(key):
            out.append(f'plugin.json extensions["com.openai"].interface lacks {key}')
    if not (agent or {}).get("author", {}).get("name"):
        out.append("plugin.json needs author.name (Codex requires it)")
    if not re.fullmatch(r"\d+\.\d+\.\d+([-+][0-9A-Za-z.-]+)?", str(version or "")):
        out.append("plugin.json version must be strict semver (Codex validates it)")
    return out


def collect_problems(root: Path) -> list[str]:
    """Check every plugin root under `root` independently, plus the shared, repo-level
    marketplace files and any maintainer `dev/` plugins nested under a product root."""
    problems: list[str] = []
    say = problems.append

    try:
        roots = find_plugin_roots(root)
    except ValueError as exc:
        return [str(exc)]
    if not roots:
        return [f"no plugin manifest found under {root} (.claude-plugin/plugin.json, plugin.json)"]

    if (root / ".mcp.json").exists():
        say("a root .mcp.json exists: Claude Code reads it as project-scope config that never expands ${CLAUDE_PLUGIN_ROOT}")

    try:
        claude_market = _load(root, ".claude-plugin/marketplace.json")
        codex_market = _load(root, ".agents/plugins/marketplace.json")
    except ValueError as exc:
        return [str(exc)]

    for plugin_root in roots:
        try:
            problems.extend(_collect_plugin_problems(plugin_root, root, claude_market, codex_market))
        except ValueError as exc:
            say(str(exc))

    problems.extend(_collect_dev_problems(root, roots, claude_market))
    return problems


def _collect_plugin_problems(root: Path, repo_root: Path, claude_market: dict | None, codex_market: dict | None) -> list[str]:
    """Every check scoped to ONE plugin at `root` (identity, whitelist, MCP, hooks,
    skills, and this plugin's own marketplace entries). `repo_root` is the outer scan
    root: marketplace files always live there, never per-plugin."""
    problems: list[str] = []
    say = problems.append

    claude = _load(root, ".claude-plugin/plugin.json")
    agent = _load(root, "plugin.json")
    present = {k: v for k, v in (("claude", claude), ("agent-plugins", agent)) if v}
    if not present:
        return [f"{root}: no plugin manifest found (.claude-plugin/plugin.json, plugin.json)"]

    codex_ext = (agent or {}).get("extensions", {}).get("com.openai")

    # -- identity: one name, one version, across every manifest THIS plugin has
    names = {k: v.get("name") for k, v in present.items()}
    if len(set(names.values())) != 1:
        say(f"{root}: plugin names differ across manifests: {names}")
    versions = {k: v.get("version") for k, v in present.items()}
    if len(set(versions.values())) != 1:
        say(f"{root}: plugin versions differ across manifests: {versions}")
    version = next(iter(versions.values()))
    name = next(iter(names.values()))

    # -- Agent Plugins 1.0: whitelisted fields, matching schema versions
    if agent:
        extra = set(agent) - AGENT_PLUGIN_FIELDS
        if extra:
            say(f"{root}/plugin.json carries fields Agent Plugins 1.0 does not define: {sorted(extra)} (hooks/skills/mcpServers live in the other manifests)")
        if not str(agent.get("$schema", "")).startswith("https://agent-plugins.org/schemas/"):
            say(f"{root}/plugin.json needs the agent-plugins.org $schema")
        agent_mcp = _load(root, "mcp.json")
        if agent_mcp:
            if _schema_version(agent_mcp.get("$schema", "")) != _schema_version(agent.get("$schema", "")):
                say(f"{root}: mcp.json and plugin.json cite different agent-plugins.org schema versions")
            for sname, server in agent_mcp.get("mcpServers", {}).items():
                if server.get("type") != "stdio" and "url" not in server:
                    say(f"{root}/mcp.json server {sname}: needs type stdio (or a url)")
                for key in server.get("env", {}):
                    if key in FORBIDDEN_MCP_ENV_KEYS:
                        say(f"{root}/mcp.json server {sname}: env key {key} is reserved by the spec")
                for text in json.dumps(server.get("args", [])), json.dumps(server.get("env", {})):
                    if re.search(r"\$\{[^}]*:-", text):
                        say(f"{root}/mcp.json server {sname}: ${{VAR:-default}} fallback syntax must stay literal under Agent Plugins 1.0")

    # -- Codex: extras now live in extensions["com.openai"], not a manifest of its own
    problems.extend(f"{root}: {msg}" for msg in _check_codex_interface(codex_ext, agent, version))
    if codex_ext and codex_ext.get("hooks") is not None and codex_ext["hooks"] != f"./{PORTABLE_HOOKS_FILE}":
        say(
            f'{root}/plugin.json extensions["com.openai"].hooks must be ./{PORTABLE_HOOKS_FILE} or absent: an explicit '
            "value replaces Codex's default discovery, and a per-event file may hold an event Codex does not know"
        )

    # -- MCP manifests: bare command, own file per ecosystem, no root .mcp.json
    if (root / ".mcp.json").exists() and root != repo_root:
        say(f"{root}: a nested .mcp.json exists; Claude Code reads it as project-scope config that never expands ${{CLAUDE_PLUGIN_ROOT}}")
    # Codex reads the same root mcp.json as agent-plugins (auto-wired by convention: no
    # manifest of its own), so it is checked once here, not under a separate key.
    mcp_files = {"claude": ".claude-plugin/mcp.json", "agent-plugins": "mcp.json"}
    launches: dict[str, dict] = {}
    for eco, rel in mcp_files.items():
        data = _load(root, rel)
        if not data:
            continue
        for sname, server in data.get("mcpServers", {}).items():
            launches.setdefault(sname, {})[eco] = server
            if "$" in str(server.get("command", "")):
                say(f"{root}/{rel} server {sname}: placeholder in `command`; no ecosystem expands one there")
            if eco != "claude":
                if any("$" in str(a) for a in server.get("args", [])) or "$" in json.dumps(server.get("env", {})):
                    say(f"{root}/{rel} server {sname}: placeholders belong only in the Claude Code manifest (.claude-plugin/mcp.json env)")
            else:
                if any("$" in str(a) for a in server.get("args", [])):
                    say(f"{root}/{rel} server {sname}: keep placeholders in env, not args (a literal ${{...}} arg means the server never started)")
    for sname, per_eco in launches.items():
        cmds = {eco: (s.get("command"), tuple(s.get("args", []))) for eco, s in per_eco.items()}
        if len(set(cmds.values())) != 1:
            say(f"{root}: server {sname} is launched differently per ecosystem: {cmds}")
        for eco, s in per_eco.items():
            joined = " ".join(map(str, s.get("args", [])))
            pin = re.search(r"==\s*([0-9][^\s\"']*)", joined)
            if pin and pin.group(1) != version:
                say(f"{root}/{mcp_files[eco]} pins {pin.group(1)} but the plugin version is {version} (add the pin to your version bumper)")
    if claude and "mcpServers" in claude and claude["mcpServers"] != "./.claude-plugin/mcp.json":
        say(f"{root}: Claude manifest mcpServers should be ./.claude-plugin/mcp.json (never a root .mcp.json)")
    if claude and "mcpServers" not in claude and (root / ".claude-plugin/mcp.json").exists():
        say(f"{root}: .claude-plugin/mcp.json exists but .claude-plugin/plugin.json never names it (mcpServers)")
    if claude and "mcpServers" in claude and not (root / ".claude-plugin/mcp.json").exists():
        say(f"{root}: .claude-plugin/plugin.json names an mcpServers file that does not exist")

    # -- hooks: portable events in hooks/hooks.json, one file per other event, named after it
    hooks_dir = root / "hooks"
    portable = _load(root, PORTABLE_HOOKS_FILE)
    extra_files = sorted(p for p in hooks_dir.glob("*.json") if p.name != "hooks.json") if hooks_dir.is_dir() else []
    portable_events = set(portable.get("hooks", {})) if portable else set()
    if portable and not portable_events <= PORTABLE_EVENTS:
        say(
            f"{root}/{PORTABLE_HOOKS_FILE} names events not every harness knows: {sorted(portable_events - PORTABLE_EVENTS)} "
            "(give each its own file named after it, e.g. hooks/worktree-remove.json)"
        )
    for path in extra_files:
        rel = f"hooks/{path.name}"
        data = _load(root, rel)
        events = list((data or {}).get("hooks", {}))
        if len(events) != 1:
            say(f"{root}/{rel}: a hook file beside hooks.json holds exactly one event, got {events}")
            continue
        event = events[0]
        if path.stem != event_file_stem(event):
            say(f"{root}/{rel}: must be named after the one event it holds: hooks/{event_file_stem(event)}.json")
        if event in PORTABLE_EVENTS:
            say(f"{root}/{rel}: {event} is an event every harness knows; it belongs in {PORTABLE_HOOKS_FILE}")
        if event in portable_events:
            say(f"{root}/{rel}: {event} is also in {PORTABLE_HOOKS_FILE}")
    hook_files = ([(PORTABLE_HOOKS_FILE, portable)] if portable else []) + [
        (f"hooks/{p.name}", _load(root, f"hooks/{p.name}")) for p in extra_files
    ]
    for path, data in hook_files:
        for groups in (data or {}).get("hooks", {}).values():
            for group in groups:
                for handler in group.get("hooks", []):
                    cmd = str(handler.get("command", ""))
                    if handler.get("type") != "command":
                        say(f"{root}/{path}: handler type must be command")
                    if not cmd.startswith(HOOK_LAUNCHER):
                        say(f"{root}/{path}: hook command should start with {HOOK_LAUNCHER!r} (one launcher, uv already required)")
                    else:
                        script = cmd[len(HOOK_LAUNCHER) :].split('"', 1)[0]
                        if not (root / script).is_file():
                            say(f"{root}/{path}: hook script {script} does not exist")
                        else:
                            problems.extend(_check_hook_script(root / script))
                    if cmd.count("${") != 1:
                        say(f"{root}/{path}: the plugin root must be the only placeholder in a hook command")
                    if not isinstance(handler.get("timeout"), int) or handler["timeout"] <= 0:
                        say(f"{root}/{path}: every handler needs a positive integer timeout")
    if hook_files:
        # Claude Code always loads hooks/hooks.json and reads the manifest field as
        # *additional* files: naming the default again fails the plugin at marketplace
        # install time ("Duplicate hooks file detected", Claude Code 2.1.270), a check
        # that neither --plugin-dir nor `claude plugin validate` runs. Codex discovers
        # hooks/hooks.json only when its extensions carry no `hooks`; an explicit value
        # replaces that discovery. So the Claude manifest names exactly the per-event
        # files and Codex's extensions name the portable file (references/hooks.md).
        expected = [f"./hooks/{p.name}" for p in extra_files]
        if claude:
            field = claude.get("hooks")
            named = [] if field is None else ([field] if isinstance(field, str) else list(field))
            if f"./{PORTABLE_HOOKS_FILE}" in named:
                say(
                    f"{root}: .claude-plugin/plugin.json names ./{PORTABLE_HOOKS_FILE}, which Claude Code loads on its own: "
                    "a marketplace install fails with 'Duplicate hooks file detected' (name only the per-event files)"
                )
            elif sorted(named) != sorted(expected):
                want = "no hooks field (hooks/hooks.json loads on its own)" if not expected else f"exactly {expected}"
                say(f"{root}: .claude-plugin/plugin.json hooks should be {want}, got {field!r}")
            elif field is not None and isinstance(field, list) and len(field) == 1:
                say(f"{root}: .claude-plugin/plugin.json hooks: one file is a string, {field[0]!r}, not a one-element array")
        if codex_ext and codex_ext.get("hooks") is not None and not portable:
            say(f'{root}: plugin.json extensions["com.openai"].hooks names ./{PORTABLE_HOOKS_FILE}, which does not exist')
        if agent and "hooks" in agent:
            say(f"{root}: Agent Plugins 1.0 defines no hooks component; drop `hooks` from plugin.json")
    elif claude and claude.get("hooks") is not None:
        say(f"{root}: .claude-plugin/plugin.json has a hooks field but hooks/ holds no hook file: {claude['hooks']!r}")

    # -- this plugin's own entry in the shared, repo-level marketplaces
    expected_rel = "./" if root == repo_root else f"./{root.relative_to(repo_root).as_posix()}"
    if claude and claude_market:
        entries = {p.get("name"): p for p in claude_market.get("plugins", [])}
        if name not in entries:
            say(f".claude-plugin/marketplace.json does not list {name}")
        elif not _source_matches(entries[name].get("source"), expected_rel):
            say(f".claude-plugin/marketplace.json: {name} source does not point at {expected_rel} (got {entries[name].get('source')!r})")
    if codex_ext and codex_market:
        entries = {p.get("name"): p for p in codex_market.get("plugins", [])}
        if name not in entries:
            say(f".agents/plugins/marketplace.json does not list {name}")
        else:
            entry = entries[name]
            expected_kind = "local" if root == repo_root else "git-subdir"
            src = entry.get("source", {})
            if src.get("source") != expected_kind:
                say(f".agents/plugins/marketplace.json: {name} source.source should be {expected_kind}, got {src.get('source')!r}")
            if not _source_matches(src, expected_rel):
                say(f".agents/plugins/marketplace.json: {name} source does not point at {expected_rel} (got {src!r})")
            policy = entry.get("policy")
            if policy is None:
                say(f".agents/plugins/marketplace.json: {name} entry lacks policy (Codex always expects it)")
            else:
                if policy.get("installation") not in {"NOT_AVAILABLE", "AVAILABLE", "INSTALLED_BY_DEFAULT"}:
                    say(f".agents/plugins/marketplace.json: {name} policy.installation is invalid: {policy.get('installation')!r}")
                if policy.get("authentication") not in {"ON_INSTALL", "ON_USE"}:
                    say(f".agents/plugins/marketplace.json: {name} policy.authentication is invalid: {policy.get('authentication')!r}")
            if "category" not in entry:
                say(f".agents/plugins/marketplace.json: {name} entry lacks category (Codex always expects it)")

    # -- skills: held equivalent across ecosystems, so the Claude manifest's own field is
    # the single source of truth (Agent Plugins 1.0 has no skills field, and Codex reads
    # this same directory by ./skills convention rather than naming it anywhere)
    if claude and claude.get("skills"):
        problems.extend(_check_skills(root / claude["skills"], f"{root}/.claude-plugin/plugin.json"))

    return problems


def _collect_dev_problems(repo_root: Path, roots: list[Path], claude_market: dict | None) -> list[str]:
    """Maintainer `dev/` plugins: claude-only (no root `plugin.json` of their own) AND
    nested under another discovered plugin root, whose version they must track.

    Both halves of that discriminator are load-bearing. Claude-only alone is not enough:
    a legitimate sibling PRODUCT plugin can be claude-only too (ecosystems restricted to
    `["claude"]`) and can legitimately carry an MCP server — nesting is what actually
    distinguishes "this directory belongs to another plugin" from "this is its own
    top-level plugin", and a top-level claude-only entry is already fully checked in its
    own right by `_collect_plugin_problems` via its own entry in `roots`.

    Candidates are enumerated from `roots` (the independently discovered plugin
    directories), NOT from marketplace entries: `roots` exists with or without a local
    marketplace, so a hub-mode repo (no `.claude-plugin/marketplace.json` at all) still
    gets its dev plugins validated — a marketplace-driven enumeration would silently
    validate nothing in the very mode this reshape exists to enable.

    The marketplace, when present, is instead used for a second, narrower pass: a
    `source` that resolves to a manifest `find_plugin_roots`'s walk never reached (outside
    `repo_root`, or inside a pruned directory name) is invisible to the roots-based pass
    above. Rather than silently trusting or silently ignoring it, it is reported as a
    named ambiguity the checker cannot resolve on its own.
    """
    problems: list[str] = []
    say = problems.append
    root_set = set(roots)

    dev_candidates: dict[Path, Path] = {}
    for candidate in roots:
        if _load(candidate, "plugin.json"):
            continue  # has its own root manifest: a real product, never a dev plugin
        ancestors = [r for r in root_set if r != candidate and r in candidate.parents]
        if not ancestors:
            continue  # claude-only but NOT nested under anything: a legitimate top-level
            # claude-only PRODUCT (e.g. ecosystems restricted to ["claude"]) — not a dev
            # plugin, and already fully checked via its own entry in `roots`
        dev_candidates[candidate] = max(ancestors, key=lambda p: len(p.parts))

    for candidate, product_root in dev_candidates.items():
        dev_manifest = _load(candidate, ".claude-plugin/plugin.json")
        product_version = _root_version(product_root)
        if product_version is not None and dev_manifest.get("version") != product_version:
            say(f"{candidate}: version {dev_manifest.get('version')} lags the product version {product_version}")
        if "mcpServers" in dev_manifest:
            say(f"{candidate}: a skills plugin should carry knowledge, never a server")
        dev_skills = (candidate / dev_manifest.get("skills", "./skills/")).resolve()
        product_claude = _load(product_root, ".claude-plugin/plugin.json")
        if product_claude and product_claude.get("skills"):
            product_skills = (product_root / product_claude["skills"]).resolve()
            if product_skills == dev_skills or dev_skills in product_skills.parents or product_skills in dev_skills.parents:
                say(f"{candidate}: the product plugin would ship the dev skills: keep the two skills trees disjoint")
        problems.extend(_check_skills(dev_skills, f"{candidate}/.claude-plugin/plugin.json"))

    if claude_market:
        for entry in claude_market.get("plugins", []):
            src_path = _resolve_local_source_path(entry.get("source"))
            if src_path is None:
                continue
            candidate = (repo_root / src_path).resolve()
            if candidate == repo_root.resolve() or candidate in root_set:
                continue  # the repo root, or an already-discovered (and already checked) plugin root
            if not _load(candidate, ".claude-plugin/plugin.json"):
                say(f"marketplace lists {entry.get('name')} at {entry.get('source')!r} but no plugin.json is there")
                continue
            say(
                f"marketplace entry {entry.get('name')!r} at {entry.get('source')!r} resolves to {candidate}, "
                "which the plugin-root scan never reached (outside the repo tree, or inside a directory "
                "find_plugin_roots skips) — the checker cannot verify this entry; check it by hand"
            )
    return problems


def _schema_version(url: str) -> str:
    parts = url.rstrip("/").split("/")
    return parts[-2] if len(parts) >= 2 else ""


def _check_skills(skills_dir: Path, owner: str) -> list[str]:
    out = []
    if not skills_dir.is_dir():
        return [f"{owner}: skills path {skills_dir} is not a directory"]
    for skill in sorted(p for p in skills_dir.iterdir() if p.is_dir()):
        md = skill / "SKILL.md"
        if not md.is_file():
            out.append(f"{skill}: no SKILL.md")
            continue
        match = _FRONTMATTER.match(md.read_text())
        if not match:
            out.append(f"{md}: no YAML front matter")
            continue
        fields = {}
        for line in match.group(1).splitlines():
            key, sep, value = line.partition(":")
            if sep and not line.startswith(" "):
                fields[key.strip()] = value.strip().strip("\"'")
        if fields.get("name") != skill.name:
            out.append(f"{md}: front matter name {fields.get('name')!r} must equal the directory name")
        if "description" not in fields:
            out.append(f"{md}: front matter needs a description that says when to load it")
    return out


def _check_hook_script(script: Path) -> list[str]:
    """`uv run --no-project python` installs nothing: only the stdlib may be imported."""
    try:
        tree = ast.parse(script.read_text())
    except SyntaxError as exc:
        return [f"{script}: syntax error ({exc})"]
    imported = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imported.update(a.name.split(".")[0] for a in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module and node.level == 0:
            imported.add(node.module.split(".")[0])
    stdlib = getattr(sys, "stdlib_module_names", None)
    if stdlib is None:
        return []
    foreign = sorted(imported - set(stdlib))
    return [f"{script}: non-stdlib imports {foreign} (the hook runs under uv run --no-project)"] if foreign else []


# --- pytest entry point (collected when this file is copied into a test suite) ---


def test_plugin_structure():
    problems = collect_problems(REPO_ROOT)
    assert not problems, "\n".join(problems)


# --- CLI ---


def main(argv: list[str] | None = None) -> int:
    import argparse

    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    args = parser.parse_args(argv)
    problems = collect_problems(args.root.resolve())
    for p in problems:
        print(f"- {p}")
    print("ok: plugin structure is consistent" if not problems else f"{len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
