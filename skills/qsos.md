---
description: QSOS orchestrator — read project state, find the entry point, and drive the workflow chain forward until a human gate is reached.
---

# /qsos

## Core Principle

You do not need to know where you are in the chain. QSOS reads the project, determines the entry point, and drives forward — pausing only when a decision genuinely belongs to a human. Tell it what you want to accomplish, or nothing at all, and it will figure out what to do next.

---

## How to invoke

Any of these work:

- "use QSOS to define this feature" → enters at `/qsos-brainstorm`
- "use QSOS to select a ticket" → finds the most ready unstarted ticket
- "use QSOS to verify yesterday's work" → finds recently implemented tickets, runs `/qsos-verify`
- "use QSOS" with no direction → scans state and picks up where work left off
- "use QSOS on QSO-007" → anchors to that ticket and determines its next stage

---

## Step 0 — Run initialisation

Before reading project state, initialise the run so all downstream events have a stable identity.

### 0.1 — Generate run_id

Derive a timestamp slug in the format `YYYYMMDD-HHMMSS` from the current date context (e.g. `20260729-143022`). If exact time is unavailable, append a 6-digit random hex suffix to the date (e.g. `20260729-a3f9c1`). The run_id must be stable for the session — do not regenerate it.

### 0.2 — Resolve log path

If an active ticket is identified (from the invocation or from scanning `work/`):
```
work/<ticket-slug>/logs/qsos-run-<run_id>.jsonl
```
where `ticket-slug` is the full `work/` directory name (e.g. `work/QSO-016-qsos-run-logging/logs/qsos-run-20260729-143022.jsonl`).

Fallback (no ticket context):
```
.qsos/logs/qsos-run-<run_id>.jsonl
```

### 0.3 — Create directory and write current-run.json

```bash
mkdir -p "$(dirname "<log_path>")"
```

Write `.qsos/current-run.json`:
```json
{"run_id": "<id>", "log_path": "<path>", "ticket": "<QSO-NNN or null>", "started_at": "<ISO timestamp>"}
```

### 0.4 — Append run_started event

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<QSO-NNN or null>","skill":"qsos","type":"run_started","data":{"chain_depth":"<full|lite>","entry_point":"<skill>","invocation":"<what user said>"}}' >> "$LOG_PATH"
fi
```

### Canonical Bash append pattern

All skills in the chain that write log events MUST use this exact pattern:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '<json-line>' >> "$LOG_PATH"
fi
```

The `2>/dev/null` suppresses errors when `.qsos/current-run.json` is absent. The `if [ -n "$LOG_PATH" ]` guard ensures graceful skip when a skill is invoked standalone (without `/qsos`). Never omit these guards.

---

## Step 1 — Read project state

Load the following:

1. All tickets in `work/` (or active Jira project) — note each status
2. All feature files in `docs/features/` — note lifecycle tags
3. Any `/qsos-plan` output present in the current context
4. Any `/qsos-verify` verdict present in the current context

Determine the **active ticket**: the one that is `in-progress`, or if none, the most advanced `ready` ticket. If multiple `ready` tickets exist, pick the one with the highest priority or lowest number.

If the user named a specific ticket, feature, or goal in their invocation, anchor to that instead.

---

## Step 2 — Classify and propose chain depth

Before determining the entry point, classify the work by type and propose a proportionate chain.

**Classify the ticket type:**

| Type | Criteria | Chain depth |
|---|---|---|
| `feat` | New observable behaviour, new files, new interfaces, external impact | Full chain |
| `fix` | Corrects broken behaviour covered by an existing feature file | Full chain (may skip brainstorm) |
| `chore` | Config change, doc update, tooling/meta work, no new observable behaviour | Lite chain |

**Chain depths:**

- **Full:** brainstorm → feature-doc → architecture → orient → plan → implement → coverage-check → review → verify → validate (optional) → doc-sync
- **Lite:** plan → implement → close (skips coverage-check, review, verify, validate, doc-sync)

**Present the assessment and require confirmation before proceeding.**

State the assessment as text:

```
QSOS ASSESSMENT

TICKET: <id> — <title>
TYPE: <feat | fix | chore> — <one-line rationale>
PROPOSED CHAIN: <full | lite> — <skills that will run, in order>
SKIPPING: <skills being omitted, or "nothing">
REASON: <why this chain depth fits the work>
```

Then use `AskUserQuestion` with a single question:

- Question: "Proceed with this chain?"
- Options:
  - "Proceed — <proposed depth>" (e.g. "Proceed — lite chain")
  - "Override to full chain"
  - "Override to lite chain"
  - "Cancel"

**Do not run a single skill until the user selects an option.** This gate is mandatory — not skippable, not inferred from prior messages.

After the user confirms, append the `chain_depth_decided` event using the canonical append pattern:

```json
{"run_id":"<id>","timestamp":"<ISO>","ticket":"<QSO-NNN or null>","skill":"qsos","type":"chain_depth_decided","data":{"ticket_type":"<feat|fix|chore>","depth":"<full|lite>","rationale":"<one line>"}}
```

---

## Step 3 — Determine entry point

Map ticket and feature state to the next skill in the confirmed chain:

| Ticket status | Feature tag | Next skill |
|---|---|---|
| todo, no feature file | — | `/qsos-brainstorm` (full only) |
| todo, feature `@proposed` | @proposed | `/qsos-feature-doc new` |
| ready, feature `@accepted`, no plan | @accepted | `/qsos-orient` → `/qsos-plan` |
| ready, feature `@accepted`, plan approved | @accepted | `/qsos-implement` |
| in-progress, feature `@in-progress` | @in-progress | `/qsos-coverage-check` → `/qsos-verify` (full only) |
| in-progress, verified CONFIRMED | @in-progress | `/qsos-validate` if requested (optional), otherwise `/qsos-doc-sync` (full only) |
| in-progress, validated | @in-progress | `/qsos-doc-sync` (full only) |
| done | @done | nothing — pick next ticket |
| bug reported | any | `/qsos-bug` |

State the entry point in one line before proceeding.

---

## Step 4 — Drive the chain

Execute the entry point skill. After it completes:

- If verdict is GO / CONFIRMED / CLEAN → move to the next skill in the confirmed chain automatically
- If verdict is BLOCKED / NEEDS CLARIFICATION / UNCONFIRMED → stop, surface the issue, wait for direction
- If the skill requires human input (plan approval, scoping questions) → wait for the response, then continue

After `/qsos-verify` returns CONFIRMED: if the developer wants human validation before closing the ticket, run `/qsos-validate` before proceeding to `/qsos-doc-sync`. `/qsos-validate` is optional — do not run it automatically; only run it when explicitly requested by the user.

For **lite chains**: stop after `/qsos-implement` completes. Update ticket status to `done` and feature tag to `@done`. Do not run coverage-check, review, verify, or doc-sync unless the user explicitly requests them.

Do not run skills outside the confirmed chain without asking first.

---

## Step 5 — Report progress at each stage

**Terminal checkpoint format (emit ONLY this — no prose, no reasoning, no context summaries):**

```
► /qsos-<skill> — starting
✓ /qsos-<skill> — <outcome in 3-5 words>
✗ /qsos-<skill> — BLOCKED: <one-line reason>
```

**Between skill transitions, emit only the checkpoint line. Do not emit reasoning, context summaries, plan recaps, or verbose progress blocks in the terminal.**

Examples of correct output:
```
► /qsos-orient — starting
✓ /qsos-orient — context loaded, no gaps
► /qsos-plan — starting
✓ /qsos-plan — presented, awaiting approval
✓ /qsos-implement — all plan items executed
✓ /qsos-verify — CONFIRMED
✗ /qsos-coverage-check — BLOCKED: no test files found
```

At each skill transition, append `skill_started` and `skill_completed` log events using the canonical append pattern:

```json
{"run_id":"<id>","timestamp":"<ISO>","ticket":"<QSO-NNN or null>","skill":"<skill>","type":"skill_started","data":{}}
{"run_id":"<id>","timestamp":"<ISO>","ticket":"<QSO-NNN or null>","skill":"<skill>","type":"skill_completed","data":{"outcome":"<go|blocked|confirmed|etc>"}}
```

If the chain stops due to a BLOCKED verdict, API error, or user cancellation, append a `run_interrupted` event:

```json
{"run_id":"<id>","timestamp":"<ISO>","ticket":"<QSO-NNN or null>","skill":"qsos","type":"run_interrupted","data":{"source":"<user|api_error|context_limit|unknown>","last_skill":"<skill>"}}
```

---

## Step 6 — Final summary

When the chain reaches a natural stopping point (ticket closed, blocked, or waiting for human input), append the `run_completed` event using the canonical append pattern:

```json
{"run_id":"<id>","timestamp":"<ISO>","ticket":"<QSO-NNN or null>","skill":"qsos","type":"run_completed","data":{"outcome":"<done|blocked|abandoned>","skills_run":["<list>"]}}
```

Then produce:

```
QSOS SUMMARY

TICKET: <id> — <title>
STARTED AT: <skill>
STOPPED AT: <skill> — <reason: done | blocked | needs input>
ARTIFACTS PRODUCED:
  - <list>
NEXT ACTION: <what the user needs to do, or "none — work complete">
```

---

## Blocking rule

**You may not skip human gates.** The chain depth confirmation in Step 2 is mandatory — never infer it from context or prior messages. `/qsos-plan` always waits for approval. `BLOCKED` verdicts always stop. Scoping questions in `/qsos-brainstorm` always wait for answers. Everything else runs unattended within the confirmed chain. The chain is only as autonomous as its gates allow — do not bypass them to appear faster.
