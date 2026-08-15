# QSOS Skills Roadmap

## What a skill is

A QSOS skill is a set of instructions that governs how an AI agent behaves during a specific stage of work. Skills are loaded into the agent's context as slash commands. They tell the agent what to produce, in what order, with what checks, and when to stop and ask.

Skills are the primary delivery mechanism for QSOS. They require no installation beyond placing a markdown file in the right directory. They work with any Claude Code session, in any project, immediately.

---

## Current state — the full chain exists

The complete chain from idea to verified delivery is built and operational.

### Orchestrator

**`/qsos`** — Given a project state or a direction ("use QSOS to advance this"), reads ticket and feature file state, determines the right entry point in the chain, and drives forward unattended until a human gate is reached. The single command that replaces step-by-step invocation.

### Chain skills

| Skill | What the agent can do |
|---|---|
| `/qsos-brainstorm` | Scope a new idea, produce a draft feature file and ticket before any code exists |
| `/qsos-feature-doc` | Audit and formally accept a feature specification; write ADRs where warranted |
| `/qsos-architecture` | Update the architecture model and ensure every structural element is justified by a decision |
| `/qsos-orient` | Load all relevant context for a ticket — feature files, ADRs, architecture — before planning |
| `/qsos-plan` | Map each specified behavior to a concrete deliverable; present for human approval |
| `/qsos-implement` | Execute the approved plan; flag deviations; mark ticket in-progress |
| `/qsos-verify` | Gather typed evidence that the claimed outcome was achieved |
| `/qsos-doc-sync` | Close the loop — reconcile spec, architecture, and ticket after verification |
| `/qsos-bug` | Triage a bug — reproduce, classify as gap or conflict, update docs, hand to plan |
| `/qsos-task` | Resolve the active ticket tracking medium and provide a consistent interface |
| `/qsos-audit` | Run a compliance check across the artifact set at any point in the chain |

---

## Near-term: utilities delegation (active program)

The chain is complete. The active work is wiring skills to native utilities (ADR-010, QSO-027):

| Skill | Utility delegation | Ticket |
|---|---|---|
| `/qsos-audit` | Tier 1 → `qsos lint` | QSO-020, QSO-027 |
| `/qsos-orient` | Context → `qsos query --ticket` | QSO-023, QSO-027 |
| `/qsos-doc-sync` | Pre-close → `qsos lint [--sync]` | QSO-020, QSO-021, QSO-027 |
| `/qsos-verify` | Coverage → `qsos ingest` + graph query | QSO-026, QSO-027 |
| `/qsos-coverage-check` | Untested scenarios → graph query | QSO-026, QSO-027 |

See [implementation-roadmap.md](./implementation-roadmap.md) for the full utilities program.

---

## Future: skills to extend or add

### Contracts and statecharts

The evidence catalog in `/qsos-verify` includes contract schema validation and statechart transition coverage — but no skill currently drafts these artifacts. A `/qsos-contract` and `/qsos-statechart` skill would complete Group 3 of the artifact model, bringing the same specification-before-implementation discipline to component boundaries and process lifecycles.

### Release attestation

`docs/releases/` exists in the standard layout but no skill currently produces release attestation documents. A `/qsos-release` skill would formalise the act of cutting a release — collecting evidence pointers, confirming all linked tickets are closed, and producing a durable record that a version was verified.

### Onboarding

A `/qsos-init` skill that sets up the standard directory structure, creates `docs/` and `work/` scaffolding, places folder READMEs, and optionally creates an `.audit-baseline.json` for projects with pre-existing violations. Reduces the cost of adopting QSOS on an existing project to a single command.

---

## Longer term: skills that delegate to utilities

As programming utilities come online (see [utilities.md](./utilities.md) and [implementation-roadmap.md](./implementation-roadmap.md)), individual skills become simpler — invoke a command, interpret JSON output, retain manual fallback.

QSO-027 tracks the skill updates. The agent interface does not change when utilities arrive.
