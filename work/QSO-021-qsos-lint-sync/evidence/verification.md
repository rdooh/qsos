# Verification Evidence — QSO-021

**Ticket:** QSO-021 — qsos lint --sync  
**Date:** 2026-08-15  
**Verdict:** CONFIRMED

| Scenario | Artifact | Result |
|---|---|---|
| Clean sync passes | [sync-clean-fixture.json](sync-clean-fixture.json) | exit 0 |
| ADR missing from DSL | [sync-adr-drift.json](sync-adr-drift.json) | adr-unlinked error |
| Import drift | [sync-import-drift.json](sync-import-drift.json) | import-drift error |
| Doc-sync delegation | [doc-sync-delegation-note.md](doc-sync-delegation-note.md) | --sync available |

Unit tests: `cargo test -p qsos-lint -- sync` — 6 passed
