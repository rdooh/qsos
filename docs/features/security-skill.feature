---
feature: QSOS Security Skill
ticket: TIX-004
status: @done
architecture_updated: false
---

# QSOS Security Skill

## Background

Security review is not routine overhead — it is a deliberate quality gate triggered when risk warrants it. `qsos-security` is an optional chain skill that activates when the architect flags a plan as security-sensitive, or when the user explicitly requests it. It never runs automatically on routine fixes.

---

## Feature: Activation heuristic

**Scenario: Skill activates when architect flag is present**
  Given `qsos-plan` output contains `SECURITY_REVIEW: recommended`
  When the chain reaches the post-review position
  Then `qsos-security` activates automatically
  And it states the reason: "Architect flagged this plan as security-sensitive"

**Scenario: Skill declines on routine work**
  Given the user invokes `/qsos-security` on a feature with no security-sensitive characteristics
  And no `SECURITY_REVIEW: recommended` flag is present in the plan
  When qsos-security evaluates the request
  Then it declines with a brief explanation of what would trigger it
  And it suggests running `/qsos-verify` instead

**Scenario: Skill always runs on explicit invocation**
  Given the user explicitly types `/qsos-security`
  Then the skill runs regardless of whether the architect flagged it
  And it notes: "Running on explicit request"

---

## Feature: Default mode — diff-scoped sonnet review

**Scenario: Security review runs in default mode**
  Given qsos-security has been activated
  When it dispatches the security-reviewer agent without flags
  Then the agent reviews only files changed in the current implementation diff
  And the agent uses sonnet model tier
  And the agent produces a Security Posture Report

**Scenario: CRITICAL security finding unconditionally halts the chain**
  Given the security-reviewer returns a CRITICAL finding
  When qsos-security processes the report
  Then it halts the chain with: "SECURITY: BLOCKED — critical finding requires remediation"
  And it displays the finding and recommended remediation
  And it routes back to qsos-implement
  And there is no bypass mechanism

---

## Feature: Deep mode — opus whole-repo review

**Scenario: Deep mode dispatches opus against whole repo**
  Given the user invokes `/qsos-security --deep`
  When qsos-security processes the flag
  Then it dispatches the security-reviewer agent with opus model tier
  And the agent scopes its review to the entire repository
  And it notes the expanded scope and model cost before proceeding

---

## Feature: Architect flagging guidance

**Scenario: Architect agent flags a plan with auth changes**
  Given qsos-plan produces a plan that introduces a new authentication mechanism
  When the architect agent reviews architectural constraints
  Then the plan output includes `SECURITY_REVIEW: recommended`
  And the reason cites: "New authentication mechanism introduced"

**Scenario: Architect agent flags a plan with external API integration**
  Given qsos-plan produces a plan that adds an outbound API call or inbound webhook
  When the architect agent reviews architectural constraints
  Then the plan output includes `SECURITY_REVIEW: recommended`
  And the reason cites: "External API trust boundary introduced"
