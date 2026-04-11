# Reports: managing_roadmaps_audit

## Documents

| File | Round | Status | Description |
|:-----|:------|:-------|:------------|
| [00-creation_time_bugs_v0.md](00-creation_time_bugs_v0.md) | 00 | Draft | Creation-time bug audit: root cause analysis for the duplicate top-level README.md observation |

## Status

Audit complete. Five findings identified across documentation and code.

Primary finding (F1): missing `dirtree-rdm init` subcommand forces a bootstrap deadlock — agents cannot use the tool to create a campaign root without a pre-existing parent README, causing spurious `__roadmap__/README.md` creation.

Secondary finding (F2): SKILL.md "The One Rule" blanket prohibition is unenforceable as written, as it forbids both the BNF-structural mutations (correct to forbid) and the prose authoring (necessary and unavoidable).

F1 and F2 drive actionable code and documentation changes. F3 is a single-line error message fix. F4–F5 are low-priority defensive/cosmetic items.
