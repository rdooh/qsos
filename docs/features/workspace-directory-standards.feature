---
feature: Workspace Directory and Testing Layout Standards
ticket: QSO-013
status: @done
architecture_updated: false
---

# Workspace Directory and Testing Layout Standards

## Background

QSOS projects require clear, consistent boundaries to prevent file structure layout drift.
This specification formalizes where test code, transient test results, and point-in-time
evidence are placed to ensure uniformity across projects.

---

## Feature: Test code location vs test configuration directory

**Scenario: Test code resides within the source tree**
  Given a QSOS-governed project
  When a developer or agent writes unit or E2E tests
  Then the test source files are placed inside the project source tree (e.g., `src/test/`, `tests/`, or adjacent to code files)
  And the `testing/` directory is reserved solely for manifests, configurations, and mocks

**Scenario: Transient test result files are git-ignored**
  Given a test run generates machine-readable outputs (e.g. `unit.json`, `coverage.json`)
  When the files are written
  Then they are saved to the `test-results/` directory at the project root
  And the `test-results/` directory is excluded in `.gitignore`

---

## Feature: Point-in-time ticket evidence

**Scenario: Verification evidence is stored in the ticket workspace**
  Given an active ticket QSO-013
  When a verifier runs verification or captures visual evidence (e.g. screenshots, curl logs)
  Then the artifacts are saved under `work/QSO-013/evidence/` or `work/QSO-013/screenshots/`
  And they are committed to version control as the ticket's durable record of success
