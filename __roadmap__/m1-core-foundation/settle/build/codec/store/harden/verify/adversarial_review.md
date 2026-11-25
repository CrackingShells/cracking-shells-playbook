# Adversarial review

**Goal**: A fresh verifier, briefed to assume something is wrong, attacks the store, lock, intent record and cache for silent data loss and relaxed rules, and the three-OS CI run is recorded.
**Pre-conditions**:
- [ ] `../crash_test.md` and `../hot_cache.md` are done
- [ ] Eliott has allowed the push of `feat/m1-core-foundation`
**Success Gates**:
- ⬜ [static] `__reports__/m1-core-foundation/02-adversarial_review_v0.md` records, for every cornerstone guard (I3, I4, I7, I10, I12, the reverse index, the newer-file refusal), the mutation injected and that the guard went red
- ⬜ [static] The report checks every leaf's gates as written against what was built, and names any rule an implementer relaxed
- ⬜ [static] The report checks that every fixture comes from a real producer, and that the seams hold: `rev` in the store and in the cache, `RecordingFs` and `RealFs` effects, the error type across crates
- ⬜ [run] `! cargo test --workspace -- --list 2>/dev/null | grep confirm_` passes
- ⬜ [static] The CI run on `feat/m1-core-foundation` is green on macOS, Linux and Windows, soak and MSRV included, and its URL is in the report
**References**: [M0b knowledge transfer](../../../../../../../../__reports__/m0b-domain-model-spec/) — verifiers challenge relaxed rules; [campaign README](../../../../../../../../__roadmap__/m1-core-foundation/README.md) — the test design and the mutations each guard names

## Step 1: Attack
**Goal**: Every cornerstone guard is shown to catch its bug.
**Implementation Logic**:
Dispatch a fresh Sonnet verifier with the campaign README, both design documents and the branch. It injects each named mutation by hand in a scratch worktree (inverted condition, reordered write, dropped lock, directory-mtime-only revalidation, struct parsed before the version), runs the guard, and reverts. It runs the ownership check on each new test and the cull gate. Findings go back to the original implementers.
**Deliverables**: `__reports__/m1-core-foundation/02-adversarial_review_v0.md`
**Consistency Checks**: `test -f __reports__/m1-core-foundation/02-adversarial_review_v0.md` (expected: PASS)
**Commit**: `docs(reports): record the m1 adversarial review`

## Step 2: Three-OS evidence
**Goal**: The I3 and Windows gates are proven on real hosts.
**Implementation Logic**:
After Eliott's go, the coordinator pushes `feat/m1-core-foundation`, dispatches the soak, and records the run URL and the CRLF probe output in the report.
**Deliverables**: `__reports__/m1-core-foundation/02-adversarial_review_v0.md`: section CI evidence
**Consistency Checks**: `grep -q 'CI evidence' __reports__/m1-core-foundation/02-adversarial_review_v0.md` (expected: PASS)
**Commit**: `docs(reports): record the m1 three-os ci run`
