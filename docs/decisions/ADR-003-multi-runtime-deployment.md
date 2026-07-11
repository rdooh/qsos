# ADR-003: Multi-runtime deployment strategy

## Status

Accepted

**Date:** 2026-07-11
**Decision makers:** Rob Dooh

## Context

QSOS is deployed as skills and agent definitions to AI runtime environments. Initially the only target was Claude Code (`~/.claude/commands/` and `~/.claude/agents/`), served by `deploy.sh`. A Gemini agent subsequently created `deploy_gemini.py` to deploy the same source artifacts to Gemini CLI's plugin system (`~/.gemini/config/plugins/qsos/`), which has a different layout: skills become subdirectories with `SKILL.md` files, and a `plugin.json` manifest is required.

Two independent deployers now exist for the same source. At two runtimes this is manageable. At three or more it becomes a maintenance liability: fixes to one deployer must be manually applied to the others, and `--check` behavior, status vocabulary, and health reporting can diverge silently.

A unified deployment architecture is needed before a third runtime target is added.

## Decision

We accept **Option B** (Single Python deployer with auto-detected targets). We will implement `deploy.py` to replace both `deploy.sh` and `deploy_gemini.py`.


## Considered Options

- **Option A: Keep separate scripts per runtime (status quo)** — `deploy.sh` for Claude, `deploy_gemini.py` for Gemini, `deploy_<runtime>.x` for each new target. Pro: each script is self-contained and independently testable. Con: shared logic (health check, stale detection, status vocabulary) duplicates across scripts; no single entry point.

- **Option B: Single Python deployer with auto-detected targets** — One script detects installed runtimes by checking for their config directories, deploys to all detected targets, applies per-target transform rules (symlink for Claude, copy+manifest for Gemini). `--target claude|gemini|all` overrides detection. Pro: single entry point, shared health-check logic, consistent status vocabulary across runtimes. Con: Python dependency (acceptable — ships with macOS); per-target transform rules must be maintained as runtime formats evolve.

- **Option C: Shared library + thin per-runtime wrappers** — Extract common logic into a module, keep per-runtime entry points. More engineering for marginal gain at current scale.

## Consequences

- If Option B is chosen: `deploy.sh` and `deploy_gemini.py` are replaced by a single `deploy.py`
- Runtime detection logic must be kept current as new runtimes ship
- `--check` output vocabulary is unified across all targets
- Adding a new runtime = adding a target adapter class, not a new script
- Per-target transform rules (Claude symlinks vs Gemini file copies + manifest) mean the deployer must understand each runtime's installation format
- Auto-detection means a newly installed runtime gets deployed to on next run — this is the desired behavior, but operators should be aware

## 6-month reversal test

Reverting to separate scripts after a unified deployer exists would require splitting shared logic back out and re-creating per-runtime scripts. Non-trivial if the codebase has grown. Decision should be made before more runtimes are added.
