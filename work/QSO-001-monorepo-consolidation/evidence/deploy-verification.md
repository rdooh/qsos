# Evidence — QSO-001 Deploy Verification

## Claim
All QSOS skills are deployed from `qsos/skills/` via symlinks in `~/.claude/commands/`. `deploy.sh` is idempotent.

## Evidence

### Symlink targets (readlink output)
```
qsos-architecture.md → .../qsos/skills/qsos-architecture.md
qsos-audit.md → .../qsos/skills/qsos-audit.md
qsos-brainstorm.md → .../qsos/skills/qsos-brainstorm.md
qsos-bug.md → .../qsos/skills/qsos-bug.md
qsos-chain-design.md → .../qsos/skills/qsos-chain-design.md
qsos-doc-sync.md → .../qsos/skills/qsos-doc-sync.md
qsos-feature-doc.md → .../qsos/skills/qsos-feature-doc.md
qsos-implement.md → .../qsos/skills/qsos-implement.md
qsos-orient.md → .../qsos/skills/qsos-orient.md
qsos-plan.md → .../qsos/skills/qsos-plan.md
qsos-task.md → .../qsos/skills/qsos-task.md
qsos-verify.md → .../qsos/skills/qsos-verify.md
qsos.md → .../qsos/skills/qsos.md
```
No broken symlinks.

### Idempotency run
Second `./deploy.sh` run: `0 linked, 13 already-ok, 0 cleaned, 0 copied, 0 removed.`

## Verdict: CONFIRMED
