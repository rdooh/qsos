workspace {
    model {
        user = person "User" "Developer managing tasks and code quality in the workspace."
        
        qsosSystem = softwareSystem "QSOS Workflow System" "Quality Spec-to-Code Orchestration System providing spec-first automation and workflow coordination." {
            
            ideExtension = container "IDE Extension" "VS Code / Antigravity panel and status indicators." "VS Code Extension API" {
                panelView = component "Workflow Dashboard" "Renders active tickets, BDD checklist status, and audit reports." "HTML/CSS/JS"
                supervisor = component "Daemon Supervisor" "Spawns, monitors, and automatically restarts the Watcher Host." "TypeScript"
            }

            watcherHost = container "Watcher Host (Hub)" "Ultra-lightweight background file monitoring and task scheduling daemon." "Rust/Tokio" {
                eventListener = component "Notify Listener" "Listens to recursive directory filesystem writes." "Rust Notify Crate"
                ruleMatcher = component "Rule Matcher" "Compares modified file paths against triggers.toml patterns." "Rust"
                taskScheduler = component "Task Scheduler & Queue" "Manages task debouncing, concurrency limits, and execution timeouts." "Rust Tokio Queue"
            }

            qsosCore = container "QSOS Core CLI" "Unified Rust binary for lint, graph, query, and test ingestion." "Rust" {
                lintEngine = component "Static Lint Engine" "ADR, Gherkin, lifecycle, and sync/drift checks." "Rust / tree-sitter"
                graphEngine = component "Graph Compiler" "Compiles artifact relationships into queryable graph registry." "Rust / petgraph"
                ingestEngine = component "Test Ingest Engine" "Maps JUnit/Jest results to Gherkin scenarios." "Rust"
            }

            spokes = container "Auditor Spokes" "qsos subcommands executed as isolated subprocesses by the watcher hub." "Rust CLI" {
                lintSpoke = component "Lint Spoke" "qsos lint — static compliance on file change." "Rust"
                graphSpoke = component "Graph Spoke" "qsos graph compile — rebuild registry on artifact change." "Rust"
            }

            mcpServer = container "QSOS MCP Server" "TypeScript MCP server exposing qsos tools to agents." "TypeScript" {
                mcpTools = component "MCP Tool Handlers" "qsos_lint, qsos_query, qsos_graph — delegates to qsos binary." "TypeScript"
            }

            triggersConfig = container "Triggers Configuration" "TOML file mapping file prefixes to automation rules." "triggers.toml File"
            globalState = container "System Workspace Storage" "Local specifications, decisions, and transient work directories." "Filesystem"
        }

        user -> qsosSystem "Uses"
        user -> panelView "Interacts with specs and status"
        
        watcherHost -> triggersConfig "Loads rules from"
        watcherHost -> globalState "Watches filesystem changes inside"
        watcherHost -> spokes "Spawns and monitors execution of"
        spokes -> qsosCore "Invokes subcommands of"
        qsosCore -> globalState "Reads artifacts and writes graph registry to"
        spokes -> watcherHost "Returns exit codes and JSON audit logs to"
        mcpServer -> qsosCore "Shells to qsos binary"
        user -> mcpServer "Agent tool calls via MCP"
        
        watcherHost -> ideExtension "Pipes execution updates to"
        supervisor -> watcherHost "Monitors process health of"
    }

    views {
        systemContext qsosSystem "SystemContext" {
            include *
            autolayout lr
        }

        container qsosSystem "Containers" {
            include *
            autolayout lr
        }

        component watcherHost "WatcherHostComponents" {
            include *
            autolayout lr
        }

        theme default
    }
}
