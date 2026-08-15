---
id: QSO-024
title: qsos-watch — Rust watcher hub daemon
status: todo
priority: medium
type: feat
impact_scope:
  - utilities/qsos-watch/
features:
  - docs/features/qsos-utilities.feature
  - docs/features/watcher_daemon.feature
adrs:
  - docs/decisions/ADR-004-hub-and-spoke-watcher.md
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-019
  - QSO-020
---

Implement the Rust hub-and-spoke watcher daemon specified in ADR-004.

## Deliverables

- `qsos-watch` binary — load `triggers.toml`, watch filesystem, schedule spokes
- Debouncing (300ms default), concurrency limits, execution timeouts
- Spawn `qsos lint`, `qsos graph compile` as spoke subprocesses
- Hub log events per [spoke-contract.md](../../docs/specs/spoke-contract.md)
- Example `triggers.toml` for QSOS repo root
- Integration tests for debounce, concurrency, timeout

## Reference

Strux `watcher/src/main.rs` — notify + tokio implementation

## Notes

Watcher feature file (`watcher_daemon.feature`) moves from @done (spec) to implementation tracked here.

## Verification

**Claim:** `qsos-watch` loads triggers, debounces events, enforces concurrency and timeouts, and dispatches spoke subprocesses per ADR-004.

**Evidence type:** Integration test output + subprocess log inspection

### Scenario coverage

| Scenario (watcher_daemon.feature + qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| File change triggers lint spoke | Save file under watched path → spoke spawned | `evidence/watcher-lint-spoke.txt` |
| Non-matching event ignored | Save outside watched path → no dispatch | `evidence/watcher-no-match.txt` |
| Rapid saves debounced | Double save within debounce → one execution | `evidence/watcher-debounce.txt` |
| Concurrency limit enforced | Second event while spoke running → queued/discarded | `evidence/watcher-concurrency.txt` |
| Hung spoke terminated | Spoke exceeds timeout → SIGKILL + log event | `evidence/watcher-timeout.txt` |

### Commands

```bash
cd utilities
cargo test -p qsos-watch
# Manual integration: qsos-watch with test triggers.toml (see evidence/watcher-integration.md)
```

**Evidence directory:** `work/QSO-024-rust-watcher-hub/evidence/`
