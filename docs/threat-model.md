# LifeReady SA — Threat Model

> Status: Living document — reviewed each release cycle.

---

## 1. System overview

LifeReady SA manages sensitive South African personal and medical
documents through five backend services and a Flutter client.

| Component | Trust boundary | Data tiers handled |
|-----------|---------------|-------------------|
| identity-service | External → Internal | GREEN (session metadata) |
| estate-service | Internal | AMBER (people, assets, role grants) |
| vault-service | Internal | RED (health/medical docs), AMBER (IDs) |
| case-service | Internal | RED + AMBER (pack exports with evidence) |
| audit-service | Internal | GREEN (append-only event log) |
| Flutter client | External (untrusted) | All tiers rendered client-side |
| PostgreSQL | Internal | All tiers at rest |
| Object storage (Azure Blob) | Internal | RED + AMBER (document blobs) |

---

## 2. Data classification (POPIA-aligned)

| Tier | Examples | Controls |
|------|----------|----------|
| **RED** | Health/biometric data, medical evidence, secret material | Explicit consent, purpose binding, revocation path, encrypted at rest, never in logs/telemetry |
| **AMBER** | IDs, addresses, bank metadata, relationship graph | Role-scoped access, audit logged |
| **GREEN** | Operational metadata, timestamps, case status | Standard access controls |

---

## 3. Threat catalogue

### T-1: Broken authentication / session hijack

| | |
|---|---|
| **STRIDE** | Spoofing |
| **Attack** | Stolen JWT used to impersonate principal |
| **Mitigations** | HS256 signing with strong secret (≥32 chars), short-lived tokens, `LIFEREADY_ENV` enforcement rejects dev fallbacks in production, step-up MFA for escalation |
| **Residual risk** | Low — token theft requires client compromise |

### T-2: Privilege escalation via RBAC bypass

| | |
|---|---|
| **STRIDE** | Elevation of privilege |
| **Attack** | Proxy/EmergencyContact attempts write operations or accesses RED-tier data |
| **Mitigations** | Server-side `require_role()`, `require_tier()`, `require_scope()` enforcement on every handler; exhaustive match on role/tier enums; unit tests for all role combinations |
| **Residual risk** | Low — policy enforcement is compile-time exhaustive |

### T-3: Path traversal in document storage

| | |
|---|---|
| **STRIDE** | Information disclosure |
| **Attack** | Crafted `blob_ref` (e.g. `../../etc/passwd`) escapes storage directory |
| **Mitigations** | `resolve_blob_ref()` uses `canonicalize()` and validates result is within `storage_dir`; rejects file:// and absolute paths outside boundary |
| **Residual risk** | Low — canonical path comparison is deterministic |

### T-4: SQL injection

| | |
|---|---|
| **STRIDE** | Tampering |
| **Attack** | Malicious input in case/document fields reaches SQL |
| **Mitigations** | All queries use parameterised `sqlx::query().bind()` — no string interpolation of user input. `format!` for table names uses only compile-time string literals from exhaustive match (commented in code) |
| **Residual risk** | Very low |

### T-5: Audit log tampering

| | |
|---|---|
| **STRIDE** | Repudiation |
| **Attack** | Attacker modifies or deletes audit events to hide actions |
| **Mitigations** | Database triggers prevent UPDATE/DELETE on `audit_events`; SHA-256 hash chain (`prev_hash` → `event_hash`) detects insertion/deletion/reordering; `audit-verifier` CLI validates chain integrity |
| **Residual risk** | Low — requires database superuser access to bypass triggers |

### T-6: Pack export with incomplete evidence

| | |
|---|---|
| **STRIDE** | Tampering / Information disclosure |
| **Attack** | Exporting a pack before mandatory evidence is attached |
| **Mitigations** | `export_case()` checks all required evidence slots are filled; returns 409 Conflict if any are NULL; MHCA 39 workflow blocks export until mandatory fields complete |
| **Residual risk** | Very low — fail-closed by design |

### T-7: Sensitive data in logs/telemetry

| | |
|---|---|
| **STRIDE** | Information disclosure |
| **Attack** | RED-tier data appears in application logs or error responses |
| **Mitigations** | Database errors return generic "database operation failed" message to clients; raw errors logged server-side with `tracing::warn` using only request_id; no RED-tier field values in log macros |
| **Residual risk** | Low — requires ongoing code review |

### T-8: POPIA security compromise (data breach)

| | |
|---|---|
| **STRIDE** | Information disclosure |
| **Attack** | Breach of personal information requiring Section 22 notification |
| **Mitigations** | POPIA incident module: create incident record, collect evidence, export notification pack for regulator/data subjects; immutable audit events for incident edits; incident workflow documented in `docs/incident-workflow.md` |
| **Residual risk** | Medium — notification timeliness depends on operational process |

### T-9: Cross-service data leakage

| | |
|---|---|
| **STRIDE** | Information disclosure |
| **Attack** | estate-service or audit-service inadvertently exposes vault-service RED-tier documents |
| **Mitigations** | Service boundaries are explicit (PRD §8.1); each service owns its tables; case-service joins vault documents only during export with explicit access checks |
| **Residual risk** | Low — enforced by database schema separation |

### T-10: Denial of service via large pack exports

| | |
|---|---|
| **STRIDE** | Denial of service |
| **Attack** | Attacker creates cases with many large documents to exhaust disk/memory |
| **Mitigations** | PRD target: ≤50 document metadata + 10 PDFs per typical bundle; export creates ZIP with deflate compression; evidence slot limits per case type (7 for MHCA39, 5 for will prep, 7 for deceased estate) |
| **Residual risk** | Medium — no explicit file size limits enforced yet |

---

## 4. Trust boundaries diagram

```text
┌─────────────────────────────────────────────────┐
│                  Internet                        │
│  ┌──────────────────┐                           │
│  │  Flutter Client   │  (untrusted)             │
│  └────────┬─────────┘                           │
│           │ HTTPS + JWT                         │
├───────────┼─────────────────────────────────────┤
│           ▼                                     │
│  ┌──────────────────┐   ┌────────────────────┐  │
│  │ identity-service  │───│ Auth layer (JWT)   │  │
│  └──────────────────┘   └────────────────────┘  │
│           │                                     │
│  ┌────────┼────────┬──────────┬─────────┐       │
│  ▼        ▼        ▼          ▼         ▼       │
│ estate  vault    case      audit    (internal)  │
│ -svc    -svc     -svc      -svc                 │
│  │        │        │          │                  │
│  └────────┼────────┼──────────┘                  │
│           ▼        ▼                             │
│    ┌──────────┐  ┌──────────────┐               │
│    │PostgreSQL │  │ Object Store │               │
│    └──────────┘  └──────────────┘               │
│                  (internal)                      │
└─────────────────────────────────────────────────┘
```

---

## 5. Review cadence

- Reviewed by Engineering + Security lead each release cycle
- Updated when new services, endpoints, or data flows are added
- Cross-referenced with `docs/ops/runbooks.md` for incident procedures
