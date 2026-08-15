# Verification Evidence — QSO-020

**Ticket:** QSO-020 — qsos lint static compliance checks  
**Date:** 2026-08-15  
**Verified by:** /qsos-verify (retroactive)  
**Evidence type:** Unit test output + CLI invocation

---

## Claim

`qsos lint` enforces ADR, Gherkin, lifecycle, and DSL rules with structured JSON output and correct exit codes.

## Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Status | Artifact |
|---|---|---|---|
| Clean project passes lint (no errors) | `qsos lint --root ..` → 0 errors, exit 0 | ✓ PASS | [lint-qsos-repo.json](lint-qsos-repo.json) |
| Violations reported with file and rule | Unit tests in adr.rs, dsl.rs, lib.rs | ✓ PASS | [cargo-test-lint.txt](cargo-test-lint.txt) |
| Rule category unit tests | `cargo test -p qsos-lint` — 3 passed | ✓ PASS | [cargo-test-lint.txt](cargo-test-lint.txt) |
| Audit skill delegates Tier 1 to lint | Deferred to QSO-027 | ○ DEFERRED | — |

## Lint run summary

- **Errors:** 0
- **Notes:** 26 (mostly DSL cross-ref suggestions — acceptable per lint policy)
- **Exit code:** 0

## Verdict

CONFIRMED (skill delegation deferred to QSO-027)
