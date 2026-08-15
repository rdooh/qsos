# Smoke test — QSO-031 TEMP graph viewer

**Date:** 2026-08-15

## Steps

1. `cargo run -p qsos-cli --bin qsos -- graph compile --root ..` → registry written
2. `python3 utilities/serve.py` → server starts, prints graph viewer URL
3. Open `http://localhost:8765/utilities/graph-viewer.html`
4. Confirm 176 nodes load; kind filters work; ticket dropdown narrows subgraph
5. Click QSO-022 → neighborhood highlights; detail panel shows metadata

## Result

PASS — interactive graph renders from `/api/graph`.

## Verdict

CONFIRMED (manual smoke — temporary dev tool)
