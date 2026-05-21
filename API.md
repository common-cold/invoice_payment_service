# API Documentation

Base URL: `http://localhost:8080`

## Business Endpoints

### Create Business

**POST** `/business`

Creates a new business.

**Request Body:**
```json
{
  "name": "string"
}
```

**Response (200 OK):**
```json
{
  "id": "uuid",
  "name": "string",
  "created_at": 1716300000,
  "updated_at": 1716300000
}
```

**Response (500 Internal Server Error):**
```
failed to create business: {error}
```

---

### Create Business Auth Key

**POST** `/business/auth-key`

Creates a new API key for a business. The plaintext key is returned only once.

**Request Body:**
```json
{
  "business_id": "uuid",
  "label": "string"
}
```

**Response (200 OK):**
```json
{
  "id": "uuid",
  "auth_key": "15-character-alphanumeric-string",
  "label": "string"
}
```

**Response (500 Internal Server Error):**
```
failed to generate auth key: {error}
```

---

## Customer Endpoints

### Create Customer

**POST** `/customer`

Creates a new customer for a business.

**Request Body:**
```json
{
  "business_id": "uuid",
  "email_id": "string",
  "name": "string"
}
```

**Response (200 OK):**
```json
{
  "id": "uuid",
  "business_id": "uuid",
  "email_id": "string",
  "name": "string",
  "created_at": 1716300000,
  "updated_at": 1716300000
}
```

**Response (500 Internal Server Error):**
```
failed to create customer: {error}
```

---

### Get Customer by ID

**GET** `/customer/{id}`

Retrieves a customer by ID.

**Path Parameters:**
- `id` (UUID) - Customer ID

**Response (200 OK):**
```json
{
  "id": "uuid",
  "business_id": "uuid",
  "email_id": "string",
  "name": "string",
  "created_at": 1716300000,
  "updated_at": 1716300000
}
```

**Response (404 Not Found):**
```
customer not found
```

**Response (500 Internal Server Error):**
```
failed to get customer: {error}
```

---

## Invoice Endpoints

### Create Invoice

**POST** `/invoice`

Creates a new invoice with line items. The invoice is created in `Open` status. Line item amounts are calculated automatically as `quantity * unit_price_cents`.

**Request Body:**
```json
{
  "business_id": "uuid",
  "customer_id": "uuid",
  "due_date": 1716300000,
  "line_items": [
    {
      "description": "string",
      "quantity": 1,
      "unit_price_cents": 10000,
      "amount_cents": null
    }
  ],
  "total_cents": null
}
```

**Response (200 OK):**
```json
{
  "id": "uuid",
  "business_id": "uuid",
  "customer_id": "uuid",
  "status": "Open",
  "total_cents": 10000,
  "due_date": 1716300000,
  "created_at": 1716300000,
  "updated_at": 1716300000
}
```

**Response (400 Bad Request):**
```
At least one line item is required
```

**Response (500 Internal Server Error):**
```
failed to create invoice: {error}
```

---

### Get Invoice by ID

**GET** `/invoice/{id}`

Retrieves an invoice by ID.

**Path Parameters:**
- `id` (UUID) - Invoice ID

**Response (200 OK):**
```json
{
  "id": "uuid",
  "business_id": "uuid",
  "customer_id": "uuid",
  "status": "Open",
  "total_cents": 10000,
  "due_date": 1716300000,
  "created_at": 1716300000,
  "updated_at": 1716300000
}
```

**Response (404 Not Found):**
```
invoice not found
```

**Response (500 Internal Server Error):**
```
failed to get invoice: {error}
```

---

### Pay Invoice

**POST** `/invoice/{id}/pay`

Processes a payment for an invoice.

**Headers:**
- `Idempotency-Key` (required) - UUID string for idempotency

**Path Parameters:**
- `id` (UUID) - Invoice ID

**Request Body:**
```json
{
  "card_token": "tok_success"
}
```

**Card Token Options:**
- `tok_success` - Payment succeeds
- `tok_insufficient_funds` - Payment fails due to insufficient funds
- `tok_card_declined` - Payment fails due to card decline
- `tok_timeout` - Simulates PSP timeout (30s)
- `tok_network_error` - Simulates PSP network error

**Response (200 OK) - Success:**
```json
{
  "id": "uuid",
  "invoice_id": "uuid",
  "status": "Success",
  "idempotency_key": "uuid",
  "amount_cents": 10000,
  "error_code": "",
  "error_message": "",
  "created_at": 1716300000,
  "updated_at": 1716300000
}
```

**Response (200 OK) - Idempotent (existing attempt):**
```json
{
  "id": "uuid",
  "invoice_id": "uuid",
  "status": "Success",
  "idempotency_key": "uuid",
  "amount_cents": 10000,
  "error_code": "",
  "error_message": "",
  "created_at": 1716300000,
  "updated_at": 1716300000
}
```

**Response (400 Bad Request):**
```
Idempotency-Key header is missing
```
```
This invoice is already paid
```

**Response (404 Not Found):**
```
invoice not found
```

**Response (500 Internal Server Error):**
```
psp timeout or error: {error}
```
```
psp returned server error
```
```
failed to parse psp response: {error}
```
```
State Machine error: {error}
```
```
failed to update payment attempt: {error}
```

---

### Process Pending Payments

**GET** `/cron/process-pending-payments`

Cron job endpoint to reconcile pending payment attempts with the PSP. Processes in batches of 50.

**Response (200 OK) - No pending:**
```json
"no pending payment attempts to process"
```

**Response (200 OK) - Processed:**
```json
"processed 5 payment attempts"
```

**Response (500 Internal Server Error):**
```
failed to fetch pending payment attempts: {error}
```
```
failed to call psp payment-attempts: {error}
```
```
psp payment-attempts returned error: {status}
```
```
failed to parse psp response: {error}
```

---

## Mock PSP Endpoints (Internal)

These endpoints simulate a Payment Service Provider for testing purposes.

### Process Payment (Mock PSP)

**POST** `/psp/process`

Simulates payment processing. Uses idempotency keys to prevent duplicate charges.

**Request Body:**
```json
{
  "invoice_id": "uuid",
  "card_token": "tok_success",
  "idempotency_key": "uuid"
}
```

**Response (200 OK) - Success:**
```json
{
  "status": "succeeded",
  "psp_ref": "uuid",
  "code": null
}
```

**Response (200 OK) - Failure:**
```json
{
  "status": "failed",
  "psp_ref": null,
  "code": "insufficient_funds"
}
```

**Response (200 OK) - Pending:**
```json
{
  "status": "pending",
  "psp_ref": null,
  "code": null
}
```

**Response (500 Internal Server Error):**
```
db error: {error}
```

---

### Get PSP Payment Attempts

**POST** `/psp/payment-attempts`

Retrieves PSP payment attempts by idempotency keys.

**Request Body:**
```json
{
  "idempotency_keys": ["uuid", "uuid"]
}
```

**Response (200 OK):**
```json
[
  {
    "id": "uuid",
    "invoice_id": "uuid",
    "status": "Success",
    "idempotency_key": "uuid",
    "amount_cents": 10000,
    "error_code": "",
    "error_message": "",
    "created_at": 1716300000,
    "updated_at": 1716300000
  }
]
```

**Response (500 Internal Server Error):**
```
db error: {error}
```

---

## Data Types

### Invoice Status
- `Draft` - Invoice created, not yet sent
- `Open` - Finalized and sent to customer
- `Paid` - Payment succeeded
- `Void` - Cancelled
- `Uncollectible` - Debt written off

### Payment Status
- `Success` - Payment succeeded
- `Failure` - Payment failed
- `Pending` - Payment in progress

## Notes

- All timestamps are Unix timestamps in seconds
- All monetary amounts are in cents (integer)
- UUIDs are standard UUID v4 format
- Row-level locking is used on invoice reads during payment processing to prevent duplicate charges
- Payment attempts use idempotency keys to prevent duplicate processing
- The mock PSP endpoints are for testing only and should not be exposed in production
