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

## Step 1 — Load context

Gather the following before dispatching the agent:

1. **Active ticket ID** — from `work/` or Jira
2. **The claim** — the `IMPLEMENTATION: all plan items executed` block from `/qsos-implement`, or ask the user (see Step 1b)
3. **Test runner** — detect from the project: `package.json` scripts, `pytest.ini`, `go.mod`, `Cargo.toml`, etc. Note which runner is available.
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
  description: "Verification pass — reads project files and runs test/build tools, writes evidence artifact, low-medium cost",
  subagent_type: "verifier",
  prompt: "Verify the following claim for ticket <id>:

Claim: <claim from Step 1>

Test runner available: <runner or 'unknown'>
Evidence directory: work/<ticket-slug>/evidence/

Apply your evidence type catalog to select the correct evidence type for this claim.
Gather actual evidence — do not summarise or infer.
Save the evidence artifact to the evidence directory.
Issue your verdict: CONFIRMED, UNCONFIRMED, or INCONCLUSIVE."
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
