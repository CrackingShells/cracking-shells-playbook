---
name: writing-release
description: >
  Write GitHub pull request bodies and Discord release announcement messages
  for software projects that follow a fork-based contribution workflow and
  publish releases with Discord announcements. Use this skill whenever the
  user wants to draft, write, or generate: (1) a PR message for a dev-branch
  contribution from a fork into the upstream repo's dev branch, (2) a PR
  message merging the upstream dev branch into its main branch for a versioned
  release, or (3) a Discord announcement for a dev pre-release or a stable
  release. Also triggers for phrases like "write the PR", "draft the release
  notes", "Discord announcement for the new version", "PR description for the
  merge", "write my PR body", or any request involving release communication
  for a project with this branching model.
---

# Writing Release

This skill produces four document types used in projects that follow a
fork-based, semver-versioned workflow with Discord release announcements.

## Quick reference — which document do you need?

| Situation | Document type | Reference file |
|---|---|---|
| Merging **your fork's `dev`** into **upstream's `dev`** | **PR Type A** — feature/fix contribution | `references/pr-fork-to-dev.md` |
| Merging **upstream `dev`** into **upstream `main`** | **PR Type B** — versioned release | `references/pr-dev-to-main.md` |
| Announcing a **dev pre-release** on Discord | **Discord dev** announcement | `references/discord-dev-release.md` |
| Announcing a **stable release** on Discord | **Discord main** announcement | `references/discord-main-release.md` |

Read the appropriate reference file before writing. They contain annotated
real examples and fill-in templates.

---

## Workflow

1. **Identify the document type** from the table above. Ask the user if
   it's unclear — the key question is always: what is the base branch?
   - Base = upstream `dev` → PR Type A
   - Base = upstream `main` → PR Type B
   - Discord message for a `-devN` version → Discord dev
   - Discord message for a stable version → Discord main

2. **Gather what you need.** Before writing, confirm you have:
   - The **version string** (e.g. `v1.3.0`, `v1.3.0-dev.2`) — preserve the
     exact format the user provides
   - The **list of changes** — commits, feature names, bug descriptions.
     The user may paste a git log, bullet points, or describe them verbally.
   - For PR Type B and stable Discord: the constituent **dev PR numbers /
     dev release links** so you can cross-reference them
   - For Discord: the **pip/npm install command** if it changed, and the
     **package name** on the registry

3. **Read the reference file** for the document type, then write the output.

4. **Always output both a title and a body together** as a single ready-to-
   copy block. The title goes first, clearly labelled:
   ```
   **Title:** [vX.Y.Z-devN] Short description of changes

   **Body:**
   ## Summary
   …
   ```
   This saves the user a copy-paste step.

5. **Review before presenting:**
   - Does the title match the version string exactly?
   - Are all mentioned features actually in the change list provided?
   - Is the tone right? (technical for PRs, friendly+accessible for Discord)
   - No invented changes — if something seems missing, ask rather than guess.

---

## Shared conventions across all documents

- **Version strings:** Preserve the exact format the user provides.
  Common patterns: `v1.0.0`, `v1.0.0-dev.1`, `v1.0.0.dev2`.

- **Emoji in Discord messages:** The project uses custom Discord emoji.
  The reference file has examples from the Hatch project — substitute your
  own project's custom emoji codes where relevant, or use standard Unicode
  emoji as a fallback.

- **Code blocks in PRs:** Use triple-backtick fenced blocks with a language
  tag when showing CLI examples (`bash`) or config snippets.

- **Headers:** PRs use `##` for top-level sections, `###` for subsections.
  Discord messages use `##` or `###` — see reference files for specifics.

- **Cross-references in Type B PRs:** When a section summarises a dev PR,
  append its number in parentheses, e.g. `**Feature name** (#42)`.

- **Never invent changes.** Only describe what the user has told you.
  If something seems missing, ask rather than guess.
