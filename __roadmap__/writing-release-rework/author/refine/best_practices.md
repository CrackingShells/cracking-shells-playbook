# Best-Practices Pass

**Goal**: Bring `SKILL.md` and all six references in line with the official Agent Skills best practices, without changing any rule, threshold or factual claim.
**Pre-conditions**:
- [ ] All three `author/` leaves are merged, and the coordinator's integration review has passed
- [ ] https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices is fetched in this session (WebFetch, or the web-fetch agent); the checklist comes from that page, not from memory
**Success Gates**:
- ⬜ The leaf report contains a findings table `rule (quoted ≤15 words from the page) → file → change made / why none was needed`, covering every rule on the page that applies to a prose-only skill
- ⬜ `uv run --with pyyaml tools/quick_validate.py skills/writing-release` passes; every reference over ~100 lines opens with `## Contents`; SKILL.md ≤500 lines
- ⬜ `git diff` over the pass shows no change to the C3 lane table, the C2 owner table assignments, or any factual claim; any such change was flagged in the report instead
**References**: [Agent Skills best practices](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices) — the rule set; [R02](../../../../skills/writing-release/SKILL.md) — the dispatcher to tune

## Step 1: Dispatcher and process references
**Goal**: Apply the page's rules to `SKILL.md`, `lifecycle.md` and `pull-requests.md`.
**Implementation Logic**:
Derive the checklist from the page first. Then edit, with expected areas being:
- the description in the third person, saying what and when, with key trigger terms;
- conciseness (cut what Claude already knows);
- the degree of freedom matched to the fragility of each instruction;
- progressive disclosure, with references one level deep;
- consistent terminology (one term per concept across files: "lane", "release-type commit", "released path", "maintainer");
- workflows as copyable checklists, with feedback loops where a check gates the next step;
- no time-sensitive statements.
Keep every C1 heading.
**Deliverables**: `skills/writing-release/SKILL.md`, `skills/writing-release/references/lifecycle.md`, `skills/writing-release/references/pull-requests.md` — edited in place; findings rows for these three files
**Consistency Checks**: `uv run --with pyyaml tools/quick_validate.py skills/writing-release && test $(wc -l < skills/writing-release/SKILL.md) -le 500` (expected: PASS)
**Commit**: `refactor(writing-release): apply skill best practices to the dispatcher and process references`

## Step 2: Tooling, CI and recovery references
**Goal**: Apply the same checklist to `setting-up.md`, `channels.md`, `ci-release-job.md` and `recovery.md`.
**Implementation Logic**:
Use the same checklist and the same terminology as step 1. Add a table of contents to any file over about 100 lines. Prefer tables and checklists to narrative where an agent will scan them. Do not touch facts; flag doubtful ones.
**Deliverables**: `skills/writing-release/references/setting-up.md`, `channels.md`, `ci-release-job.md`, `recovery.md` — edited in place; findings rows for these four files
**Consistency Checks**: `python3 -c "import re,os,sys;bad=[(p,l) for p in sys.argv[1:] for l in re.findall(r'\]\(([^)#\s]+)',open(p).read()) if not l.startswith(('http:','https:','mailto:')) and not os.path.exists(os.path.join(os.path.dirname(p),l))];print('broken:',bad);sys.exit(1 if bad else 0)" skills/writing-release/SKILL.md skills/writing-release/references/*.md && uv run tools/package_skill.py skills/writing-release dist/` (expected: PASS)
**Commit**: `refactor(writing-release): apply skill best practices to the tooling, CI and recovery references`
