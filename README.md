# QSOS — Quality Sauce Operating System

QSOS is the developer-layer quality system. It operates at the point where code is actually written — between the human, the AI agent, and the tooling — rather than at the process and compliance layer above it.

The goal is not to replace the QMS. It is to make the QMS intent real at the moment of work. Where Jira tracks what was planned and Ketryx tracks what was approved and SOPs describe how things should be done, QSOS ensures that the agent doing the work actually follows those standards — automatically, verifiably, and without requiring the developer to manually enforce them.

---

## What's in this repo

```
skills/       — QSOS chain skills (deployed to ~/.claude/commands/)
agents/       — Specialist agent definitions (deployed to ~/.claude/agents/)
utilities/    — Future CLI and MCP server source
extension/    — Future VS Code side panel
docs/         — Capabilities, roadmap, decisions, standards, feature files
work/         — Active and closed tickets (TIX-NNN-slug/)
deploy.sh     — Deploy skills and agents to Claude Code (symlink mode by default)
deploy_gemini.py — Deploy skills and agents to Gemini CLI plugin system
```

## Deploying

```bash
./deploy.sh           # symlink all artifacts to ~/.claude/
./deploy.sh --check   # health check — reports status without making changes, exits 1 if issues
./deploy.sh --copy    # copy instead of symlink
./deploy.sh --clean   # remove all deployed QSOS artifacts
```

A unified multi-runtime deployer (`deploy.py`) is planned — see [TIX-007](work/TIX-007-unified-deployer/ticket.md) and [ADR-003](docs/decisions/ADR-003-multi-runtime-deployment.md).

## The chain

```
brainstorm → feature-doc → architecture → orient → plan → implement → review → [security] → verify → doc-sync
```

Each step is a skill prefixed `qsos-`. Invoke via `/qsos` to let the orchestrator determine the entry point, or invoke individual skills directly.

## How QSOS grows

Skills are alignment artifacts first. They encode the principles and procedures that govern quality work — what must exist before implementation, what counts as done, what needs to be recorded. A skill can be adopted with no tooling at all: the agent reads it, follows it, and the quality is there.

Tooling arrives progressively to automate what the skills describe manually. A check that an agent performs by reading twenty files becomes a CLI command. A CLI command becomes an MCP tool call. A manual audit step becomes a pre-commit hook. Each wave makes the skills faster and more reliable — but the procedures stay the same. The tooling is an implementation of the skill, not a replacement for it.

This means the skills are the source of truth for *what* should happen. The tooling is the source of truth for *whether* it happened efficiently. When the two diverge, the skill wins.

The practical consequence: you can start using QSOS today, with nothing installed, and get most of the value. Every utility added later ratchets that value forward without changing the underlying principles.

---

## Standards reference

[`docs/standards/project-structure.md`](docs/standards/project-structure.md) is the canonical reference for artifact formats, file locations, and lifecycle rules that all QSOS-governed projects follow.
