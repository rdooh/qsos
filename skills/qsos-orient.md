---
description: Project-aware context loader — reads the ticket, feature files, ADRs, and architecture into context before planning begins.
---

# /orient

## Core Principle

An agent that starts implementing without reading the existing documentation is not planning — it is guessing. This skill ensures that before any plan is written, the full relevant context is loaded: what was agreed (tickets), what was specified (feature files), what was decided (ADRs), and what the structural model looks like (DSL). Gaps and stale artifacts are surfaced here, not discovered mid-implementation.

---

## When this runs

After `/qsos-feature-doc` has set a feature `@accepted` and before `/qsos-plan` produces an implementation plan. Always runs in sequence — never skipped.

---

Log `skill_started` before resolving the ticket:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-orient','type':'skill_started','data':{}}))" >> "$LOG_PATH"
fi
```

## Step 1 — Resolve the ticket

Call `/task read` with the ticket ID from context, or infer the ticket from the task description. If no ticket ID can be determined, ask for one before continuing — planning without a ticket reference is planning without a record.

Load from the ticket:
- Title and description
- Linked feature files (`features:` list)
- Linked ADRs (`adrs:` list)
- `architecture_updated` field
- Dependencies (`depends_on:` list) — check that all are `done` or explicitly waived

### Graph-assisted context (preferred)

If `qsos query` is available, load linked artifacts from the compiled graph:

```bash
qsos query --ticket <ticket-id> --root <project-root>
```

Use the returned `nodes` list as the load index — read full file contents only for artifacts in the subgraph (`feature`, `adr`, `scenario` nodes). Use `summary` counts to confirm expected coverage before reading.

**Fallback:** If `qsos` is not installed, the graph registry is missing, or query fails, load artifacts manually via Steps 2–4 below and note `graph delegation skipped`.

---

## Step 2 — Load feature files

For each feature file linked in the ticket, read the full file. Also read any `.feature` file whose name or `Feature:` title suggests topical overlap with the ticket description.

Note for each:
- Lifecycle tag — `@proposed`, `@accepted`, `@in-progress`, or `@done`
- Number of scenarios loaded
- Any scenario that references external state assumptions (a `Given` clause not satisfied by any visible `Then`)

---

## Step 3 — Load ADRs

For each ADR linked in the ticket, read the full record. Also load any ADR referenced within the loaded feature files (by filename or ADR number).

Note for each:
- Status — `Proposed`, `Accepted`, `Superseded`, or `Rejected`
- Decisions that directly constrain implementation choices (technology, pattern, structure)
- Superseded ADRs — note what replaced them; do not apply their decisions

---

## Step 4 — Load architecture

Read `docs/architecture/architecture.dsl`. Identify containers and components that are:
- Touched by the feature being built
- Tagged `Target` (planned but not yet implemented)
- Referenced in any loaded ADR

If the DSL file does not exist, note the absence — this is a gap, not a blocker unless the ticket's `architecture_updated` field is `true`.

---

## Step 5 — Flag staleness and gaps

Check the following conditions and note any that apply:

- Any linked feature file is `@proposed` — **implementation cannot proceed; needs `/qsos-feature-doc` first**
- Any linked ADR has status `Proposed` — **decision is unresolved; surface before planning**
- The `architecture_updated` field is `false` but the ticket clearly touches the structural model — **flag for review**
- Any `Target` DSL element has no corresponding `Accepted` ADR — **flag as incomplete**
- Any dependency ticket is not `done` — **flag as unresolved blocker**
- Feature file references a concept not in any ADR or DSL element — **flag as undocumented assumption**

For each gap found, emit a `gap_discovered` event:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-orient','type':'gap_discovered','data':{'gap_type':'<feature|architecture|test|doc>','description':'<one-line gap description>'}}))" >> "$LOG_PATH"
fi
```

---

## Step 6 — Produce context summary

```
ORIENT SUMMARY

TICKET: <id> — <title> [status]

FEATURE FILES LOADED:
  - <path> [@tag] — <N scenarios>
  [...]

ADRS LOADED:
  - <path> [Accepted] — <one-line summary of decision>
  [...]

ARCHITECTURE ELEMENTS (relevant to this work):
  - <container/component name> [Current | Target] — <description>
  [...]

GAPS FLAGGED:
  - <gap description or "none">

READY FOR /plan: yes | no — <reason if no>
```

Do not proceed to `/qsos-plan` until the summary shows `READY FOR /plan: yes`.

After emitting the summary, log `skill_completed` (if READY FOR /plan: yes) or `skill_blocked` (if no):

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  # If READY FOR /plan: yes:
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-orient','type':'skill_completed','data':{'outcome':'ready'}}))" >> "$LOG_PATH"
  # If READY FOR /plan: no — replace the above with:
  # python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-orient','type':'skill_blocked','data':{'reason':'<why not ready>','resolution_required':'<redirect skill>'}}))" >> "$LOG_PATH"
fi
```

---

## Blocking rule

**You may not begin planning if any linked feature file is `@proposed` (not yet `@accepted`).** You may not begin planning if a required ADR has status `Proposed`. An unresolved architectural decision is not a detail to defer — it is a constraint that shapes the plan. Surface the blocker, direct to the right skill (`/qsos-feature-doc` or `/qsos-architecture`), and stop.
