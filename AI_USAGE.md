# AI Tool Usage

1. Windsurf 

## Design Documentation

Created `design.md` with the following sections based on analysis of the actual codebase:

1. **Data Model** - JSON-like structure for all 8 database tables with columns, indexes, constraints, and relations
2. **Invoice State Machine** - States, valid transitions, and invalid transitions
3. **Payment Correctness & Failure Modes** - Analysis of concurrent payments, timeouts, crashes, idempotency, and state enforcement
4. **Webhook Design** - Placeholder (not implemented)
5. **API Key Model** - Generation, storage, transmission, rotation, revocation, and blast radius
6. **What You Cut and Why** - Webhook implementation and business auth key usage
7. **Production Readiness Gap** - Refunds, business auth keys, and Void/Uncollectible flagging

## Code Changes

Added row-level locking (`FOR UPDATE`) to `get_invoice_by_id_with_tx` in `db/src/lib.rs` to prevent duplicate charges when concurrent payment requests use different idempotency keys.
