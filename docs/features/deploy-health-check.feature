---
feature: Deploy health check
ticket: QSO-006
status: @done
architecture_updated: false
---

# Deploy Health Check

## Background

`deploy.sh` deploys QSOS artifacts (skills and agents) as symlinks to their system locations. Once deployed, artifacts can silently break if source files are deleted, renamed, or if the symlinks themselves are removed. There is currently no way to inspect system state without making changes. A `--check` mode gives the developer a read-only health report and a reliable exit code so that drift is detectable without triggering remediation.

---

## Feature: Deploy health check mode

**Scenario: All artifacts healthy — check passes**
  Given all QSOS skill and agent files are correctly symlinked to their destinations
  When the developer runs `./deploy.sh --check`
  Then each artifact prints an `ok` status line
  And the summary line shows all counts as ok with zero issues
  And the command exits with code 0

**Scenario: A skill symlink is missing — check reports it**
  Given a skill file exists in `qsos/skills/`
  And no corresponding symlink exists at `~/.claude/commands/`
  When the developer runs `./deploy.sh --check`
  Then that skill prints a `missing` status line
  And the summary shows 1 missing
  And the command exits with code 1
  And the output includes "run ./deploy.sh to fix."

**Scenario: A symlink points to a deleted source file — check reports broken**
  Given a symlink exists at `~/.claude/commands/qsos-example.md`
  And the source file it points to no longer exists
  When the developer runs `./deploy.sh --check`
  Then that artifact prints a `broken` status line with the target path shown
  And the command exits with code 1

**Scenario: A symlink points to the wrong source — check reports wrong-target**
  Given a symlink exists at `~/.claude/commands/qsos-example.md`
  And it points to a path that is not `qsos/skills/qsos-example.md`
  When the developer runs `./deploy.sh --check`
  Then that artifact prints a `wrong-target` status line showing both actual and expected targets
  And the command exits with code 1

**Scenario: A stale symlink has no matching source file — check reports stale**
  Given a symlink at `~/.claude/commands/qsos-deleted.md` points into `qsos/skills/`
  And `qsos/skills/qsos-deleted.md` no longer exists
  When the developer runs `./deploy.sh --check`
  Then that artifact prints a `stale` status line
  And the command exits with code 1

**Scenario: Check mode makes no changes**
  Given one or more artifacts are missing or broken
  When the developer runs `./deploy.sh --check`
  Then no symlinks are created, modified, or removed
  And the filesystem state is identical before and after the command

**Scenario: Developer fixes issues found by check**
  Given `./deploy.sh --check` exits 1 and reports issues
  When the developer runs `./deploy.sh` (default mode)
  Then all reported issues are resolved
  And a subsequent `./deploy.sh --check` exits 0
