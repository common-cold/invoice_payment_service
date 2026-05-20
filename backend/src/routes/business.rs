use actix_web::{HttpResponse, post, web};
use common::{CreateBusinessArgs, CreateBusinessKeyArgs, hash_string};
use serde_json::json;

use crate::{AppData, service::generate_key};

#[post("/business")]
pub async fn create_business(
    app_data: web::Data<AppData>,
    body: web::Json<CreateBusinessArgs>,
) -> HttpResponse {
    match app_data.db.create_business(body.into_inner()).await {
        Ok(business) => HttpResponse::Ok().json(business),
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("failed to create business: {err}"))
        }
    }
}

#[post("/business/auth-key")]
pub async fn create_business_auth_key(
    app_data: web::Data<AppData>,
    body: web::Json<CreateBusinessKeyArgs>,
) -> HttpResponse {
    let auth_key_result = generate_key();
    if let Err(e) = auth_key_result {
        return HttpResponse::InternalServerError().body(format!("failed to generate auth key: {e}"));
    }

    let auth_key = auth_key_result.unwrap();
    
    let hashed_key_result = hash_string(&auth_key);
    if let Err(e) =  hashed_key_result {
        return HttpResponse::InternalServerError().body(format!("failed to hash auth key: {e}"));
    }

    let hashed_key = hashed_key_result.unwrap();


    match app_data.db.create_business_auth_key(body.business_id, hashed_key, body.label.clone()).await {
        Ok(business_key) => HttpResponse::Ok().json(json!({
            "id": business_key.id,
            "auth_key": auth_key,
            "label": business_key.label
        })),
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("failed to create auth key: {err}"))
        }
    }
}
