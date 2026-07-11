# QSOS Architecture: Hub-and-Spoke Watcher

> [!NOTE]
> Following the workspace standard, the primary architectural source of truth for the QSOS system is modeled in [architecture.dsl](file:///Users/robdooh/Documents/GitHub/Personal%20Projects/qsos/docs/architecture/architecture.dsl) using the Structurizr C4 DSL format.
>
> The Mermaid diagrams below are **derived projections** of that C4 model, kept inline for immediate visual rendering.

Below is the C4 Container & Component diagram of the QSOS Hub-and-Spoke Watcher system, illustrating how filesystem events flow through the queue to invoke isolated compilation and validation utilities.

```mermaid
graph TD
    User([User]) -->|Interacts with UI| PV[Workflow Dashboard]
    
    subgraph Antigravity IDE Extension
        PV
        SV[Daemon Supervisor]
    end

    subgraph Watcher Host - Hub (Rust Daemon)
        NL[Notify Listener]
        RM[Rule Matcher]
        TS[Task Scheduler & Queue]
        
        NL --> RM
        RM --> TS
    end

    subgraph Isolated Spokes (Child Subprocesses)
        GA[Gherkin Auditor]
        CC[Structurizr C4 Compiler]
        TC[Ticket Compiler]
    end

    TOML[(triggers.toml Config)] -.->|Rules| RM
    FS[(Filesystem - docs & work)] -->|Events| NL

    SV -->|Spawns & Restarts| WatcherHost
    TS -->|Debounced Execution| GA
    TS -->|Limit = 1 | CC
    TS -->|Timeout Protection| TC

    WatcherHost[Watcher Host] -->|Task Status Logs| PV
    GA & CC & TC -->|Write Spec Files / Diagrams| FS
    GA & CC & TC -->|JSON Exit Logs| TS
```
