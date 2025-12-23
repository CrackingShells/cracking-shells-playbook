---
applyTo: '**/*'
description: 'Reporting mechanics: paths, naming/versioning, README convention, reports vs design docs'
---

# Reporting: Structure and Naming

## 1. Default locations

### Reports (temporary / iterative)
Save reports under:
- `__reports__/<topic>/<round>-<descriptive_name>_v<version>.md`

Where:
- `<topic>` is a descriptive snake_case folder name (prefer prefixes: `feature_`, `bug_fix_`, `refactor_`, `analysis_`).
- `<round>` is `00`, `01`, `02`, … (increments per user prompt/work session).
- `<descriptive_name>` is snake_case and describes the report.
- `<version>` is `v0`, `v1`, `v2`, … (increments per iteration/refinement of the same report).

### Design docs (durable)
Save long-lived design artifacts under:
- `__design__/...`

Use `__design__/` for:
- Architecture overviews and stable design decisions
- Roadmaps and long-term plans

Use `__reports__/` for:
- Iterative analysis and test definitions during active work
- Debugging / execution logs and validation evidence
- Knowledge transfer notes from an LLM-driven cycle

## 2. Directory creation rules
- Auto-create missing `__reports__/<topic>/` directories.
- Keep all reports for one work session in the same `<topic>` directory.

## 3. README convention (recommended)
Each `__reports__/<topic>/` should include a `README.md` that:
- Lists documents in chronological order
- Marks the latest versions as current
- Provides a short “status” section (Phase 1/2/3 etc.)

## 4. Minimal naming examples
- `__reports__/feature_version_command/00-architecture_analysis_v0.md`
- `__reports__/feature_version_command/01-test_definition_v0.md`
- `__reports__/feature_version_command/01-test_definition_v1.md`

## 5. Report hygiene
- Prefer fewer, higher-quality reports over many small fragments.
- If multiple reports are created, ensure each has a distinct purpose and a navigation README.
