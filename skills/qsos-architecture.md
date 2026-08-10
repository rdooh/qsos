---
description: Architecture model owner — updates the Structurizr DSL and creates ADRs when a feature changes the structural model.
---

# /architecture

## Core Principle

The architecture model is not a diagram — it is a claim about the system. Every element in `architecture.dsl` either exists in the codebase right now (`Current`) or is planned and approved (`Target`). A `Target` element with no ADR is an undocumented assumption. A `Current` element with no corresponding code is a lie. This skill keeps those claims honest.

---

## When this runs

When a feature introduces a new container, component, or relationship. When a `Target` element has been implemented and should be promoted to `Current`. When an architectural decision is being made that changes the structural model. May run in parallel with `/qsos-feature-doc` — architecture and feature specification are sibling concerns, not sequential.

---

Emit the `skill_started` log event:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-architecture','type':'skill_started','data':{}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

## Step 1 — Determine the trigger

Identify which of these applies:

- **New element** — a container, component, or relationship that does not yet exist in the DSL
- **Target → Current** — a previously `Target` element has been implemented
- **Decision** — an architectural choice is being made that affects the model (technology, boundary, protocol)
- **Removal** — an element is being removed from the system
- **Audit** — checking the DSL for correctness without a specific change trigger

State the trigger before proceeding.

---

## Step 2 — Read the current DSL

Read `docs/architecture/architecture.dsl` in full. If it does not exist, note this and create the file with a minimal `workspace {}` scaffold before continuing.

Identify:
- All elements currently in the model
- Which are tagged `Current` vs. `Target`
- Any `Target` elements with no corresponding `Accepted` ADR (a pre-existing gap — note it)
- Any ADRs linked in the context that reference DSL element names

---

## Step 3 — Identify affected elements

Name the specific containers, components, or relationships that need to change. Be precise — use the exact names and IDs from the DSL, or the names being proposed for new elements.

For new elements, confirm: does this warrant a structural model change? Adding a button does not. Adding a new service does. Applying the test from `docs/standards/project-structure.md`: if removing this element in six months would require migrating data or refactoring multiple files, it belongs in the model.

---

## Step 4 — Apply the duality rule

For each affected element, apply:

**New `Target` element:**
- Add to DSL with the `"Target"` tag
- A corresponding `Accepted` ADR is required — if one doesn't exist, create it now (see Step 6)
- Do not add a `Target` element without an ADR — this is a hard stop

**`Target` → `Current` promotion:**
- Change the tag from `"Target"` to `"Current"`
- Verify the implementation exists in the codebase before making this change
- Note in the ADR's Consequences section if appropriate

**New `Current` element:**
- Add to DSL with the `"Current"` tag
- This means it already exists — verify before adding
- Does not require a new ADR unless the decision to add it warrants one

**Removal:**
- Remove the element from the DSL
- Write or update an ADR to record the decision and reason
- Check for other elements that reference the removed one — update their relationships

---

## Step 5 — Update the DSL

Make the minimum changes needed. Preserve all existing content not related to the triggered change. Use Structurizr DSL format per `docs/standards/project-structure.md`.

After updating the DSL, emit the `file_modified` log event:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-architecture','type':'file_modified','data':{'path':'docs/architecture/architecture.dsl'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

DSL rules:
- Single file: `docs/architecture/architecture.dsl`
- `Current` and `Target` elements coexist in the same file — no separate files
- Relationships follow element order: `source -> target "label" "technology" "tag"`
- Do not edit files in `docs/architecture/diagrams/` — they are generated outputs only

---

## Step 6 — ADR check and creation

Apply the 6-month reversal test from `docs/standards/project-structure.md`:

> *If this decision were reversed in six months, would it require migrating data, refactoring multiple files, or changing how other features work?*

If yes — create or update an ADR. If no — skip.

When creating an ADR:
- Use the next sequential number from `docs/decisions/`
- Name: `ADR-NNN-short-slug.md`
- Format: MADR per `project-structure.md`
- Status: `Accepted` (you are recording a decision that has been made, not proposing one)
- Reference the DSL element names in the Decision section

After creating the ADR, emit the `adr_created` log event (replace `<adr-path>` and `<one-line decision summary>` with actual values):

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-architecture','type':'adr_created','data':{'adr_path':'<adr-path>','decision_summary':'<one-line decision summary>'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

---

## Step 7 — ADR coverage check

After updating the DSL, verify cross-coverage in both directions:

**DSL → ADR (justification check)**
For each `softwareSystem`, `container`, or `component` element you added or changed, confirm that at least one accepted ADR references it by name or ID in its body text. If none does, the element is unjustified — create or update an ADR to cover it, or flag `[DSL_UNJUSTIFIED]` in the verdict if deferring.

**ADR → DSL (reference check)**
For each ADR created or updated in this skill run, confirm that its `ADR-NNN` identifier appears somewhere in `architecture.dsl`. If not, add a comment reference in the DSL or note `[ADR_UNLINKED]` in the verdict.

This is a lightweight pass — it does not require reading every existing ADR, only the ones touched in this session and the elements just modified.

---

## Step 8 — Deliver verdict

```
ARCHITECTURE UPDATE

TRIGGER: <new element | Target → Current | decision | removal | audit>

DSL CHANGES:
  - <element name> [tag change: Target → Current | added as Target | added as Current | removed]

ADR: not required | <path> — created | updated — <ADR-NNN: title>

DUALITY AUDIT:
  Target elements with no ADR: none | <list — BLOCKED>
  Current elements unverifiable in code: none | <list — NOTE>

COVERAGE AUDIT:
  DSL elements with no justifying ADR: none | <list — [DSL_UNJUSTIFIED]>
  ADRs not referenced in DSL: none | <list — [ADR_UNLINKED]>

GENERATED VIEWS: not regenerated — run `strux generate-diagrams` to update docs/architecture/diagrams/

VERDICT: GO | BLOCKED — <reason>
```

---

Emit the `skill_completed` log event:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-architecture','type':'skill_completed','data':{'outcome':'go'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

---

## Blocking rule

**You may never add a `Target` element to the DSL without a corresponding `Accepted` ADR.** A planned element without a recorded decision is an undocumented assumption — it will not survive the next architecture review. **You may never manually edit files in `docs/architecture/diagrams/`** — those files are generated by Strux and any manual change will be overwritten. If Strux is not available, note that views are stale and must be regenerated when it is.
