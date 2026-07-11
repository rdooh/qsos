---
description: Test coverage gate — verify that new pure functions and new feature scenarios have corresponding tests before the ticket reaches /qsos-verify.
---

# /qsos-coverage-check

## Core Principle

Code that has no test is a black box. A feature can pass `/qsos-verify` visually while its core logic is completely untested — the next person who touches it has no safety net. This skill runs after `/qsos-implement` and before `/qsos-verify`. It does not run tests. It checks that tests *exist* for the code that was just written, and produces a PASS or GAPS FOUND verdict before the ticket moves forward.

---

## When this runs

After `/qsos-implement` has completed all plan items. Before `/qsos-verify`.

In the chain: `implement → coverage-check → review → verify → doc-sync`.

Can also be run standalone against any ticket or changed file set.

---

## Step 1 — Identify changed files

Determine what was changed during this ticket's implementation. Use the approved plan from `/qsos-plan` if present in context. Otherwise run:

```bash
git diff --name-only HEAD~1
```

or read the plan items for file paths. Filter to source files only — exclude test files, docs, configs, and generated output (`out/`, `dist/`, `*.vsix`, etc.).

List every changed source file. This is the **change set**.

---

## Step 2 — Locate the test directories

Detect the test structure for this project. Common patterns:

| Pattern | Where to look |
|---|---|
| TypeScript / Node.js | `src/test/unit/`, `src/test/integration/`, `test/` |
| Python | `tests/`, `test_*.py` adjacent to source |
| Go | `*_test.go` adjacent to source |
| Rust | `#[cfg(test)]` in same file, or `tests/` |

If no test directory exists at all, flag `[NO_TEST_HARNESS]` and stop — this is a blocker. Direct to set up the test harness before proceeding.

---

## Step 3 — Pure function coverage check

For each changed source file, scan for **newly added or modified exported pure functions** — functions that:

- Are exported (`export function`, `export const ... = (`, `def `, `pub fn`, etc.)
- Do **not** import or call platform APIs (`vscode`, `fs`, `http`, `os`, `child_process`, `subprocess`, `std::io`, etc. — anything that talks to the outside world)

These are the highest-value unit test targets: no mocking required, deterministic, fast.

For each pure function found, check whether a test file in the unit test directory contains the function name as a string. A test that imports and calls the function by name satisfies this check.

Flag any pure function with no matching test reference as `[UNCOVERED_PURE_FUNCTION]`.

---

## Step 4 — Feature scenario coverage check

Load the feature file(s) linked to the active ticket. For each `Scenario:` block:

1. Identify the **core action** — the `When` step verb + noun (e.g. "runs devkit doctor", "calls _compute with null cost")
2. Search the integration test directory for a test case that references that scenario's core action or a close paraphrase of it (function name, command name, or scenario title substring)

A scenario is **covered** if at least one test case exercises it. A scenario is **uncovered** if no test references its core behavior.

Flag uncovered scenarios as `[UNCOVERED_SCENARIO: <scenario name>]`.

**Note:** this is a textual proximity check, not runtime coverage. It catches obvious gaps (no test file at all for a new command) — it does not prove full branch coverage.

---

## Step 5 — Assess severity

Classify each gap:

| Type | Severity | Default action |
|---|---|---|
| `[NO_TEST_HARNESS]` | **Blocker** — no tests can exist | Stop, direct to harness setup |
| `[UNCOVERED_PURE_FUNCTION]` | **Required** — pure functions must have unit tests | Block ticket progression |
| `[UNCOVERED_SCENARIO]` | **Required** for happy path; **Advisory** for edge cases | Block on happy path gaps; note edge case gaps |

An edge case scenario is one whose `Scenario:` name includes words like: "when no … exists", "advises", "reports clean", "already", "skips", "idempotent", "gracefully".

---

## Step 6 — Produce coverage report

```
COVERAGE CHECK REPORT

TICKET: <id> — <title>
CHANGE SET: <N> source files

PURE FUNCTION COVERAGE:
  pass — all <N> pure functions have unit test references
  | <N> gap(s):
  - [UNCOVERED_PURE_FUNCTION] <FunctionName> in <file> — no reference found in <test-dir>

SCENARIO COVERAGE:
  pass — all <N> scenarios have integration test references
  | <N> gap(s):
  - [UNCOVERED_SCENARIO: <name>] — no test reference found (required | advisory)

BLOCKERS: <N> — must resolve before /qsos-verify | none
NOTES: <N> advisory gaps — worth addressing but do not block

COVERAGE VERDICT: PASS | GAPS FOUND — <N> blocker(s), <N> note(s)
```

---

## Step 7 — On GAPS FOUND

Do not proceed to `/qsos-verify`. For each **blocker gap**:

1. State the file and function/scenario that needs a test
2. Ask: "Write the missing test(s) now, or defer to a follow-up ticket?"

If the user chooses to write them now, write the test(s), re-run the relevant test command to confirm they pass, then re-run the coverage check. If they pass and no new gaps are found, issue PASS and proceed to `/qsos-verify`.

If the user defers: create a follow-up ticket (next TIX number) scoped to "add missing tests for <function/scenario>" with `priority: high` and `depends_on: []`. Note it in the coverage report, then proceed to `/qsos-verify` with an explicit `COVERAGE DEFERRED` annotation so the record is clear.

---

## Blocking rule

**You may not advance to `/qsos-verify` with uncovered pure functions or uncovered happy-path scenarios unless the user has explicitly chosen deferral and a follow-up ticket has been created.** Advisory gaps (edge cases) do not block but must appear in the report. Silent omission of gaps is not acceptable — a gap that is not reported is a gap that will not be fixed.
