# Adversarial Review and Evals

**Goal**: Attack the finished skill as a whole (V2) and run the five acceptance evals, recording both in the verification report. Skill files are read-only here.
**Pre-conditions**:
- [ ] `standalone.md` is merged; the skill content is final apart from V2's fixes
- [ ] `skills/writing-release/evals/evals.json` holds the five Phase 0 evals
**Success Gates**:
- ⬜ `__reports__/writing-release-rework/01-verification_v0.md` has a V2 section: every finding as `severity → file:section → claim → evidence → suggested fix`, plus an explicit statement of what was attacked and found sound
- ⬜ The same report has an Evals section: per eval, the answer produced with the skill loaded, and each assertion as pass or fail
- ⬜ No skill file is modified by this leaf
**References**: [Campaign contract §Gotchas](../../../../README.md) — C1–C5 are what to attack; [evals](../../../../../../skills/writing-release/evals/evals.json) — the acceptance tests

## Step 1: Adversarial review
**Goal**: Assume the skill is wrong somewhere, and find where.
**Implementation Logic**:
Attack, in order:
1. contradictions between references, and violations of the C2 owner table (a fact restated outside its owner);
2. restating `writing-history` instead of naming it;
3. whether the single-channel default and the lane thresholds are argued with reasons rather than asserted;
4. Navigation drift (every SKILL.md row routes to the section that actually answers it);
5. whether the description would trigger on the C1 trigger list and stay silent on unrelated git questions;
6. any rule or fact the two content passes changed (diff them against the `author/` merge);
7. residual C5 contamination the grep cannot see.
A review that finds nothing must say what it tried.
**Deliverables**: `__reports__/writing-release-rework/01-verification_v0.md` — section `V2 adversarial review`
**Consistency Checks**: `grep -c '^## V2 adversarial review' __reports__/writing-release-rework/01-verification_v0.md` (expected: PASS)
**Commit**: `docs(reports): record the adversarial review of writing-release`

## Step 2: Acceptance evals
**Goal**: Run the five evals against the skill and record pass or fail per assertion.
**Implementation Logic**:
For each eval, answer its `prompt` as an agent that has loaded `skills/writing-release/SKILL.md` and followed its routing; one fresh Sonnet subagent per eval is preferred. Check every assertion literally (`contains`, `contains_any`, `not_contains`, `not_contains_any`), and record the answer excerpt and the verdict. When an assertion fails, say whether the content or the assertion looks wrong; the coordinator adjudicates.
**Deliverables**: `__reports__/writing-release-rework/01-verification_v0.md` — section `Evals`
**Consistency Checks**: `grep -c '^## Evals' __reports__/writing-release-rework/01-verification_v0.md` (expected: PASS)
**Commit**: `docs(reports): record the writing-release eval results`
