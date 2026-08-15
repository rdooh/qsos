# Project Structure Standards

This document is the canonical reference for how projects in this ecosystem are structured. It defines:
- What artifact families exist and where they live
- The document-space quadrant model
- Artifact formats, lifecycle tags, and cross-artifact relationships
- How skills use them
- How Strux monitors them

Every skill in `common-skills` that touches documentation, architecture, or task tracking references this file. When a convention changes, update it here — the skills inherit the change.

---

## The Document-Space Quadrant Model

All project artifacts fall on two axes:

**Axis 1: Durable vs. Point-in-time**
- *Durable* — describes what the system is, was decided, or formally attests to. Read long after it was written.
- *Point-in-time* — relevant during active work. Value decays after the work closes.

**Axis 2: Specification vs. Evidence**
- *Specification* — claims about what should be true (features, decisions, architecture)
- *Evidence* — proof that something was true at a specific moment (test results, screenshots, release attestations)

| | Durable | Point-in-time |
|---|---|---|
| **Specification** | `docs/` | *(empty — git history serves this role)* |
| **Evidence** | `docs/releases/` | `work/<ticket-folder>/` |

**The fourth quadrant is intentionally empty.** Point-in-time specification is answered by `git log`, not a folder. Store the current truth; use version control as the time machine.

This model is defined in Strux ADR-023 and is the authority for all QSOS-governed projects.

---

## The Skill–Strux Division of Labour

**QSOS (work)** tells agents *how to produce correct artifacts* — prescriptive and proactive.

**Strux (personal)** is a parallel R&D project that explored the same artifact standards independently. It validated lint, graph, watcher, and test-ingestion mechanics that QSOS is now rebuilding natively.

They are **permanently separate systems** (work vs. personal IP). They share *standards* (this document, quadrant model) but never share *runtime implementations*. As QSOS utilities mature, skills delegate mechanical checks to QSOS-native tooling — not to Strux.

Until utilities ship, `/qsos-audit` covers the most important checks manually.

---

## Component identity and ticket prefix

Each QSOS-governed project declares its ticket prefix in `catalog-mesh.yaml` at the repository root:

```yaml
metadata:
  prefix: "QSO-"
```

For QSOS itself, the prefix is **`QSO-`**. Ticket IDs, folder names, and markdown filenames all use this prefix: `QSO-019-rust-utilities-scaffold`. The manifest file remains `work/tix-manifest.json` (ecosystem-standard schema name).

See [ADR-011](../decisions/ADR-011-catalog-mesh-ticket-prefix.md).

---

## Standard Project Layout

```
docs/                              — durable specification + durable evidence
  features/                        — Gherkin feature files
  decisions/                       — MADR ADRs
  architecture/
    architecture.dsl               — Structurizr DSL (single source of truth)
    diagrams/                      — Generated Mermaid views (never edit manually)
  contracts/                       — JSON Schema contract files
  statecharts/                     — XState-compatible statechart files
  releases/                        — formal release attestations
  standards/                       — project-specific standards and linting reports

work/                              — point-in-time work (transient)
  tix-manifest.json                — compiled ticket registry (auto-generated)
  QSO-NNN-slug/                    — one folder per ticket
    QSO-NNN-slug.md                — always present; frontmatter + description
    screenshots/                   — UI evidence captured during verification (committed)
    evidence/                      — verify runs, test logs captured during verification (committed)
    logs/                          — debug output (typically git-ignored)

testing/                           — test harness configurations and manifests (never test code)
  manifest.json                    — declarative test harness posture manifest

test-results/                      — transient test runner output (always git-ignored)
  unit.json                        — unit test output (e.g. mocha/jest JSON reporter)
  integration.json                 — integration test output
  coverage.json                    — coverage statistics

.audit-baseline.json               — optional: acknowledged pre-existing violations (see below)
logs/                              — tool status files and diagnostic reports (git-ignored)
```

Each top-level directory carries a `README.md` that self-identifies its quadrant role.

### Directory boundaries and routing rules

> [!IMPORTANT]
> **Test Code vs. Test Configuration**:
> - All test code files (unit, integration, E2E/Playwright) **must live inside the application source tree** (e.g., `src/test/`, `tests/`, or adjacent to the source files).
> - The `testing/` directory is reserved solely for configurations, test harness manifests, and mocks. **Never place executable test suites in `testing/`**.
>
> **Transient Output vs. Committed Evidence**:
> - Machine-readable test runner outputs (`unit.json`, `coverage.json`) are transient and **must go to the `test-results/` directory**. This directory is machine-specific and **must be git-ignored**.
> - Point-in-time proof of success captured during the verification phase (screenshots, curl payloads, CLI execution logs) **must go to the active ticket folder** (`work/QSO-NNN/evidence/` or `work/QSO-NNN/screenshots/`) and **must be committed to git**. This ensures a durable history of the ticket's verification.

---

## Artifact Families

### Group 1 — Requirements + Features
*What does the system do, from the user's perspective?*

| Artifact | Location | Format | Strux sensor |
|---|---|---|---|
| Ticket | `work/QSO-NNN-slug/QSO-NNN-slug.md` | Markdown with YAML frontmatter | `strux-tix` |
| Feature file | `docs/features/feature-name.feature` | Gherkin + lifecycle tags | `gherkin-rules` |

### Group 2 — Architecture + Decisions
*How is the system built, and why were those choices made?*

| Artifact | Location | Format | Strux sensor |
|---|---|---|---|
| Architecture model | `docs/architecture/architecture.dsl` | Structurizr DSL | `diagram-rules` |
| Architectural Decision Record | `docs/decisions/ADR-NNN-slug.md` | MADR | `adr-rules` |

### Group 3 — Contracts + Statecharts
*What are the boundaries and rules between components?*

| Artifact | Location | Format | Strux sensor |
|---|---|---|---|
| Contract | `docs/contracts/CON-NNN-slug.contract.json` | JSON Schema | `contract-rules` |
| Statechart | `docs/statecharts/STATE-NNN-slug.statechart.json` | XState JSON | `statechart-rules` |

### Group 4 — Release Evidence
*What was formally attested at a specific version?*

| Artifact | Location | Format |
|---|---|---|
| Release attestation | `docs/releases/v{version}.md` | Markdown — version, date, evidence pointers |

---

## Feature Files

### Format
QSOS feature files use a hybrid markdown/Gherkin format with YAML frontmatter. The lifecycle tag lives in the frontmatter `status:` field — not as a bare Gherkin `@tag` on line 1. This is the canonical convention for QSOS projects.

```markdown
---
feature: [Feature Title]
ticket: QSO-NNN
status: @proposed
architecture_updated: false
---

# [Feature Title]

## Background

[Why this feature exists and what problem it solves.]

---

## Feature: [Capability name]

**Scenario: [Happy path scenario name]**
  Given [precondition]
  When [action taken]
  Then [expected outcome]

**Scenario: [Error path scenario name]**
  Given [precondition]
  When [action that should fail]
  Then [expected error behavior]
```

Filename matches the feature slug (kebab-case), with `.feature` extension.

### Lifecycle tags

The `status:` frontmatter field carries the lifecycle tag:

| Value | Set by | Meaning |
|---|---|---|
| `@proposed` | `/qsos-brainstorm` | Draft — under discussion, not yet approved |
| `@accepted` | `/qsos-feature-doc` | Approved — implementation may proceed |
| `@in-progress` | `/qsos-implement` | Currently being implemented |
| `@done` | `/qsos-doc-sync` | Implemented and verified |
| `@deprecated` | `/qsos-doc-sync` | No longer active |

**Rule:** A feature file must be `@accepted` before implementation begins. It must not move to `@done` until `/qsos-verify` returns CONFIRMED. Nothing ships `@proposed`.

### Quality rules (enforced by Strux `gherkin-rules`)
- One `Feature:` per file
- No duplicate scenario names within a file
- No duplicate feature names across the project
- `Scenario Outline:` must have an `Examples:` table
- `Background:` must not be empty; not used for single-scenario files
- No duplicate tags on a single scenario or feature block
- Terminology must be consistent across all feature files

### Audit checks (performed by `/feature-doc` until Strux takes over)
1. **Terminology** — same nouns/verbs as existing files for the same concepts
2. **State assumptions** — every `Given` clause is satisfiable by another scenario's `Then`
3. **Behavioral contradiction** — no `Then` contradicts a `Then` in another file for the same trigger
4. **Scope creep** — behavior that belongs to a different feature area (note only, not a blocker)

---

## Architectural Decision Records (ADRs)

### Naming convention
`ADR-NNN-short-slug.md` — three-digit zero-padded number, sequential, no gaps.

### Format (MADR)
```markdown
# ADR-NNN: [Title]

## Status

[Proposed | Accepted | Superseded | Rejected]

**Date:** YYYY-MM-DD
**Decision makers:** [Names]

## Context

[The situation that forces a decision. Constraints. Problem being addressed.
Mention any C4 DSL element names that are affected.]

## Decision

[What was decided. Reference DSL element names where relevant.]

## Considered Options

- **Option A: [Name]** — [description, pros/cons]
- **Option B: [Name]** — [description, pros/cons]

## Consequences

- [What becomes easier]
- [What becomes harder]
- [Known trade-offs]
```

### When to write an ADR
*If this decision were reversed in six months, would it require migrating data, refactoring multiple files, or changing how other features work?* If yes — write an ADR.

---

## Architecture Model (Structurizr DSL)

### Location
`docs/architecture/architecture.dsl` — single file, single source of truth.

### Current / Target duality
Every element is tagged `Current` or `Target`:
- **`Current`** — exists in the codebase now
- **`Target`** — planned; corresponds to an accepted ADR but not yet implemented

**Rule:** Every `Target` element must have a corresponding `Accepted` ADR. Every `Current` element must match an implemented component verifiable in the codebase.

### Generated views
`docs/architecture/diagrams/` — Mermaid files generated from the DSL. Never edit manually.

---

## Tickets

### Ticket as folder
Each ticket is a directory under `work/`:

```
work/QSO-NNN-slug/
  QSO-NNN-slug.md    — always present; named to match directory so open tabs are self-identifying
  screenshots/       — optional
  evidence/          — optional
  logs/              — optional (typically git-ignored)
```

The `QSO-NNN-slug.md` file is the minimum viable ticket. Subfolders accumulate as the work generates artifacts.

### Medium preference order
Skills resolve the task tracking medium in this order:

1. **Jira** — if MCP is configured and a project key is resolvable
2. **QSO ticket files** — if `work/` directory exists with `tix-manifest.json`
3. **Local plan** — if a `plan.md` with checkboxes exists
4. **None** — ask the user to declare

On resolution: *"I'll use [medium] for task tracking — proceed?"* Continue unless redirected.

### Capability matrix
| Operation | Jira | QSO ticket files | Local plan |
|---|---|---|---|
| find eligible work | ✓ | ✓ | ✓ |
| read for direction | ✓ | ✓ | ✓ (limited) |
| create | ✓ | ✓ | ✓ (add checkbox) |
| start (mark in-progress) | ✓ | ✓ | ✓ |
| update (attach artifact/note) | ✓ | ✓ | — |
| link to ADR/feature | ✓ | ✓ | — |
| close with evidence pointer | ✓ | ✓ | ✓ (check off) |
| sprint / priority / watchers | ✓ | — | — |

### QSO ticket file format
```markdown
---
id: QSO-NNN
title: [Ticket Title]
status: [todo | ready | in-progress | done]
priority: [low | medium | high]
type: [feat | fix | chore | refactor]
impact_scope:
  - [packages/component-name]
features:
  - [docs/features/relevant-feature.feature]
adrs:
  - [docs/decisions/ADR-NNN-relevant-decision.md]
architecture_updated: [true | false]
depends_on:
  - [QSO-NNN]
jira: [PROJ-123]           # optional
---

[Description of work. Bullet points for sub-tasks.]
```

### Ticket readiness gates
A ticket is `ready` (eligible for implementation) when:
- Feature file is linked and `@accepted`
- ADR impact has been assessed (`architecture_updated` field populated)
- No open blocking dependencies

The `/plan` skill checks readiness before producing an implementation plan.

---

## Release Evidence

Formal attestations live in `docs/releases/`. Each file covers one released version:

```markdown
---
version: 1.2.0
date: YYYY-MM-DD
verified_by: /verify
---

## Evidence

- Test results: work/QSO-NNN-slug/evidence/unit.json
- Screenshot: work/QSO-NNN-slug/screenshots/post-deploy.png
```

---

## Contracts

### Naming convention
`CON-NNN-slug.contract.json` — sequential, referenced in ADRs and DSL relationship annotations.

### Format
JSON Schema (draft-07). Defines data shape at a component boundary.

---

## Statecharts

### Naming convention
`STATE-NNN-slug.statechart.json` — sequential.

### Format
JSON, XState-compatible. Models the lifecycle of a process or entity.

---

## Audit Baseline

`.audit-baseline.json` is an optional file at the project root. It exists only when a project adopts QSOS standards against an existing codebase that already has violations.

**What it does:** Records pre-existing violations at adoption time so the compliance tooling suppresses them rather than failing the build on day one. New violations — in new files or introduced into edited files — always fail immediately. Baseline entries are removed progressively as the team fixes the legacy issues.

**What agents should know:**
- If the file exists, do not treat it as a problem or flag it as unknown
- Do not create it unless explicitly asked — it is a human decision to acknowledge legacy violations
- Do not add entries to it during normal workflow — it is not a way to suppress legitimate new failures

The file is committed to version control so all contributors share the same suppression set.

---

## Cross-artifact relationships

```
Ticket  ──links to──►  Feature file  ──@accepted before──►  Implementation
   │                       │
   └──links to──►  ADR  ──references──►  DSL element
                    │
                    └──governs──►  Contract / Statechart
```

When `/doc-sync` runs post-implementation, it verifies this graph is internally consistent: tickets closed, feature files `@done`, DSL `Target` elements promoted to `Current`, no orphaned ADRs.

---

## Skill chain reference

| Stage | Skill | Reads | Writes / Updates |
|---|---|---|---|
| Brainstorm | `/brainstorm` | Existing features, ADRs | Draft ticket (`work/`), draft feature (`@proposed`) |
| Feature spec | `/feature-doc` | Feature files, ADRs | Feature file (`@accepted`), ADR if needed |
| Architecture | `/architecture` | `architecture.dsl`, ADRs | `architecture.dsl`, ADR |
| Context load | `/orient` | Ticket, features, ADRs, DSL | Nothing — loads into context |
| Planning | `/plan` | Ticket (readiness), features, ADRs | Nothing — produces plan for approval |
| Implementation | `/implement` | Plan, feature file | Ticket → `in-progress` |
| Coverage check | `/coverage-check` | Change set, feature file, test dirs | PASS or GAPS FOUND — blocks `/verify` on uncovered pure functions and happy-path scenarios |
| Verification | `/verify` | — | Evidence artifact (`work/QSO-NNN/evidence/`) |
| Doc sync | `/doc-sync` | All of the above | Feature → `@done`, DSL Target → Current, ticket → `done` |
| Bug triage | `/bug` | Feature files, ticket | Gap scenario or conflict note in feature file |

`/task` is not a stage — it is a cross-cutting adapter called by every skill that needs to read or write task state.

---

## Agent Definitions

Agent definitions live in `agents/` at the project root and are deployed by `deploy.py` to the appropriate runtime directory (`~/.claude/agents/`, `~/.gemini/config/plugins/qsos/agents/`).

### Format

```markdown
---
name: agent-name
description: One-line description of the agent's role and scope.
model: mid
tools:
  - Read
  - Write
  - Bash
---

[Agent instructions body]
```

### Model tier rule

> [!IMPORTANT]
> The `model:` field in agent source files **must always be an abstract tier** (`low`, `mid`, or `high`). **Never write a concrete model ID** (e.g. `claude-sonnet-5`, `claude-3-5-sonnet`, `gemini-pro`) in a source agent file.

Concrete model IDs are resolved at deploy time from `qsos.config.yml`, which is git-ignored and operator-controlled. This separation means:
- Agent definitions are portable across environments, regions, and API providers
- Model selection is a deliberate operator decision, made in one place
- `deploy.py` will hard-fail if a concrete model ID is detected in a source file, blocking deployment

**Tiers and their intended use:**

| Tier | Intended use |
|---|---|
| `low` | Fast, lightweight tasks — ticket lookups, simple formatting, log parsing |
| `mid` | Standard reasoning — code review, feature doc, plan generation |
| `high` | Complex judgment — security audit, architecture, full codebase analysis |

The concrete models mapped to each tier are defined in `qsos.config.yml` by the operator. See `qsos.config.yml.example` for the template.
