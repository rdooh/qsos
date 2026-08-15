---
id: QSO-010
title: Model tier enforcement — prevent concrete model IDs in agent source files
status: done
priority: high
type: feat
impact_scope:
  - deploy.py
  - docs/standards/project-structure.md
features: []
adrs:
  - docs/decisions/ADR-005-agent-model-tier-config.md
architecture_updated: false
depends_on:
  - QSO-008
---

Prevent future AI sessions from silently writing concrete model IDs (e.g. `claude-sonnet-5`)
into agent source files, undoing the tier abstraction introduced in QSO-008 and causing
unexpected cost.

Three changes:

1. `deploy.py` — `validate_agent_sources()` scans all files in `agents/` before any write
   and hard-fails if any `model:` field contains a value not in `{low, mid, high}`. Runs
   in both deploy and --check modes.

2. `deploy.py` — tier resolution table printed before any writes. Confirmation prompt
   required before deployment proceeds or artifacts are cleaned. `--check` is unaffected
   (read-only, no prompt).

3. `docs/standards/project-structure.md` — new Agent Definitions section codifies the
   model tier rule in the canonical standards document read by all QSOS-aware AI sessions.
