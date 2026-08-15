---
feature: QSOS Review Skill
ticket: QSO-003
status: @done
architecture_updated: false
---

# QSOS Review Skill

## Background

After implementation, the QSOS chain goes directly to verification. There is no step that inspects code quality, maintainability, correctness patterns, or DRY violations. The `qsos-review` skill closes this gap by dispatching the `code-reviewer` agent against the implementation diff before verification runs.

---

## Feature: Review skill chain integration

**Scenario: Review skill runs after implementation**
  Given `qsos-implement` has completed and emitted its completion block
  When `qsos-review` is invoked
  Then it dispatches the code-reviewer agent against the current diff
  And it waits for findings before proceeding

**Scenario: Review skill is not invoked without an approved plan**
  Given no `qsos-implement` completion block is present in context
  When `qsos-review` is invoked
  Then it halts with: "No implementation block found — run /qsos-implement first"

---

## Feature: Finding confidence gates

**Scenario: CRITICAL finding with confidence 7+ halts the chain**
  Given the code-reviewer agent returns a CRITICAL finding with confidence 8
  When qsos-review processes the findings
  Then it emits `REVIEW: BLOCKED — 1 critical finding`
  And it displays the finding with its fix recommendation
  And it routes back to qsos-implement for remediation
  And it does not proceed to qsos-verify

**Scenario: Finding with confidence 5-6 shows with caveat**
  Given the code-reviewer agent returns an INFORMATIONAL finding with confidence 5
  When qsos-review processes the findings
  Then it displays the finding with: "Medium confidence — verify this is actually an issue"
  And it does not block chain progression

**Scenario: Finding with confidence below 5 is informational only**
  Given the code-reviewer agent returns a finding with confidence 3
  When qsos-review processes the findings
  Then the finding appears in an appendix section only
  And it does not block chain progression

**Scenario: No findings — chain proceeds**
  Given the code-reviewer agent returns NO FINDINGS
  When qsos-review processes the output
  Then it emits `REVIEW: CLEAN`
  And it proceeds to the next chain skill

---

## Feature: Multi-specialist confirmation

**Scenario: Same finding from two confidence scores gets boosted**
  Given two findings share the same fingerprint (path:line:category)
  When qsos-review deduplicates findings
  Then it keeps the finding with the higher confidence score
  And it tags it as confirmed with boosted confidence (capped at 10)
  And it notes both detections in the output
