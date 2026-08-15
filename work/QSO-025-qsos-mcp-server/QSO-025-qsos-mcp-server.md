---
id: QSO-025
title: qsos-mcp — TypeScript MCP server
status: open
priority: medium
type: feat
impact_scope:
  - utilities/qsos-mcp/
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-020
  - QSO-023
---

Implement TypeScript MCP server exposing QSOS utilities as typed agent tool calls.

## Deliverables

- `utilities/qsos-mcp/` — TypeScript package with MCP SDK
- Tools: `qsos_lint`, `qsos_query`, `qsos_graph`, `qsos_ingest_status`
- Each tool shells to `qsos` binary, parses JSON stdout, returns typed results
- No duplicated rule logic — thin delegation layer only
- README with Cursor/Claude MCP config example
- Unit tests with mocked subprocess output

## Reference

Strux `packages/strux-mcp/lib/index.js` — tool surface design (`list_gaps`, `query_graph`, `remediate`)

## Notes

Do not port Strux's OODA `remediate` tool — auto-fix is out of scope for work QSOS.

## Verification

**Claim:** `qsos-mcp` exposes lint, query, graph, and ingest_status tools that delegate to the `qsos` binary and return parsed JSON.

**Evidence type:** Unit test output (mocked subprocess) + MCP tool invocation

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Agent invokes lint via MCP | `qsos_lint` tool → JSON violations | `evidence/mcp-lint-tool.json` |
| Agent invokes query via MCP | `qsos_query` tool → graph JSON | `evidence/mcp-query-tool.json` |
| Agent invokes graph compile via MCP | `qsos_graph` tool → registry summary | `evidence/mcp-graph-tool.json` |
| No duplicated rule logic | TypeScript sources only shell to `qsos` binary | `evidence/mcp-source-audit.md` |
| Unit tests with mocked subprocess | `npm test` in qsos-mcp | `evidence/mcp-unit-test.txt` |

### Commands

```bash
cd utilities/qsos-mcp
npm test
# MCP smoke: invoke qsos_lint via configured Cursor MCP session
```

**Evidence directory:** `work/QSO-025-qsos-mcp-server/evidence/`
