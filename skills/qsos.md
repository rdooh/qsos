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
- "use QSOS on TIX-007" → anchors to that ticket and determines its next stage

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

## Step 2 — Determine entry point

Map ticket and feature state to the next skill in the chain:

| Ticket status | Feature tag | Next skill |
|---|---|---|
| todo, no feature file | — | `/qsos-brainstorm` |
| todo, feature `@proposed` | @proposed | `/qsos-feature-doc new` |
| ready, feature `@accepted`, no plan | @accepted | `/qsos-orient` → `/qsos-plan` |
| ready, feature `@accepted`, plan approved | @accepted | `/qsos-implement` |
| in-progress, feature `@in-progress` | @in-progress | `/qsos-verify` (if unverified) |
| in-progress, verified CONFIRMED | @in-progress | `/qsos-doc-sync` |
| done | @done | nothing — pick next ticket |
| bug reported | any | `/qsos-bug` |

State the entry point and the reasoning in one line:

> "TIX-007 is ready with an accepted feature file and no plan — entering at /qsos-orient."

Then confirm with the user: "Proceed?" and continue unless redirected.

---

## Step 3 — Drive the chain

Execute the entry point skill. After it completes:

- If verdict is GO / CONFIRMED / CLEAN → move to the next skill in the chain automatically
- If verdict is BLOCKED / NEEDS CLARIFICATION / UNCONFIRMED → stop, surface the issue, wait for direction
- If the skill requires human input (plan approval, scoping questions) → wait for the response, then continue

Do not stop between skills when the path is clear. The user said "take it to the end" — take it to the end.

---

## Step 4 — Report progress at each stage

After each skill completes, emit a one-line status before moving on:

```
✓ /qsos-orient — context loaded, no gaps
✓ /qsos-plan — presented, awaiting approval
  [user approves]
✓ /qsos-implement — all plan items executed
✓ /qsos-verify — CONFIRMED (test results: test-results/unit.json)
✓ /qsos-doc-sync — CLEAN, TIX-007 closed
```

---

## Step 5 — Final summary

When the chain reaches a natural stopping point (ticket closed, blocked, or waiting for human input), produce:

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

**You may not skip human gates.** `/qsos-plan` always waits for approval. `BLOCKED` verdicts always stop. Scoping questions in `/qsos-brainstorm` always wait for answers. Everything else runs unattended. The chain is only as autonomous as its gates allow — do not bypass them to appear faster.
