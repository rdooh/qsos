---
id: TIX-015
title: Verify hardening and qsos-validate human-in-the-loop skill
status: done
priority: high
type: feat
impact_scope:
  - qsos-verify skill
  - verifier agent
  - qsos-validate skill (new)
  - qsos chain (validate added between verify and doc-sync)
features:
  - docs/features/validate-skill.feature
  - docs/features/verify-evidence-standards.feature
adrs:
  - docs/decisions/ADR-008-validation-skill-and-evidence-format.md
architecture_updated: false
depends_on: []
---

Two related improvements to the QSOS verification layer, motivated by agents claiming
victory on logic alone rather than demonstrable evidence.

1. **qsos-validate** — new standalone skill that derives a human validation checklist
   adversarially from weak spots in prior verify evidence. Shows full checklist upfront.
   Executes automated steps itself with direct file:// links. Uses AskUserQuestion for
   each human step one at a time. Saves results as CTRF JSON to
   work/<ticket-slug>/evidence/validation-ctrf.json.

2. **qsos-verify hardening** — every evidence claim must include a direct file:// or
   http:// link. Hedging language ("should work", "likely fixed", "appears to be working")
   is prohibited. Logic-only verdicts must produce UNCONFIRMED, not CONFIRMED. Verifier
   agent prompt updated accordingly.
