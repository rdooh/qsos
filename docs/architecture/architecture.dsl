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

            spokes = container "Auditor & Compiler Utilities (Spokes)" "Decoupled validation scripts executed as isolated processes." "Node.js / CLI" {
                featureAuditor = component "Gherkin Auditor" "Parses BDD scenarios and checks lifecycle compliance tags." "Node.js"
                dslCompiler = component "Structurizr C4 Compiler" "Generates Mermaid diagrams from architecture.dsl source." "Node.js / Python"
                ticketCompiler = component "Ticket Manifest Compiler" "Compiles active tickets and work registries." "Node.js"
            }

            triggersConfig = container "Triggers Configuration" "TOML file mapping file prefixes to automation rules." "triggers.toml File"
            globalState = container "System Workspace Storage" "Local specifications, decisions, and transient work directories." "Filesystem"
        }

        user -> qsosSystem "Uses"
        user -> panelView "Interacts with specs and status"
        
        watcherHost -> triggersConfig "Loads rules from"
        watcherHost -> globalState "Watches filesystem changes inside"
        watcherHost -> spokes "Spawns and monitors execution of"
        
        spokes -> globalState "Audits features, ADRs, and writes diagrams to"
        spokes -> watcherHost "Returns exit codes and JSON audit logs to"
        
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
