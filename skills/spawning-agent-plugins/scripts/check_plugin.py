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
`.claude-plugin/`, `.codex-plugin/` or `plugin.json`).
"""

from __future__ import annotations

import ast
import json
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
        if any((candidate / m).exists() for m in (".claude-plugin", ".codex-plugin", "plugin.json")):
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


def collect_problems(root: Path) -> list[str]:
    problems: list[str] = []
    say = problems.append
    try:
        claude = _load(root, ".claude-plugin/plugin.json")
        codex = _load(root, ".codex-plugin/plugin.json")
        agent = _load(root, "plugin.json")
        claude_market = _load(root, ".claude-plugin/marketplace.json")
        codex_market = _load(root, ".agents/plugins/marketplace.json")
    except ValueError as exc:
        return [str(exc)]
    present = {k: v for k, v in (("claude", claude), ("codex", codex), ("agent-plugins", agent)) if v}
    if not present:
        return ["no plugin manifest found (.claude-plugin/plugin.json, .codex-plugin/plugin.json, plugin.json)"]

    # -- identity: one name, one version, everywhere
    names = {k: v.get("name") for k, v in present.items()}
    if len(set(names.values())) != 1:
        say(f"plugin names differ across manifests: {names}")
    versions = {k: v.get("version") for k, v in present.items()}
    if len(set(versions.values())) != 1:
        say(f"plugin versions differ across manifests: {versions}")
    version = next(iter(versions.values()))
    name = next(iter(names.values()))

    # -- Agent Plugins 1.0: whitelisted fields, matching schema versions
    if agent:
        extra = set(agent) - AGENT_PLUGIN_FIELDS
        if extra:
            say(f"plugin.json carries fields Agent Plugins 1.0 does not define: {sorted(extra)} (hooks/skills/mcpServers live in the other manifests)")
        if not str(agent.get("$schema", "")).startswith("https://agent-plugins.org/schemas/"):
            say("plugin.json needs the agent-plugins.org $schema")
        agent_mcp = _load(root, "mcp.json")
        if agent_mcp:
            if _schema_version(agent_mcp.get("$schema", "")) != _schema_version(agent.get("$schema", "")):
                say("mcp.json and plugin.json cite different agent-plugins.org schema versions")
            for sname, server in agent_mcp.get("mcpServers", {}).items():
                if server.get("type") != "stdio" and "url" not in server:
                    say(f"mcp.json server {sname}: needs type stdio (or a url)")
                for key in server.get("env", {}):
                    if key in FORBIDDEN_MCP_ENV_KEYS:
                        say(f"mcp.json server {sname}: env key {key} is reserved by the spec")
                for text in json.dumps(server.get("args", [])), json.dumps(server.get("env", {})):
                    if re.search(r"\$\{[^}]*:-", text):
                        say(f"mcp.json server {sname}: ${{VAR:-default}} fallback syntax must stay literal under Agent Plugins 1.0")

    # -- MCP manifests: bare command, own file per ecosystem, no root .mcp.json
    if (root / ".mcp.json").exists():
        say("a root .mcp.json exists: Claude Code reads it as project-scope config that never expands ${CLAUDE_PLUGIN_ROOT}")
    mcp_files = {"claude": ".claude-plugin/mcp.json", "codex": ".codex-plugin/mcp.json", "agent-plugins": "mcp.json"}
    launches: dict[str, dict] = {}
    for eco, rel in mcp_files.items():
        data = _load(root, rel)
        if not data:
            continue
        for sname, server in data.get("mcpServers", {}).items():
            launches.setdefault(sname, {})[eco] = server
            if "$" in str(server.get("command", "")):
                say(f"{rel} server {sname}: placeholder in `command`; no ecosystem expands one there")
            if eco != "claude":
                if any("$" in str(a) for a in server.get("args", [])) or "$" in json.dumps(server.get("env", {})):
                    say(f"{rel} server {sname}: placeholders belong only in the Claude Code manifest (.claude-plugin/mcp.json env)")
            else:
                if any("$" in str(a) for a in server.get("args", [])):
                    say(f"{rel} server {sname}: keep placeholders in env, not args (a literal ${{...}} arg means the server never started)")
    for sname, per_eco in launches.items():
        cmds = {eco: (s.get("command"), tuple(s.get("args", []))) for eco, s in per_eco.items()}
        if len(set(cmds.values())) != 1:
            say(f"server {sname} is launched differently per ecosystem: {cmds}")
        for eco, s in per_eco.items():
            joined = " ".join(map(str, s.get("args", [])))
            pin = re.search(r"==\s*([0-9][^\s\"']*)", joined)
            if pin and pin.group(1) != version:
                say(f"{mcp_files[eco]} pins {pin.group(1)} but the plugin version is {version} (add the pin to your version bumper)")
    if claude and "mcpServers" in claude and claude["mcpServers"] != "./.claude-plugin/mcp.json":
        say("Claude manifest mcpServers should be ./.claude-plugin/mcp.json (never a root .mcp.json)")
    if codex and "mcpServers" in codex and codex["mcpServers"] != "./.codex-plugin/mcp.json":
        say("Codex manifest mcpServers should be ./.codex-plugin/mcp.json so the Claude placeholder never reaches Codex")
    if claude and "mcpServers" not in claude and (root / ".claude-plugin/mcp.json").exists():
        say(".claude-plugin/mcp.json exists but .claude-plugin/plugin.json never names it (mcpServers)")
    if codex and "mcpServers" not in codex and (root / ".codex-plugin/mcp.json").exists():
        say(".codex-plugin/mcp.json exists but .codex-plugin/plugin.json never names it (mcpServers)")
    if claude and "mcpServers" in claude and not (root / ".claude-plugin/mcp.json").exists():
        say(".claude-plugin/plugin.json names an mcpServers file that does not exist")
    if codex and "mcpServers" in codex and not (root / ".codex-plugin/mcp.json").exists():
        say(".codex-plugin/plugin.json names an mcpServers file that does not exist")

    # -- hooks: portable events in hooks/hooks.json, one file per other event, named after it
    hooks_dir = root / "hooks"
    portable = _load(root, PORTABLE_HOOKS_FILE)
    extra_files = sorted(p for p in hooks_dir.glob("*.json") if p.name != "hooks.json") if hooks_dir.is_dir() else []
    portable_events = set(portable.get("hooks", {})) if portable else set()
    if portable and not portable_events <= PORTABLE_EVENTS:
        say(
            f"{PORTABLE_HOOKS_FILE} names events not every harness knows: {sorted(portable_events - PORTABLE_EVENTS)} "
            "(give each its own file named after it, e.g. hooks/worktree-remove.json)"
        )
    for path in extra_files:
        rel = f"hooks/{path.name}"
        data = _load(root, rel)
        events = list((data or {}).get("hooks", {}))
        if len(events) != 1:
            say(f"{rel}: a hook file beside hooks.json holds exactly one event, got {events}")
            continue
        event = events[0]
        if path.stem != event_file_stem(event):
            say(f"{rel}: must be named after the one event it holds: hooks/{event_file_stem(event)}.json")
        if event in PORTABLE_EVENTS:
            say(f"{rel}: {event} is an event every harness knows; it belongs in {PORTABLE_HOOKS_FILE}")
        if event in portable_events:
            say(f"{rel}: {event} is also in {PORTABLE_HOOKS_FILE}")
    hook_files = ([(PORTABLE_HOOKS_FILE, portable)] if portable else []) + [
        (f"hooks/{p.name}", _load(root, f"hooks/{p.name}")) for p in extra_files
    ]
    for path, data in hook_files:
        for groups in (data or {}).get("hooks", {}).values():
            for group in groups:
                for handler in group.get("hooks", []):
                    cmd = str(handler.get("command", ""))
                    if handler.get("type") != "command":
                        say(f"{path}: handler type must be command")
                    if not cmd.startswith(HOOK_LAUNCHER):
                        say(f"{path}: hook command should start with {HOOK_LAUNCHER!r} (one launcher, uv already required)")
                    else:
                        script = cmd[len(HOOK_LAUNCHER) :].split('"', 1)[0]
                        if not (root / script).is_file():
                            say(f"{path}: hook script {script} does not exist")
                        else:
                            problems.extend(_check_hook_script(root / script))
                    if cmd.count("${") != 1:
                        say(f"{path}: the plugin root must be the only placeholder in a hook command")
                    if not isinstance(handler.get("timeout"), int) or handler["timeout"] <= 0:
                        say(f"{path}: every handler needs a positive integer timeout")
    if hook_files:
        # The two loaders read the manifest's `hooks` field with opposite semantics.
        # Claude Code always loads hooks/hooks.json and reads the field as *additional*
        # files: naming the default again fails the plugin at marketplace install time
        # ("Duplicate hooks file detected", Claude Code 2.1.270), a check that neither
        # --plugin-dir nor `claude plugin validate` runs. Codex discovers hooks/hooks.json
        # only when the manifest defines no `hooks`; an explicit value replaces that
        # discovery. So the Claude manifest names exactly the per-event files and the
        # Codex manifest names the portable file (references/hooks.md).
        expected = [f"./hooks/{p.name}" for p in extra_files]
        if claude:
            field = claude.get("hooks")
            named = [] if field is None else ([field] if isinstance(field, str) else list(field))
            if f"./{PORTABLE_HOOKS_FILE}" in named:
                say(
                    f".claude-plugin/plugin.json names ./{PORTABLE_HOOKS_FILE}, which Claude Code loads on its own: "
                    "a marketplace install fails with 'Duplicate hooks file detected' (name only the per-event files)"
                )
            elif sorted(named) != sorted(expected):
                want = "no hooks field (hooks/hooks.json loads on its own)" if not expected else f"exactly {expected}"
                say(f".claude-plugin/plugin.json hooks should be {want}, got {field!r}")
            elif field is not None and isinstance(field, list) and len(field) == 1:
                say(f".claude-plugin/plugin.json hooks: one file is a string, {field[0]!r}, not a one-element array")
        if codex and codex.get("hooks") is not None:
            if codex["hooks"] != f"./{PORTABLE_HOOKS_FILE}":
                say(
                    f".codex-plugin/plugin.json hooks must be ./{PORTABLE_HOOKS_FILE} or absent: an explicit value replaces "
                    "Codex's default discovery, and a per-event file may hold an event Codex does not know"
                )
            elif not portable:
                say(f".codex-plugin/plugin.json names ./{PORTABLE_HOOKS_FILE}, which does not exist")
        if agent and "hooks" in agent:
            say("Agent Plugins 1.0 defines no hooks component; drop `hooks` from plugin.json")
    elif claude and claude.get("hooks") is not None:
        say(f".claude-plugin/plugin.json has a hooks field but hooks/ holds no hook file: {claude['hooks']!r}")

    # -- marketplaces
    if claude and claude_market:
        entries = {p.get("name"): p for p in claude_market.get("plugins", [])}
        if name not in entries:
            say(f".claude-plugin/marketplace.json does not list {name}")
        elif entries[name].get("source") != "./":
            say(f".claude-plugin/marketplace.json: {name} source should be ./ (the repo root is the plugin)")
    if codex and codex_market:
        entries = {p.get("name"): p for p in codex_market.get("plugins", [])}
        if name not in entries:
            say(f".agents/plugins/marketplace.json does not list {name}")
        else:
            entry = entries[name]
            if entry.get("source", {}).get("source") != "local":
                say(".agents/plugins/marketplace.json: source.source should be local")
            for key in ("policy", "category"):
                if key not in entry:
                    say(f".agents/plugins/marketplace.json: {name} entry lacks {key} (Codex always expects it)")
    if codex:
        for key in ("displayName", "shortDescription", "longDescription", "developerName", "category", "capabilities"):
            if not codex.get("interface", {}).get(key):
                say(f".codex-plugin/plugin.json interface lacks {key}")
        if not codex.get("author", {}).get("name"):
            say(".codex-plugin/plugin.json needs author.name")
        if not re.fullmatch(r"\d+\.\d+\.\d+([-+][0-9A-Za-z.-]+)?", str(codex.get("version", ""))):
            say(".codex-plugin/plugin.json version must be strict semver")

    # -- skills
    for manifest, rel in ((claude, ".claude-plugin/plugin.json"), (codex, ".codex-plugin/plugin.json")):
        if manifest and manifest.get("skills"):
            problems.extend(_check_skills(root / manifest["skills"], rel))
    if claude and codex and claude.get("skills") != codex.get("skills"):
        say("Claude and Codex manifests point at different skills directories")

    # -- dev plugin
    if claude_market:
        for entry in claude_market.get("plugins", []):
            src = entry.get("source", "")
            if src in ("./", "") or not isinstance(src, str):
                continue
            dev_manifest = _load(root, f"{src.strip('/')}/.claude-plugin/plugin.json")
            if not dev_manifest:
                say(f"marketplace lists {entry.get('name')} at {src} but no plugin.json is there")
                continue
            if dev_manifest.get("name") != entry.get("name"):
                say(f"{src}: manifest name {dev_manifest.get('name')} differs from marketplace entry {entry.get('name')}")
            if dev_manifest.get("version") != version:
                say(f"{src}: version {dev_manifest.get('version')} lags the product version {version}")
            if "mcpServers" in dev_manifest:
                say(f"{src}: a skills plugin should carry knowledge, never a server")
            dev_skills = (root / src.strip("/") / dev_manifest.get("skills", "./skills/")).resolve()
            if claude and claude.get("skills"):
                product_skills = (root / claude["skills"]).resolve()
                if product_skills == dev_skills or dev_skills in product_skills.parents or product_skills in dev_skills.parents:
                    say("the product plugin would ship the dev skills: keep the two skills trees disjoint")
            problems.extend(_check_skills(dev_skills, f"{src}/.claude-plugin/plugin.json"))
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
