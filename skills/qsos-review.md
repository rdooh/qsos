---
description: Post-implement code quality gate — dispatches the code-reviewer agent against the implementation diff and gates chain progression on findings.
---

# /qsos-review

## Core Principle

Implementation produces code. Code has quality. Quality is not assumed — it is checked. This skill sits between `/qsos-implement` and `/qsos-verify` and dispatches a specialist code reviewer against the actual diff. CRITICAL findings route back to implementation. Everything else proceeds with findings noted.

---

## When this runs

After `/qsos-implement` has completed and emitted its completion block. Before `/qsos-security` (if activated) and `/qsos-verify`.

---

## Logging — skill_started

At the start of execution, before Step 1, emit a `skill_started` log entry:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-review\",\"type\":\"skill_started\",\"data\":{}}" >> "$LOG_PATH"
fi
```

---

## Step 1 — Confirm implementation block

Check that a `/qsos-implement` completion block is present in context:

```
IMPLEMENTATION: all plan items executed
```

If no such block is present, halt:

```
REVIEW: CANNOT RUN
No implementation block found in context.
Run /qsos-implement first, then return here.
```

---

## Step 2 — Dispatch code-reviewer agent

Before dispatching the agent, emit a `subagent_spawned` log entry:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-review\",\"type\":\"subagent_spawned\",\"data\":{\"label\":\"code-reviewer\",\"model\":\"<tier>\",\"scope_files\":[\"<diff scope>\"],\"purpose\":\"post-implement code quality review\"}}" >> "$LOG_PATH"
fi
```

Dispatch the `code-reviewer` agent using the Agent tool:

```
Agent(
  description: "Code review against implementation diff — ~1 diff, no files written, low cost",
  subagent_type: "code-reviewer",
  prompt: "Review the current branch diff against main. Run: DIFF_BASE=$(git merge-base origin/main HEAD) && git diff \"$DIFF_BASE\". Apply your full checklist. Output one JSON finding per line, or NO FINDINGS."
)
```

Wait for the agent to complete before proceeding.

---

## Step 3 — Parse and classify findings

Parse the agent output line by line. Each line is either a JSON finding object or `NO FINDINGS`.

**If output is `NO FINDINGS`:**
→ Skip to Step 5 (CLEAN path)

**For each JSON finding, classify by confidence and emit a log entry:**

For findings with severity CRITICAL or HIGH (confidence >= 5), emit a `gap_discovered` entry:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-review\",\"type\":\"gap_discovered\",\"data\":{\"gap_type\":\"<correctness|maintainability|security|etc>\",\"description\":\"<one-line finding summary>\"}}" >> "$LOG_PATH"
fi
```

For informational findings (LOW or MEDIUM severity, confidence < 5 or severity not CRITICAL/HIGH), emit an `insight` entry:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-review\",\"type\":\"insight\",\"data\":{\"category\":\"codebase\",\"summary\":\"<one-line observation>\",\"actionable\":true}}" >> "$LOG_PATH"
fi
```

**For each JSON finding, classify by confidence:**

| Confidence | Treatment |
|---|---|
| 7–10 | Main findings — shown normally |
| 5–6 | Main findings — shown with caveat: "Medium confidence — verify this is actually an issue" |
| 3–4 | Appendix — shown in a separate low-confidence section, does not block |
| 1–2 | Suppress — do not output |

**Deduplication by fingerprint:**
If two findings share the same `fingerprint` value (`path:line:category`):
- Keep the one with the higher confidence score
- Tag it: `CONFIRMED BY MULTIPLE PASSES`
- Boost confidence by +1, capped at 10
- Note the original scores in the output

---

## Step 4 — Apply the gate

**BLOCKED condition:** Any finding with `severity: CRITICAL` AND `confidence >= 7`.

If BLOCKED:
```
REVIEW: BLOCKED — N critical finding(s)

[For each CRITICAL finding, display:]
[CRITICAL] (confidence: N/10) path:line — summary
  Category: category
  Fix: fix recommendation
  Fingerprint: fingerprint
```

Then:
```
Route: return to /qsos-implement
These findings must be remediated before /qsos-verify can run.
```

**Stop. Do not proceed to /qsos-verify.**

---

## Step 5 — Output and proceed

**CLEAN path (NO FINDINGS):**
```
REVIEW: CLEAN
No findings. Proceeding to next step.
```

**INFORMATIONAL path (findings present, none CRITICAL 7+):**
```
REVIEW: N informational finding(s)

MAIN FINDINGS:
[For each finding with confidence 5+, in order of confidence descending:]
[SEVERITY] (confidence: N/10) path:line — summary
  Category: category
  Fix: fix recommendation
  [If medium confidence:] Note: Medium confidence — verify this is actually an issue

APPENDIX — LOW CONFIDENCE (confidence 3–4, do not block):
[If any:]
[SEVERITY] (confidence: N/10) path:line — summary

Proceeding to next step.
```

After either CLEAN or INFORMATIONAL output, continue the chain:
- If `SECURITY_REVIEW: recommended` is present in the plan → run `/qsos-security`
- Otherwise → run `/qsos-verify`

---

## Logging — skill_completed

After emitting the Step 4 or Step 5 output, emit a `skill_completed` log entry:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-review\",\"type\":\"skill_completed\",\"data\":{\"outcome\":\"<clean|blocked>\",\"finding_count\":<N>}}" >> "$LOG_PATH"
fi
```

Outcome values: `clean` (no CRITICAL findings at confidence 7+), `blocked` (one or more CRITICAL findings at confidence 7+). `finding_count` is the total number of findings emitted (all severities, confidence 3+).

---

## Blocking rule

**You may not proceed to `/qsos-verify` if any CRITICAL finding has confidence 7 or above.** You may not suppress a CRITICAL finding by reclassifying it. You may not bypass this gate — CRITICAL findings require remediation, not acknowledgement.
