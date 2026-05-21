CREATE TABLE invoice_line_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    quantity INT NOT NULL,
    unit_price_cents BIGINT NOT NULL,
    amount_cents BIGINT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    updated_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);

CREATE INDEX idx_invoice_line_items_invoice_id ON invoice_line_items(invoice_id);