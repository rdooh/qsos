---
id: QSO-023
title: qsos query — graph query CLI
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
  - QSO-022
---

Implement `qsos query` — targeted graph queries for agent context loading.

## Deliverables

- `qsos query --ticket QSO-NNN` — all artifacts linked to a ticket
- `qsos query --file path` — governing ADRs, scenarios, tests for a file
- `qsos query --blast-radius path` — downstream impact of a change
- JSON stdout with nodes, edges, and artifact summaries
- Auto-compile graph if registry missing or stale

## Reference

Strux `packages/strux-graph/` — blast-radius and coverage commands

## Unblocks

QSO-025 (MCP query tool), QSO-027 (`/qsos-orient` delegation)

## Verification

**Claim:** `qsos query` returns targeted graph subsets for ticket, file, and blast-radius queries.

**Evidence type:** Unit test output + CLI invocation

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Orient skill queries graph by ticket | `qsos query --ticket QSO-NNN` on fixture → linked artifacts | `evidence/query-ticket.json` |
| File query returns governing artifacts | `qsos query --file` → ADRs, scenarios, tests | `evidence/query-file.json` |
| Blast radius query returns downstream impact | `qsos query --blast-radius` → affected downstream nodes | `evidence/query-blast-radius.json` |
| Auto-compile when registry stale | Delete registry, query → recompiles then returns | `evidence/query-auto-compile.txt` |
| Orient skill falls back when query unavailable | Skill doc fallback section (QSO-027) | `evidence/orient-fallback-note.md` |

### Commands

```bash
cd utilities
cargo test -p qsos-graph -- query
cargo run -p qsos-cli --bin qsos -- query --ticket QSO-020 --root ..
cargo run -p qsos-cli --bin qsos -- query --file utilities/qsos-lint/src/lib.rs --root ..
cargo run -p qsos-cli --bin qsos -- query --blast-radius docs/decisions/ADR-010-polyglot-utilities-architecture.md --root ..
```

**Evidence directory:** `work/QSO-023-qsos-query-cli/evidence/`
