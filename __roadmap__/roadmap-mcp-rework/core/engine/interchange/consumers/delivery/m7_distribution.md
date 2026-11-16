# M7 Distribution

**Goal**: The four crates are published by CI, and an installed plugin starts the server version it pins, on macOS, Linux and Windows, and keeps doing so across plugin updates with no action from the consumer.
**Pre-conditions**:
- [ ] `../m4_mcp_server.md` and `../m5_cli.md` are `done`
- [ ] Reminder `skills-rdm-M7` read
- [ ] Eliott has agreed to publish to crates.io
**Success Gates**:
- ⬜ A merge to `main` releases the crates (`rdm-schema`, then `rdm-engine`, then `rdm-mcp` and `rdm-cli`), attaches prebuilt binaries and checksums to the GitHub release, and only then releases the plugin pinned to that version
- ⬜ A plugin installed from a scratch marketplace starts the pinned server on a clean machine
- ⬜ After a plugin update the new server version starts with no manual step
- ⬜ With no network, an installed binary of the same major version starts with a warning; otherwise the failure names the exact install command
- ⬜ `tools/check_plugin.py` passes with the MCP manifests of `managing-roadmaps`, and the drift job is green
**References**: [R01 §Launch and distribution](../../../../../../../__design__/rdm/architecture.md) — the publication order and the self-provisioning launch

## Step 1: Plan the milestone

**Goal**: The campaign `__roadmap__/m7-distribution/` exists and Eliott has approved it.
**Implementation Logic**:
In plan mode, author the campaign with `managing-roadmaps`, deciding: the release tool for the crates beside the skills' `multi-semantic-release` (follow the `writing-release` skill: tooling derived from the repository, CI releases after a merge, no local release commands); tag shapes and the two version lines (`rdm-schema` alone; `rdm-engine`, `rdm-mcp` and `rdm-cli` on one product version, which the plugin pins) and whether the chosen release tool supports them; the order inside one workflow run, since a push made with the workflow token starts no other workflow; binary targets and checksums; the self-provisioning launch (download, verification, cache location, same-major fallback) or a better scheme if one is found; the MCP entries in `plugin.spec.json` and what `tools/assemble_plugin.py` and `tools/check_plugin.py` must learn; changes to `.github/workflows/release.yml`, `release-config.js`, `.githooks/pre-commit`, `Makefile` and `CONTRIBUTING.md`.
Adversarial verification is required for the launch (it downloads and runs a binary: checksum handling, cache poisoning, partial downloads). Reviewed-PR lane: this changes CI and release machinery.
**Deliverables**: `__roadmap__/m7-distribution/README.md` and its leaves
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/m7-distribution/` (expected: PASS)
**Commit**: `chore(roadmap): plan the m7-distribution milestone`

## Step 2: Build and land

**Goal**: The milestone campaign is executed and lands on `main`.
**Implementation Logic**:
Execute the campaign on `feat/m7-distribution`, then land it on `main` through the lane the `writing-release` skill assigns, as one merge commit. Git management follows the `writing-history` skill. The Success Gates above must pass on `main` after the merge. If a design decision changed, `__design__/rdm/architecture.md` changes in the same merge.
**Deliverables**: the release configuration for the crates; the reworked `.github/workflows/release.yml`; the launch code in `crates/rdm-mcp`; the MCP manifests under `plugins/managing-roadmaps/`; updated `plugin.spec.json`, `tools/`, `CONTRIBUTING.md`
**Consistency Checks**: `uv run tools/check_plugin.py --root . --spec plugin.spec.json` (expected: PASS)
**Commit**: `ci(release): land rdm distribution`

## Step 3: Close out

**Goal**: The milestone is recorded.
**Implementation Logic**:
Write a knowledge-transfer report under `__reports__/m7-distribution/`. Mark this leaf `done`. Propose the reminder reconciliation to Eliott: `skills-rdm-M7` complete; `skills-rdm-M8` startable once their other gates clear. Apply only on his yes, through the scheduling-work writer.
**Deliverables**: `__reports__/m7-distribution/` knowledge-transfer report; this leaf's status in `__roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/delivery/README.md`
**Consistency Checks**: `bash skills/managing-roadmaps/scripts/dirtree-rdm.sh validate __roadmap__/roadmap-mcp-rework/core/engine/interchange/consumers/delivery/` (expected: PASS)
**Commit**: `docs(reports): close the m7-distribution milestone`
