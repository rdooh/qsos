---
feature: Evidence-grounded verify verdicts
ticket: TIX-015
status: @done
architecture_updated: false
adrs:
  - docs/decisions/ADR-008-validation-skill-and-evidence-format.md
---

@done
Feature: Evidence-grounded verify verdicts
  As a QSOS developer
  I want qsos-verify to require independently-observable evidence for every claim
  So that CONFIRMED means the work demonstrably functions, not that it logically should

  Scenario: Every evidence claim includes a direct link or file path
    Given the verifier agent is assembling evidence
    When it records a claim (e.g. "tests passed", "build succeeded")
    Then it must include a direct file:// path or http:// URL the developer can open
    And it must not state a claim without the link — the link IS the evidence

  Scenario: Hedging language is prohibited in verdicts
    Given the verifier agent is issuing a verdict
    Then it must not use phrases: "should work", "likely fixed", "appears to be working",
    "probably", "seems to", "I believe", "logically"
    And if it cannot confirm without hedging it must issue UNCONFIRMED, not CONFIRMED

  Scenario: Logic-only verdict is rejected
    Given the verifier agent has reasoned that a fix is correct
    But has not run any tool, read any output file, or followed any link
    When it attempts to issue CONFIRMED
    Then the verdict must be UNCONFIRMED
    And it must state: "No independently-observable evidence gathered"

  Scenario: Verifier surfaces what it cannot check
    Given the verifier agent cannot run a test or open a file
    When it assembles its evidence record
    Then it explicitly lists what it could not verify
    And it issues INCONCLUSIVE rather than CONFIRMED for those claims
