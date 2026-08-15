---
id: QSO-002
title: Define five QSOS agent definitions
status: done
priority: high
type: feat
impact_scope:
  - qsos/agents/
features:
  - docs/features/agents.feature
adrs:
  - docs/decisions/ADR-002-agent-definitions.md
architecture_updated: true
depends_on:
  - QSO-001
---

Create five agent definition files in `qsos/agents/`. Each specifies model tier, tool restrictions, and a system prompt externalising the role persona currently duplicated inline across skill dispatch calls.

- `product-owner.md` — sonnet, Read/Write/Edit only; Gherkin rules, lifecycle tags, scenario completeness check
- `architect.md` — sonnet, Read/Write/Edit/Bash; MADR v4, 6-month reversal test, boundary constraint vocabulary
- `code-reviewer.md` — sonnet, Read/Bash only (no Write); JSON finding schema, confidence gates, no-write constraint
- `security-reviewer.md` — sonnet default / opus on --deep, Read/Bash only; CSO persona, security checklist categories, diff-scoped by default
- `verifier.md` — sonnet, Read/Bash/Write; full evidence type catalog from qsos-verify.md, blocking language rule
- `deploy.sh` (QSO-001) updated to deploy agents to `~/.claude/agents/`
