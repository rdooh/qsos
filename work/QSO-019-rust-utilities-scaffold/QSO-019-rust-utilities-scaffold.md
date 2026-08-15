---
id: QSO-019
title: Rust utilities workspace scaffold
status: done
priority: high
type: feat
impact_scope:
  - utilities/
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: true
depends_on: []
---

Scaffold the QSOS utilities Rust workspace and unified `qsos` binary entry point.

## Deliverables

- `utilities/Cargo.toml` — workspace root with crate members
- `utilities/qsos-cli/` — binary with subcommand dispatch stub (`lint`, `graph`, `query`, `ingest`)
- `utilities/qsos-lint/`, `qsos-graph/`, `qsos-ingest/`, `qsos-watch/` — empty crate stubs
- Shared JSON output types (violations, graph nodes/edges)
- Exit codes aligned with [spoke-contract.md](../../docs/specs/spoke-contract.md)
- `cargo test` + `cargo clippy` in CI
- Update [utilities/README.md](../../utilities/README.md) with build instructions

## Reference

Strux monorepo layout: `strux/packages/` (Node) — use for responsibility boundaries, not code.

## Notes

- Python files (`serve.py`, `log-viewer.html`) remain unchanged
- TypeScript MCP crate (`qsos-mcp/`) scaffolded in QSO-025

## Verification

**Claim:** Rust workspace scaffold compiles, dispatches all subcommands, and passes CI-quality checks.

**Evidence type:** Unit test output + CLI invocation

### Scenario coverage

| Scenario | Verify method | Evidence artifact |
|---|---|---|
| Workspace compiles | `cargo build --workspace` | `evidence/cargo-build.txt` |
| Unit tests pass | `cargo test --workspace` | `evidence/cargo-test.txt` |
| Clippy clean | `cargo clippy --workspace -- -D warnings` | `evidence/cargo-clippy.txt` |
| Subcommands dispatch | `qsos lint`, `qsos graph compile`, `qsos query`, `qsos ingest` return structured output or documented stub | `evidence/cli-dispatch.txt` |

### Commands

```bash
cd utilities
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo run -p qsos-cli --bin qsos -- --help
```

**Evidence directory:** `work/QSO-019-rust-utilities-scaffold/evidence/`
