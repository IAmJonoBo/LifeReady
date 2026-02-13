# ADR-005: E2E Assurance Quality Gates and Release Governance

- Status: Accepted
- Date: 2026-02-12
- Owners: Engineering + QA + Security

## Context

Current checks are useful but insufficient to guarantee production-readiness
for internet exposure and compliance-sensitive workflows.

## Decision

- Enforce release-blocking gates across quality, security, contracts, and operability.
- Require deterministic E2E critical-path validation in staging.
- Require evidence-backed go/no-go checklist sign-off.

## Quality Gates

- Code quality: `cargo fmt --check`, clippy, Rust tests, Flutter analyze/tests.
- Coverage: workspace and per-service thresholds as defined in `PLAN.md`.
- Security: SAST, SCA, secrets scan, DAST/fuzz, SBOM generation.
- Contract integrity: OpenAPI validation and generated-code drift checks.
- E2E: identity -> vault -> case -> audit -> verifier critical flows green.
- Ops readiness: runbooks, alerts, backup/restore, rollback validation complete.

## Acceptance Criteria

- No release if any quality gate fails.
- All critical workflow scenarios pass in staging.
- Security and compliance artifacts are attached to release evidence.
- Engineering, QA, and Security sign-offs are present.

## Verification Evidence

- CI workflow results and stored artifacts.
- Staging E2E run report with scenario outcomes.
- Signed go/no-go checklist linked in release record.
