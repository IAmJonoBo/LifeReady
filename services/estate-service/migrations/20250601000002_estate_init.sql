CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE principals (
  principal_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  email text UNIQUE NOT NULL,
  display_name text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE people (
  person_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  principal_id uuid NOT NULL REFERENCES principals(principal_id) ON DELETE CASCADE,
  full_name text NOT NULL,
  email text,
  phone_e164 text,
  relationship text,
  created_at timestamptz NOT NULL DEFAULT now()
);

DO $$ BEGIN
  CREATE TYPE asset_category AS ENUM (
    'property','bank_account','insurance_policy','vehicle','business_interest','digital_account','other'
  );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE sensitivity_tier AS ENUM ('green','amber','red');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE assets (
  asset_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  principal_id uuid NOT NULL REFERENCES principals(principal_id) ON DELETE CASCADE,
  category asset_category NOT NULL,
  label text NOT NULL,
  notes text,
  sensitivity sensitivity_tier NOT NULL DEFAULT 'amber',
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE instructions (
  instruction_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  principal_id uuid NOT NULL REFERENCES principals(principal_id) ON DELETE CASCADE,
  title text NOT NULL,
  body text NOT NULL,
  sensitivity sensitivity_tier NOT NULL DEFAULT 'amber',
  created_at timestamptz NOT NULL DEFAULT now()
);

DO $$ BEGIN
  CREATE TYPE role_name AS ENUM ('principal','proxy','executor_nominee','emergency_contact');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE grant_status AS ENUM ('invited','accepted','suspended','revoked');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE role_grants (
  grant_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  principal_id uuid NOT NULL REFERENCES principals(principal_id) ON DELETE CASCADE,
  person_id uuid NOT NULL REFERENCES people(person_id) ON DELETE CASCADE,
  role role_name NOT NULL,
  status grant_status NOT NULL DEFAULT 'invited',
  access_level text NOT NULL CHECK (access_level IN ('read_only_packs','read_only_all','limited_write')),
  allowed_tiers sensitivity_tier[] NOT NULL DEFAULT ARRAY['green','amber']::sensitivity_tier[],
  expires_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_people_principal ON people(principal_id);
CREATE INDEX idx_assets_principal ON assets(principal_id);
CREATE INDEX idx_instructions_principal ON instructions(principal_id);
CREATE INDEX idx_role_grants_principal ON role_grants(principal_id);
