# LifeReady SA — Data Retention Matrix

> Status: Living document — aligned with POPIA and system requirements.

---

## 1. Purpose

This matrix defines how long each data category is retained, the legal
basis for retention, and the disposal method. It supports POPIA compliance
(Condition 7: Security Safeguards; Section 14: Retention).

---

## 2. Retention schedule

### 2.1 Identity & authentication

<!-- markdownlint-disable MD013 -->
| Data element | Tier | Retention period | Legal basis | Disposal method |
| :--- | :--- | :--- | :--- | :--- |
| User account (email, phone hash) | AMBER | Active + 7 years after last login | POPIA §14; ECTA | Soft-delete → hard-delete after 7 years |
| Session tokens (JWT) | GREEN | Until expiry (configurable, default 5 min) | Operational | Auto-expired; not persisted |
| MFA secrets (TOTP seeds) | RED | Active while MFA enabled | POPIA §19 security | Cryptographic erasure on MFA reset |
| Device registrations | GREEN | Active + 90 days after removal | Operational | Hard-delete |
<!-- markdownlint-enable MD013 -->

### 2.2 Estate vault (people, assets, instructions)

<!-- markdownlint-disable MD013 -->
| Data element | Tier | Retention period | Legal basis | Disposal method |
| :--- | :--- | :--- | :--- | :--- |
| Person records | AMBER | Active + 7 years after estate closure | POPIA §14; Tax Admin Act | Soft-delete → anonymise |
| Asset records | AMBER | Active + 7 years after estate closure | Tax Admin Act §29 | Soft-delete → anonymise |
| Instruction records | AMBER | Active + 7 years | POPIA §14 | Soft-delete → anonymise |
| Role grants | AMBER | Active + 7 years after revocation | POPIA §14 | Soft-delete → hard-delete |
<!-- markdownlint-enable MD013 -->

### 2.3 Document vault

<!-- markdownlint-disable MD013 -->
| Data element | Tier | Retention period | Legal basis | Disposal method |
| :--- | :--- | :--- | :--- | :--- |
| Document metadata | AMBER | Active + 7 years | POPIA §14 | Soft-delete → anonymise |
| Document versions (blobs) | RED/AMBER | Active + 7 years | POPIA §14; MHCA | Cryptographic erasure of object store keys |
| Document ACLs | GREEN | Lifetime of document | Operational | Cascade-delete with document |
| SHA-256 checksums | GREEN | Lifetime of document | Integrity verification | Cascade-delete with document |
<!-- markdownlint-enable MD013 -->

### 2.4 Cases & packs

<!-- markdownlint-disable MD013 -->
| Data element | Tier | Retention period | Legal basis | Disposal method |
| :--- | :--- | :--- | :--- | :--- |
| Case records | AMBER | Active + 10 years | Administration of Estates Act §35 | Soft-delete → anonymise |
| MHCA 39 case metadata | RED | Active + 10 years | MHCA §66 records retention | Soft-delete → anonymise |
| Will prep case metadata | AMBER | Active + 10 years | Admin of Estates Act | Soft-delete → anonymise |
| Deceased estate metadata | AMBER | Active + 10 years | Admin of Estates Act | Soft-delete → anonymise |
| POPIA incident records | RED | Active + 5 years | POPIA §22 notification evidence | Immutable (audit-grade) |
| Emergency pack metadata | AMBER | Active + 7 years | POPIA §14 | Soft-delete → anonymise |
| Export artifacts (ZIPs) | RED/AMBER | 90 days after export | Operational (re-exportable) | Hard-delete from object store |
| State transitions | GREEN | Lifetime of case + 7 years | Audit evidence | Cascade with case |
| Evidence slots | RED/AMBER | Lifetime of case | Operational | Cascade-delete with case |
<!-- markdownlint-enable MD013 -->

### 2.5 Audit log

<!-- markdownlint-disable MD013 -->
| Data element | Tier | Retention period | Legal basis | Disposal method |
| :--- | :--- | :--- | :--- | :--- |
| Audit events | GREEN | 10 years minimum | POPIA §14; ECTA §16 | Immutable — never deleted (append-only) |
| Audit hash chain | GREEN | Lifetime of audit log | Integrity verification | Never deleted |
| Exported audit proofs | GREEN | 90 days (re-exportable) | Operational | Hard-delete from object store |
<!-- markdownlint-enable MD013 -->

---

## 3. POPIA-specific notes

### 3.1 Right to deletion (Section 24)

Data subjects may request erasure. The response depends on tier:

- **GREEN data**: Delete immediately
- **AMBER data**: Anonymise (replace PII with hashes); retain structural record for audit chain integrity
- **RED data**: Cryptographic erasure (destroy encryption keys); retain anonymised record

### 3.2 Cross-border transfers

Not applicable in v0.1 — all data stored in Azure South Africa North
(Johannesburg). If cross-border storage is introduced, POPIA §72
(adequate protection) must be assessed.

### 3.3 POPIA incident records

POPIA Section 22 security compromise records are classified as
**immutable** and retained for the full 5-year period regardless of
data subject deletion requests (legal obligation exemption under
POPIA §11(1)(c)).

---

## 4. Disposal procedures

<!-- markdownlint-disable MD013 -->
| Method | Description | Verification |
| :--- | :--- | :--- |
| **Soft-delete** | Set `deleted_at` timestamp; exclude from queries | Verify via database query |
| **Anonymise** | Replace PII fields with `REDACTED-{hash}`; retain structural keys | Spot-check anonymised records |
| **Hard-delete** | `DELETE FROM` + vacuum; blob removal from object store | Verify row count + storage audit |
| **Cryptographic erasure** | Destroy envelope key; ciphertext becomes unrecoverable | Verify key destruction in KMS audit log |
<!-- markdownlint-enable MD013 -->

---

## 5. Review cadence

- Reviewed annually by Engineering + Legal + Compliance
- Updated when new data categories or retention obligations are identified
- Cross-referenced with `docs/threat-model.md` and `docs/ops/runbooks.md`
