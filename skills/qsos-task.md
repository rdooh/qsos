---
description: Cross-cutting task tracking adapter — resolves the active medium (Jira, TIX files, or local plan) and provides a consistent interface for all task operations.
---

# /task

## Core Principle

Task tracking is not bureaucracy — it is a record that work happened, what it was, and what it produced. An agent that silently skips task updates is creating invisible work: work that cannot be reviewed, audited, or handed off. Every operation in the chain that changes the state of work must leave a mark.

---

## When this runs

`/qsos-task` is not a stage in the chain — it is called by other skills whenever they need to read or write task state. You can also invoke it directly to find eligible work, check a ticket's status, or close a task after verification.

Calls from other skills look like:
- `/task find` — what is eligible to work on?
- `/task read <id>` — load ticket content and linked artifacts
- `/task start <id>` — mark in-progress
- `/task update <id> <note>` — attach a note, artifact, or link
- `/task close <id> <evidence>` — mark done with evidence pointer

---

## Step 1 — Resolve the medium

Scan the project in this order and stop at the first match:

1. **Jira** — check `.mcp.json` or environment for Jira MCP configuration; check if a project key is resolvable from context (e.g. a ticket ID like `PROJ-123` in the task description)
2. **TIX files** — check for `work/` directory containing `tix-manifest.json`
3. **Local plan** — check for `plan.md` or `PLAN.md` in the working directory containing markdown checkboxes (`- [ ]`)
4. **None detected** — ask the user to declare which medium to use

State the result in one line before proceeding:

> "I'll use [Jira project KEY / TIX files at work/ / local plan at plan.md] — proceed?"

Continue unless redirected. Do not ask again for the remainder of the session once resolved.

---

## Step 2 — Execute the operation

### `find` — list eligible work

Return tickets/tasks that are `ready` or `todo` and have no unresolved blocking dependencies.

- **Jira:** query open issues in the project, filter by status
- **TIX files:** read `work/tix-manifest.json`, return entries with `status: ready` or `status: todo`
- **Local plan:** list unchecked checkboxes (`- [ ]`)

### `read <id>` — load ticket content

Return the full ticket: title, description, status, linked feature files, linked ADRs, `architecture_updated` field, dependencies.

- **Jira:** fetch the issue; note the description and any linked docs in fields or comments
- **TIX files:** read `work/<id>-slug/<id>-slug.md` including YAML frontmatter
- **Local plan:** find the matching checkbox line and any sub-items beneath it

### `start <id>` — mark in-progress

Update the ticket status to `in-progress`.

- **Jira:** transition the issue to In Progress
- **TIX files:** update `status: in-progress` in the frontmatter of `work/<id>-slug/<id>-slug.md`; update entry in `work/tix-manifest.json`
- **Local plan:** change `- [ ]` to `- [~]` (in-progress marker)

### `update <id> <note>` — attach a note or artifact

Add information to the ticket without changing its status.

- **Jira:** post a comment
- **TIX files:** append a note to the body of `work/<id>-slug/<id>-slug.md` under a `## Updates` section
- **Local plan:** not supported — note this and continue

### `close <id> <evidence>` — mark done

Mark the ticket complete and attach a pointer to the evidence artifact.

- **Jira:** transition to Done; post a comment with the evidence reference
- **TIX files:** update `status: done`; append the evidence reference to the ticket body
- **Local plan:** change `- [~]` or `- [ ]` to `- [x]`

---

## Step 3 — Report capability gaps

If the requested operation is not supported by the resolved medium, state this explicitly before continuing:

> "The [medium] does not support [operation]. [What I'll do instead or what was skipped]."

Do not silently omit the operation. Do not pretend it happened.

---

## Capability reference

| Operation | Jira | TIX files | Local plan |
|---|---|---|---|
| find eligible work | ✓ | ✓ | ✓ |
| read for direction | ✓ | ✓ | ✓ (limited) |
| create | ✓ | ✓ | ✓ (add checkbox) |
| start | ✓ | ✓ | ✓ |
| update (attach note/artifact) | ✓ | ✓ | — |
| link to ADR/feature | ✓ | ✓ | — |
| close with evidence pointer | ✓ | ✓ | ✓ (check off) |
| sprint / priority / watchers | ✓ | — | — |

For full TIX file format and ticket readiness gate definitions, see `common-skills/standards/project-structure.md`.

---

## Blocking rule

**You may never silently skip a task tracking operation.** If the medium is unavailable, unreachable, or does not support the requested operation, you must state that explicitly before moving on. An agent that says nothing has not updated the ticket — it has hidden work.
