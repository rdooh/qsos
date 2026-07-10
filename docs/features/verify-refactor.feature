---
feature: Verify Refactor and Doc-Sync Update
ticket: TIX-005
status: @done
architecture_updated: false
---

# Verify Refactor and Doc-Sync Update

## Background

`qsos-verify` currently contains both the dispatcher logic (load context, invoke verification) and the domain expertise (evidence type catalog, blocking rules). As agent definitions are introduced, the expertise belongs in the agent and the skill becomes a thin context-loading dispatcher. Separately, `qsos-doc-sync` has no explicit check for unrecorded architectural decisions made during implementation.

---

## Feature: qsos-verify as thin dispatcher

**Scenario: qsos-verify dispatches verifier agent with loaded context**
  Given qsos-implement has completed with a claim statement
  When qsos-verify runs
  Then it reads the active ticket ID, the claim from the implementation block, and the project's test runner
  And it dispatches the verifier agent with that context
  And the verifier agent applies the evidence type catalog to select the right evidence type
  And the verdict is returned as CONFIRMED, UNCONFIRMED, or INCONCLUSIVE

**Scenario: qsos-verify works standalone without implementation context**
  Given no qsos-implement completion block is present in context
  When the user invokes `/qsos-verify` directly
  Then qsos-verify asks the user to state the claim being verified
  And proceeds to dispatch the verifier agent with that claim

**Scenario: INCONCLUSIVE verdict triggers escalation**
  Given the verifier agent returns INCONCLUSIVE
  When qsos-verify receives the verdict
  Then it surfaces the ambiguity and the artifact to the user
  And it asks what additional evidence would resolve it
  And it does not proceed to qsos-doc-sync

**Scenario: Behavioral parity before and after refactor**
  Given the same claim and project context
  When qsos-verify is run before and after the refactor
  Then the verdict type, evidence type selected, and output format are equivalent

---

## Feature: qsos-doc-sync unrecorded decision check

**Scenario: Unrecorded architectural decision discovered post-implementation**
  Given qsos-doc-sync is running the post-implementation reconciliation
  When it performs the unrecorded decision check
  And an architectural choice was made during implementation that has no corresponding ADR
  Then it routes to qsos-architecture before closing the ticket
  And it states: "Unrecorded decision detected — capturing before close"

**Scenario: No unrecorded decisions — check passes cleanly**
  Given qsos-doc-sync is running the post-implementation reconciliation
  When it performs the unrecorded decision check
  And all architectural choices during implementation are already recorded
  Then it states: "No unrecorded decisions" explicitly
  And it proceeds to ticket close

**Scenario: Unrecorded decision check is distinct from architecture model update**
  Given qsos-doc-sync performs both steps
  Then the architecture model update step updates existing planned elements to current
  And the unrecorded decision check asks: "did we make NEW decisions not yet in any ADR?"
  And the two steps do not overlap or substitute for each other
