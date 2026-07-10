# QSOS Capabilities

## The problem this solves

A QMS defines how software should be developed — what must be documented, reviewed, traced, and verified before it ships. The gap is always the same: the standards exist at the process layer, but the work happens at the developer layer. Between the SOP and the commit, a great deal can go undocumented, unreviewed, and unverified — not through negligence, but because there was no mechanism to enforce it at the point of work.

AI coding agents make this gap larger. An agent that writes and ships code faster than a human can review it is also an agent that can accumulate undocumented decisions, unverified behavior, and untraceable changes faster than any human team. Without a quality layer operating at the agent level, the QMS becomes a retrospective audit of what the agent already did.

QSOS is that quality layer.

---

## What QSOS enforces at the point of work

### 1. Nothing is built without a specification

Before an agent writes a single line of implementation code, QSOS requires:
- A feature file describing the observable behavior being built, in Gherkin format
- A ticket linking the work to the specification
- Any architectural decision that affects the work documented as an ADR

These are not post-hoc documentation tasks. They are gates. The agent cannot proceed without them.

### 2. Architectural decisions are recorded before they are implemented

Any decision that affects system structure — a new component, a changed boundary, a technology choice — requires an Architectural Decision Record before implementation begins. The architecture model (`architecture.dsl`) reflects both what exists now and what is planned, with every planned element backed by an accepted decision.

This means the architecture is always traceable: every structural element has a documented reason, and every documented decision has a visible element in the model.

### 3. Implementation follows an approved plan

Before coding begins, the agent maps each specified behavior to a concrete deliverable — named files, specific changes. That plan is presented for human approval. The agent cannot begin implementation until approval is explicit.

This is the human gate in an otherwise automated chain. It is not a bureaucratic checkpoint — it is the moment where the developer confirms that the agent's interpretation of the specification matches their intent.

### 4. Completion requires evidence, not assertion

An agent that says "this is done" without evidence is making a claim, not a statement of fact. QSOS requires that every completion is backed by a typed evidence artifact: a test run with named results, an API response, a screenshot, a build output. The type of evidence is matched to the type of claim.

The work is done when the evidence says so. Not before.

### 5. Documentation closes the loop

After verification, QSOS reconciles what was specified against what was built. Feature files are updated to reflect reality. The architecture model is updated to mark planned elements as current. Tickets are closed with evidence pointers. Any decision made during implementation that wasn't captured is recorded retrospectively.

The result: at the end of every piece of work, the specification, the architecture, the ticket, and the implementation all agree.

### 6. Compliance is continuously auditable

At any point, QSOS can run a compliance check across the artifact set:
- ADR naming, sequence, and required sections
- Gherkin style and structural correctness
- Feature lifecycle consistency (nothing ships without being accepted; nothing closes without being verified)
- Architecture model coverage (every element justified by a decision; every decision visible in the model)

This is not a release-gate audit. It is a continuous health check that can be run by any agent or developer at any time.

---

## What this means for the QMS

QSOS does not replace Jira, Ketryx, or SOPs. It operates below them, at the layer where the actual work is produced. The artifacts it enforces — feature files, ADRs, verified evidence — become the raw material that the process layer can reference.

The practical implication: when an auditor asks "how do you know this feature behaves as specified," the answer is not "a developer said so" — it is a feature file, an approval record, a test run, and a closed ticket, all produced automatically as a byproduct of how the work was done.

Quality is not added at the end. It is built in at the point of work.
