# Triggers Schema Specification

**Document type:** Specification  
**Status:** Draft  
**Ticket:** QSO-009  
**ADR:** [ADR-004 — Hub-and-Spoke Watcher Daemon Architecture](../decisions/ADR-004-hub-and-spoke-watcher.md)

---

## Overview

The Hub watcher reads its rules from a `triggers.toml` file located at the workspace root.
Each rule maps a filesystem path pattern to a command that will be executed when a matching
event occurs. The Hub does not implement any automation logic itself — it is a scheduler.
All domain logic lives in the Spoke processes it spawns.

---

## Rule structure

Each rule is a TOML table entry under `[[rules]]`.

```toml
[[rules]]
name        = "Feature Spec Audit"        # required — human-readable identifier, unique per file
path        = "docs/features/"            # required — path prefix or glob to watch
event       = "modify"                    # required — see Event types below
command     = "qsos lint"                 # required — shell command executed as a Spoke subprocess
debounce_ms = 300                         # optional — default: 300ms
concurrency = 1                           # optional — default: 1; max concurrent instances of this rule
timeout_ms  = 10000                       # optional — default: 10000ms (10s); 0 = no timeout
overflow    = "discard"                   # optional — default: "discard"; see Overflow policy below
```

---

## Field reference

### `name` (string, required)
A unique, human-readable label for this rule. Used in log output and error messages. Must be
unique within a `triggers.toml` file. No spaces enforced by the schema, but conventional
kebab-case is preferred for log legibility.

### `path` (string, required)
The filesystem path to watch. Interpreted as a prefix — any file whose absolute path begins
with this value (relative to the workspace root) will match. Trailing slashes are normalised.
Glob patterns are not currently supported; prefix matching only.

### `event` (string, required)
The filesystem event type that triggers this rule. Supported values:

| Value    | Fires when                              |
|----------|-----------------------------------------|
| `modify` | An existing file's content is changed   |
| `create` | A new file is created at the path       |
| `delete` | A file at the path is removed           |
| `any`    | Any of the above                        |

### `command` (string, required)
The shell command to execute as a Spoke subprocess. The Hub appends the absolute path of the
changed file as the final argument. The command must be executable from the workspace root.

Example: if `command = "node utilities/auditor.js"` and the changed file is
`docs/features/login.feature`, the Hub invokes:
```
node utilities/auditor.js /abs/path/to/docs/features/login.feature
```

### `debounce_ms` (integer, optional, default: 300)
The debounce window in milliseconds. If multiple events matching this rule fire within the
window, only one command invocation is scheduled — for the final event after the window
elapses. Minimum: 0 (no debounce). Recommended: 300–500ms for editor-triggered writes.

### `concurrency` (integer, optional, default: 1)
The maximum number of concurrent instances of this rule's command that may run simultaneously.
Most rules should use the default of 1. Rules with concurrency > 1 receive no ordering
guarantees between instances.

### `timeout_ms` (integer, optional, default: 10000)
The maximum time in milliseconds a Spoke process may run before the Hub forcefully terminates
it with SIGKILL. A value of `0` disables the timeout entirely. When a process is killed, the
Hub logs a `task-timeout` error event (see [Spoke Contract](spoke-contract.md)).

### `overflow` (string, optional, default: `"discard"`)
Policy applied when a new event arrives for a rule that is already at its concurrency limit:

| Value     | Behaviour                                                      |
|-----------|----------------------------------------------------------------|
| `discard` | The new invocation request is dropped. A log entry is emitted. |
| `queue`   | The new invocation is queued and runs when a slot opens.       |

Queue depth is not capped in this specification. Implementors should treat this as a known risk
for long-running Spoke processes and add a cap if needed.

---

## Minimal example

```toml
[[rules]]
name    = "DSL Compiler"
path    = "docs/architecture/architecture.dsl"
event   = "modify"
command = "python utilities/dsl_compiler.py"

[[rules]]
name        = "Ticket Manifest Compiler"
path        = "work/"
event       = "any"
command     = "node utilities/ticket_compiler.js"
debounce_ms = 500
concurrency = 1
timeout_ms  = 5000
overflow    = "discard"
```

---

## Validation rules

The Hub validates `triggers.toml` at startup. Violations cause the Hub to exit with a
non-zero exit code and a structured error on stderr. The following conditions are invalid:

- `name` is missing or empty
- `path` is missing or empty
- `event` is not one of the supported values
- `command` is missing or empty
- Two rules share the same `name` value
- `debounce_ms` is negative
- `concurrency` is less than 1
- `timeout_ms` is negative
- `overflow` is not `"discard"` or `"queue"`

---

## Known gaps

- **Glob pattern support** is not in scope for QSO-009. Path prefix matching covers current
  use cases. Glob support may be added in a future ticket when needed.
- **IPC between Hub and IDE Extension** is not specified here. The Daemon Supervisor in the
  IDE Extension monitors the Hub process; the protocol for streaming execution updates is
  deferred to a future ADR.
