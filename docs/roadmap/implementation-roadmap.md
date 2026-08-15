# QSOS Implementation Roadmap — Utilities Program

**Status:** Active  
**ADR:** [ADR-010](../decisions/ADR-010-polyglot-utilities-architecture.md)  
**Last updated:** 2026-08-15

---

## Current state

| Layer | Status |
|---|---|
| **Skill chain** (18 skills) | ✅ Complete |
| **Agent definitions** (5 agents) | ✅ Complete |
| **Deploy pipeline** (`deploy.py`) | ✅ Complete |
| **Run logging** (JSONL schema + viewer) | ✅ Complete |
| **Watcher specifications** (ADR-004, triggers schema, spoke contract) | ✅ Specified, not implemented |
| **Utilities — Phase 1–2** (scaffold + `qsos lint`) | ✅ Done — QSO-019, QSO-020 |
| **Utilities — Phase 3** (graph + query) | ✅ Done — QSO-022, QSO-023 |
| **Utilities — Phase 3–8** (ingest, sync, MCP, watch, init) | 🔲 In progress — QSO-024–025 open; QSO-021, QSO-026–028, QSO-030 ✅ |
| **Verification standard** | ✅ BDD scenarios, ticket verify sections, manifest floor (ADR-010 §6) |

**Utilities progress:** ~75% — lint, graph, query, ingest, sync, init wizard, and pre-commit hooks shipped; MCP and watcher remain.

---

## Strategic intent

Skills encode procedure. Utilities automate mechanical checks. Strux (personal) validated the design; QSOS (work) rebuilds natively. Systems never integrate at runtime.

**Primary metric:** reduce agent token spend on `/qsos-audit`, `/qsos-orient`, `/qsos-doc-sync`, and `/qsos-verify` by replacing file-by-file reasoning with deterministic CLI output.

---

## Recommended build sequence

| Order | Ticket | Rationale |
|---|---|---|
| **Now** | QSO-025 | MCP server — expose lint/query/graph to agents |
| Next | QSO-024 | Watcher hub — automate lint + graph compile on save |

---

## Phase 1 — Foundation ✅

**Ticket:** [QSO-019](../work/QSO-019-rust-utilities-scaffold/QSO-019-rust-utilities-scaffold.md) — **done**

- Rust Cargo workspace under `utilities/`
- Unified `qsos` binary entry point (subcommand dispatch)
- JSON output types shared across crates
- Spoke exit codes aligned with [spoke-contract.md](../specs/spoke-contract.md)
- `cargo test`, `cargo clippy` pass

---

## Phase 2 — Static lint (`qsos lint`) ✅

**Ticket:** [QSO-020](../work/QSO-020-qsos-lint-static/QSO-020-qsos-lint-static.md) — **done**

Reference: Strux `strux-sensors` + `strux-curator`

- ADR integrity (naming, sequence, metadata, sections, superseded links)
- Gherkin style (10 rules — matches `/qsos-audit` Step 3)
- Feature lifecycle cross-check against `work/tix-manifest.json`
- DSL coverage (Target elements require Accepted ADRs)
- Structured violation output: `{ file, line, rule, description, severity }`
- Exit 0 clean / 1 violations

**Skill integration ticket:** [QSO-027](../work/QSO-027-skill-utility-integration/QSO-027-skill-utility-integration.md) ✅ done

---

## Phase 3 — Knowledge graph (`qsos graph`, `qsos query`) ✅

**Tickets:** [QSO-022](../work/QSO-022-qsos-graph-compiler/QSO-022-qsos-graph-compiler.md) ✅, [QSO-023](../work/QSO-023-qsos-query-cli/QSO-023-qsos-query-cli.md) ✅

Reference: Strux `strux-graph`

- Compile nodes: tickets, features, scenarios, ADRs, DSL elements, contracts
- Compile edges: ticket→feature, feature→ADR, ADR→dsl_element, scenario→file
- Output: `work/graph-registry.json`
- Queries: `--ticket QSO-NNN`, `--file path`, `--blast-radius path`
- `/qsos-orient` delegates context assembly to `qsos query`

---

## Phase 4 — Test traceability (`qsos ingest`) ✅

**Ticket:** [QSO-026](../work/QSO-026-test-ingestion-scenario-mapping/QSO-026-test-ingestion-scenario-mapping.md) — **done**

Reference: Strux `strux-dynamix`

- Ingest JUnit XML and Jest JSON from `test-results/`
- Map test cases to Gherkin scenarios via tags/names
- Add `VERIFIES` edges to graph registry
- `/qsos-verify` and `/qsos-coverage-check` query coverage instead of inferring

---

## Phase 5 — Drift detection (`qsos lint --sync`) ✅

**Ticket:** [QSO-021](../work/QSO-021-qsos-lint-sync/QSO-021-qsos-lint-sync.md) — **done**

Reference: Strux `strux-sensors` sync rules (`auditCodeSync`, `auditADRLinks`)

- Code imports vs DSL container relationships
- Accepted ADR coverage in DSL
- Unjustified DSL elements
- `/qsos-doc-sync` runs `--sync` before close

---

## Phase 6 — MCP server (`qsos-mcp`) 🔲

**Ticket:** [QSO-025](../work/QSO-025-qsos-mcp-server/QSO-025-qsos-mcp-server.md)

Reference: Strux `strux-mcp`

- TypeScript MCP server in `utilities/qsos-mcp/`
- Tools: `qsos_lint`, `qsos_query`, `qsos_graph`, `qsos_ingest_status`
- Each tool shells to `qsos` binary, returns parsed JSON
- No duplicated rule logic in TypeScript

---

## Phase 7 — Watcher hub (`qsos-watch`) 🔲

**Ticket:** [QSO-024](../work/QSO-024-rust-watcher-hub/QSO-024-rust-watcher-hub.md)

Reference: Strux `watcher/src/main.rs`

- Implement ADR-004 hub in Rust
- Load `triggers.toml`, debounce, concurrency, timeout
- Spawn `qsos lint`, `qsos graph compile` as spokes
- Update [triggers-schema.md](../specs/triggers-schema.md) examples to use `qsos` commands

---

## Phase 8 — Project bootstrap and git hooks ✅

**Tickets:** [QSO-028](../work/QSO-028-qsos-init-precommit/QSO-028-qsos-init-precommit.md) ✅, [QSO-030](../work/QSO-030-new-project-setup-wizard/QSO-030-new-project-setup-wizard.md) ✅

- `qsos init` scaffolds QSOS-governed project layout (QSO-030)
- `qsos init --check` / `--dry-run` for adoption and inspection
- `qsos init --hooks` installs pre-commit hook running `qsos lint --staged` (QSO-028)
- `qsos init --hooks --baseline` writes `.audit-baseline.json` for legacy adoption

---

## Deferred

| Item | Reason |
|---|---|
| Hyperloop dashboard | ADR-007 — separate track |
| VS Code extension | Optional visibility layer |
| OODA auto-remediation | Wrong fit for work human-gate model |
| Diagram generation | Lower priority; Strux `generate-diagrams` as future reference |
| Permanent graph visualization | Dev OS Pillar 4 (Visual Surface Engine); QSOS has TEMP `graph-viewer.html` only ([QSO-031](../work/QSO-031-temp-dev-tool-graph-viewer/QSO-031-temp-dev-tool-graph-viewer.md)) |

---

## Ticket index

| ID | Title | Phase | Status | Depends on |
|---|---|---|---|---|
| QSO-019 | Rust utilities workspace scaffold | 1 | ✅ done | — |
| QSO-020 | `qsos lint` static checks | 2 | ✅ done | QSO-019 |
| QSO-021 | `qsos lint --sync` drift detection | 5 | ✅ done | QSO-020 |
| QSO-022 | `qsos graph` compiler | 3 | ✅ done | QSO-019 |
| QSO-023 | `qsos query` CLI | 3 | ✅ done | QSO-022 |
| QSO-024 | Rust watcher hub | 7 | open | QSO-019, QSO-020 |
| QSO-025 | `qsos-mcp` TypeScript server | 6 | open | QSO-020, QSO-023 |
| QSO-026 | Test ingestion + scenario mapping | 4 | ✅ done | QSO-022 |
| QSO-027 | Skill integration (audit, orient, doc-sync, verify) | 2–4 | ✅ done | QSO-020, QSO-023 |
| QSO-028 | `qsos init` + pre-commit hook | 8 | ✅ done | QSO-020 |
| QSO-029 | catalog-mesh ticket prefix migration | — | ✅ done | — |
| QSO-030 | `qsos init` new project setup wizard | 8 | ✅ done | QSO-019, QSO-020 |
| QSO-031 | TEMP dev tool — artifact graph viewer | — | ✅ done | QSO-022 |

---

## Success criteria

- `/qsos-audit` Tier 1 completes in &lt;2s on QSOS repo (vs. multi-minute agent read)
- `/qsos-orient` loads context via `qsos query --ticket` without reading every linked file
- `/qsos-doc-sync` blocks close on `qsos lint` violations
- `/qsos-verify` reports scenario coverage from graph, not inference
- All utilities installable as a single `qsos` binary + optional `qsos-mcp` npm package

---

## Verification

All utilities tickets include `## Verification` sections mapping BDD scenarios to evidence artifacts. Test floor: `testing/manifest.json` → `utilities_runner: cargo`. See [ADR-010 §6](../decisions/ADR-010-polyglot-utilities-architecture.md).
