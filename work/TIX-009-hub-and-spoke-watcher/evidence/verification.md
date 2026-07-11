# Verification Evidence — TIX-009

**Ticket:** TIX-009 — Design Hub-and-Spoke Watcher Daemon specifications  
**Date:** 2026-07-11  
**Verified by:** /qsos-verify  
**Evidence type:** Documentation completeness audit (spec-only ticket, no executable code)

---

## Claim

All 5 scenarios in `docs/features/watcher_daemon.feature` are addressed by the delivered
specification documents.

## Scenario coverage

| Scenario | Spec coverage | Status |
|---|---|---|
| Matching a file event to a registered rule | `triggers-schema.md` — `path`, `event`, `command` fields; rule matching semantics | ✓ PASS |
| Non-matching event silently ignored | `triggers-schema.md` — prefix-matching semantics imply non-match = no dispatch | ✓ PASS |
| Rapid saves trigger only one execution | `triggers-schema.md` — `debounce_ms` field, 300ms default, debounce window semantics | ✓ PASS |
| Second invocation held while first is running | `triggers-schema.md` — `concurrency` and `overflow` fields; queue/discard policy | ✓ PASS |
| Hung subprocess forcefully terminated | `triggers-schema.md` — `timeout_ms` + SIGKILL behaviour; `spoke-contract.md` — `task-timeout` Hub log event with rule name and elapsed time | ✓ PASS |

## Artifacts delivered

- `docs/specs/triggers-schema.md` — complete `triggers.toml` rule schema (field reference, validation rules, examples)
- `docs/specs/spoke-contract.md` — Spoke subprocess contract (invocation, exit codes, JSON stdout, Hub log events)
- `utilities/README.md` — updated with watcher daemon section and links to both specs
- `docs/features/watcher_daemon.feature` — reformatted to YAML frontmatter standard, lifecycle → @in-progress
- `docs/standards/project-structure.md` — Agent Definitions section added (TIX-010, same session)

## Known gaps documented in specs

- Queue depth not capped (flagged as implementor risk)
- `rule` field in Spoke stdout is optional until Hub injects `QSOS_RULE_NAME`
- IPC between Hub and IDE Extension not yet specified (deferred to future ADR)
- Glob pattern support deferred

## Verdict

CONFIRMED
