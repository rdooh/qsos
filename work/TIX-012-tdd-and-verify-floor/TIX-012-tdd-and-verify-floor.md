---
id: TIX-012
title: TDD discipline in qsos-implement + test runner floor in qsos-verify
status: todo
priority: high
type: feat
impact_scope:
  - skills/qsos-implement.md
  - skills/qsos-verify.md
  - agents/verifier.md
features:
  - docs/features/tdd-and-verify-floor.feature
adrs: []
architecture_updated: false
depends_on:
  - TIX-011
---

Two focused changes to the QSOS chain to make testing a first-class concern:

1. `skills/qsos-implement.md` — add TDD instruction: for each plan item, write a failing
   test first (red), then write implementation to make it pass (green). If no test runner
   is available, flag the gap and note the deviation explicitly. Non-testable items require
   an explicit deviation declaration.

2. `skills/qsos-verify.md` — before dispatching the verifier agent, check
   `testing/manifest.json`. If a unit runner is declared, pass it as a mandatory evidence
   requirement to the verifier. Compilation alone cannot satisfy verify when a runner exists.

3. `agents/verifier.md` — add rule: if test runner output is required (passed by qsos-verify),
   run it first. UNCONFIRMED immediately on test failure. Cannot issue CONFIRMED on
   compilation alone when tests are declared.

Notes:
- Depends on TIX-011 (manifest must exist before verify can read it)
- The TDD instruction in implement is behavioral (no hard gate yet — that comes with utilities)
- Hard gates (pre-commit hook enforcement, coverage thresholds) are a later concern once
  the utilities layer exists (see TIX-007 roadmap)
