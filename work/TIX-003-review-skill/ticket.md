---
id: TIX-003
title: Add qsos-review skill — post-implement code quality gate
status: done
priority: high
type: feat
impact_scope:
  - qsos/skills/
features:
  - docs/features/review-skill.feature
adrs: []
architecture_updated: false
depends_on:
  - TIX-001
  - TIX-002
---

Create `qsos/skills/qsos-review.md` — sits between `qsos-implement` and `qsos-verify` in the chain. Dispatches the `code-reviewer` agent against the current implementation diff. Structured findings gate progression.

- Dispatches `code-reviewer` agent with `subagent_type: "code-reviewer"`
- Finding schema: `{severity, confidence, path, line, category, summary, fix, fingerprint}`
- Confidence gates: 7+ CRITICAL blocks chain; 5-6 shown with caveat; <5 informational/appendix only
- CRITICAL findings route back to `qsos-implement` with findings attached
- Guards: requires `qsos-implement` completion block in context before running
- Update `qsos-implement` handoff to reference `qsos-review` as next step
- Add to `deploy.sh` registry
