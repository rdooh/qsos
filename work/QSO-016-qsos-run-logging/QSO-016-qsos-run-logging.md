---
id: QSO-016
title: Structured JSONL event logging for QSOS runs
status: done
priority: medium
type: feat
impact_scope:
  - all skills (must emit log events)
  - qsos.md (run_id generation, log path, terminal output protocol)
  - deploy.py (may need to deploy log-writing helper or shared logic)
features:
  - docs/features/qsos-run-logging.feature
adrs:
  - docs/decisions/ADR-009-qsos-run-logging-schema.md
architecture_updated: false
depends_on: []
---

Two-track output for every QSOS run:

1. **Terminal** — lean checkpoint lines only. One line per skill transition, gate
   decision, or verdict. No prose, no reasoning, no verbose context.

2. **JSONL log** — append-only, one JSON object per line, written incrementally
   as the run progresses. Persists after the session ends. Stored at:
   `work/<ticket-slug>/logs/qsos-run-<run_id>.jsonl`

Every event shares a common envelope: run_id, timestamp, ticket, skill, type, data.
The event taxonomy (~40 types across 9 categories) is defined in ADR-009.

High-signal events (deviations, verdicts, failures, process health issues) carry
concise structured payloads. Low-signal events (skill_started, file_created) carry
minimal data — the type itself is the signal.

Motivation: runs currently produce a firehose of conversational prose that scrolls
off screen, is never reviewed, and burns tokens on output nobody reads. The JSONL
log replaces this as the audit trail and enables future analysis, rendering, and
retrospective pattern detection.
