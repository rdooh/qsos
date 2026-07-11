# ADR-007: Visual Workspace Dashboard

Date: 2026-07-11  
Status: Proposed  
Decision makers: Rob Dooh  

## Context

As the QSOS-governed workspace grows (incorporating multiple submodules like `strux`, `synapse`, `agent-os`, and `hyperloop`), it generates a highly structured footprint of directories, manifests, specifications, decisions, test results, and point-in-time ticket evidence on disk.

While this data is machine-readable and easily processed by individual scripts, developers and agents lack a single, consolidated visual view of the workspace's state. There is no easy way to track development velocity, inspect coverage check gaps, map ADRs to architecture blocks, or view evidence logs without digging through the directory tree.

We need a design for a visual dashboard that consumes local workspace telemetry and updates automatically as files change.

## Considered Options

- **Option A: Web Browser App** — A standard React/Vite web application served locally.
  - *Pro:* Easy to build, uses standard web tools.
  - *Con:* Lacks deep native integration. Cannot easily watch the filesystem or launch local commands without a custom local backend server.

- **Option B: VS Code Extension Panel** — Renders directly as a webview panel inside VS Code.
  - *Pro:* High developer proximity (lives where coding happens), inherits VS Code workspace context, can use VS Code Extension APIs to watch files and run tests.
  - *Con:* Coupled to VS Code. If developers use other IDEs or terminal environments, the dashboard is inaccessible.

- **Option C: Hyperloop (Tauri/React Desktop Application) (chosen)** — A standalone desktop application built using Rust (Tauri) and React.
  - *Pro:* Stands as an independent visual command center. Tauri provides direct access to native filesystem APIs (via Rust `notify` crate) and command execution. Decoupled from the IDE but runs in the developer's system tray. Can easily integrate with the Hub-and-Spoke Watcher Daemon (TIX-009) via IPC.
  - *Con:* Requires compiling a Tauri bundle.

## Decision

We propose **Option C**: using the `hyperloop` repository as the container for the visual command center. The dashboard will operate as a read-only telemetry engine that:
1. Watches the workspace filesystem recursively using a background thread (or subscribes to the Watcher Daemon).
2. Parses structured files: `work/tix-manifest.json`, `work/TIX-NNN/TIX-NNN.md`, `testing/manifest.json`, `docs/decisions/ADR-NNN.md`, `docs/features/name.feature`.
3. Parses transient outputs: `test-results/unit.json`, `test-results/integration.json`.
4. Renders interactive cards for:
   - **Backlog & Velocity**: burn-down charts, active ticket queues, cycle times.
   - **Architectural map**: interactive node layouts showing component state (`Target` vs `Current`), linked decisions (ADRs), and linked specs (Features).
   - **Verification & Posture**: coverage audit indicators (pure function coverage gaps, skipped tests).
   - **Durable Evidence Gallery**: gallery of screenshots and verification markdown reports.

## Consequences

- The codebase effectively becomes the database; no external SQL database or state server is required to run the dashboard.
- Future agents will have a visual representation of their success, which can be verified via headless browser tests running against the Tauri app.
- System RAM usage will increase slightly to host the Tauri Webview process.
