# Copilot Scaffold Prompt

## Goal

Generate a production-grade scaffold matching the PRD.

## Instructions to Copilot

1. Create the monorepo layout exactly as specified.
2. Generate OpenAPI 3.1 specs for all services in `packages/contracts/`.
3. Scaffold Rust Axum services from OpenAPI with:
   - strict request validation
   - structured error types
   - tracing and metrics
4. Implement Postgres schema migrations (SQL) for estate, vault, case, and audit.
5. Implement `audit-service` hash chaining plus the `audit-verifier` CLI.
6. Create Flutter app shell with:
   - auth screens, role invitations, document upload, and case creation
   - a design-system package with tokens and components
7. Add CI for lint/format, unit tests, contract tests, and Flutter web build.
8. Add docs for threat model, retention matrix, runbooks, and incident workflow.

## Deterministic scaffolding and drift prevention

- OpenAPI Generator must be pinned via `tools/openapi/openapitools.json` and
  installed with npm in `tools/openapi`.
- OpenAPI validation and generation are run via Make targets:
  `make validate-openapi`, `make generate-axum`.
- Flutter tokens are generated from DTCG JSON and must be kept in sync via
  `make generate-flutter-tokens`.
- CI must fail on generated drift using `git diff --exit-code` after OpenAPI
  generation and token generation.
