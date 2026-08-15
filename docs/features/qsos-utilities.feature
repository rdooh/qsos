---
feature: QSOS Programming Utilities
ticket: QSO-019
status: @in-progress
architecture_updated: true
---

# QSOS Programming Utilities

Background: The QSOS skill chain is complete. Mechanical compliance checks are still performed manually by agents. This feature covers the native QSOS utilities program — Rust CLI core, Rust watcher hub, TypeScript MCP server — specified in ADR-010.

Reference: Strux (personal) validated these mechanics. QSOS rebuilds natively; no runtime dependency on Strux.

---

## Feature: Static lint CLI

**Scenario: Clean project passes lint**
  Given a QSOS-governed project with valid ADRs, Gherkin features, and lifecycle tags
  When the developer runs `qsos lint`
  Then the command exits with code 0
  And produces no violations on stdout

**Scenario: Violations are reported with file and rule**
  Given a feature file missing a lifecycle tag
  When the developer runs `qsos lint`
  Then the command exits with code 1
  And stdout contains a JSON violation with file path, rule name, and description

**Scenario: Audit skill delegates Tier 1 to lint**
  Given `qsos lint` is installed
  When an agent runs `/qsos-audit`
  Then Tier 1 checks invoke `qsos lint`
  And only Tier 2 cross-entity checks are performed manually

**Scenario: Audit skill falls back when lint is unavailable**
  Given `qsos lint` is not installed or not on PATH
  When an agent runs `/qsos-audit`
  Then Tier 1 checks run manually per the skill fallback
  And the skill notes that utility delegation was skipped

---

## Feature: Drift detection lint sync

**Scenario: Code imports drift from DSL is reported**
  Given a project where a source file imports a module not declared in the architecture DSL
  When the developer runs `qsos lint --sync`
  Then the command exits with code 1
  And stdout contains a JSON violation describing the import drift

**Scenario: Accepted ADR missing from DSL is reported**
  Given a project with an Accepted ADR not referenced in architecture.dsl
  When the developer runs `qsos lint --sync`
  Then the command exits with code 1
  And stdout contains a JSON violation for the missing ADR reference

**Scenario: Doc-sync runs sync before close**
  Given `qsos lint --sync` is available
  When an agent runs `/qsos-doc-sync` before closing a ticket
  Then the skill invokes `qsos lint --sync`
  And blocks close when sync violations are present

---

## Feature: Knowledge graph queries

**Scenario: Graph compiles artifact relationships**
  Given a project with tickets, features, ADRs, and an architecture DSL
  When the developer runs `qsos graph compile`
  Then a graph registry file is written
  And it contains nodes for each artifact type and edges linking them

**Scenario: Orient skill queries graph by ticket**
  Given a compiled graph registry
  When an agent runs `/qsos-orient` for ticket QSO-NNN
  Then the skill invokes `qsos query --ticket QSO-NNN`
  And loads only the returned linked artifacts into context

**Scenario: File query returns governing artifacts**
  Given a compiled graph registry
  When the developer runs `qsos query --file path/to/source.rs`
  Then stdout contains governing ADRs, scenarios, and linked tests for that file

**Scenario: Blast radius query returns downstream impact**
  Given a compiled graph registry
  When the developer runs `qsos query --blast-radius docs/decisions/ADR-010-polyglot-utilities-architecture.md`
  Then stdout contains downstream tickets, features, and files affected by a change to that ADR

**Scenario: Orient skill falls back when query is unavailable**
  Given `qsos query` is not installed or the graph registry is missing
  When an agent runs `/qsos-orient` for a ticket
  Then the skill loads linked artifacts by manual file read per the fallback

---

## Feature: Test-to-scenario traceability

**Scenario: Test results map to scenarios**
  Given JUnit XML test results in `test-results/`
  And a compiled graph registry with Gherkin scenarios
  When the developer runs `qsos ingest`
  Then VERIFIES edges are added linking test cases to scenarios
  And `/qsos-verify` can query scenario coverage from the graph

**Scenario: Coverage query reports scenario pass fail status**
  Given ingested test results linked to scenarios in the graph registry
  When the developer runs `qsos query --coverage`
  Then stdout reports which scenarios are verified, failing, or untested

---

## Feature: Watcher hub dispatches spokes

**Scenario: File change triggers lint spoke**
  Given `qsos-watch` is running with a rule for `docs/features/` on modify
  When a feature file is saved
  Then the hub debounces the event
  And spawns `qsos lint` as an isolated subprocess
  And logs the spoke exit code

**Scenario: Hung spoke is terminated**
  Given a spoke rule with a configured timeout
  When the spoke exceeds the timeout without exiting
  Then the hub forcefully terminates the subprocess
  And logs a timeout error event

---

## Feature: MCP server exposes utilities

**Scenario: Agent invokes lint via MCP**
  Given `qsos-mcp` is running and connected to an agent session
  When the agent calls the `qsos_lint` tool
  Then the MCP server shells to `qsos lint`
  And returns structured JSON violations to the agent

**Scenario: Agent invokes query via MCP**
  Given `qsos-mcp` is running and connected to an agent session
  When the agent calls the `qsos_query` tool with a ticket ID
  Then the MCP server shells to `qsos query --ticket`
  And returns structured JSON graph results to the agent

**Scenario: Agent invokes graph compile via MCP**
  Given `qsos-mcp` is running and connected to an agent session
  When the agent calls the `qsos_graph` tool
  Then the MCP server shells to `qsos graph compile`
  And returns the graph registry summary to the agent

---

## Feature: Project bootstrap and commit gates

**Scenario: Setup wizard scaffolds a QSOS-governed project**
  Given an empty directory
  When the developer runs `qsos init --name my-poc --prefix POC-`
  Then the standard QSOS directory tree is created
  And catalog-mesh.yaml is written with the chosen prefix
  And the generated layout passes `qsos lint` with zero errors

**Scenario: Init check reports layout gaps on partial repos**
  Given a repository missing required QSOS directories
  When the developer runs `qsos init --check`
  Then stdout lists missing paths against project-structure.md
  And no files are modified

**Scenario: Pre-commit hook lints staged files only**
  Given a project where `qsos init --hooks` has been run
  When the developer commits staged documentation changes
  Then the pre-commit hook runs `qsos lint` on staged files only
  And the commit is blocked when violations are present

**Scenario: Baseline suppresses pre-existing violations on adoption**
  Given a legacy project with pre-existing lint violations
  When the developer runs `qsos init --hooks` with a baseline file
  Then the pre-commit hook reports only new violations not in the baseline
