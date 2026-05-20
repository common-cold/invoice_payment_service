use actix_web::{HttpResponse, get, post, web};
use common::CreateBusinessCustomerArgs;
use uuid::Uuid;

use crate::AppData;

#[post("/customer")]
pub async fn create_customer(
    app_data: web::Data<AppData>,
    body: web::Json<CreateBusinessCustomerArgs>,
) -> HttpResponse {
    match app_data.db.create_business_customer(body.into_inner()).await {
        Ok(customer) => HttpResponse::Ok().json(customer),
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("failed to create customer: {err}"))
        }
    }
}

#[get("/customer/{id}")]
pub async fn get_customer_by_id(
    app_data: web::Data<AppData>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();
    match app_data.db.get_business_customer_by_id(id).await {
        Ok(customer) => {
            let is_found = customer.is_some();
            if is_found {
                HttpResponse::Ok().json(customer.unwrap())
            } else {
                HttpResponse::NotFound().body("customer not found")
            }
        },
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("failed to get customer: {err}"))
        }
    }
}
