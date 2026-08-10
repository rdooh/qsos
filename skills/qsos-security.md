---
description: Optional security gate — dispatches the security-reviewer agent when architect-flagged or explicitly invoked. Never runs automatically on routine work.
---

# /qsos-security

## Core Principle

Security review is not routine overhead. It is a deliberate gate that activates when risk warrants it. This skill never runs automatically on bug fixes, UI changes, or refactors. It activates when an architect flags a plan as security-sensitive, or when you explicitly invoke it. When it does run, a CRITICAL finding has no bypass.

---

## When this runs

Between `/qsos-review` and `/qsos-verify`, when either:
- The current plan output contains `SECURITY_REVIEW: recommended` (set by the architect agent), or
- The user explicitly invokes `/qsos-security`

---

Emit the `skill_started` log event:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-security','type':'skill_started','data':{}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

## Step 1 — Evaluate activation

**If invoked explicitly by the user:**
```
SECURITY: Running on explicit request.
```
Proceed to Step 2.

**If `SECURITY_REVIEW: recommended` is present in the plan output:**
```
SECURITY: Architect flagged this plan as security-sensitive.
Reason: <reason from plan output>
```
Proceed to Step 2.

**If neither condition is met:**
```
SECURITY: Not activated.

This skill runs when:
  - The architect flags a plan with SECURITY_REVIEW: recommended, or
  - You explicitly invoke /qsos-security

Neither condition applies here. Proceeding to /qsos-verify.
```
Stop and hand off to `/qsos-verify`.

---

## Step 2 — Determine mode

Check for flags in the invocation:

| Flag | Mode | Scope | Model |
|---|---|---|---|
| (none) | default | implementation diff only | sonnet |
| `--deep` | deep | whole repository | opus |
| `--full` | full | whole repository | sonnet |

If `--deep` is requested, state before dispatching:
```
SECURITY: Deep mode — whole-repo scan with opus model tier.
This is more thorough and more expensive than default mode.
```

---

## Step 3 — Dispatch security-reviewer agent

Before dispatching the agent, emit the `subagent_spawned` log event (substitute the actual diff scope files):

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-security','type':'subagent_spawned','data':{'label':'security-reviewer','model':'mid','scope_files':['<diff scope>'],'purpose':'security audit'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

**Default / --full mode:**
```
Agent(
  description: "Security review of implementation diff — reads changed files + grep, no files written, medium cost",
  subagent_type: "security-reviewer",
  prompt: "Review the implementation diff for security vulnerabilities. Scope: diff only. Run: DIFF_BASE=$(git merge-base origin/main HEAD) && git diff \"$DIFF_BASE\" --name-only to get changed files, then review each. Produce a Security Posture Report."
)
```

**--deep mode:**
```
Agent(
  description: "Deep security review — whole-repo opus scan, reads all source files, high cost",
  subagent_type: "security-reviewer",
  model: "claude-opus-4-8",
  prompt: "Deep security review of the entire repository. Scope: full repo, confidence threshold 2/10 (surface more findings). Produce a Security Posture Report."
)
```

Wait for the agent to complete before proceeding.

---

## Step 4 — Process the Security Posture Report

Read the agent's `VERDICT` line at the end of the report.

**If `VERDICT: CRITICAL-FINDINGS-PRESENT`:**

For each CRITICAL or HIGH finding, emit a `gap_discovered` log event (substitute actual gap type and description):

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-security','type':'gap_discovered','data':{'gap_type':'architecture','description':'<one-line finding>'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

Then emit `skill_blocked`:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-security','type':'skill_blocked','data':{'reason':'critical security findings','resolution_required':'fix before proceeding'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

```
SECURITY: BLOCKED — critical finding requires remediation

[Display each CRITICAL finding from the report:]
[CRITICAL] path:line — summary
  Attack vector: ...
  Remediation: ...
  Confidence: N/10

Route: return to /qsos-implement
These findings must be remediated before /qsos-verify can run.
There is no bypass for CRITICAL security findings.
```
**Stop. Do not proceed.**

**If `VERDICT: CLEAR`:**

Emit `skill_completed`:

```bash
LOG_PATH=$(python3 -c "import json; print(json.load(open('.qsos/current-run.json'))['log_path'])" 2>/dev/null)
if [ -n "$LOG_PATH" ]; then
  mkdir -p "$(dirname "$LOG_PATH")"
  python3 -c "import json,datetime; d=json.load(open('.qsos/current-run.json')); print(json.dumps({'run_id':d['run_id'],'timestamp':datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ'),'ticket':d.get('ticket',''),'skill':'qsos-security','type':'skill_completed','data':{'outcome':'clean'}}))" >> "$LOG_PATH" 2>/dev/null
fi
```

```
SECURITY: CLEAR
[Display any NEEDS-ATTENTION and INFORMATIONAL findings for awareness]
Proceeding to /qsos-verify.
```
Hand off to `/qsos-verify`.

---

## When the architect should set SECURITY_REVIEW: recommended

The `architect` agent adds this flag to plan output when the plan involves:

- New authentication or authorisation mechanism
- External API integration (inbound webhook or outbound HTTP call)
- Data persistence layer change (new model, schema migration, storage boundary)
- New service boundary or changed trust boundary
- Any feature tagged `@security-sensitive` in its feature file

---

## Blocking rule

**There is no bypass for CRITICAL security findings.** A CRITICAL finding from the security-reviewer unconditionally halts the chain and routes to `/qsos-implement`. The user may choose to invoke `/qsos-security --full` or `--deep` to get a broader view, but they may not skip remediation of findings already surfaced.
