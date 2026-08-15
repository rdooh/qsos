---
id: QSO-013
title: Codify standard directories for tests, transient results, and ticket evidence
status: done
priority: medium
type: feat
impact_scope:
  - docs/standards/project-structure.md
  - testing/
features:
  - docs/features/workspace-directory-standards.feature
adrs: []
architecture_updated: false
depends_on: []
---

Formalize and document standard boundaries for directory paths:
1. Test source files belong in the source tree (adjacent to code or inside src/test).
2. `testing/` is reserved for harness metadata/manifests.
3. Transient machine-readable outputs go to git-ignored `test-results/`.
4. Point-in-time verification evidence belongs in committed `work/QSO-NNN/evidence/`.
