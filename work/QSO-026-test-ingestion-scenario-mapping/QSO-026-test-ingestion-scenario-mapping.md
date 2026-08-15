---
id: QSO-026
title: Test ingestion and scenario mapping
status: open
priority: high
type: feat
impact_scope:
  - utilities/qsos-ingest/
  - utilities/qsos-cli/
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-022
---

Implement `qsos ingest` — map test results to Gherkin scenarios in the graph.

## Deliverables

- Parse JUnit XML and Jest JSON from `test-results/`
- Match test cases to scenarios via tags, names, or configured mapping rules
- Add `VERIFIES` edges to graph registry (test → scenario)
- `qsos query --coverage` or `--scenario <name>` — report pass/fail coverage
- Unit tests with fixture test reports

## Reference

- Strux `packages/strux-dynamix/` — ingestor, resolver, Jest/JUnit adapters
- QSOS `testing/manifest.json` — declared test harness posture

## Unblocks

QSO-027 (`/qsos-verify`, `/qsos-coverage-check` graph queries)

## Verification

**Claim:** `qsos ingest` maps test results to Gherkin scenarios and adds VERIFIES edges; coverage queries report pass/fail/untested status.

**Evidence type:** Unit test output + CLI invocation on fixture test reports

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Test results map to scenarios | Ingest JUnit/Jest fixtures → VERIFIES edges in registry | `evidence/ingest-verifies-edges.json` |
| Coverage query reports scenario status | `qsos query --coverage` → pass/fail/untested report | `evidence/query-coverage.json` |
| JUnit XML adapter | Fixture JUnit XML parsed correctly | `evidence/ingest-junit.txt` |
| Jest JSON adapter | Fixture Jest JSON parsed correctly | `evidence/ingest-jest.txt` |

### Commands

```bash
cd utilities
cargo test -p qsos-ingest
cargo run -p qsos-cli --bin qsos -- ingest --root ../testing/fixtures/ingest-junit
cargo run -p qsos-cli --bin qsos -- query --coverage --root ../testing/fixtures/ingest-junit
```

**Evidence directory:** `work/QSO-026-test-ingestion-scenario-mapping/evidence/`
