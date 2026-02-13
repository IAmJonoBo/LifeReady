# ADR-004: Durable Audit and Fail-Closed Privileged Flows

- Status: Accepted
- Date: 2026-02-12
- Owners: Audit + Service Leads

## Context

In-memory or best-effort audit emission is insufficient for
compliance-sensitive privileged actions and weakens forensic integrity.

## Decision

- Route all privileged audit events to the append-only durable audit service.
- Preserve hash-chain integrity and canonical serialization for verification.
- Fail closed for security-critical actions when audit persistence fails.
- Provide periodic audit-chain verification and alerting on mismatch.

## Consequences

- Availability of privileged actions depends on audit-service durability.
- Operational runbooks must include audit-chain incident response.

## Acceptance Criteria

- Privileged actions cannot complete without durable audit persistence.
- Audit verifier validates head hash and chain continuity.
- Chain integrity alerts trigger on corruption or missing links.

## Verification Evidence

- Service integration tests with induced audit-service failures.
- Verifier tests over generated and tampered chains.
- CI gate: audit integration and verifier suites must pass.
