---
description: Parallel implementation — Opus decomposes an approved plan into isolated sub-jobs, Sonnet subagents execute each in a worktree, main agent merges and verifies.
---

# /qsos-implement-fan

## Core Principle

This skill is a parallel variant of `/qsos-implement`. It is appropriate when an approved plan contains three or more items that can be executed in isolation. It adds one step: before any code is written, an Opus subagent analyses the approved plan for true independence, builds a sub-job manifest, and returns it for human review. Only after that manifest is confirmed does execution fan out. Nothing is "done" here — done is `/qsos-verify`'s word.

---

## When to use this skill vs `/qsos-implement`

Use `/qsos-implement-fan` when:
- The approved plan has 3+ items
- Items touch different files with no shared exported interfaces
- Turnaround time matters and the overhead of worktree setup is worth it

Use `/qsos-implement` (sequential) when:
- The plan is small (1–2 items)
- Items share a type file, barrel export, or interface boundary that makes isolation risky
- The plan is exploratory or high-uncertainty

---

## When this runs

After `/qsos-plan` has produced a plan and the user has explicitly approved it. Before `/qsos-coverage-check` and `/qsos-verify`.

---

## Step 1 — Confirm plan exists and is approved

Verify that a `/qsos-plan` output is present in context and that the user has explicitly approved it. If no plan exists, redirect to `/qsos-plan` and stop. If approval is ambiguous, ask before proceeding.

State the plan reference: ticket ID, feature file, number of plan items.

---

## Step 2 — Mark ticket in-progress

Call `/task start` with the active ticket ID.

If the project uses Jira, confirm the ticket is assigned. An unassigned Jira ticket is a stuck ticket.

---

## Step 3 — Update feature lifecycle

Update the feature file's lifecycle tag from `@accepted` to `@in-progress`.

---

## Step 4 — Spawn Opus decomposition subagent

Spawn a subagent using the Agent tool with `model: "opus"`. Pass it:


- The full approved plan (all scenario → file → change mappings)
- The feature file content
- Any ADR constraints identified in the plan

The subagent's job is to produce a **sub-job manifest**. Its instructions:

```
You are a decomposition analyst. You have an approved implementation plan. Your job is NOT to write code — it is to analyse the plan for parallel execution safety and produce a structured sub-job manifest.

Perform the following analysis:

1. DEPENDENCY GRAPH
   For each plan item, identify:
   - Which files it reads from (imports, references)
   - Which files it writes to
   - Whether it modifies any exported interface or shared type
   Draw implicit edges: two items that write to the same file, or where one item's output is another item's input, are DEPENDENT. Items with no edges between them are PARALLEL-SAFE.

2. SUB-JOB GROUPING
   Group parallel-safe items into sub-jobs. Items with dependency edges must be in the same sub-job (executed sequentially within it) or flagged as requiring sequential top-level ordering.
   Each sub-job must have:
   - A short label (e.g. "sub-job-1: parser module")
   - The plan items it covers (by number/reference)
   - The files it will touch (exhaustive list)
   - Execution order within the sub-job (if more than one item)

3. TDD CONTRACT
   For each sub-job, specify:
   - TEST FILE: the path where the failing test must be written
   - FAILING ASSERTION: the specific assertion that must fail before implementation (be concrete — name the function, describe the expected value)
   - PASSING CRITERION: what passing looks like (the assertion succeeds, no regressions in related tests)
   - NON-TESTABLE ITEMS: any items that cannot be covered by a test (list with reason)

4. COVERAGE VERIFICATION
   Confirm that the union of all sub-jobs covers every plan item exactly once. Flag any gap (plan item with no sub-job) or overlap (plan item in two sub-jobs) as a MANIFEST ERROR.

5. ISOLATION RISKS
   Flag anything that would make true isolation difficult:
   - A shared type file that multiple sub-jobs need to modify
   - A barrel export (index.ts) that needs updating for each sub-job
   - Test infrastructure that must exist before any sub-job can run (e.g. a test helper, a fixture)

Output format — return a structured manifest with these exact sections:
DEPENDENCY GRAPH: <edges or "none">
SUB-JOBS: <numbered list with label, items, files, execution order>
TDD CONTRACTS: <one per sub-job>
COVERAGE CHECK: PASS | MANIFEST ERROR: <details>
ISOLATION RISKS: <list or "none">
PRE-CONDITIONS: <any setup that must happen before sub-jobs fan out, or "none">
```

Do not proceed past this step until the subagent returns the manifest.

---

After the subagent returns the manifest, write it to `.qsos/manifest.json` at the project root before presenting to the user. Create the `.qsos/` directory if it does not exist. The file format:

```json
{
  "ticket": "<id>",
  "generated_at": "<ISO timestamp>",
  "dependency_graph": "<edges or none>",
  "sub_jobs": [
    {
      "label": "<label>",
      "items": ["<item ref>"],
      "files": ["<file path>"],
      "execution_order": ["<item ref in order>"],
      "tdd_contract": {
        "test_file": "<path>",
        "failing_assertion": "<description>",
        "passing_criterion": "<description>",
        "non_testable_items": []
      }
    }
  ],
  "pre_conditions": [],
  "isolation_risks": [],
  "coverage_check": "PASS | MANIFEST ERROR: <details>"
}
```

This file persists the manifest across context compression. Subagents (Step 7) read their sub-job from this file rather than relying solely on in-context briefing.

---

## Step 5 — Review manifest and await confirmation

Present the manifest to the user. State:

```
DECOMPOSITION MANIFEST

<manifest content>

---
Sub-jobs ready to execute in parallel: <count>
Sequential items (within sub-jobs): <count>
Pre-conditions required: yes | no

Proceed with this decomposition? (yes / adjust / abort)
```

If the Opus subagent flagged a MANIFEST ERROR or ISOLATION RISK that makes parallelisation unsafe, recommend falling back to `/qsos-implement` and stop.

Do not fan out until the user confirms.

---

## Step 6 — Execute pre-conditions

If the manifest listed pre-conditions (shared test helpers, fixture setup, interface stubs), execute those now in the main agent before spawning sub-job agents. Confirm each pre-condition is complete before proceeding.

---

## Step 7 — Fan out Sonnet subagents

For each sub-job in the manifest, spawn a Sonnet subagent using the Agent tool with `isolation: "worktree"`. Each subagent receives:

- Its sub-job label and the plan items it covers
- The exhaustive file list for its sub-job
- Its TDD contract (test file path, failing assertion, passing criterion)
- The feature file (read-only reference)
- Any relevant ADR constraints
- This instruction set:

```
You are executing sub-job: <label>

Your scope is STRICTLY LIMITED to the files listed in your sub-job. You must not modify any file outside this list. If you discover that a change outside your scope is required, surface it as a DEVIATION and stop — do not make the change.

TDD LOOP — for each plan item in your sub-job:
1. Write the failing test as specified in your TDD contract. Run it. Confirm it fails (Red).
2. Write the minimum implementation code to make it pass. Run it. Confirm it passes (Green).
3. Confirm no regressions in files you have touched.
4. Declare the item done before moving to the next.

Non-testable items: declare DEVIATION: non-testable, state the reason, then make the change.

DEVIATION PROTOCOL — if you discover that your plan items map to something different than anticipated:
  DEVIATION: <what was planned> → <what is actually needed>
  REASON: <why>
  PROCEEDING: yes (minor, same file, no interface impact) | BLOCKED (needs approval)

For BLOCKED deviations, stop and return your partial output. Do not guess.

When all items are complete, return:
  SUB-JOB COMPLETE: <label>
  ITEMS: <list of completed items>
  FILES MODIFIED: <exhaustive list>
  DEVIATIONS: none | <list>
  TEST STATUS: all green | <failures>
```

Run all sub-job subagents concurrently. Each works in its own worktree.

---

## Step 8 — Collect and review sub-job results

Wait for all subagents to return. For each:

- Confirm SUB-JOB COMPLETE (not partial)
- Check FILES MODIFIED against the manifest — flag any unexpected file
- Note any DEVIATION, especially BLOCKED ones

If any sub-job returned BLOCKED, surface the deviation to the user and resolve before merging. Do not proceed with a partial merge.

---

## Step 9 — Merge worktrees

Merge each sub-job's worktree back into the working branch sequentially. For each merge:

- Run the project's test suite against the merged state
- Confirm no regressions introduced by the merge
- If a merge conflict arises, resolve it explicitly — do not auto-resolve in a way that drops changes

After all merges, run the full test suite once more against the complete merged state.

---

## Step 10 — Cross-file impact check

Verify:

- No file outside the approved plan was unintentionally modified
- No exported interface or type was changed without a corresponding plan item
- No new dependency was introduced that wasn't present before

Surface any unplanned change now.

---

## Step 11 — Commit with ticket reference

Every commit must include the Jira ticket key in the subject line:

```
feat(EN-72): implement parser module and validator
```

If the sub-jobs were committed separately in their worktrees, ensure the merge commits also carry the ticket reference.

Before pushing, scan outgoing commits for missing Jira keys.

---

## Step 12 — Hand off to verification

Do not declare the implementation complete. When all sub-jobs are merged and tests are green, state:

```
IMPLEMENTATION: all sub-jobs executed and merged

SUB-JOBS COMPLETED: <list>
UNPLANNED CHANGES: none | <list>
DEVIATIONS: none | <list>
MERGE CONFLICTS RESOLVED: none | <list>

Next step: run /qsos-coverage-check
```

Then run `/qsos-coverage-check`. After coverage check passes, run `/qsos-review`.

---

## Blocking rules

- You may not begin execution without a confirmed decomposition manifest.
- You may not fan out sub-job agents without explicit user approval of the manifest.
- A BLOCKED deviation in any sub-job halts the merge step for that sub-job.
- You may not declare implementation complete without a CONFIRMED verdict from `/qsos-verify`.
- You may not skip the cross-file impact check or the final full test run after merge.
