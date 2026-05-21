use std::time::Duration;

use actix_web::{HttpResponse, post, web};
use common::{CardToken, PaymentServiceCode, PaymentServiceRequest, PaymentServiceResponse, PspPaymentStatus};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppData;

#[derive(Deserialize)]
pub struct IdempotencyKeysRequest {
    pub idempotency_keys: Vec<Uuid>
}

#[post("/psp/payment-attempts")]
pub async fn get_psp_payment_attempts(
    app_data: web::Data<AppData>,
    body: web::Json<IdempotencyKeysRequest>,
) -> HttpResponse {
    match app_data.db.get_psp_payment_attempts_by_idempotency_keys(body.idempotency_keys.clone()).await {
        Ok(attempts) => HttpResponse::Ok().json(attempts),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {}", e))
    }
}

#[post("/psp/process")]
pub async fn process_payment(
    app_data: web::Data<AppData>,
    body: web::Json<PaymentServiceRequest>) ->  HttpResponse {
        let existing_attempt = app_data.db.get_psp_payment_attempt_by_idempotency_key(body.idempotency_key).await;
        println!("Came 1");
        if let Ok(Some(attempt)) = existing_attempt {
            match attempt.status {
                PspPaymentStatus::Success => {
                    let response = PaymentServiceResponse {
                        status: common::PaymentServiceStatus::Succeeded,
                        psp_ref: Some(attempt.id),
                        code: None
                    };
                    return HttpResponse::Ok().json(response);
                },
                PspPaymentStatus::Failure => {
                    let code = if attempt.error_code == "InsufficentFunds" {
                        Some(PaymentServiceCode::InsufficentFunds)
                    } else if attempt.error_code == "CardDeclined" {
                        Some(PaymentServiceCode::CardDeclined)
                    } else {
                        None
                    };
                    let response = PaymentServiceResponse {
                        status: common::PaymentServiceStatus::Failed,
                        psp_ref: None,
                        code: code
                    };
                    return HttpResponse::Ok().json(response);
                },
                PspPaymentStatus::Pending => {
                    let response = PaymentServiceResponse {
                        status: common::PaymentServiceStatus::Pending,
                        psp_ref: None,
                        code: None
                    };
                    return HttpResponse::Ok().json(response);
                }
            }
        }
        println!("Came 2");
        let db_result = app_data.db.create_psp_payment_attempt(
            body.invoice_id,
            common::PspPaymentStatus::Pending,
            body.idempotency_key,
            None,
            None,
            None
        ).await;

        if let Err(e) = db_result {
            println!("{}", format!("Errrr: {}", e));
            return HttpResponse::InternalServerError().body(format!("db error: {}", e));
        }
        println!("Came 3");
        let psp_payment_attempt = db_result.unwrap();
        println!("Came 4");
        match body.card_token {
            CardToken::TokSuccess => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let response = PaymentServiceResponse {
                    status: common::PaymentServiceStatus::Succeeded,
                    psp_ref: Some(psp_payment_attempt.id),
                    code: None
                };
                let _ = app_data.db.update_psp_payment_attempt_status(body.idempotency_key, common::PspPaymentStatus::Success, None, None).await;
                return HttpResponse::Ok().json(response);
            },
            CardToken::TokInsufficientFunds => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let response = PaymentServiceResponse {
                    status: common::PaymentServiceStatus::Failed,
                    psp_ref: None,
                    code: Some(PaymentServiceCode::InsufficentFunds)
                };
                let _ = app_data.db.update_psp_payment_attempt_status(body.idempotency_key, common::PspPaymentStatus::Failure, Some(String::from("InsufficentFunds")), Some(String::from("InsufficentFunds"))).await;
                return HttpResponse::Ok().json(response);
            },
            CardToken::TokCardDeclined => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                 let response = PaymentServiceResponse {
                    status: common::PaymentServiceStatus::Failed,
                    psp_ref: None,
                    code: Some(PaymentServiceCode::CardDeclined)
                };
                let _ = app_data.db.update_psp_payment_attempt_status(body.idempotency_key, common::PspPaymentStatus::Failure, Some(String::from("CardDeclined")), Some(String::from("CardDeclined"))).await;
                return HttpResponse::Ok().json(response);
            },
            CardToken::TokTimeout => {
                tokio::time::sleep(Duration::from_millis(30000)).await;
                let response = PaymentServiceResponse {
                    status: common::PaymentServiceStatus::Succeeded,
                    psp_ref: Some(psp_payment_attempt.id),
                    code: None
                };
                let _ = app_data.db.update_psp_payment_attempt_status(body.idempotency_key, common::PspPaymentStatus::Success, None, None).await;
                return HttpResponse::Ok().json(response);
            },
            CardToken::TokNetworkError => {
                tokio::time::sleep(Duration::from_millis(30000)).await;
                return HttpResponse::InternalServerError().finish();
            }
        }
}