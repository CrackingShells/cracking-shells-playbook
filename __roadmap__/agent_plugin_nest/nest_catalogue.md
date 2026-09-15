# Nest Catalogue

**Goal**: Author the `cracking-shells` marketplace in `CrackingShells/Nest` listing all seven org plugins by git source, with no `version` and no `sha` on any entry.
**Pre-conditions**:
- [ ] `/Users/hacker/Documents/src/CrackingShells/Nest` is a clone of `CrackingShells/Nest`
- [ ] The five playbook plugin names are fixed: `managing-roadmaps`, `writing-history`, `writing-release`, `writing-reports`, `spawning-agent-plugins`
**Success Gates**:
- ⬜ Both marketplace files parse as JSON and declare `"name": "cracking-shells"` [run]
- ⬜ The Claude file lists seven plugins; the Codex file lists the six it can express [run]
- ⬜ No plugin entry anywhere carries a `version`, `sha` or `ref` key [run]
- ⬜ Every playbook entry uses a `git-subdir` source whose `path` is `plugins/<name>` [run]
- ⬜ Every entry inlines `displayName`, `description` and `category` for Codex's pre-install browse view [run]
**References**: [R01 §Decisions already settled](~/.claude/plans/good-news-overall-it-s-gleaming-wreath.md) — entry shape and why no version/sha

## Step 1: Author the Claude Code marketplace

**Goal**: Give Nest the single `cracking-shells` catalogue Claude Code resolves by name.

**Implementation Logic**:
Write `.claude-plugin/marketplace.json` with `name: "cracking-shells"`, an `owner` block naming Eliott Jacopin, a `description` saying this is the CrackingShells org catalogue, and a `plugins` array of seven entries. The five playbook plugins use a `git-subdir` source — `{"source":"git-subdir","url":"https://github.com/CrackingShells/cracking-shells-playbook.git","path":"plugins/<name>"}` — because each plugin is a subdirectory of the playbook monorepo and Claude Code sparse-clones that path. colgrep-mcp uses `{"source":"github","repo":"CrackingShells/colgrep-mcp"}` since its plugin is the repo root, and `colgrep-mcp-dev` — the maintainer plugin colgrep-mcp's own marketplace lists at `./dev` — becomes a `git-subdir` into that same repo at path `dev`. Omitting it would silently strip maintainers of a plugin they have today; relinquishing a marketplace must not quietly drop its contents. Omit `version`, `ref` and `sha` from every entry: with none set, Claude Code resolves the plugin's own `plugin.json` version first and falls back to the source commit SHA, so users update whenever the source moves and Nest never needs a release-time edit. Inline `displayName`, `description` and `category` on each entry — harmless for Claude Code, and the only metadata Codex can render before it clones.
**Deliverables**: `/Users/hacker/Documents/src/CrackingShells/Nest/.claude-plugin/marketplace.json` — keys `name`, `owner`, `description`, `plugins[]`; seven entries named `managing-roadmaps`, `writing-history`, `writing-release`, `writing-reports`, `spawning-agent-plugins`, `colgrep-mcp`, `colgrep-mcp-dev`
**Consistency Checks**: `python3 -c "import json;d=json.load(open('/Users/hacker/Documents/src/CrackingShells/Nest/.claude-plugin/marketplace.json'));assert d['name']=='cracking-shells';assert len(d['plugins'])==7;assert not any(k in p for p in d['plugins'] for k in ('version','sha','ref'))"` (expected: PASS)
**Commit**: `feat(marketplace): add the cracking-shells Claude Code catalogue`

## Step 2: Author the Codex marketplace

**Goal**: Mirror the catalogue for Codex, whose source variants and required entry fields differ.

**Implementation Logic**:
Write `.agents/plugins/marketplace.json` with `name: "cracking-shells"`, an `interface.displayName` of `"CrackingShells"`, and the same plugins. Codex has no `github` shorthand: playbook entries use `{"source":"git-subdir","url":"https://github.com/CrackingShells/cracking-shells-playbook.git","path":"plugins/<name>"}` and colgrep-mcp uses `{"source":"url","url":"https://github.com/CrackingShells/colgrep-mcp.git"}`. Every entry needs a `policy` block (`installation: "AVAILABLE"`, `authentication: "NONE"`) and a `category`; Codex's loader defaults `policy` but the reference treats both as expected. Inline the same display metadata — Codex does not clone at listing time, so an entry without it renders blank until install. Remember `category` here permanently overrides the plugin manifest's own, so it must be the value you actually want shown.
**Deliverables**: `/Users/hacker/Documents/src/CrackingShells/Nest/.agents/plugins/marketplace.json` — keys `name`, `interface.displayName`, `plugins[]` each with `name`, `source`, `policy`, `category`
**Consistency Checks**: `python3 -c "import json;d=json.load(open('/Users/hacker/Documents/src/CrackingShells/Nest/.agents/plugins/marketplace.json'));assert d['name']=='cracking-shells';assert len(d['plugins'])>=6;assert all('policy' in p and 'category' in p for p in d['plugins'])"` (expected: PASS)
**Commit**: `feat(marketplace): add the cracking-shells Codex catalogue`

## Step 3: Write the Nest README with the migration note

**Goal**: Tell a reader how to add the catalogue, and tell existing colgrep-mcp users why they must remove the old one first.

**Implementation Logic**:
Write `README.md` explaining that Nest is the CrackingShells plugin catalogue and owns the `cracking-shells` marketplace name. Give the two install snippets — `claude plugin marketplace add CrackingShells/Nest` then `claude plugin install <plugin>@cracking-shells`, and the Codex equivalents. Add a prominent migration section: a client keeps whichever marketplace it registered under a given name at add time, so anyone who previously added `cracking-shells` from `CrackingShells/colgrep-mcp` must run `claude plugin marketplace remove cracking-shells` before adding Nest, or they will keep the old, smaller catalogue with no error shown. List every plugin with one line each, marking `colgrep-mcp-dev` as maintainer-only.
**Deliverables**: `/Users/hacker/Documents/src/CrackingShells/Nest/README.md` — sections Install, Migrating from the colgrep-mcp marketplace, Plugins
**Consistency Checks**: `grep -q "marketplace remove cracking-shells" /Users/hacker/Documents/src/CrackingShells/Nest/README.md` (expected: PASS)
**Commit**: `docs(readme): describe the catalogue, install paths and the marketplace migration`
