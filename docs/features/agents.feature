---
feature: QSOS Agent Definitions
ticket: QSO-002
status: @done
architecture_updated: true
---

# QSOS Agent Definitions

## Background

QSOS skills currently embed role personas as inline prose in each dispatch call. The same "you are an independent senior architect" framing is duplicated across multiple skills. Agent definition files externalise these roles: stable system prompts, model tier selection, and tool restrictions — defined once, reused everywhere, eligible for prompt caching.

---

## Feature: product-owner agent

**Scenario: product-owner agent is dispatched for feature authoring**
  Given the product-owner agent is deployed to `~/.claude/agents/`
  When a skill dispatches the agent with a feature authoring task
  Then the agent applies Gherkin best practices (behavior not implementation, declarative language)
  And the agent checks scenario completeness: happy path, error path, at least one edge case
  And the agent enforces the `@proposed → @accepted` lifecycle before accepting a feature
  And the agent does not make shell calls or modify non-document files

---

## Feature: architect agent

**Scenario: architect agent is dispatched for ADR authoring**
  Given the architect agent is deployed to `~/.claude/agents/`
  When a skill dispatches the agent with an architectural decision to record
  Then the agent produces a valid MADR v4 format ADR
  And the agent applies the 6-month reversal test before recommending a decision
  And the agent cross-references existing ADRs for conflicts or dependencies
  And the agent can grep the repository to inspect current interfaces and boundaries

---

## Feature: code-reviewer agent

**Scenario: code-reviewer agent produces structured findings**
  Given the code-reviewer agent is deployed to `~/.claude/agents/`
  When a skill dispatches the agent against an implementation diff
  Then the agent outputs findings as one JSON object per line
  And each finding contains: severity, confidence, path, category, summary
  And optional fields (line, fix, fingerprint) are included when determinable
  And findings with confidence below 3 are suppressed entirely
  And the agent does not modify any files

**Scenario: code-reviewer agent finds no issues**
  Given a clean implementation diff
  When the code-reviewer agent reviews it
  Then the agent outputs `NO FINDINGS` and nothing else

---

## Feature: security-reviewer agent

**Scenario: security-reviewer agent runs in default mode**
  Given the security-reviewer agent is deployed to `~/.claude/agents/`
  When a skill dispatches the agent without flags
  Then the agent scopes its review to the implementation diff only
  And the agent uses sonnet model tier
  And the agent produces a Security Posture Report
  And the agent does not modify any files

**Scenario: security-reviewer agent runs in deep mode**
  Given a skill dispatches the security-reviewer agent with --deep flag
  Then the agent uses opus model tier
  And the agent expands scope to the whole repository
  And findings confidence threshold is lowered (more findings surfaced)

---

## Feature: verifier agent

**Scenario: verifier agent issues a CONFIRMED verdict**
  Given the verifier agent is deployed to `~/.claude/agents/`
  When a skill dispatches the agent with a claim and evidence type
  Then the agent selects the appropriate evidence type from its catalog
  And the agent gathers actual evidence (does not summarise or infer)
  And if evidence unambiguously confirms the claim, the agent issues `CONFIRMED`
  And the agent includes the evidence artifact inline or as a file reference

**Scenario: verifier agent refuses to declare done without evidence**
  Given the verifier agent has been dispatched
  When the agent cannot gather evidence (tool unavailable, test suite missing)
  Then the agent stops and surfaces the gap explicitly
  And the agent does not use the words "done", "fixed", "resolved", "working", or "complete"
  And the agent asks whether to set up the missing tool or proceed with lower-confidence check
