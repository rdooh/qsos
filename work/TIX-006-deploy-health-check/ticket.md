---
id: TIX-006
title: Add deploy.sh health check mode
status: done
priority: high
type: feat
impact_scope:
  - deploy.sh
features:
  - docs/features/deploy-health-check.feature
adrs: []
architecture_updated: false
depends_on: []
---

Add `./deploy.sh --check` mode that reads current system state without making changes, reports each artifact as `ok`, `missing`, `broken`, `wrong-target`, or `stale`, and exits 1 if any issues are found so that CI or manual runs can detect deployment drift.

- Implement `check_artifact()` function — read-only status report per artifact
- Implement `check_stale_links()` function — detect stale symlinks pointing into our src dirs
- Add `--check` branch in main logic
- Print health summary line and exit 1 on issues, exit 0 when clean
- Print "run ./deploy.sh to fix." hint when issues are found
