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