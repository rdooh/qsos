---
description: Compliance pre-flight — check ADR integrity, Gherkin style, feature lifecycle consistency, and DSL coverage at any point in the chain.
---

# /qsos-audit

## Core Principle

The chain produces artifacts. This skill checks them. It is not a gate tied to any single stage — run it anytime: before planning to confirm docs are clean, before closing to confirm nothing drifted, or as a standalone health check on a project you have just inherited.

---

## When this runs

At any point. Recommended before `/qsos-plan` (catch doc problems before implementation) and before `/qsos-doc-sync` (catch drift before closing). Also useful as a standalone audit triggered directly.

---

## Step 1 — Determine scope

Identify what to audit. Three modes:

- **Full** — audit all artifact families in the project (default when invoked standalone)
- **Ticket** — audit only artifacts linked to a specific ticket (run this when anchored to active work)
- **File** — audit a specific feature file, ADR, or DSL (run this when you have just written or changed one artifact)

State the mode and scope before proceeding.

---

## Step 1b — Tier 1: `qsos lint` delegation

If the `qsos` binary is available on PATH, run static checks via the utility instead of manual file reads:

```bash
# Full project (default)
qsos lint --root <project-root>

# Single file mode
qsos lint --file <path> --root <project-root>
```

Parse the JSON `violations` array and map results into the audit report sections below (ADR integrity, Gherkin style, feature lifecycle, DSL coverage). Violation `rule` names correspond to lint rules (e.g. `adr-naming-convention`, `feature-lifecycle-tag`, `dsl-unjustified`).

**When lint succeeds (exit 0):** report those sections as `pass` without re-reading files.

**When lint fails (exit 1):** translate violations into the report format; do not duplicate lint logic manually.

**Fallback:** If `qsos` is not installed or not on PATH, note `utility delegation skipped` and proceed with manual Steps 2–5.

Tier 2 manual checks (always run after Tier 1 when relevant): cross-entity reasoning lint cannot yet perform — e.g. ticket-scoped scope filtering, topical overlap features not linked in manifest, architectural judgement calls.

---

## Step 2 — ADR integrity checks

Read all files in `docs/decisions/`.

Run these checks:

1. **Naming convention** — every file (except `README.md`) must match `ADR-NNN-slug.md` (three-digit zero-padded number, lowercase slug, `.md` extension)
2. **Monotonic sequence** — numbers must start at `001`, contain no duplicates, and have no gaps; flag each missing number individually
3. **Required metadata** — each ADR must have: `Status`, `Date`, `Decision makers` (in frontmatter or as bold inline markers)
4. **Valid status** — must be one of: `Proposed`, `Accepted`, `Superseded`, `Rejected`
5. **Required sections** — must have `## Context`, `## Decision`, `## Consequences` (non-empty)
6. **Considered options** — `Accepted` and `Superseded` ADRs must have a non-empty `## Considered Options` section
7. **Superseded links** — an ADR with status `Superseded` must reference at least one other ADR number, and that file must exist

Flag each violation with: `[ADR-NNN] rule-name — description`.

---

## Step 3 — Gherkin style checks

Read all `.feature` files in `docs/features/`.

Run these checks per file:

1. **Not empty** — file must not be blank or whitespace-only
2. **One feature per file** — exactly one `Feature:` keyword
3. **Lifecycle tag present** — file must begin with one of `@proposed`, `@accepted`, `@in-progress`, `@done`, `@deprecated`
4. **No duplicate scenario names** — scenario names within a file must be unique
5. **Scenario Outline has Examples** — every `Scenario Outline:` must have an `Examples:` table; standard `Scenario:` must not
6. **Background not used for single-scenario files** — `Background:` requires at least two scenarios
7. **Outline variable alignment** — every `<placeholder>` in steps must exist as an `Examples:` column header, and every column header must appear as a placeholder
8. **Indentation** — `Feature:` at 0, `Scenario:`/`Background:` at 2, steps at 4, table rows at 6
9. **No trailing whitespace** — no line ends with spaces or tabs
10. **Single newline at EOF**

Flag each violation with: `[feature-name.feature:line] rule-name — description`.

---

## Step 4 — Feature lifecycle consistency

Cross-check feature files against ticket state. For each feature file:

1. **Stale `@proposed`** — if a linked ticket has `status: done`, the feature must not still be `@proposed`. Flag as `[STALE_TAG]`.
2. **Implementation without acceptance** — if a ticket is `in-progress` or `done` but the feature is still `@proposed`, it was never formally accepted. Flag as `[SKIPPED_ACCEPTANCE]`.
3. **Orphaned feature** — feature file exists but is not linked to any ticket. Note as `[ORPHAN]` (not a blocker, but worth knowing).

Read `work/tix-manifest.json` to load ticket states for this check. If the manifest does not exist, note this and skip the cross-check.

---

## Step 5 — DSL coverage check

Read `docs/architecture/architecture.dsl` and all accepted ADRs.

Run these checks:

1. **Every DSL element referenced in an ADR** — each `softwareSystem`, `container`, or `component` element must be referenced by name or ID in at least one accepted ADR's body. Flag unjustified elements as `[DSL_UNJUSTIFIED]`.
2. **Every accepted ADR referenced in the DSL** — each accepted ADR should appear (by `ADR-NNN`) somewhere in `architecture.dsl`. Flag unreferenced ADRs as `[ADR_UNLINKED]` (note only — not a blocker).
3. **Target elements without ADR** — any element tagged `Target` must have a corresponding `Accepted` ADR. Flag missing links as `[TARGET_NO_ADR]` — this is a blocker.

If `architecture.dsl` does not exist, note the absence and skip this step.

---

## Step 6 — Produce audit report

```
QSOS AUDIT REPORT

SCOPE: full | ticket <id> | file <path>

ADR INTEGRITY:
  pass | <N> violation(s):
  - [ADR-NNN] <rule> — <description>

GHERKIN STYLE:
  pass | <N> violation(s):
  - [<file>:<line>] <rule> — <description>

FEATURE LIFECYCLE:
  pass | <N> issue(s):
  - [STALE_TAG] <feature> — ticket <id> is done but feature is still @proposed
  - [SKIPPED_ACCEPTANCE] <feature> — ticket in-progress but feature never reached @accepted
  - [ORPHAN] <feature> — not linked to any ticket

DSL COVERAGE:
  pass | <N> issue(s):
  - [DSL_UNJUSTIFIED] <element> — no accepted ADR references this element
  - [ADR_UNLINKED] ADR-NNN — not referenced in architecture.dsl
  - [TARGET_NO_ADR] <element> — Target tag with no corresponding Accepted ADR — BLOCKED

BLOCKERS: <N> — must resolve before proceeding | none
NOTES: <N> items worth addressing | none

AUDIT VERDICT: CLEAN | ISSUES FOUND — <blocker count> blocker(s), <note count> note(s)
```

---

## Blocking rule

**`[TARGET_NO_ADR]` violations are always blockers** — a Target DSL element with no ADR is an undocumented architectural assumption and must be resolved before implementation proceeds. All other violations are reported and categorised but do not automatically block — the calling skill or the user decides whether to proceed. Do not suppress violations or mark the audit CLEAN when violations exist.
