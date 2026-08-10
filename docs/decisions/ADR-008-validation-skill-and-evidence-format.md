# ADR-008: qsos-validate as standalone skill with CTRF evidence format

Date: 2026-07-29
Status: Accepted
Decision makers: Rob Dooh

## Context

QSOS verification (`qsos-verify`) is fully automated — it dispatches a verifier agent and
returns a verdict. This works well for code-level claims but cannot cover observable
behaviour that requires a human to look at a running system (UI rendering, Storybook
components, interactive flows). A second problem: agents have been issuing CONFIRMED
verdicts based on logical reasoning rather than independently-observable evidence, with no
mechanism to require concrete proof.

Two decisions need to be made:

1. Where does human validation live — integrated into qsos-verify, or as a separate skill?
2. What format should the validation evidence record use?

## Decision 1 — Standalone skill

`qsos-validate` is a standalone skill, not integrated into `qsos-verify`.

**Rationale:**
- `qsos-verify` dispatches a subagent. Subagents cannot call `AskUserQuestion` — only the
  main agent can. Integrating interactive human steps into verify would require restructuring
  the entire verify flow.
- Keeping them separate preserves optionality: automated verify always runs; human validate
  is opt-in, invoked when the developer judges that human eyes are warranted.
- Chain position: verify → validate (optional) → doc-sync.

## Decision 2 — CTRF for validation evidence

Validation results are saved as CTRF (Common Test Results Format) JSON at
`work/<ticket-slug>/evidence/validation-ctrf.json`.

**Rationale:**
- CTRF is tool-agnostic and explicitly designed to unify results from any test runner or
  manual process. Using it avoids inventing a proprietary schema.
- The `extra` field accommodates human notes and partial confirmations without schema changes.
- `type: manual` distinguishes human steps from automated ones within the same file.
- CTRF files are readable by any CTRF-compatible reporter without custom tooling.

**Schema in use:**
```json
{
  "results": {
    "tool": { "name": "qsos-validate" },
    "summary": { "tests": N, "passed": N, "failed": N, "skipped": 0, "pending": 0, "other": N },
    "tests": [
      {
        "name": "<step name>",
        "status": "passed | failed | other",
        "type": "automated | manual",
        "message": "<evidence or human note>",
        "extra": { "link": "<file:// or http:// url>", "note": "<optional freetext>" }
      }
    ]
  }
}
```

`other` is used for "Partial — adding note" human responses.

## Consequences

**Positive:**
- Interactive human validation is possible without restructuring qsos-verify
- CTRF files can be consumed by external tooling without custom parsers
- Validate is opt-in — no ceremony overhead for work that doesn't need human eyes

**Negative:**
- Two verification skills to understand and maintain
- Migrating saved validation-ctrf.json files if CTRF schema changes (unlikely — CTRF is
  stable)
- Human steps are only as useful as the checklist quality — a lazy checklist produces a
  rubber-stamp, not real validation

## 6-month reversal test

Switching from CTRF to a custom format after tickets have accumulated validation records
requires migrating all existing files and any tooling reading them — non-trivial.
Switching from standalone to integrated verify requires restructuring the interactive flow
and the subagent boundary — significant refactor. Both decisions are load-bearing.
