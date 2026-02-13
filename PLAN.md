# LifeReady Production Readiness, Gap Analysis, and Red-Team Plan (High Assurance)

## Summary

This plan closes the gap between the current repository state and an
internet-exposed, production-ready v0.1 system with complete engineering
assurance across security, reliability, testing, and QA.

Current baseline (verified):

- Automated tests pass across workspace (`cargo test --workspace --all-targets --all-features`) and Flutter smoke test passes.
- Rust coverage is materially below production expectation: total region coverage
  `57.48%`, with major deficits in `case-service` (`19.70%`),
  `vault-service` (`30.75%`), and `audit-service` (`43.86%`) from
  `cargo llvm-cov`.
- CI has quality checks but no explicit security or release-hardening gates yet (`/Volumes/APFS Space/GitHub/LifeReady/.github/workflows/ci.yml`).
- Several runtime flows are local-dev oriented and unsafe for internet production (for example `file://` URLs in APIs).

---

## Gap Analysis (Current State -> Required State)

<!-- markdownlint-disable MD013 -->

| Domain                     | Evidence                                                                                                                                                                                                                           | Gap                                                                                      | Production Target                                                                             |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| API security model         | `/Volumes/APFS Space/GitHub/LifeReady/packages/lifeready-auth/src/lib.rs`                                                                                                                                                          | HS256 shared secret, no key rotation/JWKS/KID, no token revocation/session store         | Asymmetric signing (EdDSA or RS256), key rotation, `kid`, revocation and session invalidation |
| Authentication robustness  | `/Volumes/APFS Space/GitHub/LifeReady/services/identity-service/src/lib.rs`                                                                                                                                                        | `verify_mfa` issues tokens without real challenge lifecycle or anti-brute-force controls | Stateful challenge validation, retry lockouts, per-user/IP throttling, abuse telemetry        |
| Data egress controls       | `/Volumes/APFS Space/GitHub/LifeReady/services/vault-service/src/lib.rs`, `/Volumes/APFS Space/GitHub/LifeReady/services/case-service/src/lib.rs`, `/Volumes/APFS Space/GitHub/LifeReady/services/audit-service/src/lib.rs`        | `file://` upload/export URLs and local path semantics are exposed                        | Signed object-storage URLs, short TTL, one-time use, strict path isolation                    |
| Audit durability           | `/Volumes/APFS Space/GitHub/LifeReady/packages/lifeready-audit/src/lib.rs`, `/Volumes/APFS Space/GitHub/LifeReady/services/estate-service/src/lib.rs`, `/Volumes/APFS Space/GitHub/LifeReady/services/identity-service/src/lib.rs` | In-memory audit sink in key services, partial persistence model                          | Central append-only audit service with mandatory writes for all privileged actions            |
| Readiness and operability  | `/Volumes/APFS Space/GitHub/LifeReady/services/*/src/lib.rs`                                                                                                                                                                       | `/healthz` only, no real dependency readiness, services can start with DB unavailable    | Distinct `/healthz` and `/readyz`, startup/rollback policy, dependency-aware readiness        |
| Coverage and quality gates | `cargo llvm-cov` baseline; `/Volumes/APFS Space/GitHub/LifeReady/.github/workflows/ci.yml`                                                                                                                                         | No coverage thresholds enforced; uneven service confidence                               | Coverage gates per crate/service and risk-weighted critical-path thresholds                   |
| Security pipeline          | `/Volumes/APFS Space/GitHub/LifeReady/.trunk/trunk.yaml`, `/Volumes/APFS Space/GitHub/LifeReady/.github/workflows/ci.yml`                                                                                                          | Security linters exist locally via Trunk but not enforced in CI release gate             | SAST/SCA/secrets/DAST/container scan and policy gate in CI                                    |
| End-to-end assurance       | Tests mostly service-local; no full multi-service workflow suite                                                                                                                                                                   | Missing true e2e across identity -> vault -> case -> audit -> verifier                   | Deterministic e2e suite in CI and pre-release environment                                     |
| QA programme               | `/Volumes/APFS Space/GitHub/LifeReady/apps/lifeready_flutter/test/widget_test.dart`                                                                                                                                                | Minimal Flutter QA depth and no formal QA exit criteria                                  | Test matrix, scripted UAT, exploratory chartering, defect triage rules                        |
| PRD completeness           | `/Volumes/APFS Space/GitHub/LifeReady/docs/PRD.md`                                                                                                                                                                                 | POPIA incident module and death-readiness capabilities not fully delivered               | Implement missing v0.1 PRD modules with tests and operational controls                        |
| Deployment/IaC             | No `infra/` directory in repo                                                                                                                                                                                                      | No production deployment artefacts, hardening, or runbooked ops                          | IaC, environment config, secret management, backup/restore and DR drills                      |

<!-- markdownlint-enable MD013 -->

---

## Red-Team Findings and Countermeasure Plan

### Priority abuse paths to close

1. Token forgery or replay against internet APIs.
2. MFA bypass and unlimited credential stuffing.
3. IDOR/path abuse using blob references and case/document IDs.
4. Data exfiltration through export endpoints with weak link semantics.
5. Cross-service privilege escalation through scope/tier mismatches.
6. Audit tampering or audit omission in non-persistent paths.
7. Resource exhaustion (export flood, large payload abuse, DB pressure).
8. Supply-chain and CI compromise leading to malicious build artefacts.

### Red-team controls to implement

- Enforce signed JWT with rotation and revocation, plus strict `iss`/`aud`/`exp` handling.
- Add rate limiting and anomaly detection at gateway and identity endpoints.
- Block absolute-path and unsafe URI forms at API boundary; enforce object-store key allowlist.
- Enforce mandatory audit emission for all privileged flows; fail closed on audit write failure for critical actions.
- Add authorisation invariants as testable policies (role+tier+scope+resource owner checks).
- Add DAST/fuzzing and adversarial regression suite as release blockers.
- Add provenance and dependency attestation (SBOM + signed artefacts).

---

## Implementation Plan (Decision Complete)

## Execution Artifacts

- ADR index: `docs/adr/README.md`
- Execution task board: `docs/e2e-production-readiness-task-board.md`
- These artifacts define implementation decisions, task dependencies, quality gates,
  acceptance criteria, and required evidence for E2E completion.

## Phase 1: Security and Platform Foundations (Week 1-2)

- Implement security architecture ADR set:
  - ADR-001 token strategy (EdDSA/RS256, JWKS, key rotation cadence, revocation design).
  - ADR-002 API edge controls (rate limiting, trusted proxy headers, CORS allowlist, request size limits).
  - ADR-003 storage/export model (object storage, signed URLs, TTL, no local file paths in API responses).
- Introduce environment model and config validation:
  - `dev`, `staging`, `production` strict mode.
  - Startup must fail if production-critical config is missing.
- Add `/readyz` endpoint contract and service implementation for all services with DB and dependency checks.
- Add production deployment skeleton:
  - `infra/terraform` for networking, compute, PostgreSQL, object storage, secrets manager, observability.
  - Per-environment variable catalog and secret reference mapping.

## Phase 2: Close High-Risk Runtime Gaps (Week 2-4)

- Identity hardening:
  - Stateful MFA challenge store, challenge expiry, replay protection.
  - Per-account/IP lockouts, backoff, and abuse alerts.
  - Session table with revocation support.
- Vault/case/audit hardening:
  - Replace `file://` workflow with signed object storage upload/download contracts.
  - Enforce strict blob reference canonicalisation and ownership checks.
  - Ensure case export pulls only case-scoped and owner-authorised artefacts.
- Audit hardening:
  - Migrate all audit writes to durable append-only path.
  - Require audit write success for security-critical operations.
  - Add integrity verification service task and alerting on mismatch.
- Network/service hardening:
  - Enforce TLS in transit, internal auth between services, and least-privilege DB credentials.
  - Add request body and field-size limits to all relevant handlers.

## Phase 3: PRD v0.1 Feature Completion (Week 3-5)

- Implement missing POPIA incident module:
  - Incident record lifecycle.
  - Notification-pack export with immutable audit trail.
- Implement death-readiness workflow:
  - Executor nominee semantics and language-safe constraints.
  - Export bundle with manifest/checksum and audit excerpt.
- Align contracts and generated code:
  - Update OpenAPI specs in `/Volumes/APFS Space/GitHub/LifeReady/packages/contracts`.
  - Regenerate and drift-check generated code in `/Volumes/APFS Space/GitHub/LifeReady/services/*/generated`.

## Phase 4: Test and Coverage Expansion (Week 2-6)

- Unit test expansion:
  - Auth and policy edge cases, serialisation boundaries, validation routines, error mapping.
- Integration tests:
  - DB-integrated tests for all write paths and conflict/error conditions.
  - Cross-service integration tests using a controlled test stack.
- E2E tests:
  - Full workflow suites:
    - Login -> MFA -> token -> people/assets/documents -> case creation -> evidence attach -> export -> audit verify.
    - POPIA incident create/update/export.
    - Death readiness create/export/access.
- Coverage gates:
  - Workspace region coverage >= 75%.
  - `case-service`, `vault-service`, `audit-service` each >= 70% region coverage.
  - Security-critical modules (`lifeready-auth`, `lifeready-policy`, export integrity paths) >= 90% line coverage.
  - New modules must not merge below 80% line coverage.
- Add CI enforcement:
  - `cargo llvm-cov --workspace --all-targets --all-features --summary-only` with fail thresholds.
  - Upload coverage report artefact.

## Phase 5: Security Testing and Red-Team Automation (Week 4-7)

- Add CI security jobs:
  - Rust dependency audit (`cargo audit`), policy checks (`cargo deny`), secrets scan, SAST, SBOM generation.
  - Container/IaC scans for deployment artefacts.
- Add DAST and API fuzzing:
  - Schemathesis/OpenAPI fuzzing against staging.
  - OWASP ZAP baseline active checks for externally reachable endpoints.
- Build adversarial regression pack:
  - JWT tamper/replay tests.
  - MFA brute-force tests.
  - IDOR and cross-tenant/cross-principal access tests.
  - Path traversal/blob-ref abuse tests.
  - Export abuse and denial-of-service test scenarios.
- Introduce periodic manual red-team exercise cadence:
  - Pre-go-live full exercise.
  - Quarterly revalidation with tracked findings.

## Phase 6: QA, Reliability, and Release Governance (Week 5-8)

- QA framework:
  - Test matrix by persona, platform, and critical workflow.
  - Scripted UAT for legal/compliance-sensitive user journeys.
  - Exploratory security and resilience charters.
- Non-functional verification:
  - Load tests for export-heavy workflows and auth endpoints.
  - Soak tests for sustained traffic and queue/DB behaviour.
  - Backup/restore drill and data integrity validation.
- Observability and SRE readiness:
  - Structured logs with PII/RED-tier scrubbing.
  - Metrics/traces dashboards and alert policies.
  - SLOs with error budget and incident response runbooks.
- Release policy:
  - Staging sign-off checklist.
  - Go/no-go board requiring security, QA, coverage, and operational approvals.

---

## Public API, Interface, and Type Changes Required

- Add readiness endpoint to all services:
  - `GET /readyz` with dependency status model.
- Replace local file references in API responses:
  - `upload_url`, `download_url` move to signed HTTPS URLs with TTL metadata.
- Extend identity/session contracts:
  - MFA challenge state, attempt counters, lockout and session revocation endpoints.
- Add incident-management endpoints and types (POPIA module).
- Add death-readiness case endpoints and export schemas.
- Standardise error contract:
  - Uniform ProblemDetails payload including stable error codes and correlation/request IDs.
- Add audit event schema constraints:
  - Explicit actor/resource fields and action taxonomy for detection.

---

## Test Cases and QA Scenarios (Must Exist Before Go-Live)

1. Valid principal workflow from login to successful case export and audit verification.
2. Invalid/expired/replayed JWT handling across all services.
3. MFA challenge replay and brute-force lockout behaviour.
4. Cross-principal document/case access attempts (IDOR negatives).
5. Tier/scope downgrade and escalation attempt matrix.
6. Blob reference/path traversal abuse attempts.
7. Export link misuse: expired, replayed, unauthorised principal.
8. Audit-chain tamper detection and verifier rejection.
9. DB outage and degraded dependency readiness gating.
10. High-volume export and auth load with latency/error SLO validation.
11. POPIA incident workflow end-to-end with immutable audit trail.
12. Death-readiness flow with executor nominee access constraints.
13. CI supply-chain checks detecting vulnerable dependency insertion.
14. Secret leakage checks in logs and telemetry outputs.
15. Backup restore + integrity verification drill.

---

## CI/CD Gate Definition (Release Blocking)

- Code quality:
  - `cargo fmt --check`, clippy, Rust tests, Flutter analyse/tests.
- Coverage:
  - All defined thresholds enforced.
- Security:
  - SAST, SCA, secrets scan, DAST/fuzz pass, SBOM generated.
- Contract integrity:
  - OpenAPI validation + generated drift checks.
- E2E:
  - Full critical-path suite green in staging.
- Operations:
  - Runbooks present, alerts validated, rollback tested.

---

## Assumptions and Defaults

- Confirmed with you:
  - Scope includes PRD gaps, not only current implementation.
  - Threat posture assumes public internet exposure.
  - Assurance target is high assurance.
- Defaults applied:
  - Production deploy target includes staged environments (`dev`, `staging`, `production`) with promotion gates.
  - PostgreSQL remains primary data store and object storage becomes mandatory for document/export artefacts.
  - OAuth/OIDC federation is out of immediate scope unless added as a separate requirement.
  - Existing lockfile mutation at `apps/lifeready_flutter/pubspec.lock` is treated
    as transient and should be discarded during implementation prep.

---

## Done Criteria for “Production-Ready e2E”

- All high/critical red-team findings are remediated or have approved compensating controls.
- Coverage and test gate thresholds are met and enforced in CI.
- E2E critical workflows (including incident and death-readiness modules) pass in staging.
- Security and compliance artefacts are generated and reviewable.
- SLO dashboards, incident runbooks, backup/restore, and rollback are validated.
- Formal go-live checklist is signed by engineering, security, and QA owners.
