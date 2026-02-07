CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE sensitivity_tier AS ENUM ('green','amber','red');
CREATE TYPE document_type AS ENUM (
  'id','proof_of_address','will','advance_directive','medical_letter','policy','statement','other'
);

CREATE TABLE documents (
  document_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  principal_id uuid NOT NULL,
  document_type document_type NOT NULL,
  title text NOT NULL,
  sensitivity sensitivity_tier NOT NULL,
  tags text[] NOT NULL DEFAULT ARRAY[]::text[],
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE document_versions (
  version_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  document_id uuid NOT NULL REFERENCES documents(document_id) ON DELETE CASCADE,
  blob_ref text NOT NULL,
  sha256 char(64) NOT NULL,
  byte_size bigint NOT NULL,
  mime_type text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(document_id, sha256)
);

CREATE INDEX idx_documents_principal ON documents(principal_id);
CREATE INDEX idx_doc_versions_doc ON document_versions(document_id);
