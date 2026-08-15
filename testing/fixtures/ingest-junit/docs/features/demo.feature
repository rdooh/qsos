---
feature: Ingest Demo
ticket: QSO-901
status: @accepted
---

# Ingest Demo

**Scenario: Graph compiles**
  Given a fixture project
  When compile runs
  Then nodes exist

**Scenario: Query works**
  Given a graph
  When querying
  Then subgraph returns
