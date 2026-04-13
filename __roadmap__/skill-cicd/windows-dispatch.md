# Add Windows PowerShell Dispatch Wrapper

**Goal**: Add `dirtree-rdm.ps1` alongside `dirtree-rdm.sh` so the Windows binary is usable from Claude Code on Windows, and verify `build.sh` accepts per-target arguments for CI use.
**Pre-conditions**:
- [ ] `skills/managing-roadmaps/scripts/dirtree-rdm/build.sh` readable
- [ ] `skills/managing-roadmaps/scripts/dirtree-rdm/bin/` exists with at least the darwin binaries
**Success Gates**:
- ✅ `skills/managing-roadmaps/scripts/dirtree-rdm.ps1` exists [static]
- ✅ PowerShell script uses `$PSScriptRoot` for reliable path resolution [static]
- ✅ `build.sh` accepts a target triple as positional arg without ignoring it [static]
**References**: [R01 §Deliverable 4](~/.claude/plans/binary-jumping-trinket.md) — Windows dispatch and build.sh open item

---

## Step 1: Read build.sh and update to accept target arguments

**Goal**: Ensure `build.sh` can be called with specific Rust target triples and a `local` shorthand without ignoring them.

**Implementation Logic**:
Read `skills/managing-roadmaps/scripts/dirtree-rdm/build.sh`. If it already accepts target triples as positional args (`$@` or `$1`), document the calling convention in a comment and skip modification. If it hard-codes all 4 targets, add argument handling: `bash build.sh local` detects `uname -s`/`uname -m` and builds only the local target; `bash build.sh <triple> [<triple>...]` builds only the listed targets; `bash build.sh` with no args retains the existing build-all behavior. This makes the CI matrix call `bash build.sh aarch64-apple-darwin x86_64-apple-darwin` correct, and `make build-rust-local` fast for developers.
**References**: [R01 §build.sh open item](~/.claude/plans/binary-jumping-trinket.md) — verify before modifying
**Deliverables**: `skills/managing-roadmaps/scripts/dirtree-rdm/build.sh` — arg-handling block (or inline comment confirming existing convention)
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm/build.sh --help 2>&1 || grep -q 'local\|target' skills/managing-roadmaps/scripts/dirtree-rdm/build.sh` (expected: PASS)
**Commit**: `fix(managing-roadmaps): update build.sh to accept per-target and local args`

---

## Step 2: Create dirtree-rdm.ps1 Windows dispatch wrapper

**Goal**: Produce a PowerShell equivalent of `dirtree-rdm.sh` that resolves the correct Windows binary and forwards all arguments.

**Implementation Logic**:
Create `skills/managing-roadmaps/scripts/dirtree-rdm.ps1`. Use `$PSScriptRoot` (not `$MyInvocation.MyCommand.Path`) as the reliable way to get the script's directory in PowerShell — it works in all invocation contexts including dot-sourcing and module imports. Set `$arch = "x64"` (only Windows target for this sprint; arm64 Windows is out of scope). Resolve `$binary = Join-Path $PSScriptRoot "dirtree-rdm\bin\dirtree-rdm-windows-$arch.exe"`. Guard with `Test-Path` and `Write-Error` + `exit 1` if missing. Forward args with `& $binary @args`.
**References**: [R01 §Deliverable 4](~/.claude/plans/binary-jumping-trinket.md) — PowerShell wrapper spec
**Deliverables**: `skills/managing-roadmaps/scripts/dirtree-rdm.ps1` — `$PSScriptRoot` path resolution, `Test-Path` guard, `& $binary @args` forwarding
**Consistency Checks**: `grep -q 'PSScriptRoot' skills/managing-roadmaps/scripts/dirtree-rdm.ps1` (expected: PASS)
**Commit**: `feat(managing-roadmaps): add Windows PowerShell dispatch wrapper for dirtree-rdm`
