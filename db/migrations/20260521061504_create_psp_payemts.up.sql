CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TYPE PSP_PAYMENT_STATUS AS ENUM('Success', 'Failure', 'Pending');


CREATE TABLE psp_payment_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    invoice_id UUID NOT NULL,
    status PSP_PAYMENT_STATUS NOT NULL,
    idempotency_key UUID NOT NULL,
    amount_cents BIGINT NOT NULL,
    error_code TEXT NOT NULL,
    error_message TEXT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    updated_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);