-- Add new case types to the enum
ALTER TYPE case_type ADD VALUE IF NOT EXISTS 'will_prep_sa';
ALTER TYPE case_type ADD VALUE IF NOT EXISTS 'deceased_estate_reporting_sa';

-- Generic evidence table for all case types (will_prep_sa, deceased_estate_reporting_sa, and future types)
CREATE TABLE case_evidence (
  evidence_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  case_id uuid NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
  slot_name text NOT NULL,
  document_id uuid,
  added_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(case_id, slot_name)
);

CREATE INDEX idx_case_evidence_case ON case_evidence(case_id);

-- Case metadata for will_prep_sa
CREATE TABLE will_prep_cases (
  case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,
  principal_person_id uuid NOT NULL,
  required_evidence_slots text[] NOT NULL,
  notes text
);

-- Case metadata for deceased_estate_reporting_sa
CREATE TABLE deceased_estate_cases (
  case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,
  deceased_person_id uuid NOT NULL,
  executor_person_id uuid NOT NULL,
  estimated_estate_value_zar numeric,
  required_evidence_slots text[] NOT NULL,
  notes text
);
