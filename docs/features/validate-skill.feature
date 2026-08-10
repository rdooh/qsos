---
feature: Human-in-the-loop validation skill
ticket: TIX-015
status: @done
architecture_updated: false
adrs:
  - docs/decisions/ADR-008-validation-skill-and-evidence-format.md
---

@done
Feature: Human-in-the-loop validation skill
  As a QSOS developer
  I want a structured human validation pass after automated verification
  So that I can confirm working behaviour with my own eyes before closing a ticket,
  and the validation record is saved as evidence

  Scenario: Agent derives checklist from verification weak spots
    Given qsos-verify has run and produced a CONFIRMED verdict
    When qsos-validate runs
    Then it reads the verify evidence and identifies weak spots
    And it derives a checklist targeting those spots — not a generic walkthrough
    And it presents the full checklist to the developer before any step runs
    And the developer can see how many steps there are before starting

  Scenario: Agent executes automated steps and provides direct links
    Given a checklist step is automated (e.g. run tests, check build output)
    When that step runs
    Then the agent executes it and shows the result inline
    And it provides a direct file:// path or http:// URL the developer can open themselves
    And it does not claim the step passed without showing the evidence

  Scenario: Developer confirms or flags each human step
    Given a checklist step requires human action (e.g. navigate to a page, click a button)
    When the agent presents that step
    Then it uses AskUserQuestion with options: Confirmed, Failed, Partial — adding note
    And it waits for the developer's response before advancing to the next step
    And it presents one step at a time — not all steps at once

  Scenario: Validation record saved as CTRF JSON
    Given all checklist steps have been completed
    When qsos-validate finishes
    Then it writes a CTRF-compliant JSON file to work/<ticket-slug>/evidence/validation-ctrf.json
    And the file records each step's name, type (automated/manual), status, and any notes
    And the overall summary counts passed, failed, and partial steps

  Scenario: Validation fails — ticket does not proceed to doc-sync
    Given one or more checklist steps returned Failed
    When qsos-validate produces its final verdict
    Then the verdict is VALIDATION FAILED
    And it lists the failed steps
    And qsos-doc-sync does not run until the failures are resolved or explicitly deferred

  Scenario: qsos-validate invoked standalone without prior verify context
    Given no qsos-verify verdict is present in context
    When a developer invokes qsos-validate directly
    Then it asks the developer to describe what was built and what evidence exists
    And it derives the checklist from that description
    And it proceeds with the same step-by-step flow
