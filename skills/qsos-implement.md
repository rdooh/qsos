---
description: Implementation contract — enter the coding phase with an approved plan and leave it only after /verify confirms.
---

# /implement

## Core Principle

Implementation without a plan is exploration. Exploration is valuable — but it is not implementation. This skill is a one-page contract: you have a plan, it has been approved, and your job is to execute it faithfully. Deviations are not forbidden — they are flagged before continuing, not after. Nothing is "done" here. Done is `/qsos-verify`'s word.

---

## When this runs

After `/qsos-plan` has produced a plan and the user has approved it. Before `/test` and `/qsos-verify`.

---

## Step 1 — Confirm plan exists and is approved

Verify that a `/qsos-plan` output is present in context and that the user has explicitly approved it. If no plan exists, redirect to `/qsos-plan` and stop. If a plan exists but approval is ambiguous, ask before writing a single line of code.

State the plan reference: which ticket, which feature file, how many items in the plan.

---

## Step 2 — Mark ticket in-progress

Call `/task start` with the active ticket ID. This is not optional — invisible work is untracked work.

If the project uses Jira, assign the ticket at creation time. An unassigned Jira ticket is a stuck ticket.

---

## Step 3 — Set feature lifecycle

Update the feature file's lifecycle tag from `@accepted` to `@in-progress`. This signals that implementation has begun and prevents another agent from concurrently starting the same work.

---

Follow the approved plan exactly, item by item. For each code change item:

1. **Verify testing manifest**: Check `testing/manifest.json` for configured runners.
2. **Write a failing test**: Before writing or modifying any implementation code, write a test in the project's unit or integration test directory that asserts the new behavior.
3. **Confirm Red state**: Run the test runner to verify the test fails. If no runner is configured, note: "No test runner configured — skipping local run."
4. **Write implementation code**: Write the minimum code needed to satisfy the test.
5. **Confirm Green state**: Run the test runner to verify the test now passes.
6. **Confirm the item is done** before moving to the next.

**When a plan item is non-testable** (e.g. comment/doc update, configuration file edit, meta-formatting), declare a minor deviation:
```
DEVIATION: non-testable item — skipping TDD loop
REASON: <why the item cannot be verified by a test case>
```

**When a deviation is necessary** — a file not in the plan needs to change, the approach needs to adjust, or a scenario maps to something different than anticipated — stop before making the change. State:

```
DEVIATION: <what was planned> → <what is actually needed>
REASON: <why the plan item does not match reality>
PROCEEDING: yes (minor, no architectural impact) | no (needs approval)
```

Minor deviations (different line numbers, additional helper function in the same file) may proceed. Deviations that add files, change interfaces, or affect other feature areas require a pause for review.

---

## Step 5 — Cross-file impact check

Before declaring implementation complete, verify:

- No file outside the approved plan was unintentionally modified
- No exported interface or type was changed without a corresponding plan item
- No new dependency was introduced that wasn't present before

If any unplanned change is discovered, surface it now — do not omit it from the record.

---

## Step 5b — Commit with ticket reference

Every commit made during implementation must include the **Jira ticket key** in the commit message subject when one exists. Local-only identifiers (work/ tickets, todo numbers) do not count.

```
feat(EN-72): place all 267 SPs into pipeline-dag.yaml
fix(EN-45): correct edge direction in dag-parser
```

If a commit covers incidental work with no Jira ticket (e.g. a dependency bump), note `(no ticket)` explicitly so it is a conscious decision, not an oversight.

**Before pushing**, scan the outgoing commits for any missing Jira key. Amend if not yet pushed; flag explicitly if already pushed.

---

## Step 6 — Hand off to verification

Do not declare the implementation complete. Do not use the words "done", "complete", "finished", or "working". When the plan items are executed, state:

```
IMPLEMENTATION: all plan items executed

UNPLANNED CHANGES: none | <list>
DEVIATIONS: none | <list>

Next step: run /qsos-coverage-check
```

Then run `/qsos-coverage-check`. If a project-specific test skill applies (e.g. `/vscode-ext-test`), run it before `/qsos-coverage-check` so test failures surface first. After coverage check passes, run `/qsos-review`.

---

## Blocking rule

**You may not begin writing code without an approved plan.** You may not declare implementation complete without a CONFIRMED verdict from `/qsos-verify`. You may not skip the cross-file impact check — an undisclosed change is a hidden risk. Any deviation from the approved plan must be surfaced before the deviated change is made, not discovered during review afterward.
