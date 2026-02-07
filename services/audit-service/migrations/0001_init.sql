CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE sensitivity_tier AS ENUM ('green','amber','red');

-- Append-only event log (application must ONLY INSERT)
CREATE TABLE audit_events (
  event_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  created_at timestamptz NOT NULL DEFAULT now(),
  actor_principal_id uuid NOT NULL,
  action text NOT NULL,
  tier sensitivity_tier NOT NULL,
  case_id uuid,
  payload jsonb NOT NULL,
  prev_hash char(64) NOT NULL,
  event_hash char(64) NOT NULL
);

CREATE INDEX idx_audit_case_time ON audit_events(case_id, created_at DESC);
CREATE INDEX idx_audit_actor_time ON audit_events(actor_principal_id, created_at DESC);

-- Optional: prevent UPDATE/DELETE at DB level
CREATE OR REPLACE FUNCTION prevent_audit_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  RAISE EXCEPTION 'audit_events is append-only';
END $$;

CREATE TRIGGER no_audit_update
BEFORE UPDATE ON audit_events
FOR EACH ROW EXECUTE FUNCTION prevent_audit_mutation();

CREATE TRIGGER no_audit_delete
BEFORE DELETE ON audit_events
FOR EACH ROW EXECUTE FUNCTION prevent_audit_mutation();
