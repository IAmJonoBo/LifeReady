# ADR-001: Token Strategy (EdDSA with JWKS and revocation)

- Status: Accepted
- Date: 2026-02-12
- Owners: Identity + Security

## Context

Current authentication uses a shared-secret token model that does not provide
robust key rotation, key identifier (`kid`) traceability, or durable revocation
semantics for high-assurance internet exposure.

## Decision

- Use asymmetric JWT signing with EdDSA (Ed25519) as primary.
- Publish active and previous keys through JWKS with explicit `kid`.
- Enforce token validation for `iss`, `aud`, `iat`, `exp`, and `nbf` where applicable.
- Introduce session table and revocation list checks in identity and policy enforcement paths.
- Rotate keys on a fixed cadence and on incident response triggers.

## Consequences

- Token issuance and verification are decoupled from shared-secret distribution.
- Revocation checks add a datastore dependency to auth-critical flows.
- Key lifecycle management becomes a controlled operational process.

## Acceptance Criteria

- All services reject tokens with invalid `iss`/`aud`/expiry claims.
- Revoked session/token IDs are denied across all service boundaries.
- JWKS endpoint serves active keys and supports seamless key rollover.
- Security tests include tampered signature, wrong audience, expired token, and replay scenarios.

## Verification Evidence

- Integration tests: identity issuance + service-side verification.
- Adversarial tests: tamper/replay/wrong-audience matrix.
- CI gate: authentication and policy test suites must pass.
