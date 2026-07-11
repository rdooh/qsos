# ADR-002: Introduce named agent definitions for QSOS specialist roles

**Date:** 2026-07-10
**Status:** Accepted
**Decision makers:** Rob Dooh

## Considered Options

- **Option A: Inline personas (status quo)** — continue embedding role personas as prose in each Agent tool dispatch call. Con: duplication across skills; no model tier control; tool restrictions are soft (prompt-only); system prompts not eligible for caching.
- **Option B: Named agent definition files (chosen)** — define roles once in `.claude/agents/*.md` with model tier, tool restrictions, and stable system prompt. Pro: single definition, cacheable, enforced tool restrictions, right-sized model per role. Con: new artifact type to maintain; `subagent_type` mismatch silently falls back to general-purpose.

## Context

QSOS skills that dispatch sub-agents (currently none in the QSOS chain; common in gstack) embed role personas as inline prose in each dispatch call. This creates three problems:

1. **Duplication** — the same "you are an independent senior architect" framing must be written each time the role is invoked
2. **No model tier control** — without agent definitions, all dispatched agents inherit the session model. A security reviewer and a brainstorming assistant cost the same per token.
3. **No tool restriction** — a code reviewer that cannot be given write access by configuration must rely on the prompt saying "don't write files" — a soft constraint

Claude Code agent definition files (`.claude/agents/*.md`) solve all three: stable system prompts eligible for caching, explicit model tier, and declarative tool restrictions.

## Decision

Five named agent definitions are introduced in `qsos/agents/`:

| Agent | Model | Role |
|---|---|---|
| `product-owner` | sonnet | BDD practitioner — feature files, Gherkin, lifecycle |
| `architect` | sonnet | ADR authoring, trade-off analysis, boundary constraints |
| `code-reviewer` | sonnet | Post-implement diff review — structured JSON findings, read-only |
| `security-reviewer` | sonnet (--deep → opus) | CSO-mode security audit — read-only, diff-scoped by default |
| `verifier` | sonnet | Evidence gathering — full evidence type catalog, verdict issuance |

`deploy.sh` deploys agent files to `~/.claude/agents/` alongside skills at `~/.claude/commands/`.

## Consequences

**Positive:**
- Role personas defined once; system prompt caching reduces per-invocation token cost
- Model tier right-sized per role (security deep-mode can escalate to Opus; mechanical tasks stay on Sonnet)
- Tool restrictions enforced at the configuration layer, not just the prompt layer
- Skills that dispatch agents become shorter — no inline persona prose

**Negative:**
- Agent definitions are a new artifact type to maintain — changes to a role require updating the agent file, not just the skill
- `subagent_type` must match the deployed filename exactly — a naming mismatch silently falls back to general-purpose

**Neutral:**
- Skills that don't dispatch agents are unaffected
- The five agents cover the roles identified in the current roadmap; additional agents can be added as new specialist roles emerge

## 6-month reversal test

Reverting to inline personas is straightforward — paste the system prompt back into each dispatch call. The cost is duplication, not architectural lock-in. This decision carries low reversal risk.
