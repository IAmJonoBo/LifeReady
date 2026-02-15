# Project Overview

This is a Rust-based monorepo for the LifeReady SA project, a "servant software" platform for preparing and executing
incapacity and death-readiness workflows. The backend is built with Rust and Axum, and the frontend is a Flutter
application. The project is "contract-first", meaning that the OpenAPI 3.1 contracts are the source of truth, and Rust
Axum stubs are generated from them.

The repository contains several services, including `identity-service`, `estate-service`, `vault-service`,
`case-service`, and `audit-service`. It also includes a Flutter application and various packages for shared
functionality.

## Building and Running

The project uses `make` to simplify the build and run process. Here are the key commands:

- **Install dependencies:**
  - OpenAPI tooling: `npm install --prefix tools/openapi`
  - Rust dependencies: `cargo build` (implicitly fetched on build)
  - Flutter dependencies: `flutter pub get` (inside `apps/lifeready_flutter`)

- **Validate contracts and generate stubs:**
  - `make validate-openapi`
  - `make generate-axum`

- **Start the local development environment:**
  - `make dev-up` (starts a PostgreSQL container)

- **Run database migrations:**
  - `make db-migrate`

- **Run the services:**
  - `cargo run -p identity_service`
  - `cargo run -p estate_service`
  - `cargo run -p vault_service`
  - `cargo run -p case_service`
  - `cargo run -p audit_service`

- **Run the Flutter app:**
  - `make generate-flutter-tokens`
  - `flutter run` (inside `apps/lifeready_flutter`)

- **Run tests:**
  - Rust: `cargo test --workspace --all-targets --all-features`
  - Flutter: `flutter test` (inside `apps/lifeready_flutter`)

- **Linting:**
  - Rust: `cargo fmt -- --check` and `cargo clippy --workspace --all-targets --all-features`
  - Markdown: `make lint-docs`
  - Flutter: `flutter analyze`

## Development Conventions

- **Contract-first:** Always update the OpenAPI specs first, then run `make generate-axum`. Do not edit the generated
  code in `services/*/generated` by hand.
- **Drift prevention:** The CI pipeline will fail if there is a diff in the generated code. Make sure to commit the
  generated code.
- **Testing:** All code should be tested. The CI pipeline runs `cargo test` and `flutter test`.
- **Pull Requests:** Keep pull requests small and focused. Ensure that CI is passing before requesting a review.
