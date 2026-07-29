---
feature: Pytest coverage threshold enforcement
ticket: TIX-014
status: @done
architecture_updated: false
---

@done
Feature: Pytest coverage threshold enforcement
  As a QSOS developer
  I want a coverage threshold declared and enforced for the qsos repo's test suite
  So that coverage gaps are caught at commit time rather than discovered during review

  Scenario: pytest runs with coverage threshold enforced
    Given pytest.ini declares --cov=. and --cov-fail-under with a threshold value
    And the test suite is run
    When all tests pass and coverage meets or exceeds the threshold
    Then pytest exits 0

  Scenario: coverage drops below threshold — commit is blocked
    Given pytest.ini declares --cov-fail-under with a threshold value
    When the test suite runs with coverage below the threshold
    Then pytest exits non-zero
    And the pre-commit hook fails
    And the developer sees the coverage shortfall before the commit lands

  Scenario: coverage_threshold is declared in testing/manifest.json
    Given pytest.ini enforces a threshold
    When a developer reads testing/manifest.json
    Then coverage_threshold is set to the integer value matching pytest.ini
    And it is not null
