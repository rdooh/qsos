# Verification Evidence — QSO-023

**Ticket:** QSO-023 — qsos query graph query CLI  
**Date:** 2026-08-15  
**Verified by:** /qsos-verify (implementation)  
**Evidence type:** Unit test output + CLI invocation

---

## Claim

`qsos query` returns targeted graph subsets for ticket, file, and blast-radius queries with summaries.

## Scenario coverage

| Scenario | Verify method | Status | Artifact |
|---|---|---|---|
| Orient skill queries graph by ticket | `qsos query --ticket QSO-022` | ✓ PASS | [query-ticket.json](query-ticket.json) |
| File query returns governing artifacts | `qsos query --file docs/features/qsos-utilities.feature` | ✓ PASS | [query-file.json](query-file.json) |
| Blast radius query returns downstream impact | `qsos query --blast-radius ADR-010 path` | ✓ PASS | [query-blast-radius.json](query-blast-radius.json) |
| Auto-compile when registry stale | `cargo test` auto_recompiles_when_registry_stale | ✓ PASS | [cargo-test-query.txt](cargo-test-query.txt) |

## Verdict

CONFIRMED
