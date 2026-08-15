---
id: QSO-027
title: Skill integration — wire skills to utilities
status: done
priority: high
type: feat
impact_scope:
  - skills/qsos-audit.md
  - skills/qsos-orient.md
  - skills/qsos-doc-sync.md
  - skills/qsos-verify.md
  - skills/qsos-coverage-check.md
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-020
  - QSO-023
---

Update QSOS skills to delegate mechanical checks to utilities when available, with manual fallbacks preserved.

## Deliverables

- `/qsos-audit` — Tier 1: `qsos lint`; Tier 2: manual cross-entity checks (unchanged)
- `/qsos-orient` — `qsos query --ticket` for context assembly; manual file read fallback
- `/qsos-doc-sync` — run `qsos lint` (+ `--sync` when available) before close
- `/qsos-verify` — query scenario coverage from graph after `qsos ingest`
- `/qsos-coverage-check` — query graph for untested scenarios
- Each skill documents: utility command, fallback behaviour, expected output format

## Reference

QSOS utilities roadmap principle: "agent interface does not change between waves"

## Notes

Can land incrementally as utilities ship (audit first, then orient, then doc-sync, then verify).

## Verification

**Claim:** QSOS skills delegate mechanical checks to utilities when available and preserve documented manual fallbacks.

**Evidence type:** Skill behaviour audit + CLI/skill integration smoke tests

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Audit skill delegates Tier 1 to lint | `qsos-audit.md` invokes `qsos lint`; smoke run | `evidence/audit-delegation.md` |
| Audit skill falls back when lint unavailable | Fallback section present; tested with qsos off PATH | `evidence/audit-fallback.md` |
| Orient skill queries graph by ticket | `qsos-orient.md` invokes `qsos query --ticket` | `evidence/orient-delegation.md` |
| Orient skill falls back when query unavailable | Fallback section present | `evidence/orient-fallback.md` |
| Doc-sync runs lint and sync before close | `qsos-doc-sync.md` invokes `qsos lint` (+ `--sync`) | `evidence/doc-sync-delegation.md` |
| Verify queries scenario coverage from graph | `qsos-verify.md` invokes graph coverage query post-ingest | `evidence/verify-delegation.md` |
| Coverage-check queries untested scenarios | `qsos-coverage-check.md` invokes graph query | `evidence/coverage-check-delegation.md` |

### Commands

```bash
# Per-skill: run skill against QSOS repo with utilities installed, capture delegation log
grep -l "qsos lint\|qsos query\|qsos ingest" skills/qsos-*.md
```

**Evidence directory:** `work/QSO-027-skill-utility-integration/evidence/`
