# Evidence — TIX-006 CLI Verification

## Claim
`deploy.sh --check` mode reports each artifact as ok/missing/broken/wrong-target/stale, exits 0 when all healthy, exits 1 when issues found, prints "run ./deploy.sh to fix." on issues, and makes no filesystem changes.

## Evidence Type
CLI behavior — actual command output captured.

---

## Test 1: Healthy system → exit 0

```
$ ./deploy.sh --check

QSOS health check

Skills (/Users/robdooh/.claude/commands):
  ok             qsos-architecture.md
  ok             qsos-audit.md
  ok             qsos-brainstorm.md
  ok             qsos-bug.md
  ok             qsos-chain-design.md
  ok             qsos-doc-sync.md
  ok             qsos-feature-doc.md
  ok             qsos-implement.md
  ok             qsos-orient.md
  ok             qsos-plan.md
  ok             qsos-review.md
  ok             qsos-security.md
  ok             qsos-task.md
  ok             qsos-verify.md
  ok             qsos.md

Agents (/Users/robdooh/.claude/agents):
  ok             architect.md
  ok             code-reviewer.md
  ok             product-owner.md
  ok             security-reviewer.md
  ok             verifier.md

Health: 20 ok, 0 missing, 0 broken, 0 wrong-target, 0 stale.
All artifacts healthy.
EXIT: 0
```

✓ All 20 artifacts ok. Exit 0.

---

## Test 2: Missing symlink → exit 1 + fix hint

```
$ mv ~/.claude/commands/qsos-verify.md /tmp/
$ ./deploy.sh --check

QSOS health check

Skills (/Users/robdooh/.claude/commands):
  ...
  missing        qsos-verify.md
  ...

Health: 19 ok, 1 missing, 0 broken, 0 wrong-target, 0 stale.
Issues found: 1 — run ./deploy.sh to fix.
EXIT: 1
```

✓ Reports "missing". Exits 1. Fix hint present.

---

## Test 3: No-changes guarantee

After `--check` ran with a missing artifact:
```
PASS: symlink still absent — --check made no changes
```

✓ `--check` made no filesystem changes.

---

## Test 4: Restore → exit 0

After running `./deploy.sh` to fix:
```
Health: 20 ok, 0 missing, 0 broken, 0 wrong-target, 0 stale.
All artifacts healthy.
EXIT: 0
```

✓ Subsequent `--check` exits 0.

---

## Additional scenarios verified earlier in session

- **broken** symlink (pointing to `/nonexistent/path/`) → "broken" + target shown, exit 1 ✓
- **wrong-target** symlink (pointing to `/tmp/some-other-file.md`) → "wrong-target" + both paths, exit 1 ✓
- **stale** symlink (pointing into qsos/skills/ with no source) → "stale" + target shown, exit 1 ✓

All 7 feature scenarios confirmed with real CLI output.

## Verdict: CONFIRMED
