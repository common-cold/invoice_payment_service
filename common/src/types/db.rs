use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Business {
    pub id: Uuid,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BusinessKey {
    pub id: Uuid,
    pub business_id: Uuid,
    pub key_hash: String,
    pub label: String,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BusinessCustomer {
    pub id: Uuid,
    pub business_id: Uuid,
    pub email_id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BusinessWebhookEndpoint {
    pub id: Uuid,
    pub business_id: Uuid,
    pub url: String,
    pub api_key: String,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "status")]
#[sqlx(rename_all = "PascalCase")]
pub enum Status {
    Success,
    Failure,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub business_id: Uuid,
    pub customer_id: Uuid,
    pub status: Status,
    pub total_cents: i64,
    pub due_date: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "status")]
#[sqlx(rename_all = "PascalCase")]
pub enum PaymentStatus {
    Success,
    Failure,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentAttempt {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub status: PaymentStatus,
    pub idempotency_key: String,
    pub amount_cents: i64,
    pub processor_id: String,
    pub error_code: String,
    pub error_message: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvoiceLineItem {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub description: String,
    pub quantity: i16,
    pub unit_price_cents: i64,
    pub amount_cents: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
