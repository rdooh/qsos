---
id: QSO-028
title: qsos init — pre-commit hook installation
status: done
priority: low
type: feat
impact_scope:
  - utilities/qsos-cli/
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-020
---

Implement `qsos init` — install pre-commit hook for staged-file linting.

## Deliverables

- `qsos init` command — install git pre-commit hook in target project
- Hook runs `qsos lint --staged` (lint only changed files for speed)
- `.audit-baseline.json` support — suppress pre-existing violations on first adoption
- `qsos init --check` — report hook status without modifying
- Document in utilities README and implementation roadmap

## Reference

Strux progressive onboarding baselines (ADR-006 in Strux) — baseline concept for legacy adoption

## Notes

Lower priority than lint/graph/ingest — ships after core utilities are stable.

## Verification

**Claim:** `qsos init` installs a pre-commit hook that lints staged files and supports baseline adoption for legacy projects.

**Evidence type:** CLI invocation + git hook integration test

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Pre-commit hook lints staged files only | Stage one file, commit → lint scoped to staged | `evidence/init-hook-staged.txt` |
| Baseline suppresses pre-existing violations | Legacy fixture + baseline → only new violations reported | `evidence/init-hook-baseline.txt` |
| Init check reports hook status | `qsos init --check` → hook installed/missing | `evidence/init-check-hooks.txt` |
| Commit blocked on violations | Staged invalid file → commit rejected | `evidence/init-hook-block.txt` |

### Commands

```bash
cd utilities
cargo test -p qsos-cli -- init
# Integration: qsos init in temp git repo, attempt commit with/without violations
```

**Evidence directory:** `work/QSO-028-qsos-init-precommit/evidence/`
