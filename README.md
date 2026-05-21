# Invoice Payment Service

A Rust-based invoice payment service with transactional payment processing, idempotency support, and a mock PSP integration.

## Features

- Transactional payment processing to prevent race conditions
- Idempotency support for safe payment retries
- Mock PSP integration with various failure scenarios
- Invoice state machine management
- Concurrent payment handling

## Quick Start with Docker

The easiest way to run the entire stack is using Docker Compose:

```bash
docker-compose up --build
```

This will start:
- PostgreSQL database
- Backend API with integrated mock PSP
- Automatic database migrations

The service will be available at `http://localhost:8080`.

## Local Development

### Prerequisites

- Rust 1.83+
- PostgreSQL 15+
- SQLx CLI

### Setup

1. Copy the environment file:
```bash
cp .env.example .env
```

2. Set up the database:
```bash
createdb invoice_payment
sqlx database create
sqlx migrate run
```

3. Run the backend:
```bash
cargo run --bin backend
```

The service will be available at `http://localhost:8080`.

## API Examples

### 1. Create a Business

```bash
curl -X POST http://localhost:8080/business \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Acme Corp"
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Acme Corp",
  "created_at": 1234567890,
  "updated_at": 1234567890
}
```

### 2. Create a Customer

```bash
curl -X POST http://localhost:8080/customer \
  -H "Content-Type: application/json" \
  -d '{
    "business_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "John Doe",
    "email_id": "john@example.com"
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "business_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "John Doe",
  "email_id": "john@example.com",
  "created_at": 1234567890,
  "updated_at": 1234567890
}
```

### 3. Create an Invoice

```bash
curl -X POST http://localhost:8080/invoice \
  -H "Content-Type: application/json" \
  -d '{
    "business_id": "550e8400-e29b-41d4-a716-446655440000",
    "customer_id": "550e8400-e29b-41d4-a716-446655440001",
    "line_items": [
      {
        "description": "Web Development Services",
        "quantity": 1,
        "unit_price_cents": 50000
      }
    ],
    "due_date": 1735689600
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "business_id": "550e8400-e29b-41d4-a716-446655440000",
  "customer_id": "550e8400-e29b-41d4-a716-446655440001",
  "status": "Open",
  "total_cents": 50000,
  "due_date": 1735689600,
  "created_at": 1234567890,
  "updated_at": 1234567890
}
```

### 4. Attempt Payment (Success Case)

```bash
curl -X POST http://localhost:8080/invoice/550e8400-e29b-41d4-a716-446655440002/pay \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: 550e8400-e29b-41d4-a716-446655440003" \
  -d '{
    "card_token": "tok_success"
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440004",
  "invoice_id": "550e8400-e29b-41d4-a716-446655440002",
  "status": "Success",
  "idempotency_key": "550e8400-e29b-41d4-a716-446655440003",
  "amount_cents": 50000,
  "error_code": null,
  "error_message": null,
  "created_at": 1234567890,
  "updated_at": 1234567890
}
```

### 5. Attempt Payment (Insufficient Funds)

```bash
curl -X POST http://localhost:8080/invoice/550e8400-e29b-41d4-a716-446655440002/pay \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: 550e8400-e29b-41d4-a716-446655440005" \
  -d '{
    "card_token": "tok_insufficient_funds"
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440006",
  "invoice_id": "550e8400-e29b-41d4-a716-446655440002",
  "status": "Failure",
  "idempotency_key": "550e8400-e29b-41d4-a716-446655440005",
  "amount_cents": null,
  "error_code": "InsufficentFunds",
  "error_message": "InsufficentFunds",
  "created_at": 1234567890,
  "updated_at": 1234567890
}
```

### 6. Attempt Payment (Card Declined)

```bash
curl -X POST http://localhost:8080/invoice/550e8400-e29b-41d4-a716-446655440002/pay \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: 550e8400-e29b-41d4-a716-446655440007" \
  -d '{
    "card_token": "tok_card_declined"
  }'
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440008",
  "invoice_id": "550e8400-e29b-41d4-a716-446655440002",
  "status": "Failure",
  "idempotency_key": "550e8400-e29b-41d4-a716-446655440007",
  "amount_cents": null,
  "error_code": "CardDeclined",
  "error_message": "CardDeclined",
  "created_at": 1234567890,
  "updated_at": 1234567890
}
```

### 7. Get Invoice Status

```bash
curl http://localhost:8080/invoice/550e8400-e29b-41d4-a716-446655440002
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "business_id": "550e8400-e29b-41d4-a716-446655440000",
  "customer_id": "550e8400-e29b-41d4-a716-446655440001",
  "status": "Paid",
  "total_cents": 50000,
  "due_date": 1735689600,
  "created_at": 1234567890,
  "updated_at": 1234567890
}
```

## Mock PSP Card Tokens

The mock PSP supports the following card tokens for testing:

- `tok_success` - Payment succeeds
- `tok_insufficient_funds` - Payment fails with insufficient funds
- `tok_card_declined` - Payment fails with card declined
- `tok_timeout` - PSP times out (7 second timeout)
- `tok_network_error` - PSP returns 500 error

## Idempotency

All payment requests require an `Idempotency-Key` header. If you retry a request with the same key, the same payment attempt will be returned without charging the card again.

## Running Tests

```bash
cargo test --package backend
```

Tests cover:
- Concurrent payment requests (ensures at most one succeeds)
- Idempotency (same request returns same response)
- PSP failure scenarios (timeout, network error)

## Architecture

- **Backend**: Actix-web based REST API
- **Database**: PostgreSQL with SQLx
- **Mock PSP**: Integrated into backend at `/psp/process`
- **Transactions**: All payment operations use database transactions for atomicity

## Environment Variables

- `DATABASE_URL` - PostgreSQL connection string (default: `postgresql://postgres:postgres@localhost:5432/invoice_payment`)
