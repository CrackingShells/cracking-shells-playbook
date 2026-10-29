# Setting Up a Release Pipeline

How a repository adopts the release lifecycle: which capabilities the pipeline needs, how to pick tooling from what the repository contains, which repository settings enforce the lifecycle, and what to write in CONTRIBUTING. Read it when a repository has no automated release, or when its release tooling is being chosen or replaced. The lifecycle itself lives in [lifecycle.md](lifecycle.md); the CI job that runs the chosen tool lives in [ci-release-job.md](ci-release-job.md).

## Contents

- [Capabilities a Release Pipeline Needs](#capabilities-a-release-pipeline-needs)
- [Choosing Tooling from Contents](#choosing-tooling-from-contents)
  - [Decision procedure](#decision-procedure)
  - [By repository contents](#by-repository-contents)
  - [What each tool covers](#what-each-tool-covers)
  - [Units and tags](#units-and-tags)
  - [Filling the CI skeleton](#filling-the-ci-skeleton)
- [Repository Settings](#repository-settings)
- [The CONTRIBUTING Section](#the-contributing-section)
- [Adoption Checklist](#adoption-checklist)

## Capabilities a Release Pipeline Needs

No single tool covers every row, and the gaps are filled by CI steps. List the rows the repository needs first, then pick tools by coverage.

| Capability | What it is | Optional when |
|:--|:--|:--|
| Next version | Computed from the commits since the last tag, using the repository's own bump mapping | Never |
| Changelog | Release notes for the new version, in a `CHANGELOG` file, in the GitHub Release, or both | Nobody outside the maintainers reads version history |
| Bump commit | A CI commit that records version files and the changelog on `main` (`chore(release): <unit>@<version> [skip ci]`) | The version lives only in the tag, so no file changes |
| Tag | One annotated or lightweight tag per released version | Never; the tag is the source of truth for "last release" |
| GitHub Release | A release page attached to the tag, with the notes | Nothing downstream links to release pages |
| Registry publish | Upload to a package registry | Nothing is installed from a registry (decide per repository) |
| One tag line per unit | Each independently versioned unit has its own tag series and bump | The repository has a single version |

Commit vocabulary and the commit-to-bump mapping are not decided here: `writing-history` → `references/semver-changelog.md` owns them, and `references/convention-setup.md` runs the interview when a repository has no convention at all. This file only chooses the machinery that executes the mapping.

## Choosing Tooling from Contents

### Decision procedure

1. **Inspect existing machinery first.** Release config files, workflow files, existing tags and a hand-kept changelog are the repository's ground truth. Extend working machinery; replacing it is a separate decision that needs the maintainer's approval.
2. **Identify the units.** A unit is something versioned and released on its own: a package, a crate, a skill, a schema set. Several manifests or a workspaces layout suggest several units; confirm with the maintainer. List each unit's released paths (the paths whose changes release it).
3. **Identify the main language** from the manifests and the sources.
4. **Prefer tooling native to that language**, so CI needs no second toolchain. This is a preference, not a rule: sustainability and leanness win. A tool that needs a second runtime is fine when the alternative is hand-rolled scripting, or when the tool is better maintained.
5. **Reject tools that cannot meet the defaults.** The default pipeline has no release PRs and no local release commands: CI releases after a merge. A tool whose only mode is a release PR, or whose commands assume a developer machine, fits only if the maintainer accepts that departure.
6. **Check what the tool leaves out** against [the capability table](#capabilities-a-release-pipeline-needs) and plan the CI steps that fill the gaps.
7. **Propose, then configure.** Present the repository contents found, the tool chosen and why, and the CI steps it needs. Write config only after the maintainer approves.

### By repository contents

| Repository contents | Candidates, best fit first | Notes |
|:--|:--|:--|
| One JS/TS package, or a workspace with a single version | `semantic-release` | Needs only Node, which the repository already has. |
| JS/TS workspace with independently versioned packages | `multi-semantic-release` (wraps `semantic-release` per package) | Community wrapper, not part of `semantic-release`; maintained with limited resources; check its recent activity before adopting, and weigh sustainability. |
| One Python package | `python-semantic-release`; `commitizen` | `python-semantic-release` also creates the GitHub Release; `commitizen` leaves push and GitHub Release to CI steps. |
| Python project managed with `uv` | `commitizen` (`uv` version provider); `python-semantic-release` (`uv` integration) | `commitizen`'s `uv` provider updates the lock file with the version; `python-semantic-release` does it in its build command. |
| Several Python packages in one repository | `python-semantic-release` (monorepo commit parser, one config per package); `commitizen` (one config per component) | Both need per-package config and a run per package. |
| Rust crate | `release-plz`; `cocogitto` as bumper | `release-plz` is native to Cargo. |
| Cargo workspace, crates that must share a version | `release-plz` with `version_group` | Crates in a group get the same version; other crates version separately. |
| Skill or plugin monorepo, units are directories | `multi-semantic-release` with a `package.json` per unit (even when the unit holds no JS); `cocogitto` with `[packages]` paths | The root `package.json` lists the units in `workspaces`, and each unit's `package.json` needs at least a name and a version. |
| Content-only, schema or catalogue repository | `cocogitto` (no manifest needed); `semantic-release` with the npm plugin disabled; or tag-only via a small CI step | Tag-only is a complete answer when nothing consumes versions beyond the tag. |
| No consumer of versions at all | A documented commit convention, no release tooling | Add tooling when something starts to depend on versions. |

Commit-message checkers are the companion to any row: `commitlint`, `commitizen` (`cz check`) and `cocogitto` (`cog check`) validate messages and bump nothing by themselves. Which one to use and how to enforce it belongs to `writing-history` → `references/convention-setup.md`.

### What each tool covers

✓ built in, ◐ through an optional plugin, option or CI step, — not covered.

| Tool | Next version | Changelog file | Bump commit | Tag | GitHub Release | Registry publish |
|:--|:--|:--|:--|:--|:--|:--|
| `semantic-release` | ✓ | ◐ `@semantic-release/changelog` (notes also go to the GitHub Release) | ◐ `@semantic-release/git` | ✓ | ✓ `@semantic-release/github` (default plugin) | ✓ npm by default; ◐ other registries via `@semantic-release/exec` |
| `multi-semantic-release` | ✓ per package | as `semantic-release` | as `semantic-release` | ✓ | as `semantic-release` | as `semantic-release` |
| `python-semantic-release` | ✓ | ✓ | ✓ (`version` commits and pushes) | ✓ | ✓ (`version` creates it by default) | — upload with a separate step; `publish` attaches artifacts to the release |
| `commitizen` | ✓ (`cz bump`) | ◐ `update_changelog_on_bump` or `--changelog` | ✓ (`cz bump` commits) | ✓ | — CI step (`gh release create`) | — |
| `release-plz` | ✓ (`update`) | ✓ (`update`) | — `update` edits files and does not commit; the CI job commits | ✓ (`release`) | ✓ (`release`) | ✓ crates.io by default; off with `publish = false` |
| `release-please` | ✓ | ✓ | ✓ only by merging its release PR | ✓ | ✓ | — |
| `cocogitto` | ✓ (`cog bump --auto`) | ✓ (`cog changelog`) | ✓ (`cog bump` commits) | ✓ | — CI step | — |

Traps behind the table:

- **Release PRs.** `release-please` works through a release PR it keeps open, so do not plan on a PR-free mode. A `release-as` setting stays in force until it is removed or raised after its PR merges. `release-plz` has a `release-pr` command, but `update` plus `release` run without one: `update` edits versions and changelogs without committing, `release` tags and creates the GitHub Release and "does not modify Cargo.toml files or push new commits", so the CI job commits and pushes between them ([ci-release-job.md](ci-release-job.md#the-bump-commit)).
- **Default publishing.** `semantic-release`'s npm plugin and `release-plz` publish by default. A repository that does not publish must turn that off (`npmPublish: false` or disabling the plugin; `publish = false`, or `git_only = true` for crates that never reach the registry so versions come from tags).
- **Default bump-commit messages differ from the convention.** `python-semantic-release` defaults to `{version}` plus a generated line, and `commitizen` to `bump: version <old> → <new>`. Set `commit_message` or `bump_message` to `chore(release): <unit>@<version> [skip ci]` (`@semantic-release/git` takes a `message` option). `cocogitto` has no message key: its bump commit is `chore(version): <version>`, `[skip ci]` is added only by `cog bump --skip-ci` (`skip_ci` only changes that string), and `disable_bump_commit = true` leaves the commit to the CI job. The bump commit and its message are owned by [ci-release-job.md](ci-release-job.md#the-bump-commit).
- **Merge commits and message checks.** `cocogitto` flags merge commits as non-conventional unless `ignore_merge_commits = true`. With the PR title as the merge subject ([Repository Settings](#repository-settings)) the merge commit usually conforms; check the range once before enforcing.
- **`semantic-release` outside CI** runs as a dry run, which is one reason releases are cut only by CI.
- **`commitizen` with no bump-eligible commits** signals "nothing to bump" with a distinct exit code, which a CI job must treat as "nothing to release". Allow it with `--no-raise <code>`, placed before the subcommand, after checking the code for your version.

### Units and tags

| Tool | Several units | Tag format |
|:--|:--|:--|
| `semantic-release` | One version per run | `v<version>` by default; `tagFormat` sets it |
| `multi-semantic-release` | One version per workspace package; commits are filtered to the package folder; a package whose local dependency changed gets a dependency-driven release (patch by default) | `<name>@<version>` |
| `python-semantic-release` | One config and run per package; `conventional-monorepo` parser filters commits with `path_filters` and `scope_prefix` | `v{version}` by default; `tag_format` sets it, per package |
| `commitizen` | One config per component, run per component; `ignored_tag_formats` hides the other components' tags | `$version` by default (no `v`); `tag_format` sets it; PEP 440 version spelling by default, `version_scheme = "semver2"` for hyphenated prereleases |
| `release-plz` | Cargo workspace; `version_group` shares a version; tags and GitHub Releases can be switched off per package, for example to keep one workspace tag | Package name plus version in a workspace, version alone for a single package; `git_tag_name` sets it |
| `release-please` | Manifest mode with a `packages` map | `<component>-v<version>` by default; `include-component-in-tag` removes the component |
| `cocogitto` | `[packages]` with a path each; `cog bump --auto` tags every changed package and adds a global tag; `public_api = false` keeps a package from moving the global version | `tag_prefix` sets a prefix such as `v`; for one tag line per unit set `generate_mono_repository_global_tag = false` and `monorepo_version_separator = "@"` |

Pick one tag shape per repository and set it explicitly rather than inheriting the default: `v<version>` for a single unit, `<unit>@<version>` for several. Every tool above can produce either shape. Tags are read back to find the last release, so changing the shape later strands the old tags.

Prerelease behaviour of each tool is in [channels.md](channels.md#prereleases-per-tool); a repository on the single-channel default never needs it.

### Filling the CI skeleton

[ci-release-job.md](ci-release-job.md#job-graph) leaves the tool-dependent steps as placeholders; fill them from the rows below. Versions are placeholders (`<x.y.z>`): pin exact ones. Run each command in the tool's non-publishing mode before the first real run. Message checkers must be configured with the repository's own vocabulary (`writing-history`); a release tool without a checker pairs with one.

| Tool | `<check commits …>` over `<base>..<head>` | `<check "$TITLE">` |
|:--|:--|:--|
| `semantic-release` | none: use `commitlint` | none: use `commitlint` |
| `multi-semantic-release` | none: use `commitlint` | none: use `commitlint` |
| `python-semantic-release` | none: use `commitlint` or `cz check` | none: use `commitlint` or `cz check` |
| `commitizen` | `cz check --rev-range <base>..<head>` | `cz check --message "$TITLE"` |
| `release-plz` | none: use `cog check` or `commitlint` | none: use `cog verify` or `commitlint` |
| `release-please` | none: use `commitlint` or `cz check` | none: use `commitlint` or `cz check` |
| `cocogitto` | `cog check --from-latest-tag` (a base..head form is not documented; use another checker for an exact PR range) | `cog verify "$TITLE"` |
| companion: `commitlint` | `npx commitlint --from <base> --to <head>` | `printf '%s' "$TITLE" \| npx commitlint` |

| Tool | `<release command>` | `<TOKEN_VARIABLE_THE_TOOL_READS>` | `<install …>`, pinned |
|:--|:--|:--|:--|
| `semantic-release` | `npx semantic-release`; the bump commit, tag, GitHub Release and publish follow the configured plugins | `GITHUB_TOKEN` (`GH_TOKEN` also works); `NPM_TOKEN` to publish to npm | Exact versions in `package.json` plus the lock file, installed with `npm clean-install` |
| `multi-semantic-release` | `npx multi-semantic-release` | as `semantic-release` | as `semantic-release` |
| `python-semantic-release` | `semantic-release version` (commits, tags, pushes, creates the GitHub Release); `semantic-release publish` only attaches artifacts to it; a PyPI upload is a separate step | `GH_TOKEN` (the default `remote.token`) | `python3 -m pip install python-semantic-release==<x.y.z>`, or its GitHub Action pinned by SHA (`github_token` input) |
| `commitizen` | `cz --no-raise <nothing-to-bump code> bump --yes`, then `git push --follow-tags`, then `gh release create <tag> --notes-file <file>` with notes from `cz changelog --dry-run <version>` | `GH_TOKEN` for `gh`; the push uses the checkout credentials | `python3 -m pip install commitizen==<x.y.z>` |
| `release-plz` | `release-plz update`; the job commits and pushes; then `release-plz release` (tag, GitHub Release, registry publish unless disabled) | `--git-token` (the action reads `GITHUB_TOKEN`); `CARGO_REGISTRY_TOKEN` to publish | `cargo install --locked release-plz --version <x.y.z>`, or `release-plz/action` pinned by SHA (its documented commands are `release` and `release-pr`) |
| `release-please` | `release-please release-pr --token … --repo-url <owner>/<repo>`; after its PR merges, `release-please github-release` with the same options | `GITHUB_TOKEN`, passed as `--token` | `npm install -g release-please@<x.y.z>` |
| `cocogitto` | `cog bump --auto`, then `git push --follow-tags`, then `gh release create <tag> --notes-file <file>` with notes from `cog changelog --at <version>` | `GH_TOKEN` for `gh`; the push uses the checkout credentials | `cocogitto/cocogitto-action` pinned by SHA (installs `cog`; its `release: true` input runs `cog bump --auto`), or `cargo install cocogitto --version <x.y.z>` |

Whichever tool pushes, the bump commit carries `[skip ci]` and the same token acts for checkout, push and Release ([ci-release-job.md](ci-release-job.md#choosing-the-token)).

## Repository Settings

These settings make the merge-commit-only invariant ([lifecycle.md](lifecycle.md#invariants)) a property of the repository rather than a habit. Changing repository settings is a change that needs the maintainer's approval ([lifecycle.md](lifecycle.md#choosing-a-lane)), and the call needs admin rights on the repository.

| Setting | Value | `PATCH repos/<owner>/<repo>` field |
|:--|:--|:--|
| Allow merge commits | on | `allow_merge_commit=true` |
| Allow squash merging | off | `allow_squash_merge=false` |
| Allow rebase merging | off | `allow_rebase_merge=false` |
| Merge commit title | the PR title | `merge_commit_title=PR_TITLE` |
| Merge commit message | the PR body | `merge_commit_message=PR_BODY` |
| Delete head branch on merge | on | `delete_branch_on_merge=true` |

```bash
gh api -X PATCH repos/<owner>/<repo> \
  -F allow_merge_commit=true -F allow_squash_merge=false -F allow_rebase_merge=false \
  -f merge_commit_title=PR_TITLE -f merge_commit_message=PR_BODY \
  -F delete_branch_on_merge=true
```

`-F` types booleans, `-f` sends strings. Set the title and message fields together: the API requires `merge_commit_title` whenever `merge_commit_message` is set. The other accepted values are `MERGE_MESSAGE` for the title, and `PR_TITLE` or `BLANK` for the message. Read the result back before relying on it:

```bash
gh api repos/<owner>/<repo> --jq '{allow_merge_commit, allow_squash_merge, allow_rebase_merge, merge_commit_title, merge_commit_message, delete_branch_on_merge}'
```

Why these values: with squash and rebase merging off, every merge is a merge commit whose subject starts with the PR title (GitHub may append ` (#<number>)`; `gh pr merge --subject` overrides it) and whose body is the PR body, so the text written in [pull-requests.md](pull-requests.md) is what the release tool and the changelog read. Deleting the branch on merge keeps topic branches from piling up.

Branch protection, required checks, the bump push and token choice interact with the direct lane and are decided in [ci-release-job.md](ci-release-job.md#permissions-tokens-and-branch-protection). Do not configure protection from this file.

## The CONTRIBUTING Section

Write these four sections in CONTRIBUTING so a contributor, or an agent in a fresh session, can derive the lifecycle without reading this manual. `writing-history` → `references/commit-authoring.md` derives vocabulary from CONTRIBUTING when no tooling config exists, so these sections are also what future authors will read.

| Section | Purpose |
|:--|:--|
| Commit messages | Names the convention, the type and scope vocabulary and how it is enforced; the content is owned by `writing-history` |
| Integration lanes | The repository's own table of released paths, release-type commits and the lane each change takes; derive it as described in [lifecycle.md](lifecycle.md#mapping-lanes-in-a-repository) |
| Versioning | The release tool, the tag format, what is published where, and that CI cuts every release |
| Branches and integration | Topic-branch naming, merge commits only, PR title and body rules, pulling `main` after a merge |

A skeleton to fill in (heading levels follow the file's own outline):

```markdown
### Commit messages
<convention and type list>. Scopes: <scope list>. Enforced by <tool> in <CI | local hook>.

### Integration lanes
<table: released paths, release-type commits, and the lane each change class takes>

### Versioning
Releases are cut by CI after a merge to `main`; never run release commands locally.
Tool: <tool>. Tag format: <v<version> | <unit>@<version>>. Changelog: <file | GitHub Releases>.
Published to: <registry | not published>. Channels: `main` only.

### Branches and integration
Topic branches are named `<type>/<topic>`. Merge commits only; the PR title is the merge subject
and the PR body is the merge body. Pull `main` after every merge.
```

## Adoption Checklist

In order; each step ends with something checkable. Steps 4 to 6 are file changes and ride in one pull request; step 8 holds the actions that cannot.

1. [ ] **Audit.** Record existing release config, workflows, tags, commit convention and the current repository settings (the read-back command above). No commit convention at all: run `writing-history` → `references/convention-setup.md` first. Then settle the vocabulary the release depends on: a non-release type for merge-commit subjects (for example `merge`) that the commit checker accepts, and a scope for the bump commit (`release`). PR titles become merge subjects ([pull-requests.md](pull-requests.md#title)), and a release type there would add changelog entries or bumps. If the vocabulary cannot take a new type, agree a `chore(<scope>)` fallback with the maintainer.
2. [ ] **Map units and released paths** (decision procedure, steps 2 and 3).
3. [ ] **Propose tooling** to the maintainer: contents found, tool, tag shape, CI gap-fillers, registry decision. Wait for approval.
4. [ ] **Write the tool config** with the tag shape and bump-commit message set explicitly, and preview it with the tool's non-publishing mode, which prints the next version and tags nothing: `semantic-release --dry-run`, `semantic-release version --print` (`python-semantic-release`), `cz bump --dry-run`, `release-plz release --dry-run`, `cog bump --dry-run --auto`.
5. [ ] **Add the release job** ([ci-release-job.md](ci-release-job.md)), filling its placeholders from [Filling the CI skeleton](#filling-the-ci-skeleton), including permissions and any branch-protection bypass.
6. [ ] **Write the CONTRIBUTING section.**
7. [ ] **Open one reviewed PR with steps 4 to 6** ([lifecycle.md](lifecycle.md#choosing-a-lane)): they change CI and release machinery. The repository has no CI to gate this first PR yet, so the maintainer reviews it by hand, config preview output included. Do not merge it yet.
8. [ ] **Do the maintainer-approved actions outside the PR, before it merges**, asking for each: (a) push a baseline tag when the repository has history but no tags, so the tool does not treat every commit as unreleased ([recovery.md](recovery.md#first-release-and-missing-tags)); (b) apply the [repository settings](#repository-settings) and read them back. Both change state the PR cannot carry.
9. [ ] **The maintainer merges the PR.** Expect the job's first run to need attention ([recovery.md](recovery.md#first-run-of-a-new-release-job)).
10. [ ] **Treat the first release-type merge as the real test.** Until a merge that releases something has produced the bump commit, the tag, the GitHub Release and, if configured, the registry upload, the pipeline is unproven. Watch that run to the end; if it fails, go to [recovery.md](recovery.md).
11. [ ] **Report** to the maintainer what was verified and what was not.
