---
feature: Structured QSOS run logging
ticket: QSO-016
status: @done
architecture_updated: false
adrs: []
---

@done
Feature: Structured QSOS run logging
  As a QSOS developer
  I want every QSOS run to emit typed events to a persistent JSONL log file
  So that runs are auditable after the session ends, terminal output stays lean,
  and log data can be analysed, rendered, or fed back to agents in the future

  Scenario: Terminal output is lean — checkpoints only
    Given a QSOS run is in progress
    When any skill starts, completes, or issues a verdict
    Then the terminal shows a single checkpoint line per event
    And it does not emit prose descriptions, reasoning, or verbose context
    And the developer can follow the run's progress without scrolling

  Scenario: Typed events are appended to a JSONL log file during the run
    Given a QSOS run has started
    When any loggable event occurs (skill start, verdict, gate, deviation, etc.)
    Then one JSON object is appended as a new line to the run's log file
    And the file is written incrementally — not only at run end
    And each line is a valid, self-contained JSON object

  Scenario: Every event shares a common envelope
    Given any JSONL event is written
    Then it contains: run_id, timestamp (ISO 8601), ticket, skill, type, and a data object
    And run_id is stable for the entire QSOS invocation
    And type is one of the defined event types in the taxonomy

  Scenario: High-signal events carry structured payloads
    Given a high-signal event occurs (deviation_flagged, verdict_issued, test_failure, etc.)
    When it is written to the log
    Then its data object contains the fields defined for that event type
    And the payload is concise — no prose treatises — but captures the key facts

  Scenario: Log file is stored at a predictable path
    Given a QSOS run is associated with a ticket
    Then the log is written to work/<ticket-slug>/logs/qsos-run-<run_id>.jsonl
    And if no ticket is active, the log is written to .qsos/logs/qsos-run-<run_id>.jsonl

  Scenario: Run log can be rendered to human-readable Markdown
    Given a completed QSOS run log exists at its JSONL path
    When a developer or tool reads the log
    Then it can be parsed event-by-event without a schema file
    And the event types and payloads are self-describing enough to render a timeline
    And signal-level events (deviation, verdict, failure) are visually distinct in any rendering

  Scenario: Process health events capture churn and rabbit holes
    Given a QSOS run revisits the same skill multiple times
    Or a plan is revised more than once
    Or the goal was already met before the chain completed
    Then a process health event (loop_detected, churn_detected, rabbit_hole_risk) is emitted
    And it includes enough context to diagnose the pattern retrospectively

  Scenario: Run interruptions are recorded
    Given a QSOS run stops unexpectedly
    Whether due to user cancellation, API error, or context limit
    Then a run_interrupted event is appended as the last line
    And it records the source of interruption and the last skill that was active
