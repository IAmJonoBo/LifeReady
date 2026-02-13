# ADR-003: Object Storage and Signed URL Export Model

- Status: Accepted
- Date: 2026-02-12
- Owners: Vault + Case + Platform

## Context

Local file-path semantics (`file://`) are unsafe for production and
incompatible with secure, time-bounded document and export distribution.

## Decision

- Replace local paths with signed HTTPS object-storage URLs.
- Use short TTL URLs with single-use semantics where feasible.
- Canonicalize and validate blob/object references against strict allowlists.
- Return bundle manifests and checksums for all exports.

## Consequences

- Requires object storage integration and URL signing key management.
- Consumers must support URL expiry and refresh behavior.

## Acceptance Criteria

- No API response exposes `file://` or absolute local paths.
- Unauthorized principal access to object references is denied.
- Export packs include manifest + checksums and pass verifier checks.

## Verification Evidence

- Integration tests for upload/download/export contracts.
- Security tests for IDOR, path traversal, and expired/replayed URLs.
- CI gate: export integrity tests and negative-access tests must pass.
