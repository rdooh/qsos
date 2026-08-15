# utilities/

QSOS programming utilities — Rust CLI core, Rust watcher hub, TypeScript MCP server.

**ADR:** [ADR-010](../docs/decisions/ADR-010-polyglot-utilities-architecture.md)  
**Roadmap:** [implementation-roadmap.md](../docs/roadmap/implementation-roadmap.md)

---

## Layout (target)

```
utilities/
├── Cargo.toml          # Rust workspace root
├── qsos-cli/             # `qsos` binary — subcommand dispatch
├── qsos-lint/            # static sensors (ADR, Gherkin, DSL, sync)
├── qsos-graph/           # graph compiler + query engine
├── qsos-ingest/          # test result → scenario mapping
├── qsos-watch/           # hub daemon (triggers.toml)
├── qsos-mcp/             # TypeScript MCP server
├── serve.py              # log viewer (Python — unchanged)
└── log-viewer.html
```

Scaffold ticket: [QSO-019](../work/QSO-019-rust-utilities-scaffold/QSO-019-rust-utilities-scaffold.md)

## Build

```bash
cd utilities
cargo build --release
cargo test
cargo clippy -- -D warnings

# Lint the QSOS repo from utilities/
cargo run -p qsos-cli --bin qsos -- lint --root ..
```

Binary: `utilities/target/release/qsos`

### `qsos init`

Scaffold a QSOS-governed project layout, or install git hooks:

```bash
# New project wizard
qsos init --name "My PoC" --prefix POC-

# Inspect gaps without writing
qsos init --check --root /path/to/project

# Install pre-commit hook (lints staged files only)
qsos init --hooks --root /path/to/project

# Legacy adoption — baseline existing violations, then install hook
qsos init --hooks --baseline --root /path/to/project
```

The pre-commit hook runs `qsos lint --staged` and blocks commits when staged files have error-level violations. `.audit-baseline.json` suppresses pre-existing violations after `--baseline`.

---

## Strux reference map

Strux is a personal R&D project. QSOS rebuilds proven designs natively — no runtime coupling.

| QSOS crate | Strux reference |
|---|---|
| `qsos-lint` | `packages/strux-sensors`, `packages/strux-curator` |
| `qsos-graph` | `packages/strux-graph` |
| `qsos-ingest` | `packages/strux-dynamix` |
| `qsos-watch` | `watcher/` |
| `qsos-mcp` | `packages/strux-mcp` |

---

## Hub-and-Spoke Watcher

Specifications (QSO-009):

- [triggers-schema.md](../docs/specs/triggers-schema.md)
- [spoke-contract.md](../docs/specs/spoke-contract.md)
- [ADR-004](../docs/decisions/ADR-004-hub-and-spoke-watcher.md)

Implementation: [QSO-024](../work/QSO-024-rust-watcher-hub/QSO-024-rust-watcher-hub.md)

Spokes invoke `qsos` subcommands (Rust binaries), not Node scripts.

---

## Current files

| File | Language | Purpose |
|---|---|---|
| `serve.py` | Python | Dev server — log viewer + graph viewer APIs |
| `log-viewer.html` | HTML/JS | Run log viewer UI |
| `graph-viewer.html` | HTML/JS | **TEMP dev tool** — artifact graph browser (not load-bearing) |

```bash
# Compile graph, then browse locally
cargo run -p qsos-cli --bin qsos -- graph compile --root ..
python3 utilities/serve.py
# → http://localhost:8765/utilities/graph-viewer.html
```

Permanent visualization → Developer Operating System Visual Surface (Pillar 4). This viewer is scratch tooling only.
