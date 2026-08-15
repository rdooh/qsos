---
id: QSO-021
title: qsos lint --sync — code and DSL drift detection
status: done
priority: medium
type: feat
impact_scope:
  - utilities/qsos-lint/
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-020
---

Implement `qsos lint --sync` — drift detection between code, DSL, and ADRs.

## Deliverables

- Code imports vs DSL container relationships (JS/TS initially via tree-sitter)
- Accepted ADR coverage in DSL (every accepted ADR referenced)
- DSL element justification (every element backed by an ADR)
- `--sync` flag on `qsos lint`; same JSON violation output format
- Unit tests with fixture projects

## Reference

Strux `packages/strux-sensors/lib/sync-rules/` — `auditCodeSync`, `auditADRLinks`

## Unblocks

Full `/qsos-doc-sync` pre-close drift check (QSO-027)

## Verification

**Claim:** `qsos lint --sync` detects code/DSL/ADR drift and emits violations in the standard JSON format.

**Evidence type:** Unit test output + CLI invocation on drift fixture projects

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Code imports drift from DSL is reported | Fixture with orphan import → violation | `evidence/sync-import-drift.json` |
| Accepted ADR missing from DSL is reported | Fixture with unreferenced ADR → violation | `evidence/sync-adr-drift.json` |
| Doc-sync runs sync before close | `/qsos-doc-sync` skill doc updated (QSO-027) | `evidence/doc-sync-delegation-note.md` |
| Clean sync project passes | Valid fixture → exit 0 | `evidence/sync-clean-fixture.json` |

### Commands

```bash
cd utilities
cargo test -p qsos-lint -- sync
cargo run -p qsos-cli --bin qsos -- lint --sync --root ../testing/fixtures/sync-clean
cargo run -p qsos-cli --bin qsos -- lint --sync --root ../testing/fixtures/sync-drift
```

**Evidence directory:** `work/QSO-021-qsos-lint-sync/evidence/`
