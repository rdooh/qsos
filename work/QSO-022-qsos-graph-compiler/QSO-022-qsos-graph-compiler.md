---
id: QSO-022
title: qsos graph — artifact graph compiler
status: done
priority: high
type: feat
impact_scope:
  - utilities/qsos-graph/
  - utilities/qsos-cli/
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-019
---

Implement `qsos graph compile` — compile artifact relationships into a queryable graph registry.

## Deliverables

- Node types: ticket, feature, scenario, ADR, dsl_element, contract
- Edge types: ticket→feature, feature→ADR, ADR→dsl_element, scenario→file
- Output: `work/graph-registry.json` (JSON, nodes + edges)
- Incremental recompile on changed files (optional optimisation — full compile acceptable for v1)
- Unit tests with fixture artifact sets

## Reference

- Strux `packages/strux-graph/` — AST link resolving, builder, ingestor
- Strux `packages/strux-synthesizer/` — graph store concepts

## Unblocks

QSO-023 (query), QSO-026 (test ingestion edges), QSO-027 (orient integration)

## Verification

**Claim:** `qsos graph compile` produces a complete artifact graph registry with all node and edge types.

**Evidence type:** Unit test output + CLI invocation on fixture artifact sets

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Graph compiles artifact relationships | Compile fixture project → non-empty registry | `evidence/graph-compile-fixture.json` |
| Node types present | Assert ticket, feature, scenario, ADR, dsl_element, contract nodes | `evidence/graph-node-types.txt` |
| Edge types present | Assert ticket→feature, feature→ADR, ADR→dsl_element, scenario→file edges | `evidence/graph-edge-types.txt` |
| Registry written to disk | `work/graph-registry.json` exists after compile | `evidence/graph-registry-path.txt` |

### Commands

```bash
cd utilities
cargo test -p qsos-graph
cargo run -p qsos-cli --bin qsos -- graph compile --root ../testing/fixtures/graph-minimal
```

**Evidence directory:** `work/QSO-022-qsos-graph-compiler/evidence/`
