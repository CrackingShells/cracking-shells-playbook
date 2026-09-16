#!/usr/bin/env python3
"""Spawn the three-ecosystem plugin structure into an existing repository.

One JSON spec in, the manifests Claude Code, Codex and Agent Plugins 1.0 read
out. The layout is the one proven on CrackingShells/colgrep-mcp (Claude Code
verified end-to-end; Codex and Agent Plugins 1.0 documented against their
specs, see references/manifests.md):

    plugin.json                      Agent Plugins 1.0 manifest (whitelisted fields only);
                                      also the manifest Codex parses, with its extras under
                                      extensions["com.openai"] when codex is an ecosystem
    mcp.json                         Agent Plugins 1.0 MCP manifest, also Codex's (auto-wired
                                      by convention: no separate Codex plugin folder is ever written) [mcp]
    .claude-plugin/plugin.json       Claude Code manifest
    .claude-plugin/marketplace.json  Claude Code marketplace (product [+ dev] plugin)
    .claude-plugin/mcp.json          Claude Code MCP manifest              [mcp]
    .agents/plugins/marketplace.json Codex marketplace
    hooks/hooks.json                 portable hook events, auto-loaded     [hooks]
    hooks/<event>.json               one file per non-portable event       [hooks]
    hooks/<script>.py                one stdlib, fail-open hook script     [hooks]
    dev/.claude-plugin/plugin.json   maintainer skills plugin              [dev]
    dev/README.md, dev/skills/       "                                     [dev]

Subcommands:
    init             write an example spec next to the repo, prefilled from git and
                     from whatever version source the tree already has
    spawn            write the manifests from a spec (refuses to overwrite unless
                     --force; marketplaces are merged, never clobbered; dev/README.md
                     is seeded once and never regenerated, --force included, once a
                     maintainer starts editing it by hand)
    install-snippet  print the README install section for the spec's names

Stdlib only, Python >= 3.9. Run it from anywhere: every path is resolved
against --root (default: the current directory).
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

AGENT_PLUGINS_SCHEMA = "https://agent-plugins.org/schemas/1.0.0/{name}.schema.json"
HOOK_LAUNCHER = 'uv run --no-project --quiet python "${{CLAUDE_PLUGIN_ROOT}}/{script}"'
HERE = Path(__file__).resolve().parent
HOOK_TEMPLATE = HERE.parent / "assets" / "hooks" / "policy_template.py"

sys.path.insert(0, str(HERE))
from check_plugin import PORTABLE_EVENTS, event_file_stem  # noqa: E402  (the checker owns both)

# --- spec ------------------------------------------------------------------------------


def _die(msg: str) -> None:
    print(f"spawn_plugin: {msg}", file=sys.stderr)
    sys.exit(2)


def load_spec(path: Path) -> dict:
    try:
        spec = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as exc:
        _die(f"cannot read spec {path}: {exc}")
    for key in ("name", "description"):
        if not spec.get(key):
            _die(f"spec needs a non-empty {key!r}")
    if not re.fullmatch(r"[a-z][a-z0-9-]*", spec["name"]):
        _die("spec name must be kebab-case (Codex and Claude Code both namespace components by it)")
    spec.setdefault("ecosystems", ["claude", "codex", "agent-plugins"])
    spec.setdefault("keywords", [])
    spec.setdefault("author", {})
    if spec.get("marketplace") and not (isinstance(spec["marketplace"], dict) and spec["marketplace"].get("hub")):
        # Hub mode (`marketplace` truthy) suppresses this repo's own marketplace files, but
        # two things still need to know which marketplace lists this plugin: dev_readme() and
        # install_snippet(), both documentation outputs, never a manifest a loader reads. The
        # generator must never infer that identity (an owner/repo-shaped `repository` fallback
        # is the product repo, not the hub, and is wrong by construction) — it reads a recorded
        # answer, and refusing to generate one is better than generating wrong install
        # instructions that fail silently in a reader's hands.
        _die(
            "spec sets `marketplace` (hub mode) but records no hub repository; add "
            '`marketplace.hub` — e.g. "marketplace": {"hub": "https://github.com/<org>/<hub-repo>"} '
            "— naming the repository that owns and hosts the marketplace listing this plugin. "
            "Ask a human which repository that is; never guess it from `repository` (this "
            "plugin's own repo) or from the marketplace name."
        )
    if spec.get("hooks"):
        _check_hooks_spec(spec["hooks"])
    if spec.get("plugins"):
        _check_plugins_spec(spec["plugins"])
    return spec


def _check_plugins_spec(plugins: list) -> None:
    """Each entry needs a kebab-case `name` and a `dir`; no two entries may share either."""
    seen_names: set = set()
    seen_dirs: set = set()
    for entry in plugins:
        name = entry.get("name")
        if not name or not re.fullmatch(r"[a-z][a-z0-9-]*", name):
            _die(f"plugins entry needs a kebab-case name, got {name!r}")
        dir_ = entry.get("dir")
        if dir_ is None or not isinstance(dir_, str):
            _die(f"plugins entry {name!r} needs a dir")
        if not entry.get("description"):
            _die(f"plugins entry {name!r} needs a non-empty description")
        if name in seen_names:
            _die(f"duplicate plugin name in plugins[]: {name!r}")
        dir_key = dir_.strip("/")
        if dir_key in seen_dirs:
            _die(f"duplicate plugin dir in plugins[]: {dir_!r}")
        seen_names.add(name)
        seen_dirs.add(dir_key)
        if entry.get("hooks"):
            _check_hooks_spec(entry["hooks"])


def plugin_entries(spec: dict) -> list[dict]:
    """Normalised per-plugin dicts: what `spawn()`'s ecosystem branches loop over.

    A single-plugin spec (no top-level `plugins`) yields a one-element list holding
    the spec itself, `dir` defaulted to `""` (a root plugin) — so single-plugin
    behaviour is byte-for-byte unchanged. A multi-plugin spec yields one normalised
    entry per `plugins[]` item, each shaped exactly like a single-plugin spec (so
    every build_* function needs no change): `author`, `homepage`, `repository`,
    `license`, `ecosystems` and `keywords` fall back to the top-level spec's value
    when the entry doesn't set its own.
    """
    if not spec.get("plugins"):
        entry = dict(spec)
        entry.setdefault("dir", "")
        entry.pop("plugins", None)
        return [entry]
    entries = []
    for raw in spec["plugins"]:
        entry = dict(raw)
        entry["dir"] = (entry.get("dir") or "").strip("/")
        for key in ("author", "homepage", "repository", "license", "ecosystems", "keywords"):
            entry.setdefault(key, spec.get(key))
        entries.append(entry)
    return entries


def _event_name(event) -> str:
    return event["event"] if isinstance(event, dict) else str(event)


def _check_hooks_spec(hooks: dict) -> None:
    """Portable events in `portable`, every other event in `extra`, one file each.

    `hooks/hooks.json` is the file Claude Code always loads and Codex discovers by
    default, so only events every hook-capable harness knows may sit in it. An
    event outside that set gets its own file named after it (`WorktreeRemove` ->
    `hooks/worktree-remove.json`), the only files the Claude Code manifest names.
    """
    if "claude_only" in hooks:
        _die("hooks.claude_only was retired: list each non-portable event under hooks.extra (one file per event)")
    if not hooks.get("script"):
        _die("hooks.script is required (the one stdlib script every handler launches)")
    portable = [_event_name(e) for e in hooks.get("portable", [])]
    extra = [_event_name(e) for e in hooks.get("extra", [])]
    if not portable and not extra:
        _die("hooks needs at least one event under portable or extra")
    foreign = sorted(set(portable) - PORTABLE_EVENTS)
    if foreign:
        _die(f"hooks.portable names events not every harness knows: {foreign}; move them to hooks.extra")
    misplaced = sorted(set(extra) & PORTABLE_EVENTS)
    if misplaced:
        _die(f"hooks.extra names portable events: {misplaced}; they belong in hooks.portable")
    if len(set(extra)) != len(extra) or set(extra) & set(portable):
        _die("each hook event appears once, in hooks.portable or in hooks.extra")


def extra_hook_files(hooks: dict) -> list[str]:
    """The per-event files, in spec order: `./hooks/worktree-remove.json` for WorktreeRemove."""
    return [f"./hooks/{event_file_stem(_event_name(e))}.json" for e in hooks.get("extra", [])]


def resolve_version(spec: dict, root: Path) -> str:
    """The plugin version: a literal, or read from the one version source the tree has."""
    if spec.get("version"):
        return str(spec["version"])
    source = spec.get("version_from")
    if not source:
        return "0.1.0"
    kind, _, rel = source.partition(":")
    path = root / (rel or {"pyproject": "pyproject.toml", "package.json": "package.json", "cargo": "Cargo.toml"}[kind])
    try:
        text = path.read_text()
    except FileNotFoundError:
        # Loud, not silent: this is the correct fallback for a version source a sibling
        # leaf hasn't created YET (a plugin whose version_from names a not-yet-built
        # file resolves to 0.1.0 by design — that is expected, not a defect), but the
        # identical symptom also covers a typo'd version_from, and a silent 0.1.0 there
        # would ship a wrong version into a manifest Nest resolves. Warn by name so a
        # typo is visible, without turning this into a hard failure that would re-block
        # every entry after a legitimately-not-yet-created one.
        print(
            f"warning: version_from {source!r} resolves to {path}, which does not exist; falling back to "
            "0.1.0. If this plugin's version source should already exist, this is a typo in version_from "
            "rather than an unfinished sibling leaf — check the path.",
            file=sys.stderr,
        )
        return "0.1.0"
    if kind == "pyproject":
        block = re.search(r"^\[project\]\n(.*?)(?=^\[|\Z)", text, re.S | re.M)
        match = block and re.search(r'^version\s*=\s*"([^"]+)"', block.group(1), re.M)
    elif kind == "package.json":
        match = re.search(r'"version"\s*:\s*"([^"]+)"', text)
    elif kind == "cargo":
        block = re.search(r"^\[package\]\n(.*?)(?=^\[|\Z)", text, re.S | re.M)
        match = block and re.search(r'^version\s*=\s*"([^"]+)"', block.group(1), re.M)
    else:
        _die(f"unknown version_from kind {kind!r} (pyproject | package.json | cargo)")
    if not match:
        _die(f"no version found in {path}")
    return match.group(1)


# --- manifest builders ---------------------------------------------------------------------


def _identity(spec: dict, version: str, *, homepage: bool = True, license_: bool = True) -> dict:
    out = {
        "name": spec["name"],
        "version": version,
        "description": spec["description"],
    }
    if spec.get("author"):
        out["author"] = spec["author"]
    if homepage and spec.get("homepage"):
        out["homepage"] = spec["homepage"]
    if spec.get("repository"):
        out["repository"] = spec["repository"]
    if license_ and spec.get("license"):
        out["license"] = spec["license"]
    if spec.get("keywords"):
        out["keywords"] = spec["keywords"]
    return out


def _sub_version(value, version: str):
    if isinstance(value, str):
        return value.replace("{version}", version)
    if isinstance(value, list):
        return [_sub_version(v, version) for v in value]
    if isinstance(value, dict):
        return {k: _sub_version(v, version) for k, v in value.items()}
    return value


def build_mcp(spec: dict, version: str, ecosystem: str) -> dict:
    mcp = spec["mcp"]
    server = {"command": mcp["command"], "args": _sub_version(mcp.get("args", []), version)}
    if "$" in server["command"]:
        _die("mcp.command carries a placeholder: no ecosystem expands placeholders in `command`")
    if ecosystem == "agent-plugins":
        server = {"type": "stdio", **server}
        if mcp.get("env"):
            server["env"] = mcp["env"]
        return {"$schema": AGENT_PLUGINS_SCHEMA.format(name="mcp"), "mcpServers": {mcp["server"]: server}}
    if mcp.get("env"):
        server["env"] = dict(mcp["env"])
    if ecosystem == "claude" and mcp.get("claude_env"):
        server["env"] = {**server.get("env", {}), **mcp["claude_env"]}
    return {"mcpServers": {mcp["server"]: server}}


def build_agent_plugin(spec: dict, version: str) -> dict:
    """The root Agent Plugins 1.0 manifest — also the one Codex parses natively.

    Codex reads a root AP-conformant `plugin.json`, takes its Codex-specific data from
    `extensions["com.openai"]`, and auto-wires `skills` -> `./skills` and `mcp_servers`
    -> `./mcp.json` by convention (neither needs a field here). So when `codex` is one of
    the spec's ecosystems, this is Codex's only manifest: no separate Codex plugin
    folder is ever written.
    """
    out = {"$schema": AGENT_PLUGINS_SCHEMA.format(name="plugin"), **_identity(spec, version)}
    eco = set(spec.get("ecosystems", ["claude", "codex", "agent-plugins"]))
    if "codex" in eco:
        out["extensions"] = {"com.openai": build_codex_extensions(spec)}
    return out


def build_codex_extensions(spec: dict) -> dict:
    """Codex's `extensions["com.openai"]` payload: presentation metadata with no root
    slot in Agent Plugins 1.0, plus the portable hooks file when the spec opts in.
    """
    codex = spec.get("codex", {})
    interface = {
        "displayName": spec.get("displayName", spec["name"]),
        "shortDescription": codex.get("shortDescription", spec["description"]),
        "longDescription": codex.get("longDescription", spec["description"]),
        "developerName": spec.get("author", {}).get("name", ""),
        "category": codex.get("category", "Developer Tools"),
        "capabilities": codex.get("capabilities", ["Read"]),
    }
    if codex.get("defaultPrompt"):
        interface["defaultPrompt"] = codex["defaultPrompt"][:3]
    extensions = {"interface": interface}
    if spec.get("hooks") and spec["hooks"].get("codex"):
        # Codex discovers hooks/hooks.json only when the manifest defines no `hooks`; an
        # explicit value *replaces* that discovery, so this names the portable file and
        # never a per-event file whose event a Codex parser may not know
        # (references/hooks.md).
        extensions["hooks"] = "./hooks/hooks.json"
    return extensions


def build_claude_plugin(spec: dict, version: str) -> dict:
    out = {"name": spec["name"]}
    if spec.get("displayName"):
        out["displayName"] = spec["displayName"]
    out.update(_identity(spec, version))
    if spec.get("mcp"):
        out["mcpServers"] = "./.claude-plugin/mcp.json"
    if spec.get("hooks"):
        # Claude Code always loads hooks/hooks.json and reads this field as *additional*
        # files: naming the default again fails the whole plugin at marketplace install
        # time ("Duplicate hooks file detected", Claude Code 2.1.270), a check that
        # neither --plugin-dir nor `claude plugin validate` runs. So the field names
        # exactly the per-event files, one string when there is one, and is omitted
        # when every event is portable (references/hooks.md#one-file-per-event-class).
        files = extra_hook_files(spec["hooks"])
        if len(files) == 1:
            out["hooks"] = files[0]
        elif files:
            out["hooks"] = files
    if spec.get("skills"):
        out["skills"] = spec["skills"]
    return out


def _git_clone_url(repository: str | None, dir_: str) -> str:
    if not repository:
        _die(f"a plugin at {dir_!r} needs a `repository` to build its marketplace source (git-subdir)")
    return repository if repository.endswith(".git") else repository + ".git"


def _claude_source(dir_: str, repository: str | None):
    """A root plugin (`dir_ == ""`) is a plain relative path; a subdirectory plugin uses
    the `git-subdir` source shape (`url` + `path`), the only one that names a subdirectory."""
    if not dir_:
        return "./"
    return {"source": "git-subdir", "url": _git_clone_url(repository, dir_), "path": f"./{dir_}"}


def _codex_source(dir_: str, repository: str | None) -> dict:
    """Codex has no `github`-style shorthand, so a subdirectory source is the identical
    `git-subdir`/`url`/`path` shape Claude Code uses; the root case stays `local`."""
    if not dir_:
        return {"source": "local", "path": "./"}
    return {"source": "git-subdir", "url": _git_clone_url(repository, dir_), "path": f"./{dir_}"}


def build_claude_marketplace(spec: dict, entries: list[tuple[dict, str]]) -> dict:
    """One marketplace entry per plugin `entries` holds, plus one per declared `dev` sibling.

    `entries` is the (normalised plugin dict, resolved version) pairs for every plugin this
    spawn wrote into the `claude` ecosystem — one element for a single-plugin spec, one per
    `plugins[]` item for a multi-plugin spec. No entry ever carries `version`, `ref` or `sha`.
    """
    market = spec.get("claude_marketplace", {})
    out = {
        "name": market.get("name", f"{spec['name']}-marketplace"),
        "description": market.get("description", f"Marketplace for {spec['name']}."),
    }
    if spec.get("author"):
        out["owner"] = spec["author"]
    out["plugins"] = [
        {
            "name": entry["name"],
            "source": _claude_source(entry.get("dir", ""), entry.get("repository") or spec.get("repository")),
            "description": entry["description"],
        }
        for entry, _ in entries
    ]
    for entry, _ in entries:
        if entry.get("dev"):
            dev = entry["dev"]
            ddir = _prefixed(entry.get("dir", ""), dev.get("dir", "dev").strip("/"))
            out["plugins"].append(
                {
                    "name": dev["name"],
                    "source": _claude_source(ddir, entry.get("repository") or spec.get("repository")),
                    "description": dev.get("marketplace_description", dev["description"]),
                }
            )
    return out


def build_codex_marketplace(spec: dict, entries: list[tuple[dict, str]]) -> dict:
    """One marketplace entry per plugin `entries` holds. See `build_claude_marketplace`."""
    codex = spec.get("codex", {})
    return {
        "name": codex.get("marketplace_name", f"{spec['name']}-marketplace"),
        "interface": {"displayName": codex.get("marketplace_displayName", spec.get("displayName", spec["name"]))},
        "plugins": [
            {
                "name": entry["name"],
                "source": _codex_source(entry.get("dir", ""), entry.get("repository") or spec.get("repository")),
                # Codex's auth-policy enum is exactly ON_INSTALL | ON_USE, with no catch-all,
                # and it parses a marketplace file in one pass: "NONE" makes every entry in
                # the file unparseable, not just this one.
                "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
                "category": entry.get("codex", {}).get("category", codex.get("category", "Developer Tools")),
            }
            for entry, _ in entries
        ],
    }


def build_dev_plugin(spec: dict, version: str) -> dict:
    dev = spec["dev"]
    out = {"name": dev["name"]}
    if dev.get("displayName"):
        out["displayName"] = dev["displayName"]
    out.update(
        {
            "version": version,
            "description": dev["description"],
        }
    )
    for key in ("author", "homepage", "repository", "license"):
        if spec.get(key):
            out[key] = spec[key]
    if dev.get("keywords"):
        out["keywords"] = dev["keywords"]
    out["skills"] = "./skills/"
    return out


def _hook_entry(event: dict, script: str) -> dict:
    handler = {
        "type": "command",
        "command": HOOK_LAUNCHER.format(script=script),
        "timeout": int(event.get("timeout", 10)),
    }
    if event.get("statusMessage"):
        handler["statusMessage"] = event["statusMessage"]
    group = {"hooks": [handler]}
    if event.get("matcher"):
        group = {"matcher": event["matcher"], "hooks": [handler]}
    return group


def build_hooks_file(events: list, script: str, description: str = "") -> dict:
    """One hook file: an optional `description`, then `hooks` keyed by event name."""
    out = {}
    if description:
        out["description"] = description
    out["hooks"] = {}
    for event in events:
        event = event if isinstance(event, dict) else {"event": event}
        out["hooks"].setdefault(event["event"], []).append(_hook_entry(event, script))
    return out


def build_portable_hooks(spec: dict) -> dict:
    hooks = spec["hooks"]
    return build_hooks_file(hooks.get("portable", []), hooks["script"], hooks.get("portable_description", ""))


def build_event_hooks(spec: dict, event) -> dict:
    """The per-event file: `description` comes from the event entry itself."""
    event = event if isinstance(event, dict) else {"event": event}
    handler = {k: v for k, v in event.items() if k != "description"}
    return build_hooks_file([handler], spec["hooks"]["script"], event.get("description", ""))


# --- writing ---------------------------------------------------------------------------------


def _dump(data: dict) -> str:
    return json.dumps(data, indent=2, ensure_ascii=False) + "\n"


def _key_diff(path: Path, text: str) -> str:
    """Which top-level keys the spec would change, so the agent can decide on --force."""
    if path.suffix != ".json":
        return ""
    try:
        current, fresh = json.loads(path.read_text()), json.loads(text)
    except json.JSONDecodeError:
        return ""
    keys = sorted(k for k in set(current) | set(fresh) if current.get(k) != fresh.get(k))
    return f"  [keys: {', '.join(keys)}]" if keys else ""


def _same(path: Path, text: str) -> bool:
    """Equal content; for JSON, equal parsed content (formatting is the repo's business)."""
    current = path.read_text()
    if current == text:
        return True
    if path.suffix == ".json":
        try:
            return json.loads(current) == json.loads(text)
        except json.JSONDecodeError:
            return False
    return False


class Writer:
    def __init__(self, root: Path, *, force: bool, dry_run: bool):
        self.root, self.force, self.dry_run = root, force, dry_run
        self.written: list[str] = []
        self.skipped: list[str] = []
        self.merged: list[str] = []
        # Divergences `--force` can never resolve, kept apart from `skipped` so the CLI does
        # not tell an operator to run --force on a file the generator refuses to overwrite.
        self.protected: list[str] = []

    def put(self, rel: str, text: str) -> None:
        path = self.root / rel
        if path.exists() and not self.force:
            if _same(path, text):
                return
            self.skipped.append(rel + _key_diff(path, text))
            return
        self.written.append(rel)
        if not self.dry_run:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)

    def put_protected(self, rel: str, text: str) -> None:
        """Like `put`, but never overwritten once it exists — not even under `--force`.

        For the one output the generator only *seeds*: a maintainer edits it by hand
        afterwards (currently just `dev/README.md`). `--force` exists to overwrite generated
        manifests the spec is the source of truth for; this file stops being one of those the
        moment it is first written, and regenerating it destroyed real hand-written prose in
        production (the dev plugin's eval-case location, a skill-creator attribution, CONTRIBUTING
        framing), caught only on diff review. Reported the same way `put`'s divergences are —
        `kept`, never silently — so an operator sees why it was left alone.
        """
        path = self.root / rel
        if path.exists():
            self.protected.append(rel + _key_diff(path, text))
            return
        self.written.append(rel)
        if not self.dry_run:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)

    def merge_marketplace(self, rel: str, fresh: dict) -> None:
        """Add the plugin entries a marketplace lacks; never rewrite the ones it has."""
        path = self.root / rel
        if not path.exists() or self.force:
            self.put(rel, _dump(fresh))
            return
        current = json.loads(path.read_text())
        have = {p.get("name") for p in current.get("plugins", [])}
        missing = [p for p in fresh["plugins"] if p["name"] not in have]
        if not missing:
            return
        current.setdefault("plugins", []).extend(missing)
        self.merged.append(rel)
        if not self.dry_run:
            path.write_text(_dump(current))


def _prefixed(dir_: str, rel: str) -> str:
    """`rel` inside a plugin's own directory: unprefixed for a root plugin (`dir_ == ""`)."""
    return f"{dir_}/{rel}" if dir_ else rel


def spawn(spec: dict, root: Path, *, force: bool, dry_run: bool) -> Writer:
    """Write every declared plugin's manifest set, one entry at a time.

    `plugin_entries(spec)` normalises the spec to a list of one (single-plugin spec)
    or more (multi-plugin spec) per-plugin dicts; each is otherwise built exactly as
    a single-plugin spec always was, with every written path prefixed by the entry's
    `dir` (empty for a root plugin, so single-plugin output is byte-for-byte
    unchanged). Each entry resolves its own version independently. Marketplace
    entries accumulate across every plugin and are written with one
    `merge_marketplace` call per ecosystem, never one per plugin.
    """
    w = Writer(root, force=force, dry_run=dry_run)

    if (root / ".mcp.json").exists():
        print(
            "warning: a root .mcp.json exists; Claude Code reads it as project-scope config that never "
            "expands ${CLAUDE_PLUGIN_ROOT} (references/traps.md#root-mcp-json). Move it or delete it.",
            file=sys.stderr,
        )

    claude_entries: list[tuple[dict, str]] = []
    codex_entries: list[tuple[dict, str]] = []

    for entry in plugin_entries(spec):
        version = resolve_version(entry, root)
        eco = set(entry.get("ecosystems", ["claude", "codex", "agent-plugins"]))
        dir_ = entry.get("dir", "")

        # Codex parses this same root Agent Plugins manifest natively (extras under
        # extensions["com.openai"]) and auto-wires mcp_servers -> ./mcp.json, so
        # "codex" shares agent-plugins's files rather than owning a separate plugin folder.
        if "agent-plugins" in eco or "codex" in eco:
            w.put(_prefixed(dir_, "plugin.json"), _dump(build_agent_plugin(entry, version)))
            if entry.get("mcp"):
                w.put(_prefixed(dir_, "mcp.json"), _dump(build_mcp(entry, version, "agent-plugins")))
        if "claude" in eco:
            w.put(_prefixed(dir_, ".claude-plugin/plugin.json"), _dump(build_claude_plugin(entry, version)))
            claude_entries.append((entry, version))
            if entry.get("mcp"):
                w.put(_prefixed(dir_, ".claude-plugin/mcp.json"), _dump(build_mcp(entry, version, "claude")))
        if "codex" in eco:
            codex_entries.append((entry, version))

        if entry.get("hooks"):
            hooks = entry["hooks"]
            if hooks.get("portable"):
                w.put(_prefixed(dir_, "hooks/hooks.json"), _dump(build_portable_hooks(entry)))
            for event in hooks.get("extra", []):
                rel = extra_hook_files({"extra": [event]})[0].removeprefix("./")
                w.put(_prefixed(dir_, rel), _dump(build_event_hooks(entry, event)))
            if (root / dir_ / "hooks" / "claude-code.json").exists():
                print(
                    "warning: hooks/claude-code.json is the retired layout (one file per harness); each non-portable "
                    "event now has its own file named after it. Delete it once the per-event files are written.",
                    file=sys.stderr,
                )
            script_rel = _prefixed(dir_, hooks["script"])
            script = root / script_rel
            if not script.exists() and not dry_run:
                script.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy(HOOK_TEMPLATE, script)
                w.written.append(script_rel + "  (from assets/hooks/policy_template.py; edit it)")
            elif not script.exists():
                w.written.append(script_rel + "  (template)")

        if entry.get("dev"):
            dev = entry["dev"]
            ddir = _prefixed(dir_, dev.get("dir", "dev").strip("/"))
            market = spec.get("claude_marketplace", {}).get("name", f"{spec['name']}-marketplace")
            market_slug = _marketplace_source_slug(spec)
            w.put(f"{ddir}/.claude-plugin/plugin.json", _dump(build_dev_plugin(entry, version)))
            w.put_protected(f"{ddir}/README.md", dev_readme(entry, ddir, market, market_slug))
            if not dry_run:
                (root / ddir / "skills").mkdir(parents=True, exist_ok=True)

        if entry.get("skills") and not (root / dir_ / entry["skills"]).is_dir():
            print(f"warning: skills path {entry['skills']} does not exist yet; the loaders warn on a missing directory.", file=sys.stderr)

    # `marketplace: {"hub": "<repository>"}` means some other repo owns the marketplace naming
    # these plugins: two repos declaring one marketplace name silently replace each other in
    # Claude Code and hard-error in Codex, so this repo writes neither marketplace file at all.
    # `load_spec` already refused a hub-mode spec recording no hub, so `marketplace["hub"]` is
    # guaranteed present whenever this is truthy (see `_marketplace_source_slug`).
    if not spec.get("marketplace"):
        if claude_entries:
            w.merge_marketplace(".claude-plugin/marketplace.json", build_claude_marketplace(spec, claude_entries))
        if codex_entries:
            w.merge_marketplace(".agents/plugins/marketplace.json", build_codex_marketplace(spec, codex_entries))

    return w


def _marketplace_source_slug(spec: dict) -> str:
    """The `claude/codex plugin marketplace add <slug>` target: the hub's repository in hub
    mode (its marketplace lists this plugin), this plugin's own `repository` otherwise.

    Hub mode never falls back to `repository` — that names the product repo, not the hub, and
    is the wrong answer by construction. `load_spec` already refused a hub-mode spec that
    records no hub, so `spec["marketplace"]["hub"]` is guaranteed present whenever
    `spec.get("marketplace")` is truthy; this function never guesses.
    """
    marketplace = spec.get("marketplace")
    repo = marketplace["hub"] if marketplace else spec.get("repository", "https://github.com/<owner>/<repo>")
    return re.sub(r"^https://github\.com/", "", repo)


def dev_readme(spec: dict, ddir: str, market: str, market_slug: str) -> str:
    dev = spec["dev"]
    return f"""# {dev["name"]}

The maintainer's dev environment for this repository, packaged as a Claude Code
plugin of skills. Here the dev environment is knowledge, and this is how an
agent installs it:

```bash
claude --plugin-dir ./{ddir}                       # from a clone
claude plugin marketplace add {market_slug}        # from the marketplace that lists it
claude plugin install {dev["name"]}@{market}
```

The product plugin (`{spec["name"]}`, repository root) never ships these skills;
end users have no use for them. `AGENTS.md` lists each skill and when it fires.
Each skill follows progressive disclosure: a short `SKILL.md` whose description
says when to load it, `references/` for depth, `scripts/` for mechanical steps.
"""


# --- init -------------------------------------------------------------------------------------


def _git(root: Path, *args: str) -> str:
    try:
        return subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return ""


def detect_version_source(root: Path) -> str | None:
    for candidate in sorted(root.glob("pyproject.toml")) + sorted(root.glob("*/pyproject.toml")):
        if re.search(r"^\[project\]", candidate.read_text(), re.M):
            return f"pyproject:{candidate.relative_to(root).as_posix()}"
    if (root / "package.json").exists() and '"version"' in (root / "package.json").read_text():
        return "package.json:package.json"
    if (root / "Cargo.toml").exists():
        return "cargo:Cargo.toml"
    return None


def init_spec(root: Path) -> dict:
    remote = _git(root, "remote", "get-url", "origin")
    https = re.sub(r"^git@github\.com:", "https://github.com/", remote).removesuffix(".git")
    name = root.resolve().name.lower().replace("_", "-")
    owner = re.sub(r"^https://github\.com/([^/]+)/.*$", r"\1", https) if https else ""
    market = _kebab(owner) if owner else f"{name}-marketplace"
    spec = {
        "name": name,
        "displayName": name,
        "description": "TODO: one sentence, reused by every manifest",
        "author": {"name": _git(root, "config", "user.name"), "email": _git(root, "config", "user.email")},
        "homepage": https,
        "repository": https,
        "license": detect_license(root),
        "keywords": ["agents"],
        "ecosystems": ["claude", "codex", "agent-plugins"],
    }
    source = detect_version_source(root)
    if source:
        spec["version_from"] = source
    else:
        spec["version"] = "0.1.0"
    if (root / "skills").is_dir():
        spec["skills"] = "./skills/"
    spec["claude_marketplace"] = {"name": market, "description": f"{owner or name} marketplace."}
    spec["codex"] = {
        "marketplace_name": market,
        "marketplace_displayName": owner or name,
        "category": "Developer Tools",
        "capabilities": ["Read"],
        "shortDescription": "TODO: subtitle",
        "longDescription": "TODO: details-page paragraph",
        "defaultPrompt": ["TODO: a starter prompt under 50 characters"],
    }
    spec["_optional_sections"] = {
        "mcp": {"server": "<server-name>", "command": "uvx", "args": ["<pypi-package>=={version}"], "claude_env": {"<ROOT_VAR>": "${CLAUDE_PROJECT_DIR}"}},
        "hooks": {
            "script": f"hooks/{name.replace('-', '_')}_policy.py",
            "codex": False,
            "portable": [
                {"event": "SessionStart", "statusMessage": "Loading the policy"},
                {"event": "SubagentStart", "statusMessage": "Loading the policy"},
                {"event": "PreToolUse", "matcher": "Grep|Bash"},
            ],
            "extra": [{"event": "WorktreeRemove", "timeout": 90, "description": "TODO: what the one non-portable event does; this file is named after it and named only from the Claude Code manifest"}],
        },
        "dev": {"dir": "dev", "name": f"{name}-dev", "displayName": f"{name} maintainer skills", "description": "TODO", "keywords": [name, "maintainer", "skills", "agents"]},
    }
    return spec


def _kebab(text: str) -> str:
    """CrackingShells -> cracking-shells: one marketplace per organization, named after it."""
    text = re.sub(r"([a-z0-9])([A-Z])", r"\1-\2", text)
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")


def detect_license(root: Path) -> str:
    path = root / "LICENSE"
    if not path.exists():
        return ""
    head = path.read_text(errors="replace")[:600]
    for needle, spdx in (
        ("GNU AFFERO", "AGPL-3.0-or-later"),
        ("GNU GENERAL PUBLIC LICENSE", "GPL-3.0-or-later"),
        ("GNU LESSER", "LGPL-3.0-or-later"),
        ("Apache License", "Apache-2.0"),
        ("MIT License", "MIT"),
        ("BSD 3-Clause", "BSD-3-Clause"),
        ("Mozilla Public License", "MPL-2.0"),
    ):
        if needle.lower() in head.lower():
            return spdx
    return "TODO"


# --- install snippet -------------------------------------------------------------------------


def install_snippet(spec: dict) -> str:
    repo = spec.get("repository", "https://github.com/<owner>/<repo>")
    repo_slug = re.sub(r"^https://github\.com/", "", repo)
    # The `marketplace add` target: the hub's repository in hub mode (this plugin's own
    # marketplace files were never written), `repo_slug` otherwise. The Agent Plugins 1.0
    # section below always uses `repo_slug` regardless of hub mode — that ecosystem has no
    # marketplace concept, so it always installs from this plugin's own repository.
    market_slug = _marketplace_source_slug(spec)
    claude_market = spec.get("claude_marketplace", {}).get("name", f"{spec['name']}-marketplace")
    codex_market = spec.get("codex", {}).get("marketplace_name", f"{spec['name']}-marketplace")
    dev = spec.get("dev")
    parts = [
        "## Install\n",
        "### Claude Code\n",
        f"```bash\nclaude plugin marketplace add {market_slug}\n```\n",
        f"```bash\nclaude plugin install {spec['name']}@{claude_market}\n```\n",
        "Add `--scope project` to the marketplace command to declare it in the repository's own "
        "`.claude/settings.json` instead of your user settings.\n",
        "### Codex\n",
        f"```bash\ncodex plugin marketplace add {market_slug}\n```\n",
        f"```bash\ncodex plugin add {spec['name']}@{codex_market}\n```\n",
        "The Codex manifests are `.agents/plugins/marketplace.json` and the root `plugin.json` "
        "(Codex extras live under its `extensions[\"com.openai\"]`).\n",
        "### Agent Plugins 1.0 clients (Cursor, GitHub Copilot, VS Code, Kiro)\n",
        "The [Agent Plugins 1.0 spec](https://agent-plugins.org/specification) defines the package "
        "(`plugin.json`, `mcp.json`) and leaves installation to each client, so the install command is the "
        f"client's own. In VS Code: Command Palette, **Chat: Install Plugin from Source**, git repository, `{repo_slug}`.\n",
    ]
    if spec.get("mcp"):
        cmd = " ".join([spec["mcp"]["command"], *spec["mcp"].get("args", [])]).replace("{version}", "<version>")
        parts += [
            "### Any MCP client\n",
            f"The server is a stdio MCP server; register `{cmd}` in the client's MCP config. "
            "The plugin manifests pin the version so that upgrading the plugin is what upgrades the server.\n",
        ]
    if dev:
        ddir = dev.get("dir", "dev").strip("/")
        parts += [
            "### Maintainers\n",
            f"```bash\nclaude --plugin-dir ./{ddir}\n```\n",
            f"```bash\nclaude plugin install {dev['name']}@{claude_market}\n```\n",
        ]
    return "\n".join(parts)


# --- main -----------------------------------------------------------------------------------------


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="repository root (default: cwd)")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p_init = sub.add_parser("init", help="write an example spec prefilled from the tree")
    p_init.add_argument("--out", default="plugin.spec.json")
    p_spawn = sub.add_parser("spawn", help="write the manifests from a spec")
    p_spawn.add_argument("--spec", required=True, type=Path)
    p_spawn.add_argument("--force", action="store_true", help="overwrite existing files")
    p_spawn.add_argument("--dry-run", action="store_true", help="list what would be written")
    p_snip = sub.add_parser("install-snippet", help="print the README install section")
    p_snip.add_argument("--spec", required=True, type=Path)
    args = parser.parse_args(argv)
    root = args.root.resolve()

    if args.cmd == "init":
        out = root / args.out
        if out.exists():
            _die(f"{out} exists; edit it or pick --out")
        out.write_text(_dump(init_spec(root)))
        print(f"wrote {out}\nfill the TODOs, move what you need out of _optional_sections, then: spawn --spec {args.out} --dry-run")
        return 0

    spec = load_spec(args.spec)
    spec.pop("_optional_sections", None)
    if args.cmd == "install-snippet":
        print(install_snippet(spec))
        return 0

    w = spawn(spec, root, force=args.force, dry_run=args.dry_run)
    verb = "would write" if args.dry_run else "wrote"
    for rel in w.written:
        print(f"{verb}   {rel}")
    for rel in w.merged:
        print(f"{'would merge' if args.dry_run else 'merged'}  {rel}")
    for rel in w.skipped:
        print(f"kept     {rel}   (differs from the spec; --force overwrites)")
    for rel in w.protected:
        print(f"kept     {rel}   (yours to edit; the generator only seeds it, --force included)")
    if not (w.written or w.merged or w.skipped or w.protected):
        print("nothing to do: every manifest already matches the spec")
    print("\nnext: python3 check_plugin.py --root . ; claude plugin validate . (see SKILL.md step 4)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
