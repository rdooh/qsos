---
description: Bug triage entry point — reproduce, classify, update docs, then hand off to /plan before any fix is written.
---

# /bug

## Core Principle

A bug is not just broken code — it is a gap or conflict in the system's specification. Either the spec said nothing about this case (gap), or the code contradicts what the spec says (conflict). Both are doc problems as much as code problems. Fixing the code without fixing the docs leaves the next agent to make the same mistake. This skill forces the classification before the fix, so the fix is never applied to a misdiagnosed problem.

---

## When this runs

When a bug is reported, observed, or encountered during implementation. This skill replaces ad-hoc bug investigation — use it before touching any code. For bugs found mid-implementation, complete the current action, then run this skill before continuing.

---

## Step 1 — Describe the bug

State in one sentence:

> "Observed: <what the system actually does>. Expected: <what it should do>."

If the bug was reported by a user or in a ticket, quote the report. Do not paraphrase in a way that assumes cause — keep the description behavioral.

---

## Step 2 — Find or create a ticket

Call `/task find` — is there an existing open ticket for this bug? Use these signals: same component, same symptom, same error message.

- **Match found** — call `/task read` to load the full ticket and any linked feature files; confirm this is the same bug before using the ticket
- **No match** — create a new ticket: `type: fix`, `status: todo`, `priority:` determined by impact. Do not link a feature file yet — that happens in Step 5.

---

## Step 3 — Reproduce and capture evidence

Reproduce the bug before diagnosing it. Do not theorize the cause from code reading alone — verify the failure is real and repeatable.

Capture a failure artifact:
- Test failure output (preferred — reproducible by any agent)
- Log or console output showing the error
- Screenshot of visible wrong state (for UI bugs)
- HTTP response demonstrating wrong behavior (for API bugs)

If you cannot reproduce the bug, state that explicitly. Do not proceed to diagnosis on an unreproducible bug — investigate reproduction first.

---

## Step 4 — Locate the feature file

Find the `.feature` file that covers the area where the bug occurs. Search `docs/features/` for:
- Features referencing the component or system area
- Features that include scenarios whose `Then` clause describes the expected behavior

If no feature file covers this area:
- Note the absence — this is itself a gap; the behavior was unspecified
- Continue to classification; the resolution will add a new scenario

---

## Step 5 — Classify the bug

Exactly one classification applies:

**Gap** — the bug represents a case the feature file does not cover. The spec is silent on what should happen. The code is not contradicting the spec — the spec simply never addressed this scenario.

**Conflict** — the code's behavior directly contradicts a scenario in the feature file. There is a `Then` clause that says X should happen, and the code produces Y instead.

State the classification and cite the specific scenario (or its absence).

---

## Step 6 — Update the feature file

**If Gap:**
- Add a new scenario to the relevant `.feature` file describing the correct expected behavior
- Use `@in-progress` or `@accepted` lifecycle — this scenario is being addressed now
- The scenario must describe observable behavior, not implementation

**If Conflict:**
- Read the conflicting scenario carefully
- Determine which is authoritative: was the spec correct and the code drifted? Or did the code implement a better behavior and the spec is outdated?
- If spec is authoritative: mark the scenario clearly as the requirement that must be met; do not change it
- If code is authoritative (better behavior): update the scenario to reflect the actual correct behavior; note the update in a comment or ticket
- If unclear: surface to the user before changing either

If no feature file existed (absence noted in Step 4), create one now with a `@proposed` tag. Then run `/qsos-feature-doc change` to audit and promote it.

---

## Step 7 — Run `/qsos-feature-doc` audit

Call `/qsos-feature-doc change` on the updated feature file. This runs the 4-check audit: terminology, state assumptions, behavioral contradiction, scope creep. Do not proceed to planning until `/qsos-feature-doc` returns GO.

---

## Step 8 — Hand off to `/qsos-plan`

Update the ticket:
- Link the updated feature file (`features:` frontmatter)
- Set status to `ready`
- Attach the failure artifact from Step 3 as an update note

Call `/task update` to record the artifact.

Then run `/qsos-plan` — do not implement the fix directly. The fix requires an approved plan.

```
BUG TRIAGE VERDICT: READY FOR /plan | BLOCKED — <reason>

OBSERVED: <one-sentence description>
EXPECTED: <one-sentence description>
REPRODUCED: yes — <artifact type> | no — <why not>
FEATURE FILE: <path> — gap scenario added | conflict resolved | created
CLASSIFICATION: Gap | Conflict
FEATURE-DOC: GO | <blocked reason>
TICKET: <id> [ready]
```

---

## Blocking rule

**You may not implement the fix before `/qsos-feature-doc` returns GO and `/qsos-plan` is approved.** Reproducing the bug and diagnosing it is allowed before touching any docs. But the moment a code change is made to fix it, those docs must already be correct. A bug fix without a corresponding spec update is a fix that will drift again.
