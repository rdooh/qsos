---
feature: Agent model tier configuration
ticket: TIX-008
status: @proposed
architecture_updated: false
---

# Agent Model Tier Configuration

## Background

Agent definitions specify a model tier (low, mid, high) rather than a concrete model ID.
A configuration file maps tiers to actual model IDs for the developer's runtime and region.
This decouples agent role definitions from deployment-specific model availability —
an agent's purpose and constraints don't change when a new model ships or a region
offers different identifiers.

---

## Feature: Tier-based model references in agent definitions

**Scenario: Agent definition references a tier, not a model ID**
  Given an agent definition file in `qsos/agents/`
  When a developer reads the frontmatter
  Then the `model:` field contains a tier value (`low`, `mid`, or `high`)
  And not a concrete model identifier

**Scenario: Unknown tier value is rejected at deploy time**
  Given an agent definition contains `model: ultra`
  When the developer runs `./deploy.sh` or `./deploy.py`
  Then the deploy fails with a clear error naming the invalid tier and the valid values
  And no artifacts are deployed until the error is resolved

---

## Feature: Tier-to-model mapping configuration

**Scenario: Developer runs setup to populate tier mapping**
  Given no `qsos/config.yml` exists
  When the developer runs the setup command
  Then they are prompted to confirm or enter model IDs for each tier (low, mid, high)
  And the resulting mapping is written to `qsos/config.yml`
  And the file is not overwritten on subsequent deploys without explicit `--reconfigure`

**Scenario: Config file maps each tier to a model ID**
  Given `qsos/config.yml` exists and is valid
  When the deploy script reads agent definitions
  Then it resolves each tier to the corresponding model ID from config
  And deploys the agent with the concrete model ID substituted

**Scenario: Config file is missing at deploy time**
  Given no `qsos/config.yml` exists
  When the developer runs `./deploy.sh` or `./deploy.py`
  Then the deploy fails with: "No config.yml found — run ./setup to configure model tiers"
  And no artifacts are deployed

**Scenario: Developer updates tier mapping without touching agent definitions**
  Given a new model is available in the developer's region
  When the developer updates `qsos/config.yml` with the new model ID for a tier
  And runs `./deploy.sh`
  Then all agents referencing that tier are redeployed with the updated model ID
  And the agent definition files are unchanged

---

## Feature: Region-aware setup guidance

**Scenario: Setup prompts include region context**
  Given the developer is running setup
  When they are asked to enter a model ID for each tier
  Then the prompt reminds them to use the correct identifier for their runtime and region
  And it notes that Bedrock deployments require region-prefixed IDs (e.g. `eu.anthropic.*`)

**Scenario: --check reports config status alongside artifact health**
  Given `./deploy.sh --check` or `./deploy.py --check` is run
  Then the output includes a config section showing each tier and its resolved model ID
  And flags any tier with no mapping as `unresolved`
