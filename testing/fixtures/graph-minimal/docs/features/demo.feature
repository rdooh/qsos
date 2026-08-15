---
feature: Graph Minimal Fixture
ticket: QSO-901
status: @accepted
---

# Graph Minimal Fixture

**Scenario: Graph compiles**
  Given a fixture project
  When compile runs
  Then nodes exist
