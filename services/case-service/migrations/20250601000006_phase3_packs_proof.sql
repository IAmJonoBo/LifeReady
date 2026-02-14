-- Phase 3: Packs + Proof — new case statuses, emergency pack metadata,
-- POPIA incident tables, and state-transition audit tracking.

-- Extend case_status with workflow states from PRD §7
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'evidence_collecting';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'draft_generated';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'awaiting_oath';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'link_issued';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'accessed';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'expired';

-- Extend case_type with POPIA incident
ALTER TYPE case_type ADD VALUE IF NOT EXISTS 'popia_incident';

-- Emergency pack metadata (contacts + directive references)
CREATE TABLE IF NOT EXISTS emergency_pack_cases (
  case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,
  directive_document_ids uuid[] NOT NULL DEFAULT ARRAY[]::uuid[],
  emergency_contacts jsonb NOT NULL DEFAULT '[]'::jsonb,
  share_link_token text,
  share_link_expires_at timestamptz,
  notes text
);

-- POPIA incident metadata
CREATE TABLE IF NOT EXISTS popia_incident_cases (
  case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,
  incident_title text NOT NULL,
  description text,
  affected_data_classes text[] NOT NULL DEFAULT ARRAY[]::text[],
  affected_user_count int,
  mitigation_steps text,
  reported_at timestamptz NOT NULL DEFAULT now(),
  required_evidence_slots text[] NOT NULL DEFAULT ARRAY[]::text[],
  notes text
);

-- State transition history for audit trail
CREATE TABLE IF NOT EXISTS case_transitions (
  transition_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  case_id uuid NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
  from_status text NOT NULL,
  to_status text NOT NULL,
  actor_principal_id uuid NOT NULL,
  reason text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_case_transitions_case ON case_transitions(case_id);
CREATE INDEX IF NOT EXISTS idx_emergency_pack_cases ON emergency_pack_cases(case_id);
CREATE INDEX IF NOT EXISTS idx_popia_incident_cases ON popia_incident_cases(case_id);
