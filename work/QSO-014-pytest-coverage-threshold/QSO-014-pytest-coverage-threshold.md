---
id: QSO-014
title: Add pytest coverage threshold enforcement
status: done
priority: medium
type: feat
impact_scope:
  - testing infrastructure
  - pre-commit hook
features:
  - docs/features/pytest-coverage-threshold.feature
adrs:
  - docs/decisions/ADR-006-test-harness-manifest-schema.md
architecture_updated: false
depends_on: []
---

Add a pytest.ini to the qsos repo declaring `--cov=.` and `--cov-fail-under` at an appropriate threshold. Update `testing/manifest.json` to set `coverage_threshold` to the matching integer value (currently null). This closes the final LOW posture gap identified by qsos-coverage-check.

The threshold value should be set to match current actual coverage so it enforces a floor without immediately failing — then tightened over time as coverage grows.
