---
id: QSO-004
title: Add qsos-security skill — optional architect-triggered security gate
status: done
priority: medium
type: feat
impact_scope:
  - qsos/skills/
features:
  - docs/features/security-skill.feature
adrs: []
architecture_updated: false
depends_on:
  - QSO-001
  - QSO-002
---

Create `qsos/skills/qsos-security.md` — optional chain skill between `qsos-review` and `qsos-verify`. Activates on architect flag or explicit invocation. Never runs automatically on routine fixes.

- Activation: `SECURITY_REVIEW: recommended` in plan output, or explicit `/qsos-security` invocation
- Declines politely on routine work with guidance on when it applies
- Default: dispatches `security-reviewer` (sonnet) scoped to implementation diff
- `--deep`: dispatches `security-reviewer` with opus model override, whole-repo scope
- `--full`: dispatches `security-reviewer` whole-repo scope (sonnet)
- CRITICAL findings unconditionally halt chain — no bypass
- Update `architect.md` agent: plans touching auth, external APIs, data persistence, or significant structural change should emit `SECURITY_REVIEW: recommended`
- Add to `deploy.sh` registry
