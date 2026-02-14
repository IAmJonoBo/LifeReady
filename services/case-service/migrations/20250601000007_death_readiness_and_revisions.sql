-- Phase 3: Death readiness case metadata and incident revision history.

-- Death readiness case metadata (executor nominee + document references)
CREATE TABLE IF NOT EXISTS death_readiness_cases (
  case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,
  executor_nominee_person_id uuid NOT NULL,
  asset_document_ids uuid[] NOT NULL DEFAULT ARRAY[]::uuid[],
  contact_document_ids uuid[] NOT NULL DEFAULT ARRAY[]::uuid[],
  notes text
);

CREATE INDEX IF NOT EXISTS idx_death_readiness_cases ON death_readiness_cases(case_id);

-- Incident revision history (append-only updates for POPIA incidents)
CREATE TABLE IF NOT EXISTS incident_revisions (
  revision_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  case_id uuid NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
  revision_number int NOT NULL,
  summary text,
  mitigation_steps text,
  affected_data_classes text[],
  affected_user_count int,
  notes text,
  actor_principal_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(case_id, revision_number)
);

CREATE INDEX IF NOT EXISTS idx_incident_revisions_case ON incident_revisions(case_id);
