

use actix_web::{HttpRequest, HttpResponse, post, web, get};
use common::{CreateInvoiceArgs, LocalInvoice, PayInvoiceRequest, PaymentServiceRequest, PaymentServiceResponse, PaymentServiceStatus, PaymentStatus, PspPaymentAttemptsRequest, PspPaymentStatus, Status};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

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


#[post("/invoice/{id}/pay")]
pub async fn pay_invoice(
    req: HttpRequest,
    app_data: web::Data<AppData>,
    invoice_id: web::Path<Uuid>,
    body: web::Json<PayInvoiceRequest>
) -> HttpResponse {

    let header_option = req
        .headers()
        .get("Idempotency-Key");

    if let None = header_option {
        return HttpResponse::BadRequest().body(String::from("Idempotency-Key header is missing"));
    }

    let idempotency_key_result = header_option.unwrap().to_str();
    if let Err(e) = idempotency_key_result {
        return HttpResponse::BadRequest().body(format!("Error: {}", e));
    }
    let idempotency_key = idempotency_key_result.unwrap();

    let idempotency_key_uuid = Uuid::parse_str(idempotency_key).unwrap();

    let invoice_id_value = invoice_id.into_inner();

    let mut tx = match app_data.db.pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(format!("failed to get transaction object: {}", e)),
    };

    let invoice_result = app_data.db.get_invoice_by_id_with_tx(invoice_id_value, &mut tx).await;
    if let Err(e) = invoice_result {
        return HttpResponse::InternalServerError().body(format!("failed to get invoice: {}", e));
    }

    let invoice_option = invoice_result.unwrap();
    if let None = invoice_option {
        return HttpResponse::NotFound().body("invoice not found");
    }

    let invoice = invoice_option.unwrap();
    match invoice.status {
        Status::Paid => {
            return HttpResponse::BadRequest().body(format!("This invoice is already paid"))
        }
        _ => {}
    }
    let mut local_invoice = LocalInvoice::from(invoice.clone());

    let invoice_total = invoice.total_cents;

    let existing_payment_attempt = app_data.db.get_payment_attempt_by_idempotency_key_with_tx(idempotency_key_uuid, &mut tx).await;
    if let Ok(Some(attempt)) = existing_payment_attempt {
        let _ = tx.commit().await;
        return HttpResponse::Ok().json(attempt);
    }

    let _ = app_data.db.create_payment_attempt_with_tx(
        invoice_id_value,
        PaymentStatus::Pending,
        idempotency_key_uuid,
        None,
        None,
        None,
        &mut tx,
    ).await;

    let payment_service_request = PaymentServiceRequest {
        invoice_id: invoice_id_value,
        card_token: body.card_token.clone(),
        idempotency_key: idempotency_key_uuid,
    };

    let client = reqwest::Client::new();
    let response_result = client
        .post("http://localhost:8080/psp/process")
        .timeout(std::time::Duration::from_secs(7))
        .json(&payment_service_request)
        .send()
        .await;

    let response = match response_result {
        Ok(resp) => resp,
        Err(e) => {
            let state_result = local_invoice.state.open(&invoice);
            if let Err(e) = state_result {
               return HttpResponse::InternalServerError().body(format!("State Machine error: {}", e)); 
            }
            local_invoice.state = state_result.unwrap();
            let _ = app_data.db.update_payment_attempt_with_tx(
                idempotency_key_uuid,
                PaymentStatus::Pending,
                None,
                None,
                None,
                &mut tx
            ).await;
            return HttpResponse::InternalServerError().body(format!("psp timeout or error: {}", e));
        }
    };

    if response.status().is_server_error() {
        let state_result = local_invoice.state.open(&invoice);
        if let Err(e) = state_result {
            return HttpResponse::InternalServerError().body(format!("State Machine error: {}", e)); 
        }
        local_invoice.state = state_result.unwrap();
        let _ = app_data.db.update_payment_attempt_with_tx(
            idempotency_key_uuid,
            PaymentStatus::Pending,
            None,
            None,
            None,
            &mut tx
        ).await;
        return HttpResponse::InternalServerError().body("psp returned server error");
    }

    let payment_service_response = response.json::<PaymentServiceResponse>().await;

    if let Err(e) = payment_service_response {
        let state_result = local_invoice.state.open(&invoice);
        if let Err(e) = state_result {
            return HttpResponse::InternalServerError().body(format!("State Machine error: {}", e)); 
        }
        local_invoice.state = state_result.unwrap();
        let _ = app_data.db.update_payment_attempt_with_tx(
            idempotency_key_uuid,
            PaymentStatus::Pending,
            None,
            None,
            None,
            &mut tx
        ).await;
        return HttpResponse::InternalServerError().body(format!("failed to parse psp response: {}", e));
    }

    let payment_service_response = payment_service_response.unwrap();

    let updated_payment_attempt = match payment_service_response.status {
        PaymentServiceStatus::Succeeded => {
            let state_result = local_invoice.state.pay(&invoice);
            if let Err(e) = state_result {
                return HttpResponse::InternalServerError().body(format!("State Machine error: {}", e)); 
            }
            local_invoice.state = state_result.unwrap();
            app_data.db.update_payment_attempt_with_tx(
                idempotency_key_uuid,
                PaymentStatus::Success,
                Some(invoice_total),
                None,
                None,
                &mut tx,
            ).await
        },
        PaymentServiceStatus::Failed => {
            let state_result = local_invoice.state.open(&invoice);
            if let Err(e) = state_result {
                return HttpResponse::InternalServerError().body(format!("State Machine error: {}", e)); 
            }
            local_invoice.state = state_result.unwrap();
            let code_str = payment_service_response.code.as_ref().map(|c| format!("{:?}", c)).unwrap_or(String::from("unknown"));
            app_data.db.update_payment_attempt_with_tx(
                idempotency_key_uuid,
                PaymentStatus::Failure,
                None,
                Some(code_str.clone()),
                Some(code_str),
                &mut tx,
            ).await
        },
        PaymentServiceStatus::Pending => {
            let state_result = local_invoice.state.open(&invoice);
            if let Err(e) = state_result {
                return HttpResponse::InternalServerError().body(format!("State Machine error: {}", e)); 
            }
            local_invoice.state = state_result.unwrap();
            app_data.db.update_payment_attempt_with_tx(
                idempotency_key_uuid,
                PaymentStatus::Pending,
                None,
                None,
                None,
                &mut tx,
            ).await
        }
    };

    let status = match local_invoice.state.name() {
        "Draft" => Status::Draft,
        "Open" => Status::Open,
        "Paid" => Status::Paid,
        "Void" => Status::Void,
        "Uncollectible" => Status::Uncollectible,
        _ => Status::Open,
    };

    let _ = app_data.db.update_invoice_status_with_tx(invoice_id_value, status, &mut tx).await;

    let _ = tx.commit().await;

    match updated_payment_attempt {
        Ok(attempt) => HttpResponse::Ok().json(attempt),
        Err(e) => HttpResponse::InternalServerError().body(format!("failed to update payment attempt: {}", e))
    }
}

#[get("/invoice/{id}")]
pub async fn get_invoice(
    app_data: web::Data<AppData>,
    invoice_id: web::Path<Uuid>,
) -> HttpResponse {
    let invoice_id_value = invoice_id.into_inner();
    
    match app_data.db.get_invoice_by_id(invoice_id_value).await {
        Ok(Some(invoice)) => HttpResponse::Ok().json(invoice),
        Ok(None) => HttpResponse::NotFound().body("invoice not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("failed to get invoice: {}", e))
    }
}

#[get("/cron/process-pending-payments")]
pub async fn process_pending_payments(
    app_data: web::Data<AppData>,
) -> HttpResponse {
    let pending_attempts_result = app_data.db.get_payment_attempts_by_status(PaymentStatus::Pending).await;

    if let Err(e) = pending_attempts_result {
        return HttpResponse::InternalServerError().body(format!("failed to fetch pending payment attempts: {}", e));
    }

    let pending_attempts = pending_attempts_result.unwrap();

    if pending_attempts.is_empty() {
        return HttpResponse::Ok().json("no pending payment attempts to process");
    }

    let client = reqwest::Client::new();
    let batch_size = 50;

    for chunk in pending_attempts.chunks(batch_size) {
        let idempotency_keys: Vec<Uuid> = chunk.iter().map(|attempt| attempt.idempotency_key).collect();

        let psp_request = PspPaymentAttemptsRequest {
            idempotency_keys: idempotency_keys.clone(),
        };

        let response = client
            .post("http://localhost:8080/psp/payment-attempts")
            .json(&psp_request)
            .send()
            .await;

        if let Err(e) = response {
            return HttpResponse::InternalServerError().body(format!("failed to call psp payment-attempts: {}", e));
        }

        let response = response.unwrap();

        if !response.status().is_success() {
            return HttpResponse::InternalServerError().body(format!("psp payment-attempts returned error: {}", response.status()));
        }

        let psp_attempts_result = response.json::<Vec<common::PspPaymentAttempt>>().await;

        if let Err(e) = psp_attempts_result {
            return HttpResponse::InternalServerError().body(format!("failed to parse psp response: {}", e));
        }

        let psp_attempts = psp_attempts_result.unwrap();

        for psp_attempt in psp_attempts {
            let payment_status = match psp_attempt.status {
                PspPaymentStatus::Success => PaymentStatus::Success,
                PspPaymentStatus::Failure => PaymentStatus::Failure,
                PspPaymentStatus::Pending => PaymentStatus::Pending,
            };

            let _ = app_data.db.update_payment_attempt(
                psp_attempt.idempotency_key,
                payment_status,
                Some(psp_attempt.amount_cents),
                Some(psp_attempt.error_code),
                Some(psp_attempt.error_message),
            ).await;
        }
    }

    HttpResponse::Ok().json(format!("processed {} payment attempts", pending_attempts.len()))
}
