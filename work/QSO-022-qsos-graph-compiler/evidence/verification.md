# Verification Evidence — QSO-022

**Ticket:** QSO-022 — qsos graph artifact graph compiler  
**Date:** 2026-08-15  
**Verified by:** /qsos-verify (implementation)  
**Evidence type:** Unit test output + CLI invocation

---

## Claim

`qsos graph compile` produces a complete artifact graph registry with all node and edge types.

## Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Status | Artifact |
|---|---|---|---|
| Graph compiles artifact relationships | Fixture compile → 5 nodes, 4 edges | ✓ PASS | [graph-compile-fixture.json](graph-compile-fixture.json) |
| Node types present | ticket, feature, scenario, adr, dsl_element | ✓ PASS | [graph-node-types.txt](graph-node-types.txt) |
| Edge types present | ticket→feature, feature→ADR, ADR→dsl_element, scenario→file | ✓ PASS | [graph-edge-types.txt](graph-edge-types.txt) |
| Registry written to disk | `work/graph-registry.json` after compile | ✓ PASS | [graph-registry-path.txt](graph-registry-path.txt) |

## QSOS repo compile

Full compile on QSOS monorepo: see [graph-compile-qsos-repo.txt](graph-compile-qsos-repo.txt).

## Unit tests

`cargo test -p qsos-graph` — 4 passed. See [cargo-test-graph.txt](cargo-test-graph.txt).

## Verdict

CONFIRMED
