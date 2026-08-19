---
description: Implementation planning gate — maps feature scenarios to deliverables and requires human approval before any code is written.
---

# /plan

## Core Principle

A plan that exists only in an agent's head is not a plan — it is an intention. This skill makes implementation intent explicit, reviewable, and approachable. The plan names specific files. It maps each scenario in the feature file to a concrete deliverable. It surfaces risks and architectural constraints before they become problems mid-implementation. And it stops — completely — until a human approves it.

---

## When this runs

After `/qsos-orient` has loaded context and confirmed `READY FOR /plan: yes`. Always before `/qsos-implement`.

---

## Step 1 — Verify readiness

Call `/task read` with the active ticket ID. Confirm:

- Feature file(s) are `@accepted` — if any are `@proposed`, redirect to `/qsos-feature-doc` and stop
- All linked ADRs have status `Accepted` — if any are `Proposed`, redirect to `/qsos-architecture` and stop
- No open blocking dependencies (`depends_on:` tickets are all `done` or absent)
- `architecture_updated` field is populated (not necessarily `true`, but a conscious decision has been made)

If any check fails, state which gate is unmet and which skill to run. Do not produce a plan for work that is not ready.

---

## Step 2 — Map scenarios to deliverables

For each scenario in the linked feature file(s):

- Name the scenario
- Identify the files to create or modify (with paths)
- Describe what change makes that scenario pass

Group by feature file if multiple are linked. Every scenario must be represented — if a scenario has no corresponding deliverable, flag it as a gap in the feature spec.

---

## Step 3 — Surface architectural constraints

For each loaded ADR, note any constraint it places on implementation choices:

- Technology mandates (must use X, must not use Y)
- Pattern mandates (must follow pattern Z)
- Boundary constraints (component A must not depend on component B)

If no constraints apply, state "none" — do not omit this section.

---

## Step 4 — Identify risks

Flag anything that could cause the plan to fail silently or require revisiting mid-implementation:

- Cross-file impacts not visible in the feature file (e.g. shared types, exported interfaces)
- Scenarios that seem ambiguous — two reasonable implementations would produce different behavior
- Missing test infrastructure (no test runner configured, no test directory)
- Implementation that may warrant a new ADR (a choice being made that would fail the 6-month reversal test)

If no risks, state "none."

---

## Step 5 — Produce the plan and await approval

```
PLAN

TICKET: <id> — <title>
FEATURE: <path> [@accepted]

SCENARIO MAP:
  Scenario: <name>
    1. <file path> — <what changes and why>
    2. <file path> — <what changes and why>

  Scenario: <name>
    3. <file path> — <what changes and why>

ITEMS: (flat numbered list derived from SCENARIO MAP above — used by /qsos-implement-fan for decomposition)
  [1] <file/path> — <action>
  [2] <file/path> — <action>
  [3] <file/path> — <action>

RISKS:
  - <risk or none>

ARCHITECTURAL CONSTRAINTS:
  - <ADR reference — constraint or none>

---
```

After presenting the plan, log the `plan_produced` event — counting the items in the flat ITEMS list and capturing each as `"<file/path> — <action>"`:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<from current-run.json>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-plan","type":"plan_produced","data":{"item_count":<N>,"items":["<item-ref: file — action>"]}}' >> "$LOG_PATH"
fi
```

Then use `AskUserQuestion` with a single question:

- Question: "Approve this plan?"
- Options:
  - "Approved — proceed to implementation"
  - "Changes needed — I'll describe them"
  - "Abort — do not implement"

**On approval — persist the plan (Open TIX v1.1 §6):**

Write the approved plan to:

```text
work/plans/YYYY-MM-DD-action-<slug>.md
```

Use today's date and a slug derived from the ticket or feature area. Frontmatter MUST include `type: action`, `status: active`, `tickets: [<id>]`, and linked `adrs:` when applicable. Body follows the action-plan template in [Open TIX SPEC §6](../../catalyst/opentix/SPEC.md).

Add `plan: work/plans/...` to the ticket frontmatter. Update `work/tix-manifest.json` `active_plan` if this is the current execution plan.

**If the user selects "Approved — proceed to implementation":**

Log the `plan_approved` event, then hand off to `/qsos-implement`:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-plan","type":"plan_approved","data":{}}' >> "$LOG_PATH"
fi
```

**If the user selects "Changes needed — I'll describe them":**

Ask the user what they want changed. Revise the plan accordingly (increment a `revision_count` starting at 1 for the first revision). Log the `plan_revised` event, then re-present the updated plan and re-ask for approval:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-plan","type":"plan_revised","data":{"revision_count":<N>,"reason":"<what user described>"}}' >> "$LOG_PATH"
fi
```

**If the user selects "Abort — do not implement":**

Log the `plan_aborted` event and stop:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  echo '{"run_id":"<id>","timestamp":"<ISO>","ticket":"<id>","skill":"qsos-plan","type":"plan_aborted","data":{"reason":"user selected abort"}}' >> "$LOG_PATH"
fi
```

Do not write any implementation code until the user selects "Approved".

---

## Blocking rule

**You may not write a single line of implementation code before the user has approved this plan.** If the feature file is not `@accepted`, there is nothing to plan — redirect to `/qsos-feature-doc`. If a scenario has no deliverable mapped to it, the plan is incomplete — fix it before presenting. Presenting a plan and then immediately implementing it without waiting for a response is not approval.
