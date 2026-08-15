---
feature: Hub-and-Spoke Filesystem Watcher Daemon
ticket: QSO-009
status: @done
architecture_updated: true
---

# Hub-and-Spoke Filesystem Watcher Daemon

## Background

The watcher daemon is a lightweight, always-on process that monitors filesystem paths across
one or more QSOS projects. When a file event matches a registered rule, the daemon schedules
the corresponding utility command asynchronously. Rules specify the path, event type, command,
debounce window, concurrency limit, and timeout. The daemon never executes commands synchronously
in the file-event callback — all execution is handed off to a background queue.

---

## Feature: Event matching and utility dispatch

**Scenario: Matching a file event to a registered rule**
  Given the watcher daemon is running with at least one rule configured
  And a rule is registered for path `docs/features/` on event `modify`
  When a file under `docs/features/` is saved
  Then the daemon detects the `modify` event
  And it schedules the rule's configured command for background execution

**Scenario: Non-matching event is silently ignored**
  Given the watcher daemon is running
  And a rule is registered for path `docs/features/` on event `modify`
  When a file outside `docs/features/` is saved
  Then the daemon does not invoke any command

---

## Feature: Debouncing

**Scenario: Rapid saves trigger only one execution**
  Given the watcher daemon is running
  And a rule with a 300ms debounce window is registered for a file
  When that file is saved twice within 100 milliseconds
  Then the daemon debounces both events
  And invokes the rule's command exactly once, after the debounce window elapses

---

## Feature: Concurrency control

**Scenario: Second invocation is held while first is running**
  Given the watcher daemon is running
  And a rule has a concurrency limit of 1
  And that rule's command is currently executing
  When a new file event triggers the same rule
  Then the daemon does not spawn a second concurrent instance
  And it queues or discards the new request per its configured overflow policy

---

## Feature: Subprocess timeout enforcement

**Scenario: Hung subprocess is forcefully terminated**
  Given the watcher daemon is running
  And a rule has a configured execution timeout
  When the rule's command is invoked and runs past the timeout without exiting
  Then the daemon forcefully terminates the subprocess
  And logs a `task-timeout` error event with the rule name and elapsed time
