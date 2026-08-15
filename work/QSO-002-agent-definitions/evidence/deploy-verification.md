# Evidence — QSO-002 Agent Deployment Verification

## Claim
Five QSOS agent definition files are deployed to `~/.claude/agents/` via symlinks from `qsos/agents/`.

## Evidence

### deploy.sh output
```
Agents → /Users/robdooh/.claude/agents
  linked      architect.md
  linked      code-reviewer.md
  linked      product-owner.md
  linked      security-reviewer.md
  linked      verifier.md

Done. 5 linked, 13 already-ok, 0 cleaned, 0 copied, 0 removed.
```

## Verdict: CONFIRMED
