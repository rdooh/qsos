# QSOS Skill Chain Design

This document captures the design decisions and current state of the QSOS workflow chain.
For artifact formats, file locations, and cross-artifact relationships, see
`docs/standards/project-structure.md` — that is the canonical reference.

---

## The chain

```
/qsos-brainstorm → /qsos-feature-doc → /qsos-architecture → /qsos-orient → /qsos-plan → [approve]
    → /qsos-implement          → /qsos-coverage-check → /qsos-review → /qsos-verify → /qsos-validate (optional) → /qsos-doc-sync
    → /qsos-implement-fan ↗
```

`/qsos-implement-fan` is an opt-in parallel variant of `/qsos-implement`. Both feed into the same downstream chain. Use `/qsos-implement-fan` when the approved plan has 3+ independent items; use `/qsos-implement` (sequential default) otherwise.

`/qsos-task` is not a stage — it is a cross-cutting adapter called by every skill that reads or writes task state.
`/qsos-bug` is an entry point for bug triage that feeds back into `/qsos-feature-doc`.
`/qsos-audit` is a compliance check that can run at any point in the chain.
`/qsos` is the orchestrator that reads project state and drives the chain forward.

---

## Skill inventory

### Chain skills

**`/qsos-brainstorm`** — Top of chain. Interactive scoping session. Produces: draft ticket (`work/`), draft feature file (`@proposed`). Reads existing feature files and ADRs before asking any questions — not a blank-slate tool.

**`/qsos-feature-doc`** — Pre-implementation docs gate. Enforces feature file + ADR exist, runs 4-check audit (terminology, state assumptions, behavioral contradiction, scope creep), promotes `@proposed` → `@accepted`. Three modes: new / change / bug.

**`/qsos-architecture`** — Owns `docs/architecture/architecture.dsl`. Updates Current/Target tagged elements when a feature changes the structural model. Creates or updates ADRs for architectural decisions. Verifies ADR↔DSL cross-coverage after every update.

**`/qsos-orient`** — Project-aware context loader. Given a ticket ID or task description, loads ticket, feature files, ADRs, and DSL into context. Flags stale docs and unresolved gates. Runs before `/qsos-plan`.

**`/qsos-plan`** — Implementation planning gate. Maps feature scenarios to deliverables, surfaces risks and architectural constraints, requires human approval before any code is written.

**`/qsos-implement`** — Coding phase contract (sequential default). Follows the approved plan item by item, marks ticket `in-progress`, flags deviations before making them, does not declare done without `/qsos-verify`. Suggests `/qsos-implement-fan` when the plan has 3+ independent items.

**`/qsos-implement-fan`** — Parallel coding variant. Spawns an Opus subagent to decompose the approved plan into isolated sub-jobs (with TDD contracts and a dependency graph), writes the manifest to `.qsos/manifest.json`, then fans out Sonnet subagents in isolated worktrees. Main agent merges results, runs the full test suite, and hands off to `/qsos-coverage-check`. Use when the plan has 3+ items touching distinct files with no shared exported interfaces.

**`/qsos-verify`** — Post-implementation evidence gate. Evidence-typed catalog: UI, API, unit/integration test, log, perf, data, build, CLI, contract/schema, statechart/lifecycle. Verdicts: CONFIRMED / UNCONFIRMED / INCONCLUSIVE.

**`/qsos-validate`** — Human-in-the-loop validation checklist with CTRF JSON output. Optional step after verify, before doc-sync. Run when the developer wants structured human sign-off on the implementation before the chain closes.

**`/qsos-doc-sync`** — Post-implementation reconciliation. Checks behavioral drift, DSL duality, ADR completeness, stale lifecycle tags, and code dependency drift. Closes ticket and sets feature `@done`. Delegates to compliance tooling when available.

### Cross-cutting skills

**`/qsos-task`** — Task tracking adapter. Resolves medium (Jira → QSO ticket files → local plan → ask), exposes consistent interface: find / read / start / update / close. All chain skills call this rather than touching ticket systems directly.

**`/qsos-bug`** — Bug triage entry point. Reproduce → classify (gap vs. conflict) → update feature file → hand to `/qsos-plan`. Does not implement the fix.

**`/qsos-audit`** — Compliance pre-flight. Checks ADR integrity (naming, sequence, required sections), Gherkin style (10 rules), feature lifecycle consistency (stale tags, skipped acceptance, orphans), and DSL coverage. Delegates to compliance tooling when available; runs manual checks otherwise. Can run at any point.

**`/qsos`** — Orchestrator. Reads project state, determines chain entry point, drives forward unattended, pauses only at human gates (plan approval, BLOCKED verdicts, scoping questions).

### Domain-specific tools (not chain skills)

**`/vscode-ext-test`** — Runs Jest unit tests + `@vscode/test-cli` integration smoke test. Gates packaging. Scaffold protocol for extensions without tests.

**`/vscode-ext-load`** — Build → test gate → package → install → verify. Gates on `/vscode-ext-test` passing.

---

## Key design decisions

### Ticket tracking is a cross-cutting concern, not a stage
`/qsos-task` is an adapter, not a workflow step. Every skill that touches task state calls it. Medium is resolved once per session and confirmed with the user. See `project-structure.md` for medium preference order and capability matrix.

### Ticket is a folder
Each ticket lives at `work/QSO-NNN-slug/QSO-NNN-slug.md`. Evidence accumulates alongside it in subfolders. See `project-structure.md` and the document-space quadrant model.

### Architecture and decisions are a separate group from features
Three artifact groups: Requirements+Features / Architecture+Decisions / Contracts+Statecharts. `/qsos-feature-doc` owns group 1. `/qsos-architecture` owns group 2. Group 3 artifacts (contracts, statecharts) are referenced by ADRs — `/qsos-verify` includes evidence types for both.

### Structurizr DSL is the architecture source of truth
Not Mermaid. DSL is semantic-first; Mermaid diagrams are generated outputs. Current/Target duality is expressed via tags in a single file, not two separate files. See `project-structure.md`.

### Feature files carry a lifecycle tag
`@proposed` → `@accepted` → `@in-progress` → `@done`. Nothing ships `@proposed`. `/qsos-feature-doc` sets `@accepted`. `/qsos-implement` sets `@in-progress` via `/qsos-task`. `/qsos-doc-sync` sets `@done` after `/qsos-verify` confirms.

### Skills prescribe; compliance tooling monitors
Skills tell the agent what to produce. Compliance tooling (when installed) audits the artifacts after the fact. `/qsos-audit` is the bridge — it runs the checks manually when tooling is absent, and delegates when it is present. The handoff point is `/qsos-doc-sync`.

### Agent OS relationship
Agent OS `/shape-spec` is the prior art for `/qsos-brainstorm`. Key difference: `/shape-spec` produces a spec folder; `/qsos-brainstorm` produces a ticket and a feature file in standard locations. `/qsos-orient` supersedes Agent OS `/inject-standards` for QSOS-governed projects — it is project-artifact-aware where `/inject-standards` is generic.

---

## Current state

All chain skills are built and installed. The chain is complete end-to-end:

| Skill | Status |
|---|---|
| `/qsos` | ✓ built |
| `/qsos-task` | ✓ built |
| `/qsos-brainstorm` | ✓ built |
| `/qsos-feature-doc` | ✓ built |
| `/qsos-architecture` | ✓ built |
| `/qsos-orient` | ✓ built |
| `/qsos-plan` | ✓ built |
| `/qsos-implement` | ✓ built |
| `/qsos-implement-fan` | ✓ built |
| `/qsos-coverage-check` | ✓ built |
| `/qsos-review` | ✓ built |
| `/qsos-verify` | ✓ built |
| `/qsos-validate` | ✓ built |
| `/qsos-doc-sync` | ✓ built |
| `/qsos-bug` | ✓ built |
| `/qsos-audit` | ✓ built |
