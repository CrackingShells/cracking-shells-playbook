# Packaging

## Context

Final milestone of the `create-reporting-skill` campaign. Entered only after all four authoring leaf tasks at root level complete. Packages the skill source files into a distributable archive.

## Reference Documents

- [reporting.instructions.md](../../../instructions/reporting.instructions.md) — Core contract (for final review before packaging)

## Goal

Produce a validated, distributable `dist/reporting.skill` file.

## Pre-conditions

- [ ] All four root-level authoring tasks complete
- [ ] `skills/reporting/SKILL.md` and all three reference files exist

## Success Gates

- ✅ `package_skill.py` runs without validation errors
- ✅ `dist/reporting.skill` exists

## Status

```mermaid
graph TD
    package_and_validate[package_and_validate.md]:::done
    classDef done fill:#166534,color:#bbf7d0
    classDef inprogress fill:#854d0e,color:#fef08a
    classDef planned fill:#374151,color:#e5e7eb
    classDef amendment fill:#1e3a5f,color:#bfdbfe
    classDef blocked fill:#7f1d1d,color:#fecaca
```

## Nodes

| Node | Type | Status |
|:-----|:-----|:-------|
| `package_and_validate.md` | 📄 Leaf Task | ✅ Done |

## Amendment Log

| ID | Date | Source | Nodes Added | Rationale |
|:---|:-----|:-------|:------------|:----------|

## Progress

| Node | Branch | Commits | Notes |
|:-----|:-------|:--------|:------|
| `package_and_validate.md` | `task/package-and-validate` | 1 | Fixed description angle brackets |
