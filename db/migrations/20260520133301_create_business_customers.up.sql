CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE business_customers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    email_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    updated_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    
    CONSTRAINT unique_business_email UNIQUE (business_id, email_id)
);