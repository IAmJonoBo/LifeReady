# E2E Production Readiness Task Board

This task board operationalizes `PLAN.md` and `docs/adr/*` into execution units with quality gates and acceptance criteria.

## Usage

- Mark each task complete only when all acceptance criteria and evidence are satisfied.
- A phase is complete only when every task in that phase is complete.
- Release eligibility requires all phase gates and the global gates in ADR-005.

## Phase 1: Security and Platform Foundations

### Task P1-1: Adopt token architecture (ADR-001)

- Owners: Identity + Security
- Dependencies: none
- Acceptance criteria:
  - Asymmetric keypair support implemented and active.
  - JWKS endpoint exposes current and previous keys with `kid`.
  - Services validate `iss` and `aud` claims.
- Evidence:
  - Unit + integration tests for key rotation and claim validation.
  - CI job artifacts for auth tests.

### Task P1-2: Implement API edge controls baseline (ADR-002)

- Owners: Platform
- Dependencies: none
- Acceptance criteria:
  - Request size limits active on all internet-facing handlers.
  - CORS allowlist environment-scoped and tested.
  - Trusted proxy handling restricted by configuration.
- Evidence:
  - Config tests and negative request tests.
  - Security lint output and CI logs.

### Task P1-3: Add readiness model and deployment skeleton

- Owners: Platform + Service leads
- Dependencies: none
- Acceptance criteria:
  - `GET /readyz` implemented for all services with dependency checks.
  - Environment strict-mode validation implemented (`dev`, `staging`, `production`).
  - `infra/terraform` skeleton exists with environment variables catalog.
- Evidence:
  - Service readiness integration tests.
  - Startup-failure tests for missing production config.

## Phase 2: Close High-Risk Runtime Gaps

### Task P2-1: Harden MFA lifecycle and session revocation

- Owners: Identity
- Dependencies: P1-1
- Acceptance criteria:
  - Stateful challenge lifecycle with expiry and replay prevention.
  - Per-user/IP lockouts and anti-brute-force policy enforced.
  - Session revocation endpoint and checks wired into auth path.
- Evidence:
  - MFA replay and brute-force regression tests.

### Task P2-2: Replace local file semantics with signed object URLs (ADR-003)

- Owners: Vault + Case + Audit
- Dependencies: P1-2
- Acceptance criteria:
  - No `file://` responses in APIs.
  - Signed URL TTL + one-time behavior validated.
  - Blob reference canonicalization and owner checks enforced.
- Evidence:
  - Contract tests and negative authorization tests.

### Task P2-3: Enforce durable fail-closed audit writes (ADR-004)

- Owners: Audit + Service leads
- Dependencies: P1-3
- Acceptance criteria:
  - Privileged actions fail when durable audit write fails.
  - Audit hash chain remains verifiable after all privileged operations.
- Evidence:
  - Failure-injection integration tests.
  - Audit-verifier chain test report.

## Phase 3: PRD v0.1 Feature Completion

### Task P3-1: POPIA incident module completion

- Owners: Case service + App
- Dependencies: P2-3
- Acceptance criteria:
  - Incident lifecycle endpoints and state transitions implemented.
  - Notification-pack export includes immutable audit references.
- Evidence:
  - API tests and E2E scenario report.

### Task P3-2: Death-readiness workflow completion

- Owners: Estate + Case + App
- Dependencies: P2-2, P2-3
- Acceptance criteria:
  - Executor nominee constraints enforced.
  - Export bundle manifest/checksum and scoped access implemented.
- Evidence:
  - Workflow tests and export-integrity verification report.

### Task P3-3: Contract-first drift-proof regeneration

- Owners: Platform + Service leads
- Dependencies: none
- Acceptance criteria:
  - OpenAPI updates committed first.
  - Generated handlers checked and drift-free in CI.
- Evidence:
  - `make validate-openapi` and generation logs.

## Phase 4: Test and Coverage Expansion

### Task P4-1: Raise service and package coverage to defined thresholds

- Owners: All service teams
- Dependencies: P2, P3 tasks
- Acceptance criteria:
  - Workspace region coverage >= 75%.
  - `case-service`, `vault-service`, `audit-service` each >= 70% region.
  - `lifeready-auth`, `lifeready-policy`, export-integrity paths >= 90% line.
- Evidence:
  - `cargo llvm-cov` summary artifact in CI.

### Task P4-2: Expand integration and cross-service E2E suite

- Owners: QA + Service leads
- Dependencies: P3-1, P3-2
- Acceptance criteria:
  - Critical path flows execute deterministically in staging.
  - Failure-path assertions included for auth, authorization, and export checks.
- Evidence:
  - Staging E2E run report and flaky-test monitor output.

## Phase 5: Security Testing and Red-Team Automation

### Task P5-1: Add CI security gate stack

- Owners: Security + Platform
- Dependencies: P1-2
- Acceptance criteria:
  - `cargo audit`, `cargo deny`, secrets scan, SBOM generation enforced.
  - Container/IaC scanning present for deployment assets.
- Evidence:
  - CI security stage artifacts and policy reports.

### Task P5-2: Add DAST and API fuzzing release blockers

- Owners: Security QA
- Dependencies: P3-3
- Acceptance criteria:
  - Schemathesis against OpenAPI in staging.
  - OWASP ZAP baseline checks passing for externally reachable routes.
- Evidence:
  - DAST/fuzz reports attached to release candidate.

### Task P5-3: Adversarial regression pack

- Owners: Security + QA
- Dependencies: P2-1, P2-2, P2-3
- Acceptance criteria:
  - JWT tamper/replay, MFA brute force, IDOR, path traversal, and export abuse tests automated.
- Evidence:
  - Regression suite report stored in CI artifacts.

## Phase 6: QA, Reliability, and Release Governance

### Task P6-1: QA program and UAT readiness

- Owners: QA
- Dependencies: P4-2
- Acceptance criteria:
  - Persona/platform workflow matrix complete.
  - Scripted UAT and exploratory charters executed.
- Evidence:
  - QA run sheets, defects, and closure report.

### Task P6-2: Reliability and operability validation

- Owners: SRE + Platform
- Dependencies: P1-3, P4-2
- Acceptance criteria:
  - Load + soak tests complete for auth and export heavy flows.
  - Backup/restore and rollback drills validated.
  - SLO dashboards and actionable alerts live.
- Evidence:
  - Performance reports and drill logs.

### Task P6-3: Go/no-go governance closure (ADR-005)

- Owners: Engineering + Security + QA
- Dependencies: all prior tasks
- Acceptance criteria:
  - All global quality gates pass.
  - No unresolved critical/high findings without approved compensating controls.
  - Formal sign-off recorded by engineering, security, and QA.
- Evidence:
  - Signed release checklist and linked CI evidence bundle.

## Global Release Gates (Must All Pass)

- Code quality gates pass.
- Coverage thresholds pass.
- Security gates pass.
- Contract drift checks pass.
- Critical-path E2E in staging passes.
- Operational readiness evidence complete.
