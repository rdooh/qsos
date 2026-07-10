---
feature: QSOS Monorepo Consolidation
ticket: TIX-001
status: @done
architecture_updated: true
---

# QSOS Monorepo Consolidation

## Background

QSOS skills currently live in `common-skills/skills/workflow/`. As QSOS grows to include agents, utilities, and a VS Code extension, a single deployment source is needed. The `qsos/` repository becomes that source.

---

## Feature: Monorepo structure

**Scenario: Developer inspects the qsos repository structure**
  Given the qsos repository has been set up
  When a developer lists the top-level directories
  Then they see: `skills/`, `agents/`, `utilities/`, `extension/`, `docs/`, `work/`
  And `skills/` contains all QSOS skill markdown files (flat, no subdirectories)
  And `agents/` contains all QSOS agent definition files
  And `utilities/` contains a README stub noting future CLI source location
  And `extension/` contains a README stub noting future VS Code extension location

---

## Feature: Deploy script — basic deployment

**Scenario: Developer runs deploy.sh on a fresh machine**
  Given no QSOS artifacts are deployed
  When the developer runs `./deploy.sh` from the qsos root
  Then all skill files are symlinked to `~/.claude/commands/`
  And all agent files are symlinked to `~/.claude/agents/`
  And each artifact prints a `linked` status line
  And exit code is 0

**Scenario: Developer runs deploy.sh when already deployed**
  Given all QSOS artifacts are correctly deployed
  When the developer runs `./deploy.sh`
  Then no symlinks are changed
  And each artifact prints an `already-ok` status line
  And exit code is 0

**Scenario: Developer runs deploy.sh after renaming a skill**
  Given a skill `qsos-old.md` was previously deployed as a symlink
  And the skill has been renamed to `qsos-new.md` in the repository
  When the developer runs `./deploy.sh`
  Then the stale `~/.claude/commands/qsos-old.md` symlink is removed
  And a new `~/.claude/commands/qsos-new.md` symlink is created
  And the removed link prints a `cleaned` status line
  And the new link prints a `linked` status line

**Scenario: Developer runs deploy.sh with --copy flag**
  Given the developer needs file copies instead of symlinks
  When the developer runs `./deploy.sh --copy`
  Then skill files are copied (not symlinked) to `~/.claude/commands/`
  And agent files are copied (not symlinked) to `~/.claude/agents/`

---

## Feature: Migration from common-skills

**Scenario: All existing qsos symlinks resolve after migration**
  Given all QSOS skills previously deployed from common-skills
  When deploy.sh is run from the qsos repository
  Then `~/.claude/commands/qsos-implement.md` resolves to `qsos/skills/qsos-implement.md`
  And all other `qsos-*` commands resolve to `qsos/skills/` equivalents
  And no broken symlinks remain under `~/.claude/commands/`

**Scenario: common-skills registry is cleaned up**
  Given QSOS skills have been successfully migrated and deployed from qsos/
  When a developer inspects `common-skills/registry.yml`
  Then no `qsos-*` entries remain
  And `common-skills/install.sh` deploys only non-QSOS skills correctly
