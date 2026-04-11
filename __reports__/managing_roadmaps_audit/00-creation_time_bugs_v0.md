# Audit: managing-roadmaps Creation-Time Bugs

| Field | Value |
|:------|:------|
| Skill | `managing-roadmaps` |
| Scope | Creation-time bugs — duplicate top-level README.md root cause |
| Date | 2026-04-06 |
| Status | Draft |
| Severity summary | 2 HIGH · 1 MEDIUM · 2 LOW |

---

## 1. System Model

### Component Ownership

```
Skills layer        skills/managing-roadmaps/
                    ├── SKILL.md                  ← LLM agent entry point; "The One Rule"
                    └── references/
                        ├── dirtree-authoring.md  ← CRUD workflows (Create/Read)
                        ├── dirtree-cli.md        ← Full CLI command reference
                        ├── dirtree-execution.md  ← BFS traversal algorithm
                        └── dirtree-tier-examples.md ← Worked examples per tier

Tool layer          skills/managing-roadmaps/scripts/dirtree-rdm/
                    └── src/
                        ├── main.rs          ← CLI commands: add, status, update, move,
                        │                      insert, validate, ls, grammar
                        ├── readme.rs        ← BNF-validated README mutations (atomic write)
                        ├── templates.rs     ← readme_template / leaf_template scaffolds
                        └── validate.rs      ← BNF grammar checker (pre/post-flight)

Data layer          __roadmap__/
                    └── <campaign>/
                        ├── README.md        ← managed exclusively by dirtree-rdm
                        ├── <leaf>.md        ← created by dirtree-rdm; prose authored by agent
                        └── <subdir>/
                            └── README.md    ← managed exclusively by dirtree-rdm
```

### `cmd_add` Pre-condition Gate

Every `dirtree-rdm add` call passes three sequential guards before writing anything:

```
cmd_add(node_path)
  ├─ parent directory exists?     NO → bail "parent directory does not exist"
  ├─ parent/README.md exists?     NO → bail "parent README.md not found; parent must be
  │                                          an existing roadmap directory"
  └─ node_path already exists?   YES → bail "node already exists"
      ↓ all pass
  [dir] create dir + write child README.md from template
  [leaf] write leaf .md from template
  └─ add_node() → atomic update to parent README.md (pre- and post-validation)
```

This gate is correct for mutations. The problem is that it also applies to **campaign root creation**, where no parent README.md can or should exist.

---

## 2. F1 — Missing `init` Subcommand (HIGH)

### Root Cause

`__roadmap__/` is a container directory and must NOT have a `README.md`. There is no parent README for a campaign to register into. Yet `cmd_add` requires `parent/README.md` to exist. Therefore `dirtree-rdm add __roadmap__/campaign --type dir` **always fails** at campaign creation time with no tool-sanctioned workaround.

### Observed Failure Sequence

An agent following SKILL.md's "use dirtree-rdm for every mutation" directive encounters:

| Step | Action | Outcome |
|:-----|:-------|:--------|
| 1 | `dirtree-rdm add __roadmap__/campaign --type dir` | **ERROR**: parent README.md not found at `__roadmap__/README.md` |
| 2 | Agent infers it must create `__roadmap__/README.md` first | Manually writes `__roadmap__/README.md` — the **spurious README** |
| 3 | Retries `dirtree-rdm add __roadmap__/campaign --type dir` | Succeeds; tool creates `__roadmap__/campaign/README.md` |
| 4 | Both `__roadmap__/README.md` and `__roadmap__/campaign/README.md` exist | User observes "duplicate top-level README.md" |

The `__roadmap__/README.md` has no BFS role and no defined purpose. It is structural noise created by the deadlock.

### Missing Command

No `init` subcommand exists. The current command surface is:

```
add · status · update · move · insert · validate · ls · grammar
```

An `init <campaign>` command that creates `__roadmap__/<campaign>/` and bootstraps a BNF-valid template README.md **without requiring a parent README.md** would close this gap.

---

## 3. F2 — Over-Constrained "The One Rule" (HIGH)

### The Rule As Written

> **Never write or edit files under `__roadmap__/` by hand.** Use `dirtree-rdm` for every mutation.

### What It Correctly Guards Against

| Section type | Risk of manual edit | Should be tool-gated? |
|:-------------|:--------------------|:----------------------|
| Mermaid status block | High — BNF syntax is brittle; wrong classDef order or `-->` edge causes validation failure | **Yes** |
| Nodes table | High — trailing slash for dirs, exact emoji format, column count | **Yes** |
| Amendment Log | Medium — date and ID format `A\d+` enforced by BNF | **Yes** |
| Context, Goal, Pre-conditions, Success Gates | Low — free prose; agent is the correct author | **No** |
| Leaf step fields (Goal, Implementation Logic, Deliverables…) | Low — free prose | **No** |

### Why The Blanket Rule Fails

Two gaps make the blanket prohibition unenforceable today:

1. **No bootstrap tool** (F1 above): initial campaign README must come from somewhere; the template write must happen outside dirtree-rdm until `init` is added.
2. **No prose-editing tool**: there is no command to set the Context paragraph, Goal line, or Pre-conditions list. Those sections are legitimately agent-authored.

The rule's intent is correct but its scope is too broad. The distinction that should be expressed:

> **BNF-structural sections** (Mermaid block, Nodes table, Amendment Log, Progress table) — always touch via `dirtree-rdm`, never by hand.
> **Prose sections** (Context, Goal, Pre-conditions, Success Gates, leaf step fields) — authored by the agent; the tool does not manage these.

---

## 4. F3 — Pre-flight Error Message Contradiction (MEDIUM)

When pre-flight BNF validation fails, `readme.rs` emits:

> `"Fix the README.md manually or run dirtree-rdm validate <dir> for details."`

Even under the correctly-scoped rule (F2), "Fix the README.md manually" directs the agent to edit the BNF-structural sections that only the tool should touch. An agent following this instruction will attempt manual Mermaid or Nodes table edits, compounding the validation failure.

The message should eliminate the manual-fix suggestion and direct to the diagnostic command only.

---

## 5. Minor Findings

| ID | File | Line | Severity | Finding | Mitigated? |
|:---|:-----|:-----|:---------|:--------|:-----------|
| F4 | `main.rs` | 139 | LOW | `std::fs::write(&child_readme, …)` in `cmd_add` has no existence check — it unconditionally overwrites any pre-existing README.md in the new child directory | Yes — `node_path_clean.exists()` at line 129 prevents reaching this point for already-existing directories under normal use |
| F5 | `templates.rs` | 17 | LOW | `readme_template` initializes Success Gates with `⬜`; `dirtree-authoring.md` example uses `✅`; both accepted by BNF (`^\- [✅⬜] .+`) | No — cosmetic inconsistency; no functional impact |

---

## 6. Finding Summary

| ID | Severity | Location | Title | Impact |
|:---|:---------|:---------|:------|:-------|
| F1 | HIGH | `main.rs:120-128`, `dirtree-authoring.md` CRUD §4-5 | Missing `init` subcommand — bootstrap deadlock | Forced manual creation of `__roadmap__/README.md`; spurious duplicate at container level |
| F2 | HIGH | `SKILL.md` "The One Rule" | Blanket prohibition is unenforceable — prose sections have no tool | Agents cannot complete the task without violating the rule; ambiguous guidance erodes trust in the skill |
| F3 | MEDIUM | `readme.rs:304` (pre-flight error) | Error message says "fix manually" — contradicts even the refined rule | Agents attempt manual BNF edits; cascading validation failures |
| F4 | LOW | `main.rs:139` (`cmd_add`) | `std::fs::write` without child README existence check | Silent overwrite risk; mitigated by prior `exists()` guard |
| F5 | LOW | `templates.rs:17` | `readme_template` uses `⬜`; authoring guide example uses `✅` | Cosmetic confusion; no functional impact |

---

## 7. Recommended Fixes

| ID | Action | Target | Nature |
|:---|:-------|:-------|:-------|
| F1-a | Add `dirtree-rdm init <campaign>` subcommand: creates `__roadmap__/<campaign>/` + BNF-valid template README.md; H1 defaults to campaign name; no parent README required | `main.rs`, `templates.rs` | Code |
| F1-b | Replace CRUD Create steps 4-5 in `dirtree-authoring.md` with `dirtree-rdm init <campaign>` + "fill in prose sections"; add note: `__roadmap__/` must NOT have a README.md | `dirtree-authoring.md` | Docs |
| F1-c | Add `init` to TOC and command reference in `dirtree-cli.md`; prepend "Bootstrap" subsection to each tier in `dirtree-tier-examples.md` | `dirtree-cli.md`, `dirtree-tier-examples.md` | Docs |
| F2 | Rewrite SKILL.md "The One Rule": distinguish BNF-structural sections (tool-gated) from prose sections (agent-authored); update task table to reference `dirtree-rdm init` | `SKILL.md` | Docs |
| F3 | Change pre-flight error: "Run `dirtree-rdm validate <dir>` for details. Do not edit Mermaid, Nodes table, or Amendment Log by hand." | `readme.rs:304` | Code |
| F4 | Add `if child_readme.exists() { bail!(…) }` before `std::fs::write` in `cmd_add` | `main.rs:138` | Code (defensive) |
| F5 | Change `readme_template` Success Gates placeholder from `⬜` to `✅` | `templates.rs:17` | Code (cosmetic) |

F1 (init subcommand + authoring guide + CLI docs) is the primary remediation — it eliminates the deadlock that produces the observed duplicate README. F2 (rule rewrite) removes the remaining guidance ambiguity. Both are prerequisite to consistent agent behavior at campaign creation time.
