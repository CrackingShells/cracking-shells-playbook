# CI matrix

**Goal**: The Rust workflow runs the suite on macOS, Linux and Windows, checks the MSRV, can run a soak on demand, and runs on pushes to `feat/**`, so the gated push of `feat/m1-core-foundation` proves the three-OS gates.
**Pre-conditions**:
- [ ] `../r01_sync.md` is done
**Success Gates**:
- ✅ [run] `actionlint .github/workflows/rust.yml` passes
- ✅ [static] A matrix job runs `cargo test --workspace --locked` on `ubuntu-latest`, `macos-latest` and `windows-latest`; an MSRV job runs `cargo test --workspace --locked` with toolchain 1.88 on Linux
- ✅ [static] `workflow_dispatch` with a `soak` input sets `RDM_CRASH_ITERS` and `PROPTEST_CASES` high; a `push` to `feat/**` triggers the workflow (a dispatch needs the workflow on the default branch, so it cannot be the first run)
- ✅ [static] `.gitattributes` pins `crates/**/tests/fixtures/**/*.json` to `eol=lf`, and the Windows job prints `git ls-files --eol` for those fixtures as the CRLF probe's evidence
- ✅ [static] The MSRV job was seen red locally: `cargo +1.88 check --workspace` fails on a throwaway commit that calls `File::lock` (stable since 1.89), and the commit is dropped. Done instead in a scratch crate with `rust-version = "1.88"` outside the repository (no commit to drop): `error[E0658]: use of unstable library feature 'file_lock'`, while stable passes
**References**: [.github/workflows/rust.yml](../../../../.github/workflows/rust.yml) — the current single-OS job; [M0 knowledge transfer](../../../../__reports__/m0-feasibility-spike/) — test on real hosts early

## Step 1: Matrix, MSRV and soak
**Goal**: One push shows the suite on three operating systems.
**Implementation Logic**:
1. Turn the job into a matrix over the three runners and add the MSRV job. Keep the `paths` filters and add `feat/**` to the push branches.
2. Add `workflow_dispatch` with a boolean `soak` input mapped to the two environment variables.
3. Add `.gitattributes` for the JSON fixtures and the `git ls-files --eol` step on Windows. The decoder must still accept CRLF, because campaigns live in other repositories this one does not configure; that is a row of the codec's non-canonical guard.
4. Do the local MSRV seen-red check and drop its commit.
**Deliverables**: `.github/workflows/rust.yml`: jobs `test` (matrix), `msrv`; `.gitattributes`
**Consistency Checks**: `actionlint .github/workflows/rust.yml` (expected: PASS)
**Commit**: `ci(rust): test on macos, linux and windows and check the msrv`
