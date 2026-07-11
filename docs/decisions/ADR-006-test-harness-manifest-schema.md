# ADR-006: Test harness manifest schema

Date: 2026-07-11
Status: Accepted
Decision makers: Rob Dooh

## Context

QSOS-governed projects need a single declared record of what testing infrastructure is in
place. Currently agents infer this by scanning project config files (package.json, pytest.ini,
playwright.config.ts, etc.) — which is fragile, inconsistent across project types, and
produces no record of deliberate decisions. A manifest file makes testing posture explicit,
machine-readable, and auditable.

Two format decisions must be made: the file format and the schema structure. Both will be
read by agents (verifier, implementer), by the qsos-coverage-check skill, and eventually
by utilities. Changing the schema after adoption requires updating every project that has
a manifest and every tool that reads it.

## Considered Options

- **Option A: YAML** — human-friendly, supports comments (useful for explaining null fields).
  Con: inconsistent with project's existing data format (tix-manifest.json is JSON); YAML
  parsing edge cases; harder for utilities to validate without a schema library.

- **Option B: JSON (chosen)** — consistent with tix-manifest.json precedent in this project;
  unambiguous parsing; easy to validate against JSON Schema; supported natively in all
  target runtimes (Node, Python, bash via jq).

## Decision

`testing/manifest.json` uses JSON. The schema fields are:

```json
{
  "unit_runner": "jest | pytest | vitest | null",
  "e2e_runner": "playwright | cypress | null",
  "coverage_threshold": 80,
  "pre_commit_hook": true,
  "pre_push_hook": false,
  "decisions": [
    "docs/decisions/ADR-006-test-harness-manifest-schema.md"
  ]
}
```

Field rules:
- All fields are required; use `null` for "not in use", not absent/missing
- `unit_runner` and `e2e_runner` are string enum values or null — not free text
- `coverage_threshold` is an integer percentage or null (null = not enforced)
- `decisions` is an array of ADR paths that record testing strategy choices
- The file is committed to version control (it is a spec, not operator config)
- Schema version is implicit at v1; a `schema_version` field will be added if breaking
  changes are needed

## Consequences

**Positive:**
- Agents have a single authoritative source for testing posture — no inference from config files
- qsos-coverage-check can diff declared vs actual state deterministically
- JSON Schema validation can enforce field presence and enum values at deploy/check time

**Negative:**
- Every QSOS-governed project must maintain this file; stale manifests are worse than
  no manifest (false assurance)
- Enum values for runners must be extended as new runners are adopted; this requires an
  ADR update or a new ADR

**Neutral:**
- Projects without any testing infrastructure can have a valid manifest with all null values
  — the coverage-check skill treats this as a LOW priority gap, not an error

## 6-month reversal test

Renaming fields or changing the enum values after multiple projects adopt the manifest
requires a migration pass across all adopter projects and all tools that read the manifest.
Non-trivial. The schema should be settled before wide adoption. This ADR is the settlement.
