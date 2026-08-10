---
description: Post-implementation reconciliation — close the loop between what was planned, what was built, and what the docs say.
---

# /doc-sync

## Core Principle

Docs that are written before implementation and never updated after it are fiction. This skill is the reconciliation pass: it re-reads what was specified, compares it to what was built, resolves the gap, and closes the record. A feature is not done when the code compiles — it is done when the spec, the architecture model, the ticket, and the implementation all agree.

---

## When this runs

After `/qsos-verify` has returned CONFIRMED. This skill does not run if verification is outstanding — closing docs on unverified work is premature.

---

Emit the `skill_started` log event:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-doc-sync','type':'skill_started','data':{}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

## Step 1 — Confirm verification is complete

Check that `/qsos-verify` has returned CONFIRMED in this session for the active ticket. If `/qsos-verify` has not run or returned UNCONFIRMED or INCONCLUSIVE, stop. State: "Doc sync cannot run until /verify returns CONFIRMED."

---

## Step 2 — Check feature lifecycle

Load the feature file(s) linked to the active ticket. The lifecycle tag should be `@in-progress`. If it is:

- `@proposed` — implementation should not have started; note as a process gap
- `@accepted` — `/qsos-implement` should have set this to `@in-progress`; note and correct
- `@in-progress` — correct; continue
- `@done` — this skill has already run; confirm this is not a duplicate run

---

## Step 3 — Check behavioral drift

For each scenario in the feature file, re-read the `Given / When / Then` steps and compare to the implemented behavior.

Ask: does the code do what the scenario says it does? Test cases passing is not the same question — tests may not cover all scenarios, and scenarios may have been written before edge cases were discovered.

For each scenario, note:
- **Match** — behavior matches specification
- **Drift** — implemented behavior differs from what the scenario specifies

If drift is found, determine what is authoritative:
- The spec was correct, the implementation drifted → fix the code or flag for the next ticket
- The implementation revealed a better behavior → update the scenario to reflect reality, note the change

Do not silently accept drift. Both outcomes require a record.

---

## Step 4 — Check DSL duality

Load `docs/architecture/architecture.dsl`. For each `Target` element that was part of the work delivered in this ticket:

- Has it been implemented? If yes, change the tag from `"Target"` to `"Current"`
- Is it partially implemented? Leave as `"Target"` and note what remains

If `architecture_updated` was `true` in the ticket frontmatter, confirm the DSL was actually updated during implementation. If it was not, do it now.

---

## Step 5 — Check ADR completeness

Review the implementation decisions made. Apply the 30-minute reversal test: was any choice made during implementation that would have warranted an ADR, but none was written? The bar is low — if reversing the decision would cost more than 30 minutes in migration, refactoring, or untangling dependencies, it deserves an ADR.

Common triggers:
- A library was chosen for a non-trivial reason
- A pattern was applied that will govern future similar work
- A component boundary was established that other code now depends on

If a decision slipped through undocumented, write the ADR now (status `Accepted`, retrospective). It is better to write it late than never.

---

## Step 5c — Unrecorded decision check

This step is distinct from Step 5. Step 5 asks whether known implementation decisions are documented. This step asks: **were any architectural decisions made during implementation that have no ADR at all?**

Review the implementation work for this ticket. Ask specifically:

- Was a library or framework chosen for a non-trivial reason?
- Was a component boundary established that other code will now depend on?
- Was a structural pattern applied that will govern future similar work?
- Was a technology excluded or constrained in a way that was not previously documented?

Apply the 6-month reversal test: *if this decision were reversed in six months, would it require migrating data, refactoring multiple files, or changing how other features work?* If yes — an ADR is needed.

**If an unrecorded decision is found:**
```
Unrecorded decision detected — capturing before close.
```
Dispatch the `architect` agent (or invoke `/qsos-architecture`) to write the ADR retrospectively (status `Accepted`, date = today). Do not close the ticket until the ADR is written.

**If no unrecorded decisions:**
```
Unrecorded decisions: none.
```
State this explicitly. Do not omit this line — a silent pass is indistinguishable from a skipped check.

---

## Step 6 — Check for stale lifecycle tags

Read `work/tix-manifest.json`. For the closing ticket and any tickets linked as dependencies that are also now `done`:

- Any feature file still tagged `@proposed` when its ticket is `done` → flag `[STALE_TAG]` and correct it now
- Any feature file still tagged `@accepted` (never moved to `@in-progress`) when its ticket is `done` → flag `[SKIPPED_LIFECYCLE]` and note it

Also scan all other feature files in `docs/features/` for the same condition — a batch close can leave orphaned tags from related work.

If the manifest does not exist, skip this step and note the absence.

---

## Step 6b — Note code dependency drift (manual check)

Read `docs/architecture/architecture.dsl`. For each container element that was changed or added during this ticket's implementation, ask: does the import/dependency structure in the code still match the `->` relationships declared in the DSL?

This is a lightweight manual check — scan the changed files for `require`, `import`, or equivalent statements and compare to DSL relationships. Flag any discrepancy as `[DEPENDENCY_DRIFT]`.

---

## Step 7 — Update lifecycle and close ticket

1. Set feature file lifecycle tag to `@done`
2. Call `/task close <id> <evidence pointer>` — the evidence pointer is the artifact reference from `/qsos-verify` (test result path, screenshot, log excerpt, etc.)

Emit a `file_modified` event for each document updated during this skill run — feature file, DSL, ticket, ADR. Run once per file, substituting the actual path:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-doc-sync','type':'file_modified','data':{'path':'<path to updated file>'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

---

## Step 9 — Produce sync report

```
DOC SYNC REPORT

FEATURE: <path> → @done
TICKET: <id> → closed [evidence: <pointer>]

DSL CHANGES:
  - <element name> [Target → Current | no change needed]
  [or: none]

ADR GAPS FILLED:
  - <ADR-NNN: title — created retrospectively>
  [or: none]

UNRECORDED DECISIONS:
  - <decision description — ADR-NNN written>
  [or: none]

BEHAVIORAL DRIFT:
  - Scenario "<name>": <match | drift — <description and resolution>>
  [or: none]

STALE TAGS:
  - [STALE_TAG] <feature> — corrected to @done
  [or: none]

DEPENDENCY DRIFT:
  - [DEPENDENCY_DRIFT] <element> — import found with no DSL relationship
  [or: none]

SYNC VERDICT: CLEAN | DRIFT FOUND — <details>
```

---

Emit the `skill_completed` log event:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-doc-sync','type':'skill_completed','data':{'outcome':'clean'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

---

## Blocking rule

**You may not mark the ticket `done` or the feature `@done` unless `/qsos-verify` has returned CONFIRMED.** Do not skip the behavioral drift check — a test suite passing is not the same as every specified scenario being implemented as written. An undocumented architectural decision discovered during this step must be recorded as an ADR before the sync report is marked CLEAN.
