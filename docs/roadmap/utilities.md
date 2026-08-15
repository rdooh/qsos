# QSOS Programming Utilities Roadmap

**ADR:** [ADR-010 — Polyglot Utilities Architecture](../decisions/ADR-010-polyglot-utilities-architecture.md)  
**Execution plan:** [implementation-roadmap.md](./implementation-roadmap.md)

## What a utility is

A QSOS utility automates checks agents currently perform manually. Skills decide *what to do with* a violation; utilities *find* violations deterministically. Skills retain manual fallbacks when utilities are not installed.

Strux (personal) validated these mechanics. QSOS rebuilds them natively under work IP. No runtime dependency on Strux.

---

## Language assignments

| Component | Language | Notes |
|---|---|---|
| `qsos` CLI (lint, graph, query, ingest) | **Rust** | Single binary, `tree-sitter` parsing, `petgraph` |
| `qsos-watch` hub | **Rust** | `notify` + `tokio`; reference Strux `watcher/` |
| `qsos-mcp` | **TypeScript** | Thin MCP wrapper; delegates to `qsos` binary |
| `deploy.py`, log server | **Python** | Unchanged |

---

## Wave 1 — Static linting CLI ✅

**Ticket:** QSO-020 **done** | **Reference:** Strux `strux-sensors`, `strux-curator`

- `qsos lint` — ADR integrity, Gherkin style (10 rules), feature lifecycle consistency
- Structured JSON violation output; exit 0/1
- `/qsos-audit` Tier 1 delegates here; manual checks become Tier 2

---

## Wave 2 — Sync and drift detection

**Ticket:** QSO-021 | **Reference:** Strux `strux-sensors` sync rules

- `qsos lint --sync` — code imports vs DSL, ADR↔DSL links, unjustified elements
- `/qsos-doc-sync` runs before close

---

## Wave 3 — Git hook integration

**Ticket:** QSO-028

- `qsos init` installs pre-commit hook
- Staged-files-only lint for speed
- `.audit-baseline.json` for legacy adoption

---

## Wave 4 — Knowledge graph

**Tickets:** QSO-022 **done**, QSO-023 open | **Reference:** Strux `strux-graph`

- `qsos graph compile` → `work/graph-registry.json` ✅
- `qsos query --ticket`, `--file`, `--blast-radius` (QSO-023)
- `/qsos-orient` delegates context assembly to query
- **TEMP dev viewer:** [QSO-031](../../work/QSO-031-temp-dev-tool-graph-viewer/QSO-031-temp-dev-tool-graph-viewer.md) — `utilities/graph-viewer.html` (not load-bearing; permanent viz → Dev OS)

---

## Wave 5 — MCP server

**Ticket:** QSO-025 | **Reference:** Strux `strux-mcp`

- TypeScript `qsos-mcp` exposing `qsos_lint`, `qsos_query`, `qsos_graph`
- Shells to Rust binary; no duplicated rule logic

---

## Wave 6 — Dashboard

**ADR:** ADR-007 (Hyperloop) — separate track, not part of utilities program

---

## Wave 7 — Test execution mapping

**Ticket:** QSO-026 | **Reference:** Strux `strux-dynamix`

- JUnit/Jest ingestion from `test-results/`
- `VERIFIES` edges: test → scenario
- `/qsos-verify` queries coverage from graph

---

## Principles

**Each wave makes a specific skill cheaper.** Utilities exist to improve the agent chain, not for their own sake.

**Manual fallbacks are maintained.** Projects without the CLI still work via skill prose.

**Strux informs design; QSOS owns code.** Reference Strux package responsibilities when designing crates; never import Strux.

**Best tool per job.** Polyglot is intentional. Agents write the code; humans invoke one binary.

---

## Workspace layout

```
utilities/
├── Cargo.toml          # Rust workspace
├── qsos-cli/             # unified qsos binary
├── qsos-lint/
├── qsos-graph/
├── qsos-ingest/
├── qsos-watch/
└── qsos-mcp/             # TypeScript MCP (package.json)
```

Scaffold: QSO-019 **done**
