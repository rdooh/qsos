---
id: TIX-017
title: Subagent log events must be emitted by parent skill, not subagent
status: done
priority: high
type: fix
impact_scope:
  - skills/qsos-implement-fan.md
  - skills/qsos-verify.md
  - skills/qsos-review.md
features: []
adrs:
  - docs/decisions/ADR-009-qsos-run-logging-schema.md
architecture_updated: false
depends_on: []
---

Subagents spawned via the Agent tool do not have access to `.qsos/current-run.json`.
The subagent_spawned, subagent_completed, and subagent_blocked events are currently
wired inside the subagent instruction blocks, where they cannot fire.

These events must be moved to the PARENT skill at the point where the parent:
- Spawns the subagent (emit subagent_spawned)
- Receives the result back (emit subagent_completed or subagent_blocked)

Affected skills:
- qsos-implement-fan.md — Step 7 (fan-out) and Step 8 (collect results)
- qsos-verify.md — verifier agent dispatch and verdict receipt
- qsos-review.md — code-reviewer agent dispatch and findings receipt

Remove the append instructions from inside the subagent prompt blocks and move
them to the parent skill steps that surround the Agent tool call.
