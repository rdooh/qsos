---
description: Ensure feature files and ADRs are correct and consistent before any implementation work is committed.
---

# /feature-doc

## Core Principle

Documentation is not a record of what was built — it is a specification of what will be built and why. Implementing a feature without a feature file is not faster; it is undocumented work that will have to be understood from scratch by the next person (or agent) who touches it. The docs exist to make every subsequent decision cheaper.

---

## When this runs

**Before any new feature work begins.** No implementation until this skill returns GO.

**Before any intentional change ships.** If behavior is changing, the feature file must reflect the new behavior before the change is committed.

**During or after bug investigation.** Investigation may precede this skill in bug mode, but the skill must run and return GO before the fix ships.

**Note on Strux:** This skill does not run structural linting or deep cross-file validation. That is the domain of the Strux project. This skill is a lightweight pre-flight — it gets you further down the road. As Strux matures, this skill will defer to it for the audit step.

---

Log `skill_started` before identifying the mode:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-feature-doc','type':'skill_started','data':{}}))" >> "$LOG_PATH"
fi
```

## Step 1 — Identify mode

Determine which mode applies:

**`new`** — A feature that does not yet exist is being added. No feature file exists yet.

**`change`** — An existing feature is being intentionally modified or extended. A feature file already exists and must be updated.

**`bug`** — A bug is being investigated or fixed. Investigation may have already happened. The goal is to determine what the docs should say and update them before the fix ships.

State the mode explicitly at the start of your output.

---

## Step 2 — Locate or establish the docs structure

Every project is expected to have:

```
docs/
  features/     # Gherkin-syntax .feature files, one per feature area
  decisions/    # MADR-formatted ADR files, named ADR-NNN-slug.md
```

If either directory is missing, create it. If the project has no `docs/` folder at all, create the full structure and note it in your output.

---

## Step 3 — Feature file

### For `new` mode

Draft a feature file. Do not write implementation code until this draft has been reviewed and accepted.

The file should follow Gherkin syntax and must begin with the `@proposed` lifecycle tag. `/qsos-brainstorm` may have already created this file with `@proposed` — if so, your job in this skill is to audit it and promote the tag to `@accepted`.

```gherkin
@proposed
Feature: <short name>
  <one or two sentences describing the purpose and value of this feature>

  Scenario: <the core happy path>
    Given <precondition>
    When <action>
    Then <expected outcome>

  Scenario: <a meaningful edge case or error path>
    ...
```

Lifecycle tags:
- `@proposed` — set by `/qsos-brainstorm` (or by this skill when no brainstorm was run). Draft state; not yet approved for implementation.
- `@accepted` — set by this skill when the audit passes and the verdict is GO. Implementation may proceed.
- `@in-progress` — set by `/qsos-implement` when coding begins.
- `@done` — set by `/qsos-doc-sync` after `/qsos-verify` returns CONFIRMED.

When this skill returns GO, update the tag from `@proposed` to `@accepted` before finalizing the file.

- Use the same terminology (nouns, verbs) as existing feature files. Read them before writing.
- Each scenario should be independently understandable — no implicit shared state between scenarios unless a Background block is used.
- Do not describe implementation details. Describe observable behavior.

### For `change` mode

Open the existing feature file. Identify which scenarios are affected by the change. Update them to reflect the new behavior. If the change adds a new case, add a new scenario. If it removes a case, remove or mark the scenario as obsolete with a comment.

After writing or updating the feature file, log `file_created` (new mode) or `file_modified` (change/bug mode):

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  # For new mode — file_created:
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-feature-doc','type':'file_created','data':{'path':'docs/features/<slug>.feature'}}))" >> "$LOG_PATH"
  # For change/bug mode — replace 'file_created' with 'file_modified' above
fi
```

### For `bug` mode

Determine which of these is true:

- **Gap** — The bug represents a case the feature file does not cover. Add a scenario that correctly describes the expected behavior (i.e. what the fix will make true).
- **Conflict** — The code's behavior contradicts the feature file. Determine which is authoritative — the spec or the implementation — and update accordingly. Surface this for review if it is not obvious.

---

## Step 4 — Audit

Run all four checks against the feature file from Step 3, comparing against all existing `.feature` files in `docs/features/`.

### Check 1 — Terminology
Does this file use the same nouns and verbs as existing files for the same concepts? Look for synonyms that refer to the same thing (e.g. "workspace" vs "project", "run" vs "execution"). Flag any mismatch.

### Check 2 — State assumptions
Does any scenario assume a state that no other scenario creates? For each `Given` clause, verify that either: (a) it describes a universally available starting condition, or (b) another scenario in the system creates that state as its `Then` outcome.

### Check 3 — Behavioral contradiction
Does any expected outcome (`Then`) in this file directly contradict a `Then` in an existing file for the same trigger? Look for the same action producing different outcomes across files.

### Check 4 — Scope creep
Does this file describe behavior that clearly belongs to a different feature area? Note it but do not block on it.

**Checks 1–3 are blockers.** If any fail, the verdict is BLOCKED. Do not proceed to implementation until resolved.

**Check 4 is a note.** Record it and continue.

---

## Step 5 — ADR (if warranted)

Apply this test to determine if an ADR is needed:

> *Would reversing this decision cost more than 30 minutes — in migration, refactoring, or untangling dependencies?*

An ADR takes minutes to write. If reversing a decision would cost even an hour of future work, the ADR pays for itself. The bar is low by design — aim for roughly twice as many ADRs as feel "obviously necessary." Prefer writing a short ADR over skipping one.

Common triggers worth recording:
- A storage format or schema was chosen
- A component boundary was established that other code will depend on
- A library or API was selected for a non-trivial reason
- A pattern was introduced that will govern similar future work

If yes — write or update an ADR.
If no — skip this step and note "ADR: not required" in the verdict.

ADR format (MADR):

```markdown
# ADR-NNN: <short title>

## Status
Accepted

## Context
<What is the situation that forces a decision? What constraints exist?>

## Decision
<What was decided?>

## Consequences
<What becomes easier? What becomes harder? What is now a known trade-off?>
```

Number sequentially from existing ADRs in `docs/decisions/`. If none exist, start at ADR-001.

After writing the ADR (if one was warranted), log `adr_created`:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-feature-doc','type':'adr_created','data':{'adr_path':'docs/decisions/ADR-NNN-<slug>.md','decision_summary':'<one line>'}}))" >> "$LOG_PATH"
fi
```

---

## Step 6 — Verdict

Produce the verdict block:

```
MODE: new | change | bug
FEATURE FILE: <path> — created | updated | unchanged
ADR: not required | <path> — created | updated
AUDIT:
  terminology:              pass | BLOCKED — <description>
  state assumptions:        pass | BLOCKED — <description>
  behavioral contradiction: pass | BLOCKED — <description>
  scope creep:              none | NOTE — <description>
GAPS NOTED: none | <list>
VERDICT: GO | BLOCKED — <reason>
```

**GO** means: the feature file is in place, the audit passed, any required ADR exists, and implementation may begin (or the fix may ship).

**BLOCKED** means: stop. Surface the specific issue. Do not proceed to implementation. Wait for direction.

After emitting the verdict block, log `skill_completed` (GO) or `skill_blocked` (BLOCKED):

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  # If GO:
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-feature-doc','type':'skill_completed','data':{'outcome':'go'}}))" >> "$LOG_PATH"
  # If BLOCKED — replace the above with:
  # python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-feature-doc','type':'skill_blocked','data':{'reason':'<audit failure>','resolution_required':'fix feature file'}}))" >> "$LOG_PATH"
fi
```

---

## Spike escape hatch

If feasibility is genuinely unknown before a feature file can be written meaningfully, a spike is permitted. Rules:

1. State explicitly that this is a spike, not implementation.
2. Spike output stays out of the main branch — a separate branch or scratch directory only.
3. Once feasibility is established, run `/qsos-feature-doc` in `new` mode before any spike findings become implementation.

A spike that quietly becomes implementation without this step is a violation of the protocol.

---

## What this skill does not do

- Does not run structural linting or deep cross-file validation (Strux's domain)
- Does not autonomously resolve conflicts — it surfaces them
- Does not write feature files from scratch without flagging them as drafts requiring review
- Does not infer that missing docs are acceptable because the feature is "small"
