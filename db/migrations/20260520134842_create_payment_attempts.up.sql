CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE PAYMENT_STATUS AS ENUM('Success', 'Failure', 'Pending');

CREATE TABLE payment_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    status PAYMENT_STATUS NOT NULL,
    idempotency_key UUID NOT NULL,
    amount_cents BIGINT NOT NULL,
    error_code TEXT NOT NULL,
    error_message TEXT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    updated_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);

CREATE INDEX idx_payment_attempts_invoice_id ON payment_attempts(invoice_id);
CREATE INDEX idx_payment_attempts_idempotency_key ON payment_attempts(idempotency_key);