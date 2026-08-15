---
id: QSO-001
title: Consolidate QSOS into monorepo structure with deploy script
status: done
priority: high
type: chore
impact_scope:
  - qsos/
  - common-skills/
features:
  - docs/features/consolidation.feature
adrs:
  - docs/decisions/ADR-001-monorepo-consolidation.md
architecture_updated: true
depends_on: []
---

Move all QSOS skills from `common-skills/skills/workflow/` into `qsos/skills/`. Establish `qsos/` as the monorepo for all QSOS artifact types (skills, agents, utilities, extension). Replace `common-skills/install.sh` for QSOS artifacts with a new `qsos/deploy.sh`.

- Create `qsos/skills/` (flat — no subdirectories), `qsos/agents/`, `qsos/utilities/`, `qsos/extension/`
- Write `deploy.sh`: symlinks skills → `~/.claude/commands/`, agents → `~/.claude/agents/`; idempotent; cleans stale links; stubs future artifact types; prints per-artifact status (`linked`, `already-ok`, `cleaned`, `skipped`)
- Move all `qsos-*.md` skill files from `common-skills/skills/workflow/` to `qsos/skills/`
- Verify all `~/.claude/commands/qsos-*` symlinks resolve correctly post-migration
- Remove QSOS entries from `common-skills/registry.yml`
- Add `utilities/README.md` and `extension/README.md` stubs noting future artifact locations
