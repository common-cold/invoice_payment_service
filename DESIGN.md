# Invoice Payment Service Design

## 1. Data Model

### businesses
```json
{
  "businesses": [
    "id: UUID",
    "name: TEXT",
    "created_at: BIGINT",
    "updated_at: BIGINT"
  ],
  "indexes": ["PK on id"],
  "relations": [
    "Referenced by business_keys (CASCADE)",
    "Referenced by business_customers (CASCADE)",
    "Referenced by invoices (CASCADE)",
    "Referenced by business_webhook_endpoints (CASCADE)"
  ]
}
```

### business_keys
```json
{
  "business_keys": [
    "id: UUID",
    "business_id: UUID",
    "key_hash: TEXT",
    "label: TEXT",
    "is_active: BOOLEAN",
    "created_at: BIGINT"
  ],
  "indexes": [
    "PK on id",
    "IDX on business_id"
  ],
  "relations": [
    "References businesses(id) ON DELETE CASCADE"
  ]
}
```

### business_customers
```json
{
  "business_customers": [
    "id: UUID",
    "business_id: UUID",
    "email_id: TEXT",
    "name: TEXT",
    "created_at: BIGINT",
    "updated_at: BIGINT"
  ],
  "indexes": [
    "PK on id",
    "IDX on business_id",
    "IDX on email_id"
  ],
  "constraints": [
    "UNIQUE (business_id, email_id)"
  ],
  "relations": [
    "References businesses(id) ON DELETE CASCADE",
    "Referenced by invoices (CASCADE)"
  ]
}
```

### business_webhook_endpoints
```json
{
  "business_webhook_endpoints": [
    "id: UUID",
    "business_id: UUID",
    "url: TEXT",
    "api_key: TEXT",
    "is_active: BOOLEAN",
    "created_at: BIGINT"
  ],
  "indexes": ["PK on id"],
  "relations": [
    "References businesses(id) ON DELETE CASCADE"
  ]
}
```

### invoices
```json
{
  "invoices": [
    "id: UUID",
    "business_id: UUID",
    "customer_id: UUID",
    "status: ENUM('Draft', 'Open', 'Paid', 'Void', 'Uncollectible')",
    "total_cents: BIGINT",
    "due_date: BIGINT",
    "created_at: BIGINT",
    "updated_at: BIGINT"
  ],
  "indexes": [
    "PK on id",
    "IDX on business_id",
    "IDX on status",
    "IDX on customer_id"
  ],
  "relations": [
    "References businesses(id) ON DELETE CASCADE",
    "References business_customers(id) ON DELETE CASCADE",
    "Referenced by payment_attempts (CASCADE)",
    "Referenced by invoice_line_items (CASCADE)",
    "Referenced by psp_payment_attempts"
  ]
}
```

### payment_attempts
```json
{
  "payment_attempts": [
    "id: UUID",
    "invoice_id: UUID",
    "status: ENUM('Success', 'Failure', 'Pending')",
    "idempotency_key: UUID",
    "amount_cents: BIGINT",
    "error_code: TEXT",
    "error_message: TEXT",
    "created_at: BIGINT",
    "updated_at: BIGINT"
  ],
  "indexes": [
    "PK on id",
    "IDX on invoice_id",
    "IDX on idempotency_key"
  ],
  "relations": [
    "References invoices(id) ON DELETE CASCADE"
  ]
}
```

### invoice_line_items
```json
{
  "invoice_line_items": [
    "id: UUID",
    "invoice_id: UUID",
    "description: TEXT",
    "quantity: INT",
    "unit_price_cents: BIGINT",
    "amount_cents: BIGINT",
    "created_at: BIGINT",
    "updated_at: BIGINT"
  ],
  "indexes": [
    "PK on id",
    "IDX on invoice_id"
  ],
  "relations": [
    "References invoices(id) ON DELETE CASCADE"
  ]
}
```

### psp_payment_attempts
```json
{
  "psp_payment_attempts": [
    "id: UUID",
    "invoice_id: UUID",
    "status: ENUM('Success', 'Failure', 'Pending')",
    "idempotency_key: UUID",
    "amount_cents: BIGINT",
    "error_code: TEXT",
    "error_message: TEXT",
    "created_at: BIGINT",
    "updated_at: BIGINT"
  ],
  "indexes": ["PK on id"],
  "relations": [
    "References invoices(id)"
  ]
}
```

## 2. Invoice State Machine

### States

<img width="1106" height="657" alt="image" src="https://github.com/user-attachments/assets/1b43bb39-b1a1-4005-a7b1-612eef461c80" />


- **draft** — invoice created, not yet sent. Line items still editable.
- **open** — finalized and sent to customer. Amount locked. Payment attempts can now be made.
- **paid** — a payment attempt succeeded. Money collected. Terminal.
- **void** — cancelled. No payment expected or possible. Terminal.
- **uncollectible** — business gave up collecting. Debt written off. Terminal.

### Transitions

- **draft → open** — business finalizes and sends invoice to customer. Line items are locked from this point.
- **draft → void** — business deletes a wrongly created draft before sending it.
- **open → paid** — PSP returns succeeded on a payment attempt, or background reconciliation confirms success for a previously pending attempt.
- **open → void** — business cancels the invoice because it was wrong/duplicate, or the underlying order was cancelled.
- **open → uncollectible** — business manually marks the invoice as bad debt after failed collection attempts.

### What is not a transition

- **open → draft** — invalid. Invoice has already been seen by the customer.
- **paid → anything** — invalid. Money was collected, no reversal in scope (no refund modelled).
- **void → anything** — invalid. Cancellation is final.
- **uncollectible → anything** — invalid. Write-off is final.

## 3. Payment Correctness & Failure Modes

### (a) Two clients call POST /invoices/{id}/pay for the same invoice at the same instant

**Same idempotency key:** The second request finds the existing payment attempt and returns it immediately without processing.

**Different idempotency keys:**
1. A row-level lock (`FOR UPDATE`) is acquired on the invoice, ensuring only one request proceeds at a time.
2. Inside the lock, we check if the invoice is in `Paid` status (reject if true).
3. We then check if any payment attempt for this invoice is in `Pending` status (reject if true).
4. If no pending attempts exist, a new payment attempt is created with `Pending` status and the transaction is committed, releasing the lock.
5. The PSP is then called, and the payment attempt status is updated to `Success`, `Failure`, or remains `Pending` based on the response.

The `Pending` check prevents concurrent requests from creating duplicate payment attempts. To avoid indefinite rejection due to stuck `Pending` attempts, a timeout mechanism can be implemented: if the reconciliation cron finds a `Pending` status beyond a configured timeout, it marks it as `Failure`, allowing the customer to retry.

### (b) The mock PSP times out (tok_timeout, 30 s)

The endpoint returns HTTP 500 with error message "psp timeout or error" after the 7-second client timeout. The payment attempt is left in `Pending` status and the transaction is rolled back, so invoice status remains unchanged. The caller can retry with the same idempotency key, and a background cron job (`/cron/process-pending-payments`) periodically reconciles pending payment attempts with the PSP.

### (c) The PSP returns success but your service crashes before persisting that

IN POST /invoices/{id}/pay the payment_attempt is first stored in db before any calls to PSP with state as Pending. If the invoice service ccomes back after crashing the background cron job (`/cron/process-pending-payments`) periodically reconciles pending payment attempts with the PSP, which will update the true sttaus of the payemnt_attempt.

### (d) An idempotency key is reused with a different request body

The service returns the existing payment attempt without processing the new request body, ignoring any different card token or amount in the new request.

### (e) An invoice in paid state receives another POST /pay

The service does nto allow creating a payment whose invoice is already in PAID state. This check is done after fetching the invoice from db.

## 4. Webhook Design

Due to time constraints I have not done this part.

## 5. API Key Model

### Generation

API keys are 15-character alphanumeric strings generated server-side using cryptographically secure random generation and returned to the client only once during creation.

### Storage

Keys are hashed with bcrypt before storage; only the hash is stored in `business_keys.key_hash`. The plaintext key is never persisted. No prefix is used.

### Transmission

The plaintext API key is returned to the client only once in the HTTP response during creation (`POST /business/auth-key`) and never transmitted again.

### Rotation

Not implemented. Keys must be manually rotated by creating a new key and deactivating the old one.

### Revocation

Keys can be revoked by setting `is_active = false` in the database, but no revocation endpoint is implemented.

### Blast Radius if Leaked

Database leaks expose only bcrypt hashes (difficult but not impossible to crack). Key leaks give attackers full access to the business's invoices, customers, and payment capabilities until deactivated via database update. No audit logging exists.

## 6. What You Cut and Why

1. **Webhook implementation** — Due to time constraints. The `business_webhook_endpoints` table exists to store webhook configuration, but the webhook delivery service, retry mechanism, and delivery tracking are not implemented.

2. **Usage of business auth keys in business related APIs** — Due to time constraints. API keys are generated and stored, but authentication middleware to verify keys on business-related API endpoints is not implemented.

## 7. Production Readiness Gap

If this shipped tomorrow, the top 3 critical gaps are:

1. **Refunds** — No refund mechanism exists despite payments being processed.

2. **Business auth keys** — Business auth keys are generated and stored but authentication middleware is not implemented. All API endpoints are currently unprotected.

3. **No mechanism for flagging invoice as Void/Uncollectible** — There is no API endpoint to transition invoices to Void or Uncollectible states, despite these being valid state transitions in the design.
