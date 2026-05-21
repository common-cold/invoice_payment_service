use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateBusinessArgs {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBusinessKeyArgs {
    pub business_id: Uuid,
    pub label: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBusinessCustomerArgs {
    pub business_id: Uuid,
    pub email_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceLineItem {
    pub description: String,
    pub quantity: i16,
    pub unit_price_cents: i64,
    pub amount_cents: Option<i64>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceArgs {
    pub business_id: Uuid,
    pub customer_id: Uuid,
    pub due_date: i64,
    pub line_items: Vec<CreateInvoiceLineItem>,
    pub total_cents: Option<i64>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayInvoiceRequest {
    pub card_token: CardToken
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentServiceRequest {
    pub invoice_id: Uuid,
    pub card_token: CardToken,
    pub idempotency_key: Uuid
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentServiceResponse {
    pub status: PaymentServiceStatus,
    pub psp_ref: Option<Uuid>,
    pub code: Option<PaymentServiceCode>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentServiceStatus {
    #[serde(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "pending")]
    Pending
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentServiceCode {
    #[serde(rename = "insufficient_funds")]
    InsufficentFunds,
    #[serde(rename = "card_declined")]
    CardDeclined
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardToken {
    TokSuccess,
    TokInsufficientFunds,
    TokCardDeclined,
    TokTimeout,
    TokNetworkError
}

#[derive(Serialize)]
pub struct PspPaymentAttemptsRequest {
    pub idempotency_keys: Vec<Uuid>
}
