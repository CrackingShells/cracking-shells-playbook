# Add Per-Skill package.json Markers

**Goal**: Create minimal `package.json` files in each of the 4 skill directories so `semantic-release-monorepo` can track per-skill versions.
**Pre-conditions**:
- [ ] Skill directory names confirmed: `committing-changes`, `managing-roadmaps`, `writing-release`, `writing-reports`
**Success Gates**:
- ✅ All 4 files exist: `skills/<name>/package.json` [static]
- ✅ Each contains `{"name":"<name>","version":"1.0.0"}` where `<name>` matches the directory and the `name:` field in `SKILL.md` [static]
- ✅ `uv run tools/package_skill.py skills/managing-roadmaps dist/` does NOT include `package.json` in the output zip (requires `vendor-tools` complete) [run]
**References**: [R01 §Deliverable 3](~/.claude/plans/binary-jumping-trinket.md) — package.json invariants

---

## Step 1: Create per-skill package.json files

**Goal**: Seed each skill with a `package.json` that `semantic-release-monorepo` will read for path-scoped versioning.

**Implementation Logic**:
Read each skill's `SKILL.md` to confirm the `name:` frontmatter field matches the directory name, then create 4 files: `skills/committing-changes/package.json`, `skills/managing-roadmaps/package.json`, `skills/writing-release/package.json`, `skills/writing-reports/package.json`. Each file is exactly `{"name":"<dir-name>","version":"1.0.0"}` — no other fields. The `version` field starts at `1.0.0`; semantic-release manages it from the first release onward. Do NOT add extra fields; the packager excludes this file from the artifact, so its content never reaches skill consumers.
**References**: [R01 §Deliverable 3](~/.claude/plans/binary-jumping-trinket.md) — name invariant
**Deliverables**: `skills/committing-changes/package.json`, `skills/managing-roadmaps/package.json`, `skills/writing-release/package.json`, `skills/writing-reports/package.json` — each with `name` and `version` fields
**Consistency Checks**: `for d in committing-changes managing-roadmaps writing-release writing-reports; do python3 -c "import json,sys; d=json.load(open('skills/$d/package.json')); assert d['version']=='1.0.0'"; done` (expected: PASS)
**Commit**: `feat(release): add per-skill package.json version markers at 1.0.0`
