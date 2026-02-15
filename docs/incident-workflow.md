# LifeReady SA — POPIA Incident Workflow

> Status: Living document — aligned with POPIA Section 22 and the LifeReady
> case-service POPIA incident module.

---

## 1. Purpose

This document describes the end-to-end workflow for handling a security
compromise (data breach) under the Protection of Personal Information Act
(POPIA), Section 22. It maps each legal obligation to an action in
LifeReady SA.

---

## 2. Legal context

POPIA Section 22 requires a responsible party who has reasonable grounds
to believe that personal information has been accessed or acquired by an
unauthorised person to:

1. **Notify the Information Regulator** as soon as reasonably possible
2. **Notify affected data subjects** unless the identity of the subjects
   cannot be established
3. Provide sufficient information for data subjects to take protective
   measures

---

## 3. Workflow overview

```text
┌──────────────┐
│  Incident     │
│  Detected     │
└──────┬───────┘
       │
       ▼
┌──────────────┐     POST /v1/cases/popia-incident
│  Create       │──────────────────────────────────►  case_type = popia_incident
│  Incident     │                                     status = draft
│  Record       │
└──────┬───────┘
       │
       ▼
┌──────────────┐     PUT /v1/cases/{id}/evidence/{slot}
│  Collect      │──────────────────────────────────►  Attach evidence to slots:
│  Evidence     │                                     • incident_report
└──────┬───────┘                                      • affected_data_summary
       │                                              • mitigation_evidence
       ▼                                              • regulator_notification_draft
┌──────────────┐     POST /v1/cases/{id}/transition   • data_subject_notification_draft
│  Transition   │──────────────────────────────────►  draft → ready
│  to Ready     │
└──────┬───────┘
       │
       ▼
┌──────────────┐     POST /v1/cases/{id}/export
│  Export       │──────────────────────────────────►  Generates:
│  Notification │                                     • popia_notification_pack.json
│  Pack         │                                     • popia_instructions.md
└──────┬───────┘                                      • manifest.json + checksums.txt
       │                                              • audit.jsonl
       ▼                                              • ZIP archive
┌──────────────┐     POST /v1/cases/{id}/transition
│  Transition   │──────────────────────────────────►  ready → exported
│  to Exported  │
└──────┬───────┘
       │
       ▼
┌──────────────┐     POST /v1/cases/{id}/transition
│  Close        │──────────────────────────────────►  exported → closed
│  Incident     │
└──────────────┘
```

---

## 4. Step-by-step procedure

### Step 1 — Detect and report internally

<!-- markdownlint-disable MD013 -->
| Action | Owner | Timeline |
| :--- | :--- | :--- |
| Identify potential compromise | Any team member | Immediately |
| Confirm scope (affected systems, data classes, user count) | Engineering lead | Within 2 hours |
| Escalate to responsible party (DPO / executive) | Engineering lead | Within 4 hours |
<!-- markdownlint-enable MD013 -->

### Step 2 — Create incident record

```bash
curl -X POST /v1/cases/popia-incident \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "incident_title": "Descriptive title of the breach",
    "description": "What happened, how it was discovered",
    "affected_data_classes": ["health", "identity", "financial"],
    "affected_user_count": 150,
    "mitigation_steps": "Access revoked, keys rotated, ...",
    "notes": "Internal reference: INC-2026-001"
  }'
```

This creates a case with `case_type = popia_incident` and `status = draft`.

Default evidence slots are created automatically:

<!-- markdownlint-disable MD013 -->
| Slot | Purpose |
| :--- | :--- |
| `incident_report` | Formal written report of the breach |
| `affected_data_summary` | Inventory of compromised personal information categories |
| `mitigation_evidence` | Proof of remedial steps taken |
| `regulator_notification_draft` | Draft letter/form for the Information Regulator |
| `data_subject_notification_draft` | Draft notification to affected individuals |
<!-- markdownlint-enable MD013 -->

### Step 3 — Collect evidence

Upload supporting documents to vault-service, then attach each to the
appropriate evidence slot:

```bash
# Attach the incident report document
curl -X PUT /v1/cases/{case_id}/evidence/incident_report \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"document_id": "<uuid>"}'
```

Repeat for each evidence slot. The export will block (409 Conflict) until
all required slots have documents attached.

### Step 4 — Review and transition to ready

Once all evidence is attached and reviewed:

```bash
curl -X POST /v1/cases/{case_id}/transition \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"to_status": "ready", "reason": "All evidence collected and reviewed"}'
```

### Step 5 — Export notification pack

```bash
curl -X POST /v1/cases/{case_id}/export \
  -H "Authorization: Bearer $TOKEN"
```

The export produces a ZIP containing:

<!-- markdownlint-disable MD013 -->
| File | Content |
| :--- | :--- |
| `popia_notification_pack.json` | Structured incident data: title, description, affected classes, user count, mitigation, evidence checklist |
| `popia_instructions.md` | POPIA Section 22 obligations and next steps |
| `manifest.json` | Case metadata, document checksums, audit head hash |
| `checksums.txt` | SHA-256 checksums for all bundle files |
| `audit.jsonl` | Hash-chained audit events (if `read:all` scope) |
| `documents/` | Attached evidence files |
<!-- markdownlint-enable MD013 -->

### Step 6 — Submit to regulator and notify data subjects

<!-- markdownlint-disable MD013 -->
| Action | Owner | Timeline |
| :--- | :--- | :--- |
| Submit notification pack to Information Regulator | DPO | As soon as reasonably possible |
| Send data subject notifications | DPO + Comms | As soon as reasonably possible |
| Record submission timestamps | DPO | Same day |
<!-- markdownlint-enable MD013 -->

### Step 7 — Close the incident

```bash
curl -X POST /v1/cases/{case_id}/transition \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"to_status": "exported", "reason": "Pack exported for submission"}'

# After regulator submission confirmed:
curl -X POST /v1/cases/{case_id}/transition \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"to_status": "closed", "reason": "Regulator notified, subjects informed"}'
```

---

## 5. State machine

```text
draft ──► ready ──► exported ──► closed
```

Each transition is:

- Validated against the POPIA incident state machine in `allowed_transitions()`
- Recorded in the `case_transitions` table with actor, timestamp, and reason
- Immutably logged in the audit trail

---

## 6. Notification content requirements (POPIA §22(4))

The notification to the Information Regulator and data subjects must
include:

<!-- markdownlint-disable MD013 -->
| Required element | Source in LifeReady |
| :--- | :--- |
| Description of the compromise | `popia_incident_cases.description` |
| Category of personal information involved | `popia_incident_cases.affected_data_classes` |
| Identity and contact details of responsible party | Organisation config (not stored per-incident) |
| Description of measures taken or proposed | `popia_incident_cases.mitigation_steps` |
| Recommendations to data subjects | `popia_instructions.md` template |
| Identity of unauthorised person (if known) | `incident_report` evidence slot |
<!-- markdownlint-enable MD013 -->

---

## 7. Audit trail

Every action in the incident workflow is captured in the append-only,
hash-chained audit log:

- Incident creation
- Evidence attachment (each slot)
- State transitions (with actor and reason)
- Pack export (with manifest SHA-256)

The audit chain can be independently verified using the `audit-verifier`
CLI:

```bash
audit-verifier verify-bundle --bundle <path-to-export-zip>
```

---

## 8. Retention

POPIA incident records are classified as **immutable** and retained for
**5 years minimum** (see `docs/retention-matrix.md` §2.4). They are exempt
from data subject deletion requests under POPIA §11(1)(c) (legal
obligation).

---

## 9. Review cadence

- Reviewed after each incident and annually
- Validated against latest Information Regulator guidance
- Cross-referenced with `docs/threat-model.md` and `docs/ops/runbooks.md`
