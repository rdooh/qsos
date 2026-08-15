---
id: QSO-029
title: Adopt catalog-mesh ticket prefix (TIX- to QSO- migration)
status: done
priority: medium
type: chore
impact_scope:
  - catalog-mesh.yaml
  - work/
  - skills/
  - docs/
features: []
adrs:
  - docs/decisions/ADR-011-catalog-mesh-ticket-prefix.md
architecture_updated: false
depends_on: []
---

Migrated all work items from generic `TIX-` prefix to component prefix `QSO-` declared in `catalog-mesh.yaml`. Renamed 28 ticket folders and files; updated manifest, skills, roadmaps, features, and standards. Ticket numbers preserved (001–028).

## Deliverables

- [x] Enhanced `catalog-mesh.yaml` (v1alpha1, ticket_pattern, description)
- [x] ADR-011 — catalog-mesh ticket prefix convention
- [x] Renamed `work/TIX-*` → `work/QSO-*`
- [x] Updated `work/tix-manifest.json` ids and paths
- [x] Updated all cross-references in docs and skills
- [x] Updated `docs/standards/project-structure.md`

## Notes

- Manifest filename stays `tix-manifest.json` (ecosystem schema name)
- Next new ticket: **QSO-029**
