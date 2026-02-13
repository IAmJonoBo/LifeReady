# ADR-002: API Edge Controls and Abuse Mitigation

- Status: Accepted
- Date: 2026-02-12
- Owners: Platform + Security

## Context

Internet exposure requires robust boundary controls to prevent abuse,
brute-force attempts, oversized payload attacks, and unsafe cross-origin
invocation.

## Decision

- Enforce request size limits and strict schema validation at all service boundaries.
- Add per-IP and per-principal rate limits for authentication and export-sensitive routes.
- Restrict CORS to explicit allowlists per environment.
- Honor trusted proxy headers only from configured reverse-proxy CIDRs.
- Add structured abuse telemetry (rate-limit hits, lockouts, anomaly counts).

## Consequences

- Some legitimate spikes may be throttled and require tuning.
- Edge configuration and service config must remain synchronized.

## Acceptance Criteria

- Authentication brute-force attempts are rate-limited and lockout policy is enforced.
- Requests above configured payload limits return deterministic errors.
- Non-allowlisted origins are rejected.
- Abuse metrics are available in dashboards and alerts.

## Verification Evidence

- Load and abuse tests for auth and export endpoints.
- Configuration tests for trusted proxies and CORS allowlists.
- CI gate: API security checks and regression suite must pass.
