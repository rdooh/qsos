---
description: Verification gate — load context, dispatch the verifier agent, and surface the verdict. Evidence expertise lives in the verifier agent.
---

# /qsos-verify

## Core Principle

Evidence is not optional post-processing — it is the definition of done. This skill loads the context the verifier agent needs and dispatches it. The agent holds the evidence type catalog and the verdict protocol. The skill holds the context: what was claimed, for which ticket, with which tools available.

---

## When this runs

After `/qsos-review` (or `/qsos-security` if activated) has passed. Before `/qsos-doc-sync`.

---

## Step 0 — Coverage-check gate

Before loading verification context, check whether `testing/manifest.json` exists in the project.

**If the manifest exists:** run `/qsos-coverage-check` now. If the report contains any HIGH posture gaps or coverage blockers, surface them to the developer:

State the gaps as text, then use `AskUserQuestion`:

- Question: "Coverage check found issues before verify — how should I proceed?"
- Options:
  - "Proceed to verify (note as COVERAGE DEFERRED)"
  - "Fix gaps first — run /qsos-coverage-check"

If the user proceeds, note `COVERAGE DEFERRED` in the evidence record. If they choose to fix gaps, wait for `/qsos-coverage-check` to return PASS, then continue.

**If the manifest does not exist:** note the absence and continue. The verifier will detect the test runner independently.

---

## Step 1 — Load context

Gather the following before dispatching the agent:

1. **Active ticket ID** — from `work/` or Jira
2. **The claim** — the `IMPLEMENTATION: all plan items executed` block from `/qsos-implement`, or ask the user (see Step 1b)
3. **Test manifest** — load `testing/manifest.json`. Note whether `unit_runner` or `e2e_runner` are defined.
4. **Evidence directory** — `work/<ticket-slug>/evidence/`

**Step 1b — Standalone invocation (no implementation block in context):**

If no `/qsos-implement` completion block is present, ask:

```
No implementation block found in context.
What claim are you verifying? State it in one sentence.
> 
```

Wait for the user's response, then use that as the claim. Continue with Step 2.

---

## Step 2 — Dispatch verifier agent

```
Agent(
  description: "Verification pass — reads project files, runs test tools, and validates test result JSONs, writes evidence, low-medium cost",
  subagent_type: "verifier",
  prompt: "Verify the following claim for ticket <id>:

Claim: <claim from Step 1>

Test manifest: testing/manifest.json
Evidence directory: work/<ticket-slug>/evidence/

INSTRUCTIONS:
1. Read testing/manifest.json to see what runners are configured.
2. If a runner is configured, run the test suite and verify that the results are written to a JSON file (e.g. test-results/unit.json or test-results/integration.json).
3. Open and parse the test result JSON file. Do NOT rely on console stdout or self-attestation.
4. The test result JSON is the mandatory VERIFICATION FLOOR:
   - If the JSON file is missing or unreadable -> verdict must be UNCONFIRMED
   - If any test has status failed or skipped -> verdict must be UNCONFIRMED
   - If test count is 0 -> verdict must be UNCONFIRMED
5. Save the evidence artifact (referencing the verified JSON file contents) to the evidence directory.
6. Issue your verdict: CONFIRMED or UNCONFIRMED."
)
```

Wait for the agent to return its verdict before proceeding.

---

## Step 3 — Handle the verdict

**CONFIRMED:**
```
VERIFY: CONFIRMED
Evidence: work/<ticket-slug>/evidence/<artifact>
```
Proceed to `/qsos-doc-sync`.

**UNCONFIRMED:**
```
VERIFY: UNCONFIRMED
<agent's description of what the artifact showed>
```
Stop. Surface to the user. Wait for direction. Do not proceed to `/qsos-doc-sync`.

**INCONCLUSIVE:**
```
VERIFY: INCONCLUSIVE
<agent's description of why the artifact is ambiguous>
Artifact: work/<ticket-slug>/evidence/<artifact>

What additional evidence would resolve this?
```
Stop. Wait for direction. Do not proceed to `/qsos-doc-sync`.

---

## Blocking rule

**You may not proceed to `/qsos-doc-sync` unless the verdict is CONFIRMED.** UNCONFIRMED and INCONCLUSIVE verdicts stop the chain. The user must direct what happens next — either attempt remediation, gather additional evidence, or consciously accept the ambiguity.
