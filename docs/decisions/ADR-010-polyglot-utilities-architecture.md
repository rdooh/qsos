# ADR-010: Polyglot Utilities Architecture and Development Program

## Status

Accepted

**Date:** 2026-08-14
**Decision makers:** Rob Dooh

## Context

The QSOS skill chain is complete and operational at work. Mechanical checks — ADR integrity, Gherkin style, lifecycle consistency, context loading, drift detection, test-to-scenario traceability — are still performed manually by agents reading files. This is slow, inconsistent, and expensive in context tokens.

A personal project (**Strux**) explored the same problem space independently. Strux validated modular static sensors, a relational artifact graph, test-result ingestion, an MCP tool surface, and a Rust hub-and-spoke watcher. QSOS and Strux are **permanently separate systems** (personal vs. work IP boundary). QSOS will **rebuild** proven mechanics natively; it will never invoke or depend on Strux at runtime.

Language choice must follow **best tool for the job**, not human cognitive-load minimisation. Agents write the code; polyglot is acceptable and expected.

## Decision

### 1. Utilities development program

QSOS enters a utilities implementation phase. Skills remain the source of truth for *what* should happen; utilities automate *mechanical verification*. Every skill that delegates to a utility retains a manual fallback.

Implementation priority (reprioritised from the original utilities roadmap):

| Phase | Deliverable | Unblocks |
|---|---|---|
| **1** | Rust workspace scaffold + spoke output contract | All utilities |
| **2** | `qsos lint` — static checks (ADR, Gherkin, lifecycle) | `/qsos-audit`, `/qsos-doc-sync` |
| **3** | `qsos graph` + `qsos query` — artifact graph compiler and queries | `/qsos-orient`, `/qsos-plan` |
| **4** | Test ingestion + scenario mapping | `/qsos-verify`, `/qsos-coverage-check` |
| **5** | `qsos lint --sync` — code/DSL drift detection | `/qsos-doc-sync` |
| **6** | `qsos-mcp` — TypeScript MCP server | All skills (typed tool calls) |
| **7** | `qsos-watch` — Rust hub daemon | Continuous dev-time compliance |
| **8** | `qsos init` — pre-commit hook installation | Commit-time gates |

### 2. Language assignments

| Component | Language | Rationale |
|---|---|---|
| **`qsos` CLI** (lint, graph, query, ingest) | **Rust** | Compiler-gate tooling: fast, single binary, no runtime deps at work. `tree-sitter` for multi-format parsing (Gherkin, Markdown ADRs, DSL). `petgraph` for graph operations. |
| **`qsos-watch` hub** | **Rust** | Proven in Strux (`watcher/`). Low overhead, process isolation, debouncing via `notify` + `tokio`. |
| **`qsos-mcp` server** | **TypeScript** | Thin wrapper over Rust CLI binaries. Mature MCP SDK. No business logic duplication — delegates to `qsos` subprocesses. |
| **`deploy.py`, log server** | **Python** | Already correct for deployment scripting and lightweight HTTP serving. Not rewritten. |

Spokes invoked by the watcher hub are language-agnostic subprocesses per [spoke-contract.md](../specs/spoke-contract.md). In practice, spokes are `qsos` subcommands (Rust binaries).

### 3. Rust workspace layout

```
utilities/
├── Cargo.toml              # workspace root
├── qsos-cli/               # unified binary: lint, graph, query, ingest
├── qsos-lint/              # static sensor rules (ADR, Gherkin, DSL, sync)
├── qsos-graph/             # graph compiler and query engine
├── qsos-ingest/            # test result ingestion (JUnit/JSON)
├── qsos-watch/             # hub daemon (triggers.toml scheduler)
└── qsos-mcp/               # TypeScript MCP server (package.json)
```

Reference Strux packages when designing each crate (ideas and rule categories, not code):

| QSOS crate | Strux reference | Key responsibilities |
|---|---|---|
| `qsos-lint` | `strux-sensors`, `strux-curator` | `auditGherkin`, `auditADRs`, `auditArchitecture`, lifecycle cross-checks |
| `qsos-graph` | `strux-graph`, `strux-synthesizer` | AST link resolving, blast radius, coverage gaps |
| `qsos-ingest` | `strux-dynamix` | JUnit/Jest report parsing, `VERIFIES` edge to scenarios |
| `qsos-watch` | `watcher/` | TOML rule loading, debounce, concurrency, timeout, spoke spawn |
| `qsos-mcp` | `strux-mcp` | `qsos_lint`, `qsos_query`, `qsos_graph` tool surface |

### 4. IP and evolution boundary

- QSOS utilities are work-owned, enterprise-deployable code.
- Strux remains a personal R&D sandbox. Parallel feature evolution is expected; runtime coupling is forbidden.
- Shared *standards* (artifact formats, quadrant model) may cross-pollinate conceptually. Shared *implementations* may not.
- When Strux and QSOS diverge on rules, each system owns its enforcement independently.

### 5. Output contract

All Rust CLI commands emit structured JSON on stdout (violations, graph nodes/edges, query results) and use exit codes defined in [spoke-contract.md](../specs/spoke-contract.md). The MCP server parses this JSON; skills may read human-formatted summaries or JSON depending on context.

### 6. Verification floor

Utilities tickets use `testing/manifest.json` fields `utilities_runner` and `utilities_root` (extension to [ADR-006](ADR-006-test-harness-manifest-schema.md)):

- **`utilities_runner`:** `"cargo"` — mandatory floor for utilities tickets is `cargo test` and `cargo clippy` in `utilities_root`
- **`utilities_root`:** `"utilities"` — path to the Rust workspace from repo root

Evidence artifacts live in `work/<ticket-slug>/evidence/`. BDD scenarios in [qsos-utilities.feature](../features/qsos-utilities.feature) map to ticket verification sections. Integration tests (watcher, MCP) may additionally require subprocess or fixture evidence beyond unit tests.

## Considered Options

- **Option A: Python for all utilities** — extend `deploy.py` ecosystem. Con: slower for compiler-gate workloads; weaker static analysis tooling; not best tool for lint/graph at scale.
- **Option B: TypeScript for all utilities** — port Strux Node packages directly. Con: runtime dep chain at work; duplicates Strux code rather than rebuilding cleanly under QSOS IP.
- **Option C: Rust core + TypeScript MCP + Python deploy (chosen)** — each language where it excels; agents handle polyglot; single `qsos` binary for mechanical work.
- **Option D: Integrate Strux as dependency** — call Strux CLI/MCP from QSOS skills. Con: violates IP separation; couples work to personal repo lifecycle.

## Consequences

- `utilities/` grows from stubs to a Rust workspace + TypeScript MCP package
- `architecture.dsl` updated: Spokes container technology changes from Node.js to Rust
- Skills gain delegation steps (`qsos lint`, `qsos query`) with manual fallbacks preserved
- `docs/roadmap/utilities.md` and new `docs/roadmap/implementation-roadmap.md` become the execution plan
- Implementation tickets QSO-019 through QSO-028 track delivery
- Ticket prefix `QSO-` from [catalog-mesh.yaml](../../catalog-mesh.yaml) ([ADR-011](../decisions/ADR-011-catalog-mesh-ticket-prefix.md))
- Python stays limited to deploy and log-server concerns — not expanded to lint/graph

## 6-month reversal test

Reversing language choice for an individual crate is moderate cost (rewrite one crate). Reversing the entire utilities program (return to manual skills only) is easy — skills already have manual fallbacks. Reversing the decision to build utilities at all would leave skills functional but token-expensive.
