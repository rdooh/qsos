---
id: QSO-031
title: "TEMP dev tool — artifact graph viewer (HTML)"
status: done
priority: low
type: chore
impact_scope:
  - utilities/graph-viewer.html
  - utilities/serve.py
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
architecture_updated: false
depends_on:
  - QSO-022
---

Scratch visualization for `work/graph-registry.json`. **Not load-bearing.** Intended for local human inspection during utilities development.

Permanent visualization belongs at the **Developer Operating System** layer (Pillar 4: Visual Surface Engine / Hyperloop-style lenses). QSOS owns the graph data contract; Dev OS owns cross-project visual projection.

## Problem

`qsos graph compile` produces useful JSON, but there is no way to visually traverse ticket→feature→ADR→scenario relationships without reading raw files or JSON.

## Deliverables

- `utilities/graph-viewer.html` — standalone browser viewer (vis-network)
- `utilities/serve.py` — `/api/graph` endpoint + startup URL hint
- Color-coded nodes by artifact kind; search and kind filters
- Click-to-focus neighborhood highlight
- Prominent **TEMPORARY DEV TOOL** banner in UI

## Non-goals

- Not a product surface; no CI dependency
- Not MCP-integrated
- Not a replacement for Dev OS graph lenses
- No edit/write-back to registry

## Dev OS elevation path

When Dev OS visual surface matures, this file should be deleted or replaced by a lens that reads the same `graph-registry.json` contract from any mesh-connected project.

## Verification

**Claim:** Developer can compile the graph and browse it interactively in a local browser.

**Evidence type:** Manual smoke test

### Scenario coverage

| Step | Verify method | Evidence artifact |
|---|---|---|
| Graph loads | Open viewer after `qsos graph compile` | `evidence/smoke-test.md` |
| Nodes colored by kind | Visual check | `evidence/smoke-test.md` |
| Click focuses neighborhood | Visual check | `evidence/smoke-test.md` |

### Commands

```bash
cd utilities
cargo run -p qsos-cli --bin qsos -- graph compile --root ..
python3 serve.py
# Open http://localhost:8765/utilities/graph-viewer.html
```

**Evidence directory:** `work/QSO-031-temp-dev-tool-graph-viewer/evidence/`
