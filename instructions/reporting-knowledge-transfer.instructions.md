---
applyTo: '**/*'
description: 'Knowledge transfer report: capture what went well/badly in an LLM-driven cycle and what to change next'
---

# Reporting: Knowledge Transfer (LLM Cycle)

A Knowledge Transfer (KT) report captures what happened during a cycle and, critically, **what to do differently next time**.

## Purpose
- Identify what worked well and what went poorly in an LLM-driven cycle (code, audit, documentation).
- Convert observations into concrete process improvements.

## Default length
- Keep it short and practical (aim ~1–2 pages).

## Recommended sections
1. **Executive summary** (what shipped / what changed)
2. **Wins** (what worked well)
3. **Pain points** (review friction, LLM failure modes, tooling gaps)
4. **Root causes** (why it happened; not just symptoms)
5. **Next-cycle changes** (specific instruction updates, workflow changes, checklists)
6. **Artifacts to preserve** (diagrams, test matrix, decision tables, scripts)
7. **Open questions** (unknowns to validate next cycle)

## Focus areas (examples)
- Prompting/instructions: what caused over-generation or drift?
- Review process: what slowed stakeholders down?
- Testing: where did the test plan explode and why?
- Documentation: what was unclear or outdated?

## What not to do
- Don’t paste large code blocks.
- Don’t rewrite the full architecture; link to Stage 1 analysis artifacts.
