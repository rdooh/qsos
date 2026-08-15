# Dashboard Roadmap

This document outlines the phases of building the **Hyperloop Dashboard**, the visual command center that consumes QSOS workspace telemetry and provides a real-time view of development status, architecture, and verification.

---

## Phase 1 — Manifest & Ticket Ingestion
*Objective: Build the parser layer that reads the workspace's structured data.*

1. **Parser Engine**: Build parser utilities to read:
   - `work/tix-manifest.json` and `work/QSO-NNN/` markdown frontmatter.
   - `testing/manifest.json` configurations.
   - `docs/decisions/ADR-NNN.md` files.
2. **Dashboard UI**:
   - Render a Kanban board representing the active tickets.
   - Render velocity metrics (cycle time, ticket completion rates).
   - Render the testing posture status (Healthy vs Gaps).

---

## Phase 2 — Interactive Architecture Mapping
*Objective: Generate a visual graph of the system architecture from the Structurizr DSL.*

1. **DSL Parser**: Read `docs/architecture/architecture.dsl` to build a node-edge graph of software systems, containers, and components.
2. **Interactive Node Layout**: Renders the graph in React (using React Flow or d3-force).
3. **Traceability Links**:
   - Colors nodes based on lifecycle (`Current` vs `Target`).
   - Clicking a node displays the ADRs that govern it and the tickets that implemented it.

---

## Phase 3 — Live Telemetry & Verification Floor
*Objective: Connect the dashboard to active test runners and filesystem events.*

1. **File Watcher**: Integrate Tauri's native filesystem watch APIs (via Rust `notify` crate) or hook into the Hub-and-Spoke Watcher Daemon (QSO-009) to trigger updates on save.
2. **Transient Parser**: Read `test-results/unit.json` and `test-results/integration.json` on modification.
3. **Live Test Cards**:
   - Stream test run status directly in the UI.
   - Flag skipped tests or test coverage gaps in red.
   - Render code-reviewer and security-reviewer findings directly in an Audit log list.

---

## Phase 4 — HEADS-UP Headless Verification
*Objective: Allow agents to verify the UI of the dashboard itself.*

1. **Playwright Integration**: Wire up Playwright tests to boot the dashboard, run assertions, and capture screenshots.
2. **Visual Evidence**: Save test run screenshots directly into the ticket's `work/QSO-NNN/screenshots/` folder.
