---
id: TIX-008
title: Agent model tier configuration — abstract low/mid/high tiers with config-driven mapping
status: done
priority: medium
type: feat
impact_scope:
  - agents/
  - deploy.sh
  - deploy_gemini.py
  - config.yml (new)
  - setup (new)
features:
  - docs/features/agent-model-tier-config.feature
adrs:
  - docs/decisions/ADR-002-agent-definitions.md
  - docs/decisions/ADR-005-agent-model-tier-config.md
architecture_updated: false
depends_on:
  - TIX-007
---

Agent definitions currently hardcode a concrete model ID in the `model:` field. This couples
agent role definitions to deployment-specific model availability and region constraints
(Bedrock EU requires different IDs than the public API).

Replace concrete model IDs in agent frontmatter with abstract tier values (`low`, `mid`, `high`).
Add a `qsos/config.yml` that maps tiers to actual model IDs, populated at install/setup time.
The deploy script resolves tiers to model IDs at deploy time and validates that all tiers
are mapped before proceeding.

When TIX-007 (unified deployer) ships, the setup and tier-resolution logic lives there.
If implemented before TIX-007, it lives in `deploy.sh` as an interim.

Notes:
- ADR-002 records the model tier table with concrete names — will need an addendum
  or superseding ADR-004 once this ships
- `config.yml` should be in `.gitignore` (region/account-specific, not portable)
- Three tiers map to haiku/sonnet/opus conceptually; exact IDs are operator-supplied
