# LifeReady SA

LifeReady SA is a South Africa-first "servant software" platform for preparing and executing
incapacity and death-readiness workflows using:

- defensible documentation packs
- role-scoped access (RBAC)
- a tamper-evident, append-only audit trail

This repository is contract-first:

- OpenAPI 3.1 contracts are the source of truth
- Rust Axum stubs are generated from contracts
- CI fails on generated drift

> Important: LifeReady SA is not a law firm or medical provider. It organizes documents,
> instructions, and evidence packs. It must never claim legal appointment or medical
> verification.

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

### Explicitly out of scope

- live biometric/IoT triggers
- medical optimization, triage scheduling, or treatment recommendations
- automatic incapacity determination
- automatic release of high-risk secrets (seed phrases, custodial credentials)

---

## Tech stack

### Client

- Flutter (Android / iOS / Web)
- Design tokens (DTCG JSON) -> generated Flutter tokens

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

apps/
lifeready_flutter/

packages/
contracts/ # OpenAPI 3.1 contracts + generator config
design-tokens/ # DTCG token source-of-truth
audit-verifier/ # Rust CLI to verify audit chains/manifests

services/
service/src # handwritten service crate code
service/generated # generated stubs from OpenAPI
service/migrations # SQL migrations where applicable

scripts/
validate-openapi.sh
generate-axum.sh
generate-flutter-tokens.sh

tools/
openapi/ # pinned openapi-generator wrapper

---

## Prerequisites

- Docker + docker compose
- Rust stable
- Flutter stable
- Node 20+

OpenAPI Generator is invoked via the pinned wrapper in tools/openapi. See OpenAPI Generator CLI usage and installation docs:

- [OpenAPI Generator usage](https://openapi-generator.tech/docs/usage/)
- [OpenAPI Generator installation](https://openapi-generator.tech/docs/installation/)

---

## Quick start (local dev)

### 1) Install OpenAPI generator wrapper

```bash
npm install --prefix tools/openapi
```

### 2) Validate contracts + generate stubs

```bash
make validate-openapi
make generate-axum
```

### 3) Start local Postgres

```bash
make dev-up
```

### 4) Configure environment

```bash
cp .env.example .env
```

### 5) Run migrations (sqlx-cli)

Install sqlx-cli once:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Run migrations:

```bash
make db-migrate
```

### 6) Run services (separate terminals)

```bash
cargo run -p identity_service
cargo run -p estate_service
cargo run -p vault_service
cargo run -p case_service
cargo run -p audit_service
```

Default ports (override with PORT):

- identity-service: 8081
- estate-service: 8082
- vault-service: 8083
- case-service: 8084
- audit-service: 8085

### 7) Flutter app

```bash
make generate-flutter-tokens
make flutter-check
```

---

## Make targets

- `make validate-openapi` / `make validate-openapi-<service>`
- `make generate-axum` / `make generate-axum-<service>`
- make generate-flutter-tokens
- make flutter-check
- make dev-up / make dev-down
- make db-migrate

---

## Drift prevention

- OpenAPI generator is pinned via tools/openapi/openapitools.json and invoked through make scripts.
- CI validates OpenAPI, regenerates stubs, and fails on git diff --exit-code.
- Flutter tokens are generated from DTCG JSON; CI fails if generated output drifts.
