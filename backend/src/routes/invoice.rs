use std::fmt::format;

use actix_web::{HttpResponse, body, post, web};
use common::CreateInvoiceArgs;

use crate::AppData;

#[post("/invoice")]
pub async fn create_invoice(
    app_data: web::Data<AppData>,
    mut body: web::Json<CreateInvoiceArgs>,
) -> HttpResponse {

    let mut invoice_total = 0;

    if body.line_items.len() == 0 {
        return HttpResponse::BadRequest().body(String::from("At least one line item is required"));
    }
    
    for item in body.line_items.iter_mut() {
        let item_total = item.quantity as i64 * item.unit_price_cents; 
        item.amount_cents = Some(item_total);
        invoice_total += item_total;
    }

    body.total_cents = Some(invoice_total);
    

    match app_data.db.create_invoice(body.into_inner()).await {
        Ok(invoice) => HttpResponse::Ok().json(invoice),
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("failed to create invoice: {err}"))
        }
    }
}
