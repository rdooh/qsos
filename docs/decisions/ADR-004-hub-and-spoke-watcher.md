# ADR-004: Hub-and-Spoke Watcher Daemon Architecture

## Status
Accepted

## Date
2026-07-10

## Context
As the workspace grows to support multiple packages and submodules, we need a way to automate dev-time tasks (specification auditing, contract compilation, C4 DSL projection generation, and linters) based on filesystem events. 

In a complex multi-project environment, running a single monolithic watch daemon is prone to bottlenecks and system crashes, while running a separate watch process for every single rule leads to process management chaos and excessive CPU overhead. Furthermore, rapid file saving can trigger duplicate overlapping script runs, causing write collisions and file locks.

## Considered Options

1. **Option A: Unified Monolithic Watcher**
   - Run a single watch daemon that recursively listens to the entire workspace and runs all checks sequentially inside the same process.
   - **Pros**: Single process to manage.
   - **Cons**: High crash risk (if one linter fails or enters an infinite loop, the entire watcher crashes) and poor concurrency.

2. **Option B: Subdivided Watchers (One per project)**
   - Run independent watchers inside each submodule directory targeting specific TOML configs.
   - **Pros**: Good domain separation.
   - **Cons**: Excessive process management overhead and high idle RAM consumption.

3. **Option C: Hub-and-Spoke Watcher Daemon (Chosen)**
   - Maintain a single, ultra-lightweight **Hub** watcher process (written in Rust or Go) whose sole job is listening to filesystem events, loading configuration rules from a TOML manifest, and scheduling task executions.
   - Pipe events to an internal task scheduler that handles **debouncing** (delaying execution by 300-500ms to group rapid edits) and **concurrency control** (limiting identical rules to a concurrency of 1).
   - Execute all validators, compilers, and linters as decoupled, isolated **Spokes** (child subprocesses) managed with hard execution timeouts.
   - **Pros**: Exceptional fault isolation, low resource overhead, zero write collision risk, and robust self-healing.
   - **Cons**: Requires building a structured IPC/reporting protocol.

## Decision
We choose **Option C** (Hub-and-Spoke Watcher Daemon) for optimal fault isolation, task queue safety, and low footprint.

## Consequences

- **Fault Isolation**: If a linter script crashes or runs out of memory, it exits with an error code. The Hub remains fully active.
- **Queuing & Debouncing**: Consecutive filesystem writes will not trigger overlapping compilations.
- **Timeout Protection**: Subprocesses that hang or exceed their configured execution limit are automatically terminated (`SIGKILL`) by the Hub.
- **Status Reporting**: Utilities will exit with standardized JSON logs or status files, allowing the IDE extension to render visual health highlights.

## 6-month reversal test
Reversing this decision would require returning to individual package watchers or consolidating all linter logic directly into the watcher process. Since the Hub is decoupled from the actual script implementations, reversing the daemon model only impacts the runner wrapper, making the decision moderately easy to reverse if needed.
