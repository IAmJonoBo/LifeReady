CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE case_type AS ENUM ('emergency_pack','mhca39','death_readiness');
CREATE TYPE case_status AS ENUM ('draft','ready','blocked','exported','closed','revoked');

CREATE TABLE cases (
  case_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  principal_id uuid NOT NULL,
  case_type case_type NOT NULL,
  status case_status NOT NULL DEFAULT 'draft',
  blocked_reasons text[] NOT NULL DEFAULT ARRAY[]::text[],
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE case_artifacts (
  artifact_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  case_id uuid NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
  kind text NOT NULL,
  blob_ref text NOT NULL,
  sha256 char(64) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

-- MHCA 39 case details (guided evidence slots; no "verification" fields)
CREATE TABLE mhca39_cases (
  case_id uuid PRIMARY KEY REFERENCES cases(case_id) ON DELETE CASCADE,
  subject_person_id uuid NOT NULL,
  applicant_person_id uuid NOT NULL,
  relationship_to_subject text,
  required_evidence_slots text[] NOT NULL,
  notes text
);

CREATE TABLE mhca39_evidence (
  evidence_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  case_id uuid NOT NULL REFERENCES cases(case_id) ON DELETE CASCADE,
  slot_name text NOT NULL,
  document_id uuid,
  added_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(case_id, slot_name)
);

CREATE INDEX idx_cases_principal ON cases(principal_id);
CREATE INDEX idx_artifacts_case ON case_artifacts(case_id);
CREATE INDEX idx_mhca39_evidence_case ON mhca39_evidence(case_id);
