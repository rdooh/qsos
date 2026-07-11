# ADR-005: Agent model tier configuration

Date: 2026-07-11
Status: Accepted
Decision makers: Rob Dooh

## Context

Agent definitions currently hardcode a concrete model ID (e.g. `model: claude-sonnet-5`) in the agent file frontmatter. This tightly couples agent role definitions to deployment-specific model availability, region constraints, and API providers (e.g. Bedrock EU model IDs vs. Anthropic direct API vs. Gemini).

To maintain portability and avoid configuration drift across various deployments of the QSOS workspace, we need a way to abstract model selection from the agent role definitions themselves.

## Decision

We replace concrete model IDs in agent definitions with abstract model tiers: `low` (efficient/fast tasks), `mid` (standard workflow/reasoning tasks), and `high` (complex design/planning/security audits).

A workspace-local `qsos.config.yml` mapping file will resolve these tiers to concrete model IDs at deploy time. This file is git-ignored to allow machine/operator-specific configuration. A `qsos.config.yml.example` will be provided as a template.

The deployer script (`deploy.py`) will read `qsos.config.yml`, dynamically replace the abstract tiers with the resolved model IDs during deployment, and validate that all mapped tiers are defined before proceeding.

## Considered Options

- **Option A: Keep hardcoded model names in agent definitions (status quo)** — Keep agent files self-contained. Con: agent definitions cannot be shared between environments with different model access profiles or regions without modifying git-tracked source files.
- **Option B: Abstract tiers (`low`/`mid`/`high`) resolved via git-ignored local configuration (chosen)** — Decouples agent behavior/tools from deployment environment. Pro: High portability, supports different cloud providers/regions. Con: Requires an extra deployment-time translation step and configuration file.

## Consequences

**Positive:**
- Complete portability of the `agents/` definitions across different API keys, deployment regions, and runtime providers.
- Ability to change the active LLM version for all agents simultaneously by editing a single YAML mapping file.

**Negative:**
- Deployment process requires a mapping config; missing configurations will halt the deployment.

**Neutral:**
- The source agent files will not be directly runnable by the runtime without going through the compiler/deployer step.

## 6-month reversal test

Reversing this decision in six months would require replacing the abstract model fields (`model: mid`) in the source agent files with hardcoded model strings and removing the translation step from `deploy.py`. The cost is low-to-moderate, which justifies the flexibility gained.
