# QSOS Programming Utilities Roadmap

## What a utility is

A QSOS utility is a programming tool — a CLI, a library, or an MCP server — that automates checks the agent skills currently perform manually. Where a skill asks an agent to read every ADR and check for sequence gaps, a utility runs that check in milliseconds and returns a structured result.

Utilities do not replace skills. They make skills faster, more reliable, and less dependent on agent reasoning for mechanical checks. The agent still decides what to do with a violation — the utility just finds it first.

Each utility wave delivers a specific set of capabilities that unlock something concrete in the skill chain.

---

## Wave 1 — Static linting CLI

**The need:** `/qsos-audit` currently runs all compliance checks manually — an agent reading files and applying rules. This is slow, prone to inconsistency, and expensive in context tokens. Mechanical checks (naming conventions, required sections, sequence gaps, indentation) should not consume agent reasoning cycles.

**What this delivers:**
- A CLI command: `qsos lint`
- Checks: ADR integrity (naming, sequence, required sections, valid statuses, superseded links), Gherkin style (all 10 rules), feature lifecycle consistency (stale tags, orphaned features)
- Output: structured violation list with file, line, rule, and description; exit 0 on clean, exit 1 on violations
- Integration: `/qsos-audit` invokes `qsos lint` instead of running manual checks; `/qsos-doc-sync` invokes it before closing

**Agent experience before:** agent reads 20 files and reasons about violations  
**Agent experience after:** agent runs one command and reads the output

---

## Wave 2 — Sync and drift detection

**The need:** `/qsos-doc-sync` currently performs a manual scan for code dependency drift — comparing imports in changed files against DSL relationships. This is the check most likely to catch meaningful architectural violations, and the hardest to do reliably by reading.

**What this delivers:**
- Additional `qsos lint --sync` mode
- Checks: code imports vs DSL relationships (actual dependency drift), accepted ADR coverage in DSL (every ADR referenced), DSL element justification (every element backed by an ADR)
- Requires: access to source files for import scanning; works across JS/TS initially

**Agent experience before:** agent scans changed files manually, likely misses indirect imports  
**Agent experience after:** deterministic drift report, no missed imports

---

## Wave 3 — Git hook integration

**The need:** violations are most useful when caught at the moment a file changes, not when the agent runs an audit at the end. A pre-commit hook turns compliance from a periodic check into a continuous one.

**What this delivers:**
- `qsos init` command that installs a pre-commit hook in the project
- Hook runs `qsos lint` on staged files only (fast — not the full project)
- New violations in staged files block the commit; baseline-suppressed violations pass
- `.audit-baseline.json` support: first run on an existing project can generate a baseline rather than blocking everything

**Agent experience:** violations surface at commit time, before they reach review

---

## Wave 4 — Knowledge graph

**The need:** `/qsos-orient` currently loads context by reading a list of files. It has no way to answer questions like "which ADRs are relevant to this component" or "which feature files have no passing tests" without reading everything. A graph of artifact relationships enables targeted, efficient context loading.

**What this delivers:**
- `qsos graph` command that compiles the knowledge graph from the project's artifacts
- Nodes: tickets, feature files, ADRs, DSL elements, contracts, statecharts
- Edges: ticket→feature, feature→ADR, ADR→DSL element, DSL element→contract
- Output: `work/graph-registry.json`
- Query interface: `qsos query --ticket TIX-007` returns all artifacts linked to that ticket

**Agent experience before:** `/qsos-orient` reads every linked file  
**Agent experience after:** `/qsos-orient` queries the graph for exactly what's relevant

---

## Wave 5 — MCP server

**The need:** even with a CLI, skills invoke shell commands and parse stdout. An MCP server exposes the same capabilities as structured tool calls — typed inputs, typed outputs, no stdout parsing.

**What this delivers:**
- `qsos-mcp` server exposing tools: `qsos_lint`, `qsos_query`, `qsos_check_ticket`, `qsos_graph_node`
- Skills call MCP tools directly instead of shell commands
- Richer outputs: violations as structured objects, graph queries as typed results
- Enables multi-agent scenarios: one agent lints while another implements

**Agent experience:** skills become simpler — a tool call replaces a shell invocation and output parsing

---

## Wave 6 — Dashboard

**The need:** the knowledge graph and compliance state are useful to humans as well as agents. A visual interface makes artifact health, coverage gaps, and ticket state visible without reading files or running CLI commands.

**What this delivers:**
- Web dashboard reading `work/graph-registry.json` and lint results
- Views: ticket kanban, artifact coverage map, compliance health, open violations
- Human-facing — no agent integration required

**Use case:** standup visibility, release readiness review, QMS audit preparation

---

## Wave 7 — Test execution mapping

**The need:** `/qsos-verify` collects evidence that tests pass. It does not know which feature file scenarios those tests cover. Connecting test results to scenarios closes the traceability loop from requirement to verification.

**What this delivers:**
- Test result ingestion: JUnit XML as the interchange format (compatible with Jest, pytest, Cargo, NUnit, and most other runners)
- Mapping: test names and tags resolved to scenario nodes in the knowledge graph
- New graph edges: `test→scenario` (VERIFIES), `scenario→test` (COVERED_BY)
- Dashboard view: which scenarios have passing tests, which are untested
- `/qsos-verify` can query coverage rather than running tests itself

**Agent experience:** verification becomes a query ("is this scenario covered by a passing test") rather than a command execution

---

## Principles

**Each wave makes a specific skill cheaper or more reliable.** Utilities are not added for their own sake — they exist to improve the agent chain.

**Manual fallbacks are maintained.** Every skill that delegates to a utility retains a manual fallback. A project without the CLI still works; it just works more slowly.

**The agent interface does not change between waves.** Skills invoke utilities when available, fall back otherwise. The developer experience is identical regardless of which waves are installed.
