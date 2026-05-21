CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE business_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    key_hash TEXT NOT NULL,
    label TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);

CREATE INDEX idx_business_keys_business_id ON business_keys(business_id);