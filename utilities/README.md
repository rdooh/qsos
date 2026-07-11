# utilities/

Future home of QSOS programming utilities — CLI tools and MCP server.

Per the roadmap in `docs/roadmap/utilities.md`:

- **Wave 1** — `qsos lint` CLI: ADR integrity, Gherkin style, feature lifecycle checks
- **Wave 2** — `qsos lint --sync`: code import vs DSL drift detection
- **Wave 3** — `qsos init`: pre-commit hook installation
- **Wave 4** — `qsos graph`: knowledge graph compiler and query interface
- **Wave 5** — `qsos-mcp`: MCP server exposing utilities as typed tool calls

Nothing lives here yet. When Wave 1 begins, source will be added here alongside a `package.json` or equivalent build manifest.

---

## Hub-and-Spoke Watcher Daemon (TIX-009)

The watcher daemon (Hub) spawns automation utilities (Spokes) as isolated subprocesses in
response to filesystem events. The Hub implementation will live in this directory once
TIX-009 moves from specification to implementation.

### Specifications

- **[triggers-schema.md](../docs/specs/triggers-schema.md)** — The `triggers.toml` rule
  schema. Defines the full field reference for path, event, command, debounce, concurrency,
  timeout, and overflow policy.

- **[spoke-contract.md](../docs/specs/spoke-contract.md)** — The Spoke subprocess interface
  contract. Defines exit codes, stdout JSON format, Hub log events, and known gaps.

### Architecture

See [ADR-004](../docs/decisions/ADR-004-hub-and-spoke-watcher.md) for the Hub-and-Spoke
design decision and the [architecture DSL](../docs/architecture/architecture.dsl) for the
full container and component model.
