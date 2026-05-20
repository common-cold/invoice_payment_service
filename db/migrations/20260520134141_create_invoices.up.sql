CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE STATUS AS ENUM('Success', 'Failure', 'Pending');

CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    customer_id UUID NOT NULL REFERENCES business_customers(id) ON DELETE CASCADE,
    status STATUS NOT NULL,
    total_cents BIGINT NOT NULL,
    due_date BIGINT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    updated_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);