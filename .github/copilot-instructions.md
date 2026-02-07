# Copilot instructions

## Big picture

- Product scope, compliance constraints, and module boundaries are defined in [docs/PRD.md](https://github.com/IAmJonoBo/LifeReady/blob/main/docs/PRD.md); treat it as the source of truth.
- The intended monorepo layout is specified in [docs/PRD.md](https://github.com/IAmJonoBo/LifeReady/blob/main/docs/PRD.md) under "Repo layout"; match names and boundaries when scaffolding.
- Service boundaries are explicit in the PRD: identity-service, estate-service, vault-service, case-service, audit-service; do not merge responsibilities across services.
- The system relies on an append-only audit log with hash chaining; any audit changes must preserve hash-chain verification behavior described in the PRD.

## Contracts and data flow

- Generate OpenAPI 3.1 contracts first in packages/contracts/ and scaffold services from them (see [docs/PRD.md](https://github.com/IAmJonoBo/LifeReady/blob/main/docs/PRD.md) API contracts section).
- Data tiers (RED/AMBER/GREEN) and RBAC rules are enforced at the API boundary; no RED-tier data in logs or telemetry.
- Case workflows (EmergencyPack, MHCA39, DeathPack) are state machines; follow the state transitions in [docs/PRD.md](https://github.com/IAmJonoBo/LifeReady/blob/main/docs/PRD.md).
- Export packs must include manifests and checksums; keep the bundle structure aligned with PRD acceptance criteria.

## Implementation patterns

- Rust services are Axum-based and must include strict request validation, structured error types, tracing, and metrics (see [docs/copilot_scaffold_prompt.md](https://github.com/IAmJonoBo/LifeReady/blob/main/docs/copilot_scaffold_prompt.md)).
- Postgres schema migrations are required for estate/vault/case/audit data models; align tables to the PRD entity list.
- audit-service owns hash chaining and audit proof exports; audit-verifier CLI must validate head hash + canonical JSON chain.
- Flutter app is a single codebase (iOS/Android/Web) with a shared design system; create a design-system package with tokens and components before feature screens.

## Workflows and tooling

- CI should include lint/format, unit tests, contract tests, and Flutter web build as described in [docs/copilot_scaffold_prompt.md](https://github.com/IAmJonoBo/LifeReady/blob/main/docs/copilot_scaffold_prompt.md).
- When adding new endpoints or fields, update OpenAPI contracts first, then regenerate/update service handlers and tests.

## Key files and references

- Requirements and acceptance criteria: [docs/PRD.md](https://github.com/IAmJonoBo/LifeReady/blob/main/docs/PRD.md)
- Scaffold checklist and non-negotiables: [docs/copilot_scaffold_prompt.md](https://github.com/IAmJonoBo/LifeReady/blob/main/docs/copilot_scaffold_prompt.md)
