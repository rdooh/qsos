---
id: TIX-011
title: Test harness manifest and qsos-coverage-check skill
status: done
priority: high
type: feat
impact_scope:
  - testing/manifest.json (new, per-project)
  - skills/qsos-coverage-check.md (new)
  - skills/qsos-verify.md (chain integration)
features:
  - docs/features/test-harness-manifest.feature
adrs:
  - docs/decisions/ADR-006-test-harness-manifest-schema.md
architecture_updated: false
depends_on: []
---

Introduce `testing/manifest.json` as the single declared record of testing infrastructure
in a QSOS-governed project. Replace agent inference from config files with a direct read
of this manifest. Build out the `qsos-coverage-check` skill to audit declared vs actual
testing posture and surface a prioritised gap list.

Deliverables:
- JSON schema for `testing/manifest.json` (unit_runner, e2e_runner, coverage_threshold,
  pre_commit_hook, pre_push_hook, decisions[])
- `skills/qsos-coverage-check.md` — reads manifest, diffs against actual project state,
  outputs HIGH/MEDIUM/LOW prioritised gaps, exits 1 on gaps
- `testing/manifest.json` created for the qsos project itself (meta: QSOS governs itself)
- Integration note in `skills/qsos-verify.md`: run coverage-check before dispatching verifier,
  surface HIGH gaps to developer before proceeding

Notes:
- Manifest uses JSON (project standard, consistent with tix-manifest.json)
- Manifest is committed to version control (not gitignored — it's a spec, not config)
- qsos-coverage-check replaces the stub that currently exists in the skill registry
- An ADR may be warranted once the manifest schema is settled (30-min reversal test applies
  to the schema format choice)
