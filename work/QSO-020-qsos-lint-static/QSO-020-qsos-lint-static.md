---
id: QSO-020
title: qsos lint — static compliance checks
status: done
priority: high
type: feat
impact_scope:
  - utilities/qsos-lint/
  - utilities/qsos-cli/
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-019
---

Implement `qsos lint` — static ADR, Gherkin, lifecycle, and DSL coverage checks.

## Deliverables

- ADR integrity: naming, monotonic sequence, metadata, required sections, superseded links
- Gherkin style: all 10 rules from `/qsos-audit` Step 3
- Feature lifecycle cross-check against `work/tix-manifest.json`
- DSL coverage: Target elements require Accepted ADRs
- JSON stdout: `{ violations: [{ file, line, rule, description, severity }] }`
- Exit 0 clean / 1 violations
- Unit tests per rule category

## Reference

- Strux `packages/strux-sensors/` — rule categories and audit function signatures
- Strux `packages/strux-curator/` — unified check orchestration
- QSOS `skills/qsos-audit.md` Steps 2–5 — rule definitions to match

## Unblocks

QSO-021 (sync), QSO-024 (watcher spoke), QSO-027 (skill integration), QSO-028 (pre-commit)

## Verification

**Claim:** `qsos lint` enforces ADR, Gherkin, lifecycle, and DSL rules with structured JSON output and correct exit codes.

**Evidence type:** Unit test output + CLI invocation on fixture projects

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Clean project passes lint | `qsos lint` on valid fixture → exit 0 | `evidence/lint-clean-fixture.json` |
| Violations reported with file and rule | `qsos lint` on invalid fixture → exit 1, JSON violations | `evidence/lint-violations-fixture.json` |
| Audit skill delegates Tier 1 to lint | Skill doc references `qsos lint`; manual spot-check | `evidence/audit-delegation-note.md` (QSO-027) |
| Rule category unit tests | `cargo test -p qsos-lint` | `evidence/cargo-test-lint.txt` |
| QSOS repo lint (no errors) | `qsos lint --root ..` — errors only, notes allowed | `evidence/lint-qsos-repo.json` |

### Commands

```bash
cd utilities
cargo test -p qsos-lint
cargo run -p qsos-cli --bin qsos -- lint --root ..
```

**Evidence directory:** `work/QSO-020-qsos-lint-static/evidence/`
