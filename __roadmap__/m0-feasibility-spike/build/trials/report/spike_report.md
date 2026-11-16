# Spike report

**Goal**: The trial data becomes a findings report with a go or no-go per host. R01 is updated where M0 changed a decision.
**Pre-conditions**:
- [ ] `../host_trials.md` is done
**Success Gates**:
- ⬜ [run] `uv run --no-project __reports__/m0-feasibility-spike/timing.py __reports__/m0-feasibility-spike/trials/probe.jsonl` prints the median and the worst time to display, cold and warm, per host, with the bootstrap 95% CI of each median and the payload size
- ⬜ [static] The report gives a go or no-go for Claude Code Desktop, VS Code Copilot and the Claude Code CLI. It includes the raw `hostCapabilities` and `hostContext` JSON per host, the protocol versions each host speaks and the view announces, the allow-rule string and the crate-name result, and gives a PASS, DEVIATION or BLOCKED verdict with evidence for every gate of `__roadmap__/roadmap-mcp-rework/m0_feasibility_spike.md`
- ⬜ [static] On any no-go, the report proposes the amendment of the later leaves it affects (M4, M6 or M7) without applying it
- ⬜ [static] R01 §Proposed State and §Risks & Mitigations match what the spike proved (the rmcp pin, the Apps calls, host support)
**References**: [writing-reports skill, findings type](../../../../../skills/) — report shape; [R01 §Proposed State, §Risks & Mitigations](../../../../../__design__/rdm/architecture.md) — what M0 may change

## Step 1: Analyse the timing
**Goal**: The timing numbers come from a script, not from hand arithmetic.
**Implementation Logic**:
1. A stdlib-only script reads the probe JSONL and groups it by host and run kind.
2. For each group it computes N, the median, the worst case, and the bootstrap 95% CI of the median (10,000 resamples, fixed seed). It also takes the payload size from `payload_bytes` and the size of the HTML.
3. It prints a markdown table.
**Deliverables**: `__reports__/m0-feasibility-spike/timing.py`: functions `load`, `bootstrap_median_ci`, `main`
**Consistency Checks**: `uv run --no-project __reports__/m0-feasibility-spike/timing.py __reports__/m0-feasibility-spike/trials/probe.jsonl` (expected: PASS)
**Commit**: `docs(reports): analyse the m0 display timing`

## Step 2: Write the findings report
**Goal**: Later milestones can read one report and know what holds on each host.
**Implementation Logic**:
1. Follow the findings type of `writing-reports`, data first.
2. Contents:
   - the gate table with a verdict for each gate
   - one go or no-go per host, with its reason
   - the host dumps
   - the timing table and why its N controls the noise
   - the allow-rule string
   - the crate names
   - the Apps calls that worked on rmcp 3.10.0
   - the findings on protocol support
3. A no-go proposes an amendment and does not apply it.
**Deliverables**: `__reports__/m0-feasibility-spike/00-findings_spike_v0.md`, `__reports__/m0-feasibility-spike/README.md`
**Consistency Checks**: `test -s __reports__/m0-feasibility-spike/00-findings_spike_v0.md` (expected: PASS)
**Commit**: `docs(reports): report the m0 feasibility spike`

## Step 3: Bring R01 in line
**Goal**: R01 states what the spike proved.
**Implementation Logic**:
1. Edit only §Proposed State, which holds the rmcp pin and the Apps mechanics, and §Risks & Mitigations, which holds host support.
2. M0b owns the other sections.
**Deliverables**: `__design__/rdm/architecture.md`: §Proposed State, §Risks & Mitigations
**Consistency Checks**: `git diff --stat main -- __design__/rdm/architecture.md` (expected: PASS)
**Commit**: `docs(design): record what the m0 spike proved`
