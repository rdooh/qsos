---
id: QSO-018
title: Instrument remaining 6 uninstrumented skills with log events
status: done
priority: high
type: fix
impact_scope:
  - skills/qsos-brainstorm.md
  - skills/qsos-orient.md
  - skills/qsos-feature-doc.md
  - skills/qsos-architecture.md
  - skills/qsos-doc-sync.md
  - skills/qsos-security.md
features: []
adrs:
  - docs/decisions/ADR-009-qsos-run-logging-schema.md
architecture_updated: false
depends_on: []
---

Six skills in the QSOS chain produce no log events, leaving gaps in the run log
for any full-chain execution. All six need the same treatment as the skills
instrumented in QSO-016: skill_started at entry, skill_completed at exit, and
any relevant mid-skill events (gate_reached/gate_passed, insight, gap_discovered,
file_created/file_modified, adr_created).

Skills to instrument:
- qsos-brainstorm.md — skill_started, gate events (scoping questions), file_created
  (feature file + ticket), adr_created if ADR produced, skill_completed
- qsos-orient.md — skill_started, gap_discovered for each flagged gap,
  assumption_flagged, skill_completed
- qsos-feature-doc.md — skill_started, file_created/modified (feature file),
  adr_created if ADR produced, skill_blocked if BLOCKED verdict, skill_completed
- qsos-architecture.md — skill_started, adr_created, file_modified (DSL),
  skill_completed
- qsos-doc-sync.md — skill_started, file_modified per doc updated,
  file_modified for ticket status change, skill_completed
- qsos-security.md — skill_started, gap_discovered per finding,
  skill_blocked if critical findings, skill_completed

Use the canonical append pattern from ADR-009 with graceful skip guards throughout.
