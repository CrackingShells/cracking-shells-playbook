---
applyTo: '**/*'
description: 'Phase 1 architecture reporting: Mermaid-first, contracts/invariants, alternatives and risks'
---

# Reporting: Architecture (Phase 1)

Phase 1 reports are evaluated primarily on **architecture clarity** and **decision quality**.

## 1. Required artifacts (default)
- **At least 1 Mermaid diagram** describing the proposed system or change
- **Contracts & invariants** (schemas + signatures + pseudo-code)
- **Alternatives considered** (with tradeoffs)
- **Risk register** (top risks + mitigations)

## 2. Diagram selection guide
Use the smallest diagram that communicates the point.

- **System/context flow** (`graph TD`): components and dependencies
- **Sequence** (`sequenceDiagram`): key request/response and component interactions
- **State** (`stateDiagram-v2`): lifecycles and transitions
- **ER** (`erDiagram`): schema/data relationships

Mermaid reference:
- [documentation-resources.instructions.md](./documentation-resources.instructions.md)

## 3. Contracts & invariants (how to write them)
Prefer *interfaces* over implementations.

Include:
- Inputs/outputs (types or schemas)
- Error model (what fails, how it surfaces)
- Invariants (what must always be true)
- Pseudo-code for non-trivial logic (focus on observable behavior)

Avoid:
- Full class/module definitions
- Full function bodies

## 4. Alternatives & decisions
For each major decision:
- Options considered
- Why the chosen approach wins
- What tradeoffs were accepted

## 5. Roadmap handoff
If the effort is complex/multi-phase, end the Stage 1 report with a roadmap handoff:
- Create a roadmap directory tree under `__roadmap__/<campaign>/` using:
  - [roadmap-generation.instructions.md](./roadmap-generation.instructions.md)
