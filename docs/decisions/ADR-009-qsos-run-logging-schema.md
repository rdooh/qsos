# ADR-009: QSOS run logging — JSONL format, event taxonomy, and log path convention

Date: 2026-07-29
Status: Accepted
Decision makers: Rob Dooh

## Context

QSOS runs produce verbose conversational prose in the terminal. Output scrolls off screen,
is not persisted, and burns tokens that the developer may never read. There is no audit
trail of what happened during a run, what decisions were made, or what health issues
occurred (churn, rabbit holes, interruptions). Future analysis — "why did we spend so many
tokens on TIX-012?" — is impossible without a structured record.

Three decisions need to be made:

1. What file format should run logs use?
2. What is the event taxonomy — types, categories, payloads?
3. Where are log files stored?

## Decision 1 — JSONL format

Run logs use JSONL (JSON Lines): one JSON object per line, newline-delimited, append-only.

**Rationale:**
- Append-only matches the streaming nature of a run — events are written as they occur,
  not buffered until the end. If a run is interrupted, partial logs are still readable.
- Each line is a valid, self-contained JSON object — no partial-read risk.
- No schema file required to parse — a reader can process line-by-line without loading
  the whole file.
- Trivially readable by AI agents, standard CLI tools (jq, grep), and custom renderers.
- Same format used by Claude session transcripts — familiar and proven.

## Decision 2 — Event taxonomy

Every event shares a common envelope:

```json
{
  "run_id": "<uuid>",
  "timestamp": "<ISO 8601>",
  "ticket": "<TIX-NNN or null>",
  "skill": "<active skill name or null>",
  "type": "<event_type>",
  "data": { ... }
}
```

Event types by category:

### Run lifecycle
| type | data fields |
|---|---|
| `run_started` | chain_depth, entry_point, invocation |
| `run_completed` | outcome, skills_run, duration_ms |
| `run_interrupted` | source (user/api_error/context_limit/unknown), last_skill |
| `chain_depth_decided` | ticket_type, depth, rationale |
| `chain_depth_overridden` | from, to, reason |

### Skill lifecycle
| type | data fields |
|---|---|
| `skill_started` | skill |
| `skill_completed` | skill, outcome |
| `skill_skipped` | skill, reason |
| `skill_blocked` | skill, reason, resolution_required |

### Subagent lifecycle
| type | data fields |
|---|---|
| `subagent_spawned` | label, model, scope_files, purpose |
| `subagent_completed` | label, outcome, deviations |
| `subagent_blocked` | label, reason |

### Human gates
| type | data fields |
|---|---|
| `gate_reached` | gate_type, options_presented |
| `gate_passed` | gate_type, user_selection |
| `gate_rejected` | gate_type, user_selection, implication |
| `gate_abandoned` | gate_type |

### Plan
| type | data fields |
|---|---|
| `plan_produced` | item_count, items |
| `plan_approved` | — |
| `plan_revised` | revision_count, reason |
| `plan_aborted` | reason |

### Implementation
| type | data fields |
|---|---|
| `item_started` | item_ref, file, action |
| `item_completed` | item_ref, tdd |
| `deviation_flagged` | severity (minor/significant/blocked), planned, actual, reason |
| `deviation_resolved` | how (proceeded/reverted/aborted) |
| `unplanned_change` | file, description |

### Tests and coverage
| type | data fields |
|---|---|
| `test_run` | runner, passed, failed, skipped, coverage_pct, result_link |
| `test_failure` | runner, failed_count, failure_summary, result_link |
| `coverage_gap` | files_uncovered, threshold, actual |

### Verdicts
| type | data fields |
|---|---|
| `verdict_issued` | verdict, evidence_links, weak_spots |
| `verdict_unconfirmed` | reason, what_was_not_checked |
| `verdict_inconclusive` | unchecked_claims, resolution_path |

### Process health
| type | data fields |
|---|---|
| `loop_detected` | skill, visit_count |
| `churn_detected` | indicator, count |
| `rabbit_hole_risk` | description, goal_already_met |
| `context_compressed` | approximate_tokens_lost |

### Insights and knowledge
| type | data fields |
|---|---|
| `insight` | category (process/codebase/architecture), summary, actionable |
| `assumption_flagged` | assumption, impact_if_wrong |
| `gap_discovered` | gap_type (feature/architecture/test/doc), description |
| `adr_referenced` | adr_path |
| `adr_created` | adr_path, decision_summary |

### Artifacts
| type | data fields |
|---|---|
| `file_created` | path |
| `file_modified` | path |
| `evidence_written` | type, path |
| `commit_made` | message, files_changed |
| `deploy_run` | target, healthy, issues_count |

**Signal discipline:** High-signal types (deviation_flagged, verdict_issued, test_failure,
loop_detected, churn_detected, rabbit_hole_risk, run_interrupted, plan_aborted,
skill_blocked) carry concise but complete payloads. Low-signal types (skill_started,
file_created) carry minimal payloads — the type itself is the signal. `insight` is
agent-judgment territory; agents should use it sparingly and write one-line summaries only.

## Decision 3 — Log file path convention

```
work/<ticket-slug>/logs/qsos-run-<run_id>.jsonl    # ticket-anchored run
.qsos/logs/qsos-run-<run_id>.jsonl                  # no active ticket
```

**Rationale:**
- Ticket-anchored logs live alongside evidence and other ticket artefacts in `work/` —
  consistent with workspace-directory-standards (ADR-013 / TIX-013).
- `run_id` in the filename means multiple runs for the same ticket don't overwrite each
  other — every run is independently inspectable.
- `.qsos/logs/` as fallback keeps exploratory runs out of `work/` but still persisted.

## Decision 4 — Terminal output protocol

Terminal output is reduced to one checkpoint line per event for skill transitions,
gate decisions, and verdicts. No prose, no reasoning, no verbose context. The full
detail lives in the JSONL log. Skills emit log events by appending to the run log file
using the `Bash` tool (or equivalent) — they do not rely on conversational output as
the persistence mechanism.

## Consequences

**Positive:**
- Runs are fully auditable after the session ends
- Terminal stays clean — no scrolling firehose
- Tokens are not burned on prose nobody reads
- Log data can be rendered, analysed, or fed back to agents later
- Process health issues (churn, loops, rabbit holes) are detectable retrospectively

**Negative:**
- Every skill must be updated to emit log events — significant cross-cutting change
- Skills must have access to run_id and log file path at invocation time (passed via
  context or a per-run state file)
- JSONL files accumulate over time — a cleanup/archive policy will eventually be needed
- The taxonomy is a first draft; new event types will be added as gaps are discovered

## 6-month reversal test

Changing the JSONL schema after runs have accumulated requires migrating all existing
log files or accepting a mixed-format corpus. Changing the log path convention breaks
any tooling that reads logs from the old path. Both decisions are load-bearing once
runs accumulate — the taxonomy and paths should be treated as a stable API.

## Worked example

A short fictional run (TIX-016, partial chain: qsos → plan → implement) showing what the JSONL log looks like in practice.

```jsonl
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:14:01.012Z","ticket":"TIX-016","skill":null,"type":"run_started","data":{"chain_depth":2,"entry_point":"qsos","invocation":"qsos TIX-016"}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:14:01.388Z","ticket":"TIX-016","skill":"qsos","type":"chain_depth_decided","data":{"ticket_type":"feature","depth":2,"rationale":"Feature ticket with approved spec — plan + implement"}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:14:02.105Z","ticket":"TIX-016","skill":"qsos-plan","type":"skill_started","data":{"skill":"qsos-plan"}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:14:18.772Z","ticket":"TIX-016","skill":"qsos-plan","type":"plan_produced","data":{"item_count":4,"items":["Add subagent taxonomy to ADR-009","Add worked example to ADR-009","Update skill-started handler to emit subagent_spawned","Write tests for subagent event types"]}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:15:04.331Z","ticket":"TIX-016","skill":"qsos-plan","type":"plan_approved","data":{}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:15:04.512Z","ticket":"TIX-016","skill":"qsos-plan","type":"skill_completed","data":{"skill":"qsos-plan","outcome":"approved"}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:15:05.001Z","ticket":"TIX-016","skill":"qsos-implement","type":"skill_started","data":{"skill":"qsos-implement"}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:15:06.204Z","ticket":"TIX-016","skill":"qsos-implement","type":"item_started","data":{"item_ref":"1","file":"docs/decisions/ADR-009-qsos-run-logging-schema.md","action":"edit"}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:15:22.890Z","ticket":"TIX-016","skill":"qsos-implement","type":"deviation_flagged","data":{"severity":"minor","planned":"Add subagent taxonomy after Skill lifecycle section","actual":"Taxonomy section order required insertion before Human gates — structural position differs","reason":"Human gates immediately followed Skill lifecycle with no gap; insertion point adjusted to preserve logical grouping"}}
{"run_id":"a3f1c9e2-8b47-4d2a-bc06-1e7f30d52a84","timestamp":"2026-07-29T09:15:23.110Z","ticket":"TIX-016","skill":"qsos-implement","type":"deviation_resolved","data":{"how":"proceeded"}}
```

Signal-level events (deviation_flagged, verdict_issued, test_failure, etc.) are identifiable by type without parsing data — a renderer can visually distinguish them from informational events.

## Related

- ADR-008 — CTRF evidence format (validate-skill) — complementary evidence format
- TIX-013 — workspace directory standards — log path follows same conventions
