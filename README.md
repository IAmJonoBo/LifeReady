# LifeReady SA

LifeReady SA is a South Africa-first "servant software" platform for preparing and
executing incapacity and death-readiness workflows using:

- defensible documentation packs
- role-scoped access (RBAC)
- a tamper-evident, append-only audit trail

This repository is **contract-first**:

- OpenAPI 3.1 contracts are the source of truth
- Rust Axum stubs are generated from contracts
- CI fails on generated drift

> Important: LifeReady SA is not a law firm or medical provider. It organises documents,
> instructions, and evidence packs. It must never claim legal appointment, legal advice, or
> medical verification.

---

## Scope (v0.1)

### In scope

- SSoT Estate Vault: people, assets, instructions, role grants
- Encrypted Document Vault: document versions + checksums
- Packs:
  - Emergency Directive Pack (read-only, retrieval-focused)
  - MHCA 39 Incapacity Administration Pack (guided evidence pack; no incapacity claims)
  - Death Readiness Pack (executor nominee language only)
- Audit spine:
  - append-only, hash-chained audit log
  - exportable audit proof bundles

### Explicitly out of scope (v0.1)

- live biometric/IoT triggers
- medical optimisation, triage scheduling, or treatment recommendations
- automatic incapacity determination
- automatic release of high-risk secrets (seed phrases, custodial credentials)

---

## Tech stack

### Client

- Flutter (Android / iOS / Web)
- Design tokens (DTCG JSON) → generated Flutter tokens

### Backend

- Rust + Axum (services)
- PostgreSQL (local dev)
- Object storage later (Azure Blob in prod; emulator optional)

### Services

- identity-service
- estate-service
- vault-service
- case-service
- audit-service

---

## Repo layout

```text
apps/
  lifeready_flutter/

packages/
  contracts/                 # OpenAPI 3.1 contracts + generator config
  design-tokens/             # DTCG token source-of-truth
  audit-verifier/            # Rust CLI to verify audit chains/manifests

services/
  <service>/src              # handwritten service crate code
  <service>/generated        # generated stubs from OpenAPI (committed)
  <service>/migrations       # SQL migrations where applicable

scripts/
  validate-openapi.sh
  generate-axum.sh
  generate-flutter-tokens.sh
  markdownlint.sh
  upgrade-toolchain.sh

tools/
  openapi/                   # pinned openapi-generator wrapper (openapitools.json)
  markdownlint/
```

---

## Prerequisites

- Docker + docker compose
- Rust stable
- Flutter stable
- Node 20+

### Setup

OpenAPI Generator is invoked via the pinned wrapper under `tools/openapi`.
Validation uses `openapi-generator-cli validate --recommend`.

---

## Quick start (local dev)

1. Install OpenAPI tooling (pinned)

   ```bash
   npm install --prefix tools/openapi
   ```

2. Validate contracts + generate stubs

   ```bash
   make validate-openapi
   make generate-axum
   ```

3. Start local Postgres

   ```bash
   make dev-up
   ```

4. Configure environment

   ```bash
   cp .env.example .env
   ```

5. Run migrations (sqlx-cli)

   Install once:

   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

   Run migrations:

   ```bash
   make db-migrate
   ```

6. Run services (separate terminals)

   ```bash
   cargo run -p identity_service
   cargo run -p estate_service
   cargo run -p vault_service
   cargo run -p case_service
   cargo run -p audit_service
   ```

Default ports (override with `<SERVICE>_PORT` or `PORT` environment variables):

- identity-service: 8081
- estate-service: 8082
- vault-service: 8083
- case-service: 8084
- audit-service: 8085

1. Flutter

   ```bash
   make generate-flutter-tokens
   make flutter-check
   ```

---

## Make targets

- `make validate-openapi` / `make validate-openapi-<service>`
- `make generate-axum` / `make generate-axum-<service>`
- `make generate-flutter-tokens`
- `make flutter-check`
- `make dev-up` / `make dev-down`
- `make db-migrate`
- `make lint-docs`
- `make upgrade-toolchain`
- `make clean-generated`

---

## Drift prevention (non-negotiable)

- OpenAPI Generator is pinned via `tools/openapi/openapitools.json`
- CI validates OpenAPI and regenerates stubs; it fails on `git diff --exit-code`
- Generated outputs under `services/*/generated` are committed
- Flutter tokens are generated from DTCG JSON; CI fails if generated output drifts

---

## Development rules

- **Fail closed**: missing permissions/evidence blocks workflows
- **No claims** of legal appointment or medical verification
- **Keep contracts explicit**: generated code stays generated
- **Prefer consistent error shapes** (ProblemDetails) and request IDs once Phase 2 lands

---

## Roadmap (high level)

- **Phase 0**: repo hygiene + generation is green and deterministic
- **Phase 1**: local dev runs (all services boot + migrations reliable)
- **Phase 2**: auth + tiered RBAC + ProblemDetails everywhere
- **Phase 3**: Vault flows + pack exports + audit persistence + verifier wiring
