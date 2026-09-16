# Release Wiring

**Goal**: Make every skill release write its version into its plugin manifests and commit the refreshed plugin tree, and give `spawning-agent-plugins` the release files it has never had.
**Pre-conditions**:
- [ ] `playbook_plugins` is done — the manifests the exec step rewrites must exist
- [ ] `skills/spawning-agent-plugins/` is committed
**Success Gates**:
- ⬜ `skills/spawning-agent-plugins/package.json` and `.releaserc.js` exist and match the other skills' shape [run]
- ⬜ `npx multi-semantic-release --dry-run` names all five packages without error [run]
- ⬜ The version-writing step stages the plugin manifests so they land in the release commit, demonstrated by exercising it directly [run]
- ⬜ Each `plugins/<name>/plugin.json` version equals `skills/<name>/package.json` version [run]
- ⬜ `CONTRIBUTING.md` describes the plugin path alongside the `.skill` path [static]
**References**: [R03 §semantic-release, package.json as source](../../../../skills/spawning-agent-plugins/references/versioning.md) — the exec-then-git-assets pattern

## Step 1: Give spawning-agent-plugins its release files

**Goal**: Put the skill into the workspace so `multi-semantic-release` can see it at all.

**Implementation Logic**:
Add `skills/spawning-agent-plugins/package.json` as `{"name":"spawning-agent-plugins","version":"1.0.0"}` and `.releaserc.js` as the one-line `module.exports = require('../../release-config')('spawning-agent-plugins');`, matching every other skill. The `skills/*` workspace glob then picks it up with no change to the root `package.json` — the only reason it was invisible to the release pipeline is that it lacked a `package.json`. `CHANGELOG.md` is not created here; `@semantic-release/changelog` writes it on the first release.

This is the leaf that answers the question that started this campaign: until these two files exist, pushing the skill publishes nothing, silently and with a green pipeline.
**Deliverables**: `skills/spawning-agent-plugins/package.json`, `skills/spawning-agent-plugins/.releaserc.js`
**Consistency Checks**: `python3 -c "import json;d=json.load(open('skills/spawning-agent-plugins/package.json'));assert d['name']=='spawning-agent-plugins'"` (expected: PASS)
**Commit**: `feat(spawning-agent-plugins): add the release marker and config`

## Step 2: Write the version into the plugin manifests on release

**Goal**: Make the version the loaders actually read move on every release.

**Implementation Logic**:
Add `tools/set_plugin_version.py` taking a plugin name and a version and writing it into `plugins/<name>/plugin.json` and `plugins/<name>/.claude-plugin/plugin.json`. Wire it into `release-config.js` as a second `@semantic-release/exec` `prepareCmd` alongside the existing packaging call, and extend the `@semantic-release/git` `assets` list so the rewritten manifests and the reassembled skill tree are committed with the release.

Mind the working directory: semantic-release runs a package's lifecycle from that package's root, `skills/<name>/`, which is why the existing `prepareCmd` reads `../../tools/package_skill.py`.

**Extending `git.assets` is necessary but NOT sufficient, and on its own achieves nothing.** `@semantic-release/git` builds its commit list from `getModifiedFiles()`, which runs `git ls-files -m -o` with `cwd` at the package root and no pathspec — and git scopes an argument-less `ls-files` to the invoking directory's own subtree. So a modified file under `plugins/` is invisible to that call, and `assets` only *filters* what it returned; a glob cannot add a path the call never reported. Measured, not inferred.

The manifest write must therefore stage itself: have the version-writing tool run `git add` on the manifests from the repo root, as an exec step ordered before the `git` plugin's own, so the staged changes ride into its `git commit`. Verify the installed `@semantic-release/git` commits the whole index rather than a pathspec before relying on this.

This is the failure mode the leaf exists to prevent: a green release whose manifest still carries the previous version, after which Codex never reinstalls because its gate compares version strings.
**Deliverables**: `tools/set_plugin_version.py` — functions `set_version(plugin_name, version, repo_root)` and `main()`; `release-config.js` — second `exec` prepareCmd and extended `git` assets
**Consistency Checks**: `node -e "const c=require('./release-config')('writing-history');const git=c.plugins.find(p=>Array.isArray(p)&&p[0]==='@semantic-release/git');if(!git[1].assets.some(a=>String(a).includes('plugins/'))) process.exit(1)"` (expected: PASS)
**Commit**: `feat(release): write the plugin version and commit the assembled tree on release`

## Step 3: Document both distribution paths

**Goal**: Stop `CONTRIBUTING.md` and `README.md` describing a world with only `.skill` zips.

**Implementation Logic**:
Update `CONTRIBUTING.md`'s release section to describe the plugin path beside the existing `.skill` path: what `plugins/` is, that it is tracked rather than build output, that the pre-commit hook refreshes it and CI fails on drift, and that nobody hand-edits it. Update `README.md`'s installation section to add the marketplace route — `claude plugin marketplace add CrackingShells/Nest` then `claude plugin install <plugin>@cracking-shells` — while keeping the `.skill` download options, which continue to work unchanged. State plainly which five skills are available as plugins, so a reader is not left guessing why the others are absent.
**Deliverables**: `CONTRIBUTING.md` — release section covering the plugin path; `README.md` — installation section with the marketplace route and the five plugin names
**Consistency Checks**: `grep -q "marketplace add CrackingShells/Nest" README.md && grep -q "plugins/" CONTRIBUTING.md` (expected: PASS)
**Commit**: `docs(release): document the plugin distribution path alongside .skill`
