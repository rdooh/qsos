---
id: QSO-005
title: Refactor qsos-verify to dispatch verifier agent + update qsos-doc-sync
status: done
priority: medium
type: refactor
impact_scope:
  - qsos/skills/
features:
  - docs/features/verify-refactor.feature
adrs: []
architecture_updated: false
depends_on:
  - QSO-001
  - QSO-002
---

Two related changes to close out the agent integration and tighten the doc-sync step.

**qsos-verify refactor:**
- Move evidence type catalog and blocking language rule into `verifier.md` agent system prompt
- Reduce `qsos-verify` skill to: context-loading (ticket, claim, test runner), dispatch call, INCONCLUSIVE escalation path
- Standalone invocation (no `qsos-implement` block) still works — skill asks user to state the claim
- Verify behavioral parity before and after: same evidence type selection, same verdict format

**qsos-doc-sync update:**
- Add explicit step: "Were any architectural decisions made during implementation not recorded as an ADR?"
- If yes: route to `qsos-architecture` before ticket close
- If no: state "no unrecorded decisions" explicitly
- This is distinct from the existing architecture model update step (which promotes Target → Current elements)
