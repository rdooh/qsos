# ADR-011: Catalog-Mesh Ticket Prefix Convention

## Status

Accepted

**Date:** 2026-08-14
**Decision makers:** Rob Dooh

## Context

QSOS declares component identity in `catalog-mesh.yaml` with prefix `QSO-`, but all work items used the generic ecosystem placeholder `QSO-`. The Unified Developer Standards Basket specifies ticket IDs and folder names must use the component prefix from catalog-mesh (`work/<PREFIX>-NNN-slug/`).

Generic `QSO-` prefixes are ambiguous in a multi-component monorepo and do not identify which component owns the work item.

## Decision

1. **Ticket prefix is `QSO-`**, sourced from `catalog-mesh.yaml` `metadata.prefix`.
2. **Folder and file naming:** `work/QSO-NNN-slug/QSO-NNN-slug.md`
3. **Manifest filename stays `work/tix-manifest.json`** — the manifest schema name is ecosystem-standard; only entry IDs and paths use `QSO-`.
4. **Feature frontmatter** `ticket:` field uses `QSO-NNN` (not `QSO-NNN`).
5. **Skills and run logs** reference `QSO-NNN` in JSONL event payloads.
6. **Existing ticket numbers are preserved** (001–028); only the prefix changes. No renumbering.

## Considered Options

- **Option A: Keep generic QSO- prefix** — Con: contradicts catalog-mesh; ambiguous across components.
- **Option B: Adopt QSO- from catalog-mesh (chosen)** — Con: one-time migration; Pro: identity-aligned, Nexus/Kanban-ready.
- **Option C: Renumber from QSO-001 fresh** — Con: breaks git history references; unnecessary.

## Consequences

- All 28 ticket folders and files renamed from `QSO-*` to `QSO-*`
- ~200 references updated across skills, docs, features, roadmaps, and manifest
- `catalog-mesh.yaml` enhanced as canonical component identity record
- Future tickets increment from `QSO-029`
- Other ecosystem repos (Strux, etc.) may migrate independently; QSOS does not depend on them doing so

## 6-month reversal test

Reversal would require renaming folders back to `QSO-` and updating references. Moderate cost; prefix change is cosmetic to ticket numbers and sequencing logic.
