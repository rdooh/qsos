---
description: Testing posture audit and coverage gate — checks that testing infrastructure is correctly declared and wired, then verifies new code has corresponding tests.
---

# /qsos-coverage-check

## Core Principle

Two distinct things can be wrong with a project's testing story: the infrastructure isn't wired up correctly, and the code that was just written isn't tested. This skill checks both — in that order. Posture first (is the declared testing infrastructure actually in place?), then coverage (do the changed files have tests?). Both must pass before `/qsos-verify` runs.

---

## When this runs

After `/qsos-implement` has completed all plan items. Before `/qsos-review` and `/qsos-verify`.

In the chain: `implement → coverage-check → review → verify → doc-sync`.

Can also be run standalone against any ticket or changed file set.

---

## Logging — skill_started

At the start of execution, before Step 0, emit a `skill_started` log entry:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-coverage-check\",\"type\":\"skill_started\",\"data\":{}}" >> "$LOG_PATH"
fi
```

---

## Step 0 — Load or create testing/manifest.json

Look for `testing/manifest.json` at the project root.

**If the manifest exists:** read it. This is the authoritative source for what testing infrastructure is declared. Do not scan `package.json`, `pytest.ini`, or other config files to infer the runner — read the manifest.

**If the manifest does not exist:** create it by running detection heuristics and prompting for fields that cannot be auto-detected:

Detection heuristics (run from project root):
- `unit_runner`: jest → any of `jest.config.*`, `"jest"` in package.json scripts; vitest → `vitest.config.*`; pytest → `pytest.ini`, `pyproject.toml` with `[tool.pytest]`
- `e2e_runner`: playwright → `playwright.config.*`; cypress → `cypress.config.*`
- `pre_commit_hook`: `.git/hooks/pre-commit` exists and is executable, or `.pre-commit-config.yaml` exists
- `pre_push_hook`: `.git/hooks/pre-push` exists and is executable
- `coverage_threshold`: check jest/vitest config for `coverageThreshold`, pytest-cov for `--cov-fail-under`

For any field that cannot be detected, set to `null` and note it needs configuration.

Write the manifest to `testing/manifest.json` following ADR-006 schema — all fields required, `null` for absent, JSON format. Inform the developer: "Created testing/manifest.json — please review and commit."

---

## Step 1 — Posture audit

Using the loaded manifest, check each declared value against actual project state:

**Unit runner check:**
- If `unit_runner` is non-null: verify the corresponding config exists
  - `jest` → `jest.config.*` or `"jest"` key in package.json scripts
  - `vitest` → `vitest.config.*`
  - `pytest` → `pytest.ini` or `pyproject.toml` with `[tool.pytest]`
  - Missing → `[POSTURE_GAP: HIGH]` unit runner declared but config not found

**E2E runner check:**
- If `e2e_runner` is non-null: verify the corresponding config exists
  - `playwright` → `playwright.config.*`
  - `cypress` → `cypress.config.*`
  - Missing → `[POSTURE_GAP: HIGH]` e2e runner declared but config not found

**Undeclared runner detection:**
- Scan for `jest.config.*`, `vitest.config.*`, `pytest.ini`, `playwright.config.*`, `cypress.config.*` regardless of manifest
- If found but not declared in manifest → `[POSTURE_GAP: MEDIUM]` runner config found but not declared — update manifest

**Pre-commit hook check:**
- If `pre_commit_hook` is `false` or `null` → `[POSTURE_GAP: MEDIUM]` no pre-commit hook wired; add hook to run test suite before commit (hook template: see utilities/ when available)

**Pre-push hook check:**
- If `pre_push_hook` is `false` or `null` → `[POSTURE_GAP: LOW]` no pre-push hook wired

**Coverage threshold check:**
- If `coverage_threshold` is `null` → `[POSTURE_GAP: LOW]` no coverage threshold enforced

Collect all posture gaps. If any HIGH gaps exist, surface them prominently in the report.

---

## Step 2 — Identify changed files

Determine what was changed during this ticket's implementation.

**When invoked after `/qsos-implement-fan`:** check for `.qsos/manifest.json` at the project root. If present, the change set is the **union of all `files` arrays across every sub-job** in the manifest. Use this list directly rather than `git diff` — the manifest is authoritative for fan-out implementations and accounts for all worktree merges.

**Otherwise:** use the approved plan from `/qsos-plan` if present in context. Otherwise run:

```bash
git diff --name-only HEAD~1
```

Filter to source files only in all cases — exclude test files, docs, configs, and generated output (`out/`, `dist/`, `*.vsix`, etc.).

List every changed source file. This is the **change set**.

---

## Step 3 — Locate the test directories

Detect the test structure for this project. Common patterns:

| Pattern | Where to look |
|---|---|
| TypeScript / Node.js | `src/test/unit/`, `src/test/integration/`, `test/` |
| Python | `tests/`, `test_*.py` adjacent to source |
| Go | `*_test.go` adjacent to source |
| Rust | `#[cfg(test)]` in same file, or `tests/` |

If no test directory exists at all, flag `[NO_TEST_HARNESS]` and stop — this is a blocker. Direct to set up the test harness before proceeding.

---

## Step 4 — Pure function coverage check

For each changed source file, scan for **newly added or modified exported pure functions** — functions that:

- Are exported (`export function`, `export const ... = (`, `def `, `pub fn`, etc.)
- Do **not** import or call platform APIs (`vscode`, `fs`, `http`, `os`, `child_process`, `subprocess`, `std::io`, etc. — anything that talks to the outside world)

These are the highest-value unit test targets: no mocking required, deterministic, fast.

For each pure function found, check whether a test file in the unit test directory contains the function name as a string. A test that imports and calls the function by name satisfies this check.

Flag any pure function with no matching test reference as `[UNCOVERED_PURE_FUNCTION]`.

---

## Step 5 — Feature scenario coverage check

Load the feature file(s) linked to the active ticket. For each `Scenario:` block:

1. Identify the **core action** — the `When` step verb + noun (e.g. "runs devkit doctor", "calls _compute with null cost")
2. Search the integration test directory for a test case that references that scenario's core action or a close paraphrase of it (function name, command name, or scenario title substring)

A scenario is **covered** if at least one test case exercises it. A scenario is **uncovered** if no test references its core behavior.

Flag uncovered scenarios as `[UNCOVERED_SCENARIO: <scenario name>]`.

**Note:** this is a textual proximity check, not runtime coverage. It catches obvious gaps (no test file at all for a new command) — it does not prove full branch coverage.

---

## Step 6 — Assess severity

Classify each gap:

| Type | Severity | Default action |
|---|---|---|
| `[POSTURE_GAP: HIGH]` | **Required** — declared runner has no config | Block ticket progression |
| `[NO_TEST_HARNESS]` | **Blocker** — no tests can exist | Stop, direct to harness setup |
| `[UNCOVERED_PURE_FUNCTION]` | **Required** — pure functions must have unit tests | Block ticket progression |
| `[UNCOVERED_SCENARIO]` | **Required** for happy path; **Advisory** for edge cases | Block on happy path gaps; note edge case gaps |
| `[POSTURE_GAP: MEDIUM]` | **Advisory** — undeclared runner or missing hook | Note, do not block |
| `[POSTURE_GAP: LOW]` | **Advisory** — threshold or push hook absent | Note, do not block |

An edge case scenario is one whose `Scenario:` name includes words like: "when no … exists", "advises", "reports clean", "already", "skips", "idempotent", "gracefully".

---

## Logging — test_run and coverage_gap

After running tests (or after Step 5 — the feature scenario coverage check), emit a `test_run` log entry with results:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-coverage-check\",\"type\":\"test_run\",\"data\":{\"runner\":\"<pytest|jest|vitest|etc>\",\"passed\":<N>,\"failed\":<N>,\"skipped\":<N>,\"coverage_pct\":<N>,\"result_link\":\"file://<absolute-path-to-results>\"}}" >> "$LOG_PATH"
fi
```

If coverage gaps are identified (uncovered pure functions, uncovered scenarios, or posture gaps at HIGH), emit a `coverage_gap` log entry per gap set:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-coverage-check\",\"type\":\"coverage_gap\",\"data\":{\"files_uncovered\":[\"<list>\"],\"threshold\":<N>,\"actual\":<N>}}" >> "$LOG_PATH"
fi
```

---

## Step 7 — Produce report

```
COVERAGE CHECK REPORT

TICKET: <id> — <title>
MANIFEST: testing/manifest.json [present | created | absent]

POSTURE AUDIT:
  HEALTHY — all declared infrastructure confirmed
  | <N> gap(s):
  - [POSTURE_GAP: HIGH] <description> — <remediation>
  - [POSTURE_GAP: MEDIUM] <description> — <remediation>
  - [POSTURE_GAP: LOW] <description> — <remediation>

COVERAGE:
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

VERDICT: PASS | GAPS FOUND — <N> blocker(s), <N> note(s)
```

If all posture gaps are LOW/MEDIUM and all coverage gaps are advisory: verdict is PASS with notes.

---

## Logging — skill_completed

After producing the Step 7 report, emit a `skill_completed` log entry:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  RUN_ID=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['run_id'])" 2>/dev/null)
  TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  echo "{\"run_id\":\"$RUN_ID\",\"timestamp\":\"$TS\",\"ticket\":\"$TICKET_ID\",\"skill\":\"qsos-coverage-check\",\"type\":\"skill_completed\",\"data\":{\"outcome\":\"<pass|fail|deferred>\"}}" >> "$LOG_PATH"
fi
```

Outcome values: `pass` (no blockers), `fail` (blockers found, not deferred), `deferred` (user chose to defer gaps to a follow-up ticket).

---

## Step 8 — On GAPS FOUND

Do not proceed to `/qsos-verify`. State each blocker gap as text, then use `AskUserQuestion`:

- Question: "Coverage blockers found — how should I proceed?"
- Options:
  - "Write missing tests now"
  - "Defer to a follow-up ticket and proceed to verify"
  - "Abort — I'll handle this manually"

If "Write missing tests now": write the test(s), re-run the test command to confirm they pass, re-run the coverage check. If PASS, proceed to `/qsos-verify`.

If "Defer": create a follow-up ticket (next QSO number) scoped to "add missing tests for <function/scenario>" with `priority: high` and `depends_on: []`. Note it in the coverage report, then proceed to `/qsos-verify` with an explicit `COVERAGE DEFERRED` annotation.

---

## Blocking rule

**You may not advance to `/qsos-verify` with HIGH posture gaps, uncovered pure functions, or uncovered happy-path scenarios unless the user has explicitly chosen deferral and a follow-up ticket has been created.** MEDIUM and LOW posture gaps do not block but must appear in the report. Silent omission of gaps is not acceptable.
