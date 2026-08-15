# Verification Evidence — QSO-026

**Ticket:** QSO-026 — Test ingestion and scenario mapping  
**Date:** 2026-08-15  
**Verdict:** CONFIRMED

| Scenario | Artifact | Result |
|---|---|---|
| Test results map to scenarios | [ingest-verifies-edges.json](ingest-verifies-edges.json) | 1 resolved |
| Coverage query reports status | [query-coverage.json](query-coverage.json) | verified/failing/untested |
| JUnit XML adapter | [ingest-junit.txt](ingest-junit.txt) | pass |
| Jest JSON adapter | [ingest-jest.txt](ingest-jest.txt) | pass |

Unit tests: `cargo test -p qsos-ingest` — 3 passed
