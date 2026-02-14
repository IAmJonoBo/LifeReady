# Audit Hashing Specification

> Single source of truth for the audit hash-chain algorithm used by
> `audit-service` (producer) and `audit-verifier` (consumer / CLI).

## 1. Overview

Every audit event is appended to an immutable, ordered log. Each event
carries a `prev_hash` (the hash of the preceding event) and an
`event_hash` (the hash of the current event combined with `prev_hash`).
Together these form a hash chain that allows any party to detect
tampering or deletion.

## 2. Canonical JSON Serialisation

Before hashing, the event payload is serialised to **canonical JSON**:

1. All object keys are sorted lexicographically (Unicode code-point
   order).
2. Nested objects are sorted recursively.
3. Arrays preserve their original order.
4. No extraneous whitespace (compact serialisation).
5. Null values are included as `null`.

### Fields included in canonical form

```json
{
  "action": "<string>",
  "actor_principal_id": "<uuid-string>",
  "case_id": "<uuid-string | null>",
  "created_at": "<RFC 3339 timestamp>",
  "event_id": "<uuid-string>",
  "payload": { ... },
  "tier": "<green | amber | red>"
}
```

> **Note:** `prev_hash` and `event_hash` are *not* included in the
> canonical form — they are computed *from* it.

## 3. Hash Computation

```text
event_hash = SHA-256(
    prev_hash_hex          // 64 hex chars of the previous event hash
    || canonical_json      // canonical JSON bytes (UTF-8)
)
```

- `prev_hash` of the first event in a chain is `"0" × 64`
  (64 ASCII zeros).
- The output is lower-case hex-encoded (64 characters).

### Pseudocode

```rust
fn compute_event_hash(prev_hash: &str, event: &AuditEvent) -> String {
    let canonical = canonical_event_json(event);
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}
```

## 4. Chain Verification

The verifier walks the log line by line:

1. Initialise `expected_prev = "0" × 64`.
2. For each event line (JSONL):
   a. Parse the event.
   b. Assert `event.prev_hash == expected_prev`.
   c. Recompute `event_hash` from `expected_prev` + canonical JSON.
   d. Assert computed hash matches `event.event_hash`.
   e. Set `expected_prev = event.event_hash`.
3. After the last line, the final `expected_prev` is the **head hash**.
4. If an expected head hash was supplied, assert it matches.

## 5. Export Manifest Verification

Export bundles contain a `manifest.json` with:

| Field                 | Description                              |
|-----------------------|------------------------------------------|
| `case_id`             | Case identifier                          |
| `case_type`           | Type of case                             |
| `exported_at`         | RFC 3339 timestamp                       |
| `audit_head_hash`     | Head hash of the included audit chain    |
| `audit_events_sha256` | SHA-256 of the `audit.jsonl` file bytes  |
| `documents[]`         | Array of document entries with checksums |

### Document entry

| Field          | Description                      |
|----------------|----------------------------------|
| `slot_name`    | Evidence slot or document index  |
| `document_id`  | UUID of the document             |
| `document_type`| Type label                       |
| `title`        | Human-readable title             |
| `sha256`       | SHA-256 of the bundled file      |
| `bundle_path`  | Relative path inside the bundle  |

### Verification steps

1. Recompute SHA-256 of `audit.jsonl` and compare to
   `audit_events_sha256`.
2. Verify the audit chain (§4) and compare head hash to
   `audit_head_hash`.
3. For each document, recompute SHA-256 of the bundled file and compare
   to the manifest entry.
4. Any mismatch → **fail closed** (reject the bundle).

## 6. Implementations

| Crate / Package     | Role     | Location                                |
|---------------------|----------|-----------------------------------------|
| `audit-service`     | Producer | `services/audit-service/src/lib.rs`     |
| `audit-verifier`    | Consumer | `packages/audit-verifier/src/lib.rs`    |

Both implementations use the same `canonicalize_value` function that
recursively sorts object keys and the same `compute_event_hash` logic.

## 7. Security Considerations

- The chain uses SHA-256, which is collision-resistant for integrity
  verification purposes.
- RED-tier payloads must never appear in logs or telemetry; the audit
  service enforces tier gating at the API boundary.
- Tampering with any single event breaks the chain from that point
  forward, ensuring fail-closed behaviour.
