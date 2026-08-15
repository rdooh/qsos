---
id: QSO-030
title: qsos init — new project setup wizard
status: open
priority: high
type: feat
impact_scope:
  - utilities/qsos-cli/
  - docs/standards/project-structure.md
features:
  - docs/features/qsos-utilities.feature
adrs:
  - docs/decisions/ADR-010-polyglot-utilities-architecture.md
  - docs/decisions/ADR-011-catalog-mesh-ticket-prefix.md
architecture_updated: false
depends_on:
  - QSO-019
  - QSO-020
---

Interactive (or flag-driven) setup wizard for new PoC and greenfield projects. Replaces the repeated agent conversation about folder layout, catalog identity, and baseline files every time a new repo is started.

## Problem

Starting a new PoC currently requires manually instructing an agent to create `docs/`, `work/`, `testing/`, `catalog-mesh.yaml`, folder READMEs, `.gitignore` entries, and an empty `tix-manifest.json`. This is slow, inconsistent, and wastes tokens on work that should be deterministic.

## Deliverables

- `qsos init` command (or `qsos init --wizard`) that scaffolds a QSOS-governed project from [project-structure.md](../../docs/standards/project-structure.md)
- Prompts (or CLI flags) for: project name, ticket prefix, component description, optional test runner
- Creates directory tree with quadrant READMEs:
  - `docs/features/`, `docs/decisions/`, `docs/architecture/diagrams/`, `docs/contracts/`, `docs/statecharts/`, `docs/releases/`, `docs/standards/`
  - `work/` with empty `tix-manifest.json`
  - `testing/manifest.json` stub
  - `test-results/` gitignored
- Writes `catalog-mesh.yaml` from template using chosen prefix (per ADR-011)
- Writes starter `docs/architecture/architecture.dsl` (minimal workspace block)
- Writes `.gitignore` entries for `logs/`, `test-results/`, `work/*/logs/`
- `qsos init --check` — report what's missing vs standard layout (adopt wizard on existing repos)
- `qsos init --dry-run` — print planned tree without writing
- Idempotent: skip existing files; report what was created vs skipped

## Relationship to other tickets

- **QSO-028** (pre-commit hook) — optional final wizard step; may merge into this command as `qsos init --hooks`
- **`/qsos-orient` skill** — document that agents should run `qsos init --check` on unfamiliar repos before planning

## Reference

- [project-structure.md](../../docs/standards/project-structure.md) — canonical layout
- Strux `init-ai` — agent context bootstrap (personal); QSOS wizard scaffolds **project** layout, not global skills
- VoiceBox / catalog-mesh.yaml — component identity template

## Success criteria

- New PoC bootstrapped in one command, no agent folder negotiation
- `qsos init --check` correctly reports gaps on partial repos
- Generated layout passes `qsos lint` with zero errors

## Verification

**Claim:** `qsos init` scaffolds a complete QSOS-governed project layout that passes lint and supports check/dry-run modes.

**Evidence type:** CLI invocation on temp directories + lint gate

### Scenario coverage

| Scenario (qsos-utilities.feature) | Verify method | Evidence artifact |
|---|---|---|
| Setup wizard scaffolds a QSOS-governed project | `qsos init` in empty dir → full tree + catalog-mesh.yaml | `evidence/init-scaffold-tree.txt` |
| Generated layout passes lint | `qsos lint` on scaffolded project → exit 0 | `evidence/init-scaffold-lint.json` |
| Init check reports layout gaps | `qsos init --check` on partial repo → gap list, no writes | `evidence/init-check-gaps.txt` |
| Dry run prints planned tree | `qsos init --dry-run` → planned paths, no files written | `evidence/init-dry-run.txt` |
| Idempotent skip existing files | Re-run init → skipped paths reported | `evidence/init-idempotent.txt` |

### Commands

```bash
cd utilities
cargo test -p qsos-cli -- init_wizard
TMP=$(mktemp -d) && cargo run -p qsos-cli --bin qsos -- init --name test-poc --prefix TST- --root "$TMP"
cargo run -p qsos-cli --bin qsos -- lint --root "$TMP"
cargo run -p qsos-cli --bin qsos -- init --check --root ..
```

**Evidence directory:** `work/QSO-030-new-project-setup-wizard/evidence/`
