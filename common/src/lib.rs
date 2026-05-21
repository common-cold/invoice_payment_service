pub mod types;
use anyhow::Result;
pub use types::*;

pub mod utils;
pub use utils::*;
use uuid::Uuid;


pub struct LocalInvoice {
    pub id: Uuid,
    pub business_id: Uuid,
    pub customer_id: Uuid,
    pub state: Box<dyn InvoiceState>,
    pub total_cents: i64,
    pub due_date: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Invoice> for LocalInvoice {
    fn from(value: Invoice) -> Self {
        let state: Box<dyn InvoiceState> = match value.status {
            Status::Draft => Box::new(Draft{}),
            Status::Open => Box::new(Open{}),
            Status::Paid => Box::new(Paid{}),
            Status::Void => Box::new(Void{}),
            Status::Uncollectible => Box::new(Uncollectible{}),
        };

        LocalInvoice {
            id: value.id,
            business_id: value.business_id,
            customer_id: value.customer_id,
            state: state,
            total_cents: value.total_cents,
            due_date: value.due_date,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

pub trait InvoiceState {
    fn open(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
        Err(format!("invalid transition from {} to {}", self.name(), "Open"))
    }
    fn pay(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
        Err(format!("invalid transition from {} to {}", self.name(), "Paid"))
    }
    fn void(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
        Err(format!("invalid transition from {} to {}", self.name(), "Void"))
    }
    fn uncollectible(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
        Err(format!("invalid transition from {} to {}", self.name(), "Uncollectible"))
    }
    
    fn name(&self) -> &str;
}

struct Draft {}
impl InvoiceState for Draft {
    fn open(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
        Ok(Box::new(Open{}))
    }
    fn void(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
        Ok(Box::new(Void{}))
    }
    fn name(&self) -> &str {
        "Draft"
    }
}

struct Open {}
impl InvoiceState for Open {
    fn open(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
        Ok(Box::new(Open{}))
    }
    fn pay(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
         Ok(Box::new(Paid{}))
    }
    fn void(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
         Ok(Box::new(Void{}))
    }
    fn uncollectible(&self, invoice: &Invoice) -> Result<Box<dyn InvoiceState>, String> {
         Ok(Box::new(Uncollectible{}))
    }
    fn name(&self) -> &str {
        "Open"
    }
}
struct Paid {}
impl InvoiceState for Paid {
    fn name(&self) -> &str {
        "Paid"
    }
}
struct Void {}
impl InvoiceState for Void {
    fn name(&self) -> &str {
        "Void"
    }
}
struct Uncollectible {}
impl InvoiceState for Uncollectible {
    fn name(&self) -> &str {
        "Uncollectible"
    }
}