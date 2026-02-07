Recommended tech stack (SA-first, “remarkable” UX, Copilot-friendly)

Client (Android, iOS, Web)

Flutter (Dart) + Material 3 + Cupertino as the single UI codebase (high-fidelity, consistent design system, strong animation/storytelling, and one-team delivery across mobile + web). ￼

Suggested libraries (standard, scaffoldable):
• Routing: go_router
• State: riverpod (or bloc if you prefer stricter event models)
• Forms/validation: reactive_forms or formz
• Local cache: drift (SQLite) + encrypted storage for device cache
• Design system: custom tokens + component catalogue (build once; enforce everywhere)

Backend (compliance + security + determinism)
• Rust API services (Axum) + PostgreSQL (SSoT) + object storage for documents (Azure Blob)
• Event log: append-only audit store + hash-chained records (tamper-evident)
• Secrets: Azure Key Vault / HSM-backed keys (release via policy + staged workflows)
• Infra: Terraform + GitHub Actions

Cloud (SA locality)

Deploy primarily to Azure South Africa North (Johannesburg) (and optionally paired region strategy as you scale). ￼

⸻

LifeReady SA — PRD v0.1 (SA-only)

Optimised for GitHub Copilot scaffolding (clear modules, contracts, acceptance tests, repo layout).

0. Product statement

LifeReady SA is servant software for South Africans to prepare and execute incapacity and death readiness workflows with defensible documentation packs, role-scoped access, and tamper-evident audit trails. The platform does not practise law or medicine; it organises evidence, instructions, and exports.

1. Legal/compliance baseline (explicit, SA-only)

1.1 POPIA (Protection of Personal Information Act, 2013)
• Special personal information includes health and biometric information and is generally prohibited unless an authorisation/exception applies; system must treat health/biometric as RED-tier data. ￼
• Security compromise notification capability is required (productised incident workflow). ￼

1.2 Mental Health Care Act 17 of 2002 (MHCA)
• Support the Section 60/61 process: application to the Master of the High Court for appointment of an administrator; workflow must never claim incapacity “verified”, only assemble evidence and drafts. ￼
• Provide MHCA 39 export pack (prefill + checklist). ￼

⸻

2. Scope (v0.1) and non-scope

In scope (ship) 1. SSoT Estate Vault (structured inventory + documents + instructions) 2. Role invitations & RBAC (Principal / Proxy / Executor nominee / Emergency contact) 3. Document Vault (encrypted, versioned, tagged) 4. Packs
• Emergency Directive Pack (advance directive documents + contacts)
• Incapacity Administration Pack (MHCA 39 guided workflow + evidence checklist + export bundle)
• Death Readiness Pack (asset map + instructions + document index; no claim of executor appointment) 5. Auditability
• Append-only audit log
• Exportable evidence bundle (audit excerpt + manifest hashes)

Out of scope (explicitly)
• Live biometric/IoT triggers
• Medical optimisation / triage scheduling / treatment recommendations
• Automatic incapacity determination
• Automatic release of high-risk secrets (seed phrases, custodial credentials)

⸻

3. System goals (measurable)
   • Pack generation: < 10 seconds for typical user bundle (≤ 50 documents metadata + 10 PDFs)
   • Break-glass read-only access: < 30 seconds from Principal approval
   • Audit integrity: every access/export is logged; audit log is hash-chained and verifiable
   • Fail-closed: missing evidence blocks exports; no silent overrides

⸻

4. Data model (SSoT)

4.1 Entities (minimum)
• Person (Principal / Proxy / ExecutorNominee / EmergencyContact)
• RoleGrant (role, scope, status, expiry, approvals)
• Document (type, tags, version, checksum, storageRef, sensitivityTier)
• Instruction (free-text + structured fields)
• Asset (category, identifiers, owner, notes)
• Case (EmergencyPack | MHCA39Case | DeathPack)
• CaseArtifact (generated PDFs, manifests, checklists)
• AuditEvent (append-only)

4.2 Data tiers
• RED: health/biometric, secret material, medical evidence attachments
• AMBER: IDs, addresses, bank account metadata, relationship graph
• GREEN: non-sensitive operational metadata

⸻

5. Security & privacy requirements (build tasks)

5.1 POPIA-driven controls
• Enforce RED/AMBER/GREEN at API boundary (deny by default)
• Health/biometric treated as special personal information: explicit consent, purpose binding, revocation path ￼
• Built-in security compromise workflow (records, templates, evidence attachments) ￼

5.2 RBAC and staged access
• Read-only packs are the only “fast path” access
• Any escalation (write access, secret material) requires:
• step-up MFA
• multi-approval (Principal + secondary trusted contact OR time-lock)
• cooldown timer
• irreversible audit entry

5.3 Audit log (tamper-evident)
• Append-only store
• Hash chain: hash*i = H(hash*{i-1} || canonical_json(event_i))
• Export “audit proof”: include head hash + verification tool output

⸻

6. Functional modules (with acceptance criteria)

6.1 Document Vault

User stories
• Principal uploads documents; tags them; marks “authoritative”; sets who can view
• Proxy can view only permitted docs; cannot export RED-tier unless explicitly granted

Acceptance criteria
• Versioning (immutable old versions)
• Checksum stored per version (SHA-256)
• Encrypted storage reference only (no direct URLs in client)
• Document-level ACL enforced server-side

⸻

6.2 Emergency Directive Pack (read-only, no clinical output)

What it is
• Export bundle containing directive PDFs + contact sheet + quick-summary (user authored)

Acceptance criteria
• Generates EmergencyPack.pdf + manifest.json + checksums.txt
• Share link is time-limited; accesses are logged
• Revocation invalidates link immediately

⸻

6.3 MHCA 39 Incapacity Administration Pack (guided case workflow)

What it is
• A “case” that collects required particulars and attachments, generates a prefilled MHCA 39 draft, and exports a submission pack

Acceptance criteria
• Workflow blocks export until mandatory fields and evidence slots are completed per MHCA s60(2) particulars and MHCA39 structure ￼
• System never claims incapacity “verified”
• Export pack includes:
• MHCA39_draft.pdf
• evidence checklist
• asset summary
• attachments manifest + checksums

⸻

6.4 Death Readiness Pack (SA language-safe)

What it is
• A preparedness bundle for the nominated executor and family (documents, contacts, asset map, instructions)

Acceptance criteria
• Must label role as “Executor nominee” (until formally appointed)
• No “credential release” in v0.1
• Exportable pack + audit excerpt

⸻

6.5 POPIA Incident Module (Security Compromise)

What it is
• Internal workflow to record incidents and support notifications

Acceptance criteria
• Create incident record, affected data classes, user impact, mitigation steps
• Export: “Notification Pack” (for regulator/data subjects)
• Immutable audit events for incident edits

⸻

7. Workflow state machines (for scaffolding)

7.1 Emergency Directive Pack

stateDiagram-v2
[*] --> Draft
Draft --> Ready: docs + contacts present
Ready --> LinkIssued: principal step-up MFA
LinkIssued --> Accessed: recipient views
LinkIssued --> Revoked: principal revokes
LinkIssued --> Expired: TTL elapsed
Revoked --> [*]
Expired --> [*]

7.2 MHCA 39 Case

stateDiagram-v2
[*] --> CaseOpened
CaseOpened --> EvidenceCollecting
EvidenceCollecting --> DraftGenerated: mandatory fields complete
DraftGenerated --> AwaitingOath: applicant must swear/affirm externally
AwaitingOath --> SubmissionPackExported
EvidenceCollecting --> Blocked: missing required evidence slots
Blocked --> EvidenceCollecting
SubmissionPackExported --> Closed

7.3 Death Readiness Pack

stateDiagram-v2
[*] --> Draft
Draft --> Ready: will OR nomination + asset map
Ready --> Exported: step-up MFA
Exported --> AccessedByNominee: scoped access
Exported --> Revoked
Revoked --> [*]

⸻

8. API contracts (scaffold targets)

8.1 Service boundaries
• identity-service (auth, MFA, devices, sessions)
• estate-service (SSoT entities)
• vault-service (documents, versions, ACL)
• case-service (packs + MHCA cases + generators)
• audit-service (append-only log + verification exports)

8.2 Minimal REST endpoints (v0.1)
• POST /v1/auth/login
• POST /v1/auth/mfa/verify
• GET /v1/me
• POST /v1/people
• POST /v1/assets
• POST /v1/documents (init upload)
• POST /v1/documents/{id}/versions (commit)
• POST /v1/roles/grants (invite + scope)
• POST /v1/cases/emergency-pack
• POST /v1/cases/mhca39
• POST /v1/cases/{id}/export
• GET /v1/audit/export?caseId=...

(Copilot note: generate OpenAPI 3.1 spec first; scaffold handlers from spec.)

⸻

9. Repo layout (monorepo, Copilot-friendly)

lifeready-sa/
apps/
lifeready_flutter/ # Flutter app (iOS/Android/Web)
services/
identity-service/ # Rust (axum)
estate-service/ # Rust (axum)
vault-service/ # Rust (axum)
case-service/ # Rust (axum)
audit-service/ # Rust (axum)
packages/
contracts/ # OpenAPI specs + JSON Schemas + shared types
audit-verifier/ # CLI: verify hash-chain + manifests
infra/
terraform/
envs/dev/
envs/prod/
github-actions/
docs/
prd.md
threat-model.md
retention-matrix.md
runbooks/

⸻

10. Definition of Done (global gates)
    • ✅ All endpoints covered by contract tests (OpenAPI)
    • ✅ RBAC enforced server-side (unit + integration tests)
    • ✅ Audit hash-chain verification passes
    • ✅ Pack exports include manifest + checksums
    • ✅ No RED-tier data in logs/telemetry
    • ✅ POPIA incident workflow functional (create + export pack)
