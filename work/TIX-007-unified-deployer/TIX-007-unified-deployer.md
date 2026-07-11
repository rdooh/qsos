---
id: TIX-007
title: Unified multi-runtime deployer (replace deploy.sh + deploy_gemini.py)
status: todo
priority: medium
type: feat
impact_scope:
  - deploy.sh
  - deploy_gemini.py
  - deploy.py (new)
features: []
adrs:
  - docs/decisions/ADR-003-multi-runtime-deployment.md
architecture_updated: false
depends_on: []
---

Replace `deploy.sh` (Claude-only) and `deploy_gemini.py` (Gemini-only) with a single `deploy.py` that:

- Auto-detects installed runtimes by checking `~/.claude/` and `~/.gemini/` (and any future runtime dirs)
- Prints: `Targets detected: claude, gemini — deploying to both. Use --target <name> to limit scope.`
- Accepts `--target claude|gemini|all` to override detection
- Applies per-target transform rules:
  - Claude: symlink `.md` files to `~/.claude/commands/` and `~/.claude/agents/`
  - Gemini: copy to `~/.gemini/config/plugins/qsos/` with subdirectory layout, `SKILL.md` per skill, `plugin.json` manifest
- `--check` mode: read-only health report for all detected targets, unified status vocabulary (ok/missing/broken/wrong-target/stale), exits 1 if any issues
- `--fix` mode (or default): remediate issues found by check
- Removes `deploy.sh` and `deploy_gemini.py` once the replacement is verified

Before starting: resolve ADR-003 (choose Option B or record a different decision).

Notes:
- Python 3 assumed (ships with macOS)
- Status vocabulary must match deploy.sh --check output to preserve any tooling/docs that reference it
- The `plugin.json` metadata currently hardcoded in deploy_gemini.py should move to a config file or be derived from qsos/ repo metadata
