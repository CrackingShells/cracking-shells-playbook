# Release Announcements Skill — Notice (v0)

Date: 2026-10-29

---
type: notice
title: Turn the retired announcement templates into their own skill
tag: gap
effort: M
reversibility: easy (a new skill; nothing depends on it)
evidence: verified
spotted-during: reworking writing-release into the release-cycle manual, which retired its two Discord references
date: 2026-10-29
status: deferred
---

## TL;DR

- Build a dedicated skill for announcing releases to a community channel.
- writing-release dropped its Discord templates; announcing is a separate job.
- Worth it only once a project has an audience that reads announcements.

## Context

The two references are not lost. They live at commit `461036c`, under `skills/writing-release/references/`:
- `discord-dev-release.md` covers pre-release posts;
- `discord-main-release.md` covers stable-release posts.

Both are annotated real examples with fill-in templates and custom-emoji conventions. Running the release cycle (lanes, PRs, CI releases) and telling people about a release are different jobs, so they were split.

## Scope Delta

- A new skill, for example `announcing-releases`, seeded from the two references at `461036c`.
- Its input is the GitHub Release notes the release cycle already produces, so it adds no release machinery.
- Decisions it would need: which channels (Discord, mailing list, social), the cadence (every release or stable only), and who posts (CI webhook or a human or agent).
- If CI posts, a webhook step after the release job: a secret, and a guard on the published flag.

## Accept / Decline

| Accept | Decline |
|:--|:--|
| Announcements get the same derived, repeatable treatment as releases, and the existing templates are reused rather than rewritten. Cost: one more skill to maintain, and a webhook secret per announcing repo. | The release cycle stays lean: GitHub Releases are the announcement, and no repo carries channel secrets or posting steps it doesn't need. Cost avoided: maintaining templates for audiences that do not exist yet. |

## If Accepted — Next Step

Open a dedicated thread. Extract the two references from `461036c`, generalise them past a single project, and author the skill with writing-reports + managing-roadmaps.

## If Declined — Next Step

Archive it. Revisit when a project first wants to announce a release to a community channel.
