---
description: Human-in-the-loop validation gate — derives a targeted checklist from verify weak spots, walks the developer through each step one at a time, and saves results as CTRF evidence.
---

# /qsos-validate

## Core Principle

Automated verification confirms tests pass. Validation confirms the thing actually works — with human eyes on a running system. This skill converts verify evidence into a targeted checklist, walks through each step one at a time, and saves the result as a CTRF JSON evidence record.

Chain position: **verify → validate (optional) → doc-sync**

---

## When this runs

- After `/qsos-verify` has returned CONFIRMED, when the developer judges human eyes are warranted (UI, interactive flows, visual output, Storybook components)
- OR standalone, invoked directly without any prior verify context

---

## Logging — skill_started

Emit before any other action:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<from current-run.json>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-validate","type":"skill_started","data":{}}' >> "$LOG_PATH"
fi
```

---

## Step 0 — Determine mode

Check whether a `qsos-verify` CONFIRMED verdict is present in context.

**Mode A — Post-verify (prior context exists):** proceed to Step 1A.

**Mode B — Standalone (no verify context):** proceed to Step 1B.

---

## Step 1A — Derive checklist from verify evidence (post-verify mode)

Read the verify evidence from `work/<ticket-slug>/evidence/`. Look for the verifier's artifact and any evidence notes.

Adopt a sceptic's mindset: what would a sceptic challenge about this verdict? Identify weak spots — areas where:
- The verifier relied on test output rather than observable behaviour
- UI or interactive state was not exercised
- Build output was assumed correct but not viewed
- Integration points were not exercised end-to-end

From those weak spots, derive a targeted checklist. Each item must be:
- Specific to this ticket (not a generic walkthrough)
- Either automated (agent can execute it) or manual (human must act)

Do not list more than 8 steps unless the ticket genuinely warrants it.

Proceed to Step 2.

---

## Step 1B — Derive checklist from developer description (standalone mode)

No prior verify context is present. Ask:

```
No verify context found in this session.

To build a targeted validation checklist, I need to understand what was built.

Please describe:
1. What was implemented (one sentence)
2. What evidence exists (test results, screenshots, build output, running app URL)
3. Which parts you're most uncertain about
```

Wait for the developer's response. Derive the checklist from that description using the same criteria as Step 1A (targeted, specific, no more than 8 steps unless warranted).

Proceed to Step 2.

---

## Step 2 — Present full checklist overview before any step runs

Before executing or presenting any individual step, show the complete checklist to the developer so they can prepare:

```
VALIDATION CHECKLIST — <N> steps

  [A] = automated (agent runs it)   [H] = human action required

  1. [A/H] <step name>
  2. [A/H] <step name>
  ...
  N. [A/H] <step name>

Starting Step 1 of <N> now.
```

State the total count. Do not begin the first step until after displaying this overview.

---

## Step 3 — Execute checklist steps

Work through the checklist one step at a time. Apply the correct protocol for each step type.

---

### Automated step protocol

For steps the agent can execute (run tests, read a file, check build output, fetch a URL):

1. Execute the step.
2. Show the result inline — do not summarise, show the actual output.
3. Provide a direct `file://` path or `http://` URL the developer can open themselves. **Never claim a step passed without a concrete evidence link.**
4. Record the result in the running evidence list (status: passed / failed, type: automated, link: the URL).
5. Emit item_completed after the step result is known:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-validate","type":"item_completed","data":{"item_ref":"Step <N>","tdd":"automated","status":"passed|failed","link":"<file:// or http:// evidence url>"}}' >> "$LOG_PATH"
fi
```

6. Advance to the next step automatically.

Example output format:
```
Step 2 of 5 — [Automated] Unit test results

$ npm test
✓ 14 tests passed, 0 failed
Results: file:///Users/dev/project/test-results/unit.json

PASSED — evidence: file:///Users/dev/project/test-results/unit.json
```

---

### Human step protocol

For steps requiring human action (navigate to a page, click a button, visual check, screen review):

Show the step using `AskUserQuestion` with:
- A concise prompt stating exactly what to do or look at
- The progress indicator "Step N of M" in the prompt
- Exactly these three options (verbatim):

  - "Confirmed — this step passed"
  - "Failed — this step did not pass"
  - "Partial — adding a note"

Wait for the developer's response before advancing. Do not batch multiple human steps.

**"Partial — adding a note"** → follow up with a free-text question: "Add your note for this step." Record the note in the evidence. Status maps to `other` in CTRF.

After the human response is received (including any note), emit item_completed:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-validate","type":"item_completed","data":{"item_ref":"Step <N>","tdd":"manual","status":"<passed|failed|other>","note":"<human note if partial>"}}' >> "$LOG_PATH"
fi
```

Example AskUserQuestion invocation:
```
Question: "Step 3 of 5 — Navigate to /reports and confirm the chart renders without errors."
Options:
  - "Confirmed — this step passed"
  - "Failed — this step did not pass"
  - "Partial — adding a note"
```

---

## Step 4 — Write CTRF evidence record

After all steps complete, write the evidence file to:

```
work/<ticket-slug>/evidence/validation-ctrf.json
```

Use this exact schema (ADR-008):

```json
{
  "results": {
    "tool": { "name": "qsos-validate" },
    "summary": {
      "tests": N,
      "passed": N,
      "failed": N,
      "skipped": 0,
      "pending": 0,
      "other": N
    },
    "tests": [
      {
        "name": "<step name>",
        "status": "passed | failed | other",
        "type": "automated | manual",
        "message": "<evidence output or human note>",
        "extra": {
          "link": "<file:// or http:// url>",
          "note": "<optional freetext>"
        }
      }
    ]
  }
}
```

Status mapping:
- "Confirmed — this step passed" → `"passed"`
- "Failed — this step did not pass" → `"failed"`
- "Partial — adding a note" → `"other"`
- Automated step succeeded → `"passed"`
- Automated step failed → `"failed"`

Every automated step must have a `link` value. Human steps that produced a note must have the note in `extra.note`.

---

After writing the CTRF file, emit evidence_written:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-validate","type":"evidence_written","data":{"type":"ctrf","path":"work/<ticket-slug>/evidence/validation-ctrf.json"}}' >> "$LOG_PATH"
fi
```

---

## Step 5 — Issue final verdict

### VALIDATION PASSED

If all steps returned passed or other (no failures):

Emit:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-validate","type":"verdict_issued","data":{"verdict":"VALIDATION PASSED","failed_steps":[]}}' >> "$LOG_PATH"
fi
```

```
VALIDATE: PASSED
Evidence: work/<ticket-slug>/evidence/validation-ctrf.json
Summary: <N> passed, <N> partial, 0 failed

Proceed to /qsos-doc-sync.
```

---

### VALIDATION FAILED

If any step returned failed:

Emit:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-validate","type":"verdict_issued","data":{"verdict":"VALIDATION FAILED","failed_steps":["<list>"]}}' >> "$LOG_PATH"
fi
```

```
VALIDATE: FAILED
Evidence: work/<ticket-slug>/evidence/validation-ctrf.json

Failed steps:
  - Step <N>: <step name> — <reason or note>
  - Step <N>: <step name> — <reason or note>

/qsos-doc-sync must not run until the above failures are resolved or explicitly deferred by the developer.
```

Stop. Surface to the developer. Wait for direction. Do not proceed to `/qsos-doc-sync`.

If the developer explicitly defers: note `VALIDATION FAILURES DEFERRED` with the step names in the evidence record, then allow chain continuation.

---

## Blocking rule

**You may not proceed to `/qsos-doc-sync` if any step returned Failed**, unless the developer has explicitly stated they are deferring the failures. Document the deferral decision in `extra.note` for each deferred failed step.
