---
feature: TDD discipline and verify test floor
ticket: TIX-012
status: @accepted
architecture_updated: false
---

# TDD Discipline and Verify Test Floor

## Background

The QSOS chain currently has no explicit instruction to write tests before implementation code,
and the verifier agent may satisfy a verify claim with compilation evidence alone — even when
a test runner is available. Two changes close these gaps: `qsos-implement` is updated to
require a failing test before each implementation item, and `qsos-verify` (plus the verifier
agent) is updated so that test runner output is a mandatory evidence floor when a runner
is declared in the project's `testing/manifest.json`.

---

## Feature: TDD discipline in qsos-implement

**Scenario: Implementer writes failing test before implementation code**
  Given an approved plan with N items
  When qsos-implement executes each plan item
  Then before writing implementation code it writes a failing test that would pass
  when the item is complete
  And it runs the test suite to confirm the test fails (red)
  And only then writes the implementation to make it pass (green)
  And it confirms the test passes before marking the item done

**Scenario: Test runner not available — TDD instruction still applies**
  Given no test runner is declared in `testing/manifest.json`
  Or the manifest does not exist
  When qsos-implement runs
  Then it notes the absence and flags it as a gap
  And it proceeds with implementation but recommends setting up a runner
  And it does not silently skip the TDD step without noting it

**Scenario: Plan item is non-testable — deviation declared**
  Given a plan item describes a change with no observable test surface
  (e.g. a comment update, a config rename)
  When qsos-implement reaches that item
  Then it declares: "DEVIATION: non-testable item — skipping TDD step"
  And it states the reason explicitly
  And it proceeds without a test

---

## Feature: Test runner floor in qsos-verify

**Scenario: Verifier runs test suite before other evidence types**
  Given `testing/manifest.json` declares a unit runner
  When qsos-verify dispatches the verifier agent
  Then the verifier runs the declared test suite first
  And includes the test output in the evidence artifact
  And only proceeds to other evidence types after the test run

**Scenario: Tests fail — verify cannot proceed**
  Given the test suite is run as part of verification
  And one or more tests fail
  When the verifier processes the results
  Then it issues UNCONFIRMED immediately
  And it surfaces the failing tests in the evidence
  And it does not attempt other evidence types

**Scenario: Compilation alone cannot satisfy verify when tests exist**
  Given `testing/manifest.json` declares a unit runner
  And the verifier agent is assessing evidence
  When the only evidence gathered is a successful build or type check
  Then the verifier must not issue CONFIRMED
  And it must attempt to run the test suite before issuing any verdict

**Scenario: No test runner declared — compilation evidence is sufficient floor**
  Given `testing/manifest.json` declares no unit runner
  Or the manifest does not exist
  When the verifier agent is assessing evidence
  Then it may use compilation or other evidence types as the floor
  And it notes the absence of a test runner in the evidence artifact
