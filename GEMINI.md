# Project Overview

This is a Rust-based monorepo for the LifeReady SA project, a "servant software" platform for preparing
and executing incapacity and death-readiness workflows. The backend is built with Rust and Axum, and
the frontend is a Flutter application. The project is "contract-first", meaning that the OpenAPI 3.1
contracts are the source of truth, and Rust Axum stubs are generated from them.

The repository contains several services, including `identity-service`, `estate-service`, `vault-service`,
`case-service`, and `audit-service`. It also includes a Flutter application and various packages for
shared functionality.

## Building and Running

The project uses `make` to simplify the build and run process. Here are the key commands:

- `make validate-openapi`: Validates all OpenAPI specifications.
- `make generate-axum`: Generates Rust Axum stubs from the OpenAPI contracts.
- `make dev-up`: Starts the local development environment using Docker Compose.
- `make db-migrate`: Runs the database migrations for all services.
- `make generate-flutter-tokens`: Regenerates Flutter tokens from DTCG JSON.
- `make flutter-check`: Runs tokens, analysis, and tests for the Flutter application.

## Core Concepts

- **RBAC**: Role-based access control is implemented throughout the services.
- **Sensitivity Tiers**: Data is classified into `green`, `amber`, and `red` tiers, with different access
  requirements.
- **Audit Chain**: All critical actions are recorded in an append-only, hash-chained audit trail.
- **Packs**: Guided evidence packs for different workflows (e.g., MHCA 39, Will Prep).

## Development Rules

- **Contract-First**: Always update the OpenAPI specs before modifying generated code.
- **Security-First**: Default to restrictive permissions.
- **Audit Everything**: Ensure all state-changing operations are logged to the audit service.

---

*Note: This project is in Phase 3 of development.*
