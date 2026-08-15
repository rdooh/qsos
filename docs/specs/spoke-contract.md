# Spoke Contract Specification

**Document type:** Specification  
**Status:** Draft  
**Ticket:** QSO-009  
**ADR:** [ADR-004 — Hub-and-Spoke Watcher Daemon Architecture](../decisions/ADR-004-hub-and-spoke-watcher.md)

---

## Overview

A Spoke is any subprocess spawned by the Hub watcher in response to a matching filesystem
event. The Hub is output-agnostic — it does not parse Spoke output for domain logic. However,
it reads exit codes and emits structured log events based on process lifecycle. This document
defines the contract that every Spoke process must honour.

---

## Invocation

The Hub spawns each Spoke as a shell subprocess. The invocation is:

```
<command> <absolute-path-to-changed-file>
```

- `command` — the exact string from the rule's `command` field in `triggers.toml`
- `absolute-path-to-changed-file` — appended by the Hub as the final argument

The Spoke inherits the Hub's working directory (the workspace root) and environment variables.
No additional environment variables are injected by the Hub in the current implementation.

---

## Exit codes

The Hub interprets Spoke exit codes as follows:

| Exit code | Meaning          | Hub action                                      |
|-----------|------------------|-------------------------------------------------|
| `0`       | Success          | Logs `task-complete` event                      |
| Non-zero  | Failure          | Logs `task-error` event with the exit code      |
| — (killed)| Timeout exceeded | Logs `task-timeout` event (Hub issued SIGKILL)  |

Spokes **must not** use exit code `0` to signal partial failure. A non-zero exit code is the
only reliable signal the Hub can observe for failure routing.

---

## Structured log output (stdout)

Spokes **should** emit a single JSON object to stdout on completion. This output is captured
by the Hub and forwarded to the IDE Extension for display. The format is:

```json
{
  "status": "ok" | "error" | "warning",
  "rule": "<rule name from triggers.toml>",
  "file": "<absolute path of the changed file>",
  "message": "<human-readable summary>",
  "details": [
    "<optional array of strings — e.g. lint findings, line numbers>"
  ],
  "elapsed_ms": 1234
}
```

### Field notes

- `status` — required. Must be `"ok"`, `"error"`, or `"warning"`.
- `rule` — optional. Should match the `name` field in the triggering rule when the Spoke knows it. The Hub does not currently inject the rule name into the Spoke's environment — see Known Gaps. Spokes that can determine their rule name (e.g. from a wrapper script) should include it.
- `file` — required. The absolute path passed in as the first argument.
- `message` — required. One-line human-readable summary, suitable for status bar display.
- `details` — optional. Array of strings for extended output (linter findings, line refs, etc).
- `elapsed_ms` — optional but recommended. Wall-clock time the Spoke spent executing.

### Non-JSON output

If a Spoke does not emit valid JSON, the Hub treats the raw stdout as an unstructured log
string and emits it with `status: "unknown"`. This is permissible for simple utility scripts
but produces degraded IDE Extension rendering.

---

## Error log output (stderr)

Spokes may emit diagnostic information to stderr freely. The Hub captures stderr and includes
it in its internal log. stderr output does not affect the Hub's routing logic — only the exit
code and stdout JSON do.

---

## Hub log event reference

These are the structured events the Hub emits to its own log (distinct from Spoke stdout):

| Event name      | Fired when                                                        |
|-----------------|-------------------------------------------------------------------|
| `task-start`    | Hub spawns the Spoke subprocess                                   |
| `task-complete` | Spoke exits with code 0                                           |
| `task-error`    | Spoke exits with non-zero code                                    |
| `task-timeout`  | Spoke exceeded `timeout_ms`; Hub sent SIGKILL                     |
| `task-discard`  | New invocation was discarded due to concurrency limit + `discard` policy |
| `task-queued`   | New invocation was queued due to concurrency limit + `queue` policy |

Each Hub log event includes: `timestamp`, `rule_name`, `event_type`, `file`, and any
relevant exit code or elapsed time.

---

## Known gaps

- **IPC between Hub and IDE Extension** — the mechanism by which the Hub forwards log events
  to the IDE Extension's Daemon Supervisor is not yet defined. The contract above describes
  the data shape; the transport (e.g. stdout pipe, Unix socket, file polling) is deferred to
  a future ADR.
- **Rule name injection** — the Hub currently does not inject the rule name into the Spoke's
  environment. Spokes must know their own rule name independently (e.g. hardcoded or inferred
  from argv[0]). This is a known weakness; a future ticket may add `QSOS_RULE_NAME` as an
  injected env var.
