---
id: QSO-032
title: QSOS orchestration for Open TIX plans (read/write/orient)
status: ready
priority: P1
type: feature
created: 2026-08-17
updated: 2026-08-19
---

# QSO-032: QSOS Orchestration for Open TIX Plans

## Description

**Plan format ownership:** [Open TIX v1.1 §6](../../../catalyst/opentix/SPEC.md) ([OPENTIX-002](../../../catalyst/opentix/work/OPENTIX-002-opentix-plans-layer-v1-1/OPENTIX-002-opentix-plans-layer-v1-1.md)). QSOS does **not** define `work/plans/` schema.

This ticket covers QSOS **skills and tooling** that read, write, and orient against Open TIX plans.

* **Council report (historical):** [`docs/council-reports/2026-08-17--council-design--provider-agnostic-canonical-plan-standard.md`](../docs/council-reports/2026-08-17--council-design--provider-agnostic-canonical-plan-standard.md)

## Deliverables

- [x] Open TIX v1.1 plans spec (OPENTIX-002 — done elsewhere)
- [ ] `/qsos-plan` persists approved plans to `work/plans/YYYY-MM-DD-action-<slug>.md` per Open TIX SPEC
- [ ] `/qsos-orient` loads active plan (`status: active`) into context when present
- [ ] `docs/standards/project-structure.md` references Open TIX §6 for plans (not a duplicate spec)
- [ ] `qsos init` creates `work/plans/README.md` stub pointing at Open TIX SPEC

## Out of scope

- Plan JSON schema in QSOS repo (lives in `catalyst/opentix/schemas/`)
- `qsos-lint` duplicating plan rules (delegate to `lint_opentix.py`)

## Acceptance

- Agent running `/qsos-plan` writes a compliant action plan file
- `/qsos-orient` surfaces active plan + linked tickets
