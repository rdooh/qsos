---
feature: Test harness manifest
ticket: QSO-011
status: @done
architecture_updated: false
---

# Test Harness Manifest

## Background

Each project using QSOS may have different testing infrastructure in place: unit runners,
E2E runners, coverage thresholds, pre-commit hooks. Currently agents infer this by scanning
config files — which is fragile and inconsistent. A `testing/manifest.json` at the project
root is the single declared record of what testing tools and gates are in place, what was
decided, and what is currently enforced. Agents read it instead of inferring. The
`qsos-coverage-check` skill audits it against reality and surfaces gaps as a prioritised list.

---

## Feature: Test harness manifest file

**Scenario: Developer inspects the test harness manifest**
  Given a project has a `testing/manifest.json` file
  When a developer reads it
  Then it declares the unit test runner in use (or null if absent)
  And it declares the E2E test runner in use (or null if absent)
  And it declares whether a pre-commit hook is wired
  And it declares whether a pre-push hook is wired
  And it declares the coverage threshold (or null if not enforced)
  And it references any ADRs that record testing strategy decisions

**Scenario: Manifest is created on project setup**
  Given a new project is being set up under QSOS
  When the developer runs the setup command or qsos-coverage-check for the first time
  Then a `testing/manifest.json` is created with detected values pre-filled
  And fields that cannot be detected are set to null with a comment indicating they need configuration
  And the manifest is committed to version control

**Scenario: Agent reads manifest to know what evidence is required**
  Given `testing/manifest.json` exists in the project
  When the verifier agent or qsos-implement reads the project context
  Then it reads the manifest to determine what runners are available
  And what evidence types are mandatory for the current ticket
  And it does not scan package.json or config files to infer this

---

## Feature: qsos-coverage-check skill

**Scenario: Coverage check detects declared runner not installed**
  Given `testing/manifest.json` declares `unit_runner: jest`
  And no jest configuration is found in the project
  When the developer runs `/qsos-coverage-check`
  Then it reports: "unit runner declared as jest but no jest config found"
  And it rates this gap as HIGH priority
  And it suggests the remediation step

**Scenario: Coverage check detects missing pre-commit hook**
  Given `testing/manifest.json` declares `pre_commit_hook: false`
  Or the field is null
  When the developer runs `/qsos-coverage-check`
  Then it reports the gap with recommended hook configuration
  And it notes the hook template available in `utilities/`

**Scenario: Coverage check detects undeclared runner in the project**
  Given a `playwright.config.ts` exists in the project
  And `testing/manifest.json` declares `e2e_runner: null`
  When the developer runs `/qsos-coverage-check`
  Then it reports: "playwright config found but not declared in manifest"
  And it prompts the developer to update the manifest

**Scenario: Coverage check passes cleanly**
  Given all declared runners are installed and configured
  And all declared hooks are wired
  And no undeclared runners are detected
  When the developer runs `/qsos-coverage-check`
  Then it reports: "POSTURE: HEALTHY — all declared testing infrastructure confirmed"
  And it issues verdict: "PASS"

**Scenario: Coverage check produces a prioritised gap report**
  Given one or more gaps are detected
  When `/qsos-coverage-check` completes
  Then it outputs gaps ordered by priority: HIGH, MEDIUM, LOW
  And each gap includes a one-line description and a suggested remediation
  And it issues verdict: "GAPS FOUND"
  And it does not make any changes to the project

**Scenario: Coverage check is run before qsos-verify**
  Given qsos-verify is about to run for a ticket
  When the chain reaches the verify step
  Then qsos-coverage-check runs first
  And if HIGH priority gaps are present the developer is notified before proceeding
  And the developer may proceed or address gaps first
