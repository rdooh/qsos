# ADR-001: Consolidate QSOS into a dedicated monorepo

## Status

Accepted

**Date:** 2026-07-10
**Decision makers:** Rob Dooh

## Context

QSOS skills have lived in `common-skills/skills/workflow/` — a general personal scripts repository. As QSOS grows to include agents, utilities (CLI, MCP server), and a VS Code extension, a single-repo source-of-truth is needed. The `common-skills/` structure has no concept of artifact types beyond skills, no deployment model for agents or binaries, and no natural home for QSOS's own documentation and roadmap.

`qsos/` already exists as the documentation home for QSOS. It is the natural consolidation point.

## Decision

The `qsos/` repository becomes the monorepo for all QSOS artifacts:

- `qsos/skills/` — canonical skill markdown files (flat, no subdirectories)
- `qsos/agents/` — agent definition files
- `qsos/utilities/` — future CLI source and MCP server
- `qsos/extension/` — future VS Code side panel
- `qsos/docs/` — existing capabilities, roadmap, decisions
- `qsos/deploy.sh` — single idempotent deployment script

`common-skills/` retains non-QSOS skills (research, thinking, coding, comms) and its own `install.sh`. It is no longer involved in QSOS deployment.

## Considered Options

- **Option A: Keep skills in common-skills** — continue using `common-skills/skills/workflow/` as the home for QSOS skills; add agent and utility deployment to `common-skills/install.sh`. Con: `common-skills/` has no concept of artifact types beyond skills; the repo's purpose becomes unclear as it grows to host agents, CLIs, and a VS Code extension.
- **Option B: Consolidate into qsos/ monorepo (chosen)** — move all QSOS artifacts into `qsos/`, which already exists as the documentation home. Pro: single source of truth, single deploy script, natural home for all artifact types as they arrive. Con: one-time symlink migration.

## Consequences

- One repository to clone, one script to run — full QSOS install
- `deploy.sh` can handle all artifact types as they arrive, without retrofitting
- QSOS documentation and implementation artifacts are co-located
- Enables future packaging (npm, brew, installer) from a single source
- One-time migration of symlinks — existing `~/.claude/commands/qsos-*` links must be re-pointed
- `common-skills/registry.yml` requires cleanup after migration
- `deploy.sh` supersedes `common-skills/install.sh` for QSOS only — the two scripts coexist and serve different artifact sets

## 6-month reversal test

Reversing this decision would require moving skills back to `common-skills/` and removing agent/utility deployment from `qsos/`. By the time utilities and an extension exist, reversal cost is high. The decision should be made before those artifacts accumulate. It is being made now.
