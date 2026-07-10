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

## Step 1 — Resolve the ticket

Call `/task read` with the ticket ID from context, or infer the ticket from the task description. If no ticket ID can be determined, ask for one before continuing — planning without a ticket reference is planning without a record.

Load from the ticket:
- Title and description
- Linked feature files (`features:` list)
- Linked ADRs (`adrs:` list)
- `architecture_updated` field
- Dependencies (`depends_on:` list) — check that all are `done` or explicitly waived

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

---

## Blocking rule

**You may not begin planning if any linked feature file is `@proposed` (not yet `@accepted`).** You may not begin planning if a required ADR has status `Proposed`. An unresolved architectural decision is not a detail to defer — it is a constraint that shapes the plan. Surface the blocker, direct to the right skill (`/qsos-feature-doc` or `/qsos-architecture`), and stop.
