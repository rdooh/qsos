---
description: Interactive scoping session — produce a draft feature file and ticket before any implementation begins.
---

# /brainstorm

## Core Principle

Ideas without structure are not specifications. Before a single line of code is written, the behavior being built must be described in terms of observable outcomes — not technical approaches. This skill converts a raw idea into the two artifacts that gate all subsequent work: a feature file that describes what the system will do, and a ticket that tracks that it gets done.

---

## When this runs

At the very beginning of new work. Before `/qsos-feature-doc`. Before `/qsos-plan`. Before any code.

If you arrive at this skill mid-implementation, stop — implementation should not have started without these artifacts. Surface the gap before continuing.

---

## Step 1 — Load existing context

Before asking a single question, read the project's existing documentation:

- All files in `docs/features/*.feature` — understand current system vocabulary (nouns, verbs, roles) and what behaviors already exist
- All files in `docs/decisions/ADR-*.md` — understand architectural decisions already made

If neither directory exists, note that this appears to be a new project and proceed. Do not create the directories yet — that happens in Step 4.

---

## Step 2 — Scope the idea

Ask the following, in order, waiting for a response before moving on:

1. **What are we building?** Describe the feature or change in plain language.
2. **Is this new behavior or a change to existing behavior?** (New feature / modifying existing / fixing a bug — if bug, use `/qsos-bug` instead)
3. **What is the expected outcome when this is done?** What will a user be able to do that they couldn't before, or what will work that was broken?
4. **Are there any constraints, dependencies, or things that must not change?**

If the answers are vague, ask one follow-up to sharpen them. Do not ask more than five questions total — capture enough to write a meaningful feature file, not an exhaustive specification.

---

## Step 3 — Check for overlaps and conflicts

Using the context loaded in Step 1, assess:

- **Overlap** — does this idea describe behavior already covered (fully or partially) by an existing feature file? If yes, this may be a `change` — redirect to `/qsos-feature-doc` in `change` mode instead.
- **Conflict** — does this idea contradict an existing ADR? (e.g. choosing a different persistence strategy than one already decided) If yes, surface the conflict before proceeding. Do not draft a feature file that assumes a decision that contradicts an existing ADR.
- **Vocabulary mismatch** — does the user's description use different terms for concepts that existing feature files name differently? Align on the correct terms now, before they are embedded in the draft.

State what was found: overlaps noted, conflicts found (BLOCKED), or clean.

---

## Step 4 — Draft the feature file

Create `docs/features/<slug>.feature` (create `docs/features/` if it does not exist).

Use the format from `common-skills/standards/project-structure.md`:

```gherkin
@proposed
Feature: <title matching the idea from Step 2>
  As a <role>
  I want to <action>
  So that <outcome>

  Scenario: <core happy path>
    Given <precondition>
    When <action>
    Then <expected outcome>

  Scenario: <meaningful edge case or error path>
    Given <precondition>
    When <action>
    Then <expected outcome>
```

Rules:
- Tag is `@proposed` — this is a draft, not yet approved for implementation
- Use vocabulary from existing feature files for shared concepts
- At minimum: one happy path scenario and one edge case or error path
- Do not describe implementation details — describe observable behavior only
- Present the draft to the user before writing the file; incorporate any corrections

---

## Step 5 — Draft the ticket

Determine the next TIX number by reading `work/tix-manifest.json` (or start at TIX-001 if none exists).

Create `work/TIX-NNN-<slug>/ticket.md` using the format from `common-skills/standards/project-structure.md`:

```markdown
---
id: TIX-NNN
title: <title>
status: todo
priority: medium
type: feat
impact_scope:
  - <affected area>
features:
  - docs/features/<slug>.feature
adrs: []
architecture_updated: false
depends_on: []
---

<Description from the scoping conversation. Include the expected outcome.>
```

Then call `/task update` to register the new ticket in the active medium.

---

## Step 6 — Deliver verdict

```
BRAINSTORM VERDICT: READY FOR /feature-doc | NEEDS CLARIFICATION — <reason>

FEATURE FILE: docs/features/<slug>.feature [@proposed] — created
TICKET: work/TIX-NNN-<slug>/ticket.md [todo] — created
OVERLAPS: none | <description>
CONFLICTS: none | BLOCKED — <ADR reference and nature of conflict>
VOCABULARY ALIGNED: yes | <terms adjusted>

Next step: run /feature-doc new to audit and promote @proposed → @accepted.
```

---

## Blocking rule

**You may not produce implementation code during this skill.** You may not write a feature file without first reading existing ones — a feature file written in ignorance of the system's vocabulary and existing behavior is worse than no feature file. You may not create a ticket without a linked feature file. If a conflict with an existing ADR is found, the verdict is BLOCKED — do not proceed until the conflict is resolved.
