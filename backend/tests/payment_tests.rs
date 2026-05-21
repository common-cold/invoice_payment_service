use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono;

const BASE_URL: &str = "http://localhost:8080";

async fn create_business(client: &Client) -> Uuid {
    let response = client
        .post(format!("{}/business", BASE_URL))
        .json(&json!({
            "name": "Test Business"
        }))
        .send()
        .await
        .expect("Failed to send business request");

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.expect("Failed to read error body");
        panic!("Failed to create business: {} - {}", status, body);
    }

    let business: serde_json::Value = response.json().await.expect("Failed to parse business");
    let business_id = business["id"].as_str().expect("Missing business id");
    Uuid::parse_str(business_id).expect("Invalid business UUID")
}

async fn create_customer(client: &Client, business_id: Uuid) -> Uuid {
    let response = client
        .post(format!("{}/customer", BASE_URL))
        .json(&json!({
            "name": "Test Customer",
            "email_id": "test@example.com",
            "business_id": business_id
        }))
        .send()
        .await
        .expect("Failed to send customer request");

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.expect("Failed to read error body");
        panic!("Failed to create customer: {} - {}", status, body);
    }

    let customer: serde_json::Value = response.json().await.expect("Failed to parse customer");
    let customer_id = customer["id"].as_str().expect("Missing customer id");
    Uuid::parse_str(customer_id).expect("Invalid customer UUID")
}

async fn create_invoice(client: &Client, business_id: Uuid, customer_id: Uuid) -> Uuid {
    let invoice_response = client
        .post(format!("{}/invoice", BASE_URL))
        .json(&json!({
            "business_id": business_id,
            "customer_id": customer_id,
            "line_items": [
                {
                    "description": "Test item",
                    "quantity": 1,
                    "unit_price_cents": 10000
                }
            ],
            "due_date": chrono::Utc::now().timestamp()
        }))
        .send()
        .await
        .expect("Failed to send invoice request");

    if !invoice_response.status().is_success() {
        let status = invoice_response.status();
        let body = invoice_response.text().await.expect("Failed to read error body");
        panic!("Failed to create invoice: {} - {}", status, body);
    }

    let invoice: serde_json::Value = invoice_response
        .json()
        .await
        .expect("Failed to parse invoice");
    let invoice_id = invoice["id"].as_str().expect("Missing invoice id");
    Uuid::parse_str(invoice_id).expect("Invalid invoice UUID")
}

#[tokio::test]
async fn test_concurrent_payment_requests() {
    // First, create business, customer, and invoice
    let client = Client::new();
    let business_id = create_business(&client).await;
    let customer_id = create_customer(&client, business_id).await;
    let invoice_uuid = create_invoice(&client, business_id, customer_id).await;

    // Fire N concurrent payment requests
    let num_requests = 10;
    let success_count = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for i in 0..num_requests {
        let client = client.clone();
        let success_count = Arc::clone(&success_count);
        let idempotency_key = Uuid::new_v4();

        let handle = tokio::spawn(async move {
            let response = client
                .post(format!("{}/invoice/{}/pay", BASE_URL, invoice_uuid))
                .header("Idempotency-Key", idempotency_key.to_string())
                .json(&json!({
                    "card_token": "TokSuccess"
                }))
                .send()
                .await
                .expect("Failed to send payment request");

            if response.status().is_success() {
                let mut count = success_count.lock().await;
                *count += 1;
            }
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await.expect("Task failed");
    }

    let final_success_count = *success_count.lock().await;
    
    // Assert at most one succeeds
    assert!(
        final_success_count <= 1,
        "Expected at most one successful payment, got {}",
        final_success_count
    );

    // Verify final invoice state
    let invoice_response = client
        .get(format!("{}/invoice/{}", BASE_URL, invoice_uuid))
        .send()
        .await
        .expect("Failed to get invoice");

    if !invoice_response.status().is_success() {
        let status = invoice_response.status();
        let body = invoice_response.text().await.expect("Failed to read error body");
        panic!("Failed to get invoice: {} - {}", status, body);
    }
    let final_invoice: serde_json::Value = invoice_response
        .json()
        .await
        .expect("Failed to parse invoice");

    // If payment succeeded, invoice should be Paid
    if final_success_count == 1 {
        assert_eq!(final_invoice["status"], "Paid");
    } else {
        // If payment failed, invoice should remain Open
        assert_eq!(final_invoice["status"], "Open");
    }

    // Verify no double-charges by checking payment attempts
    // (This would require a GET endpoint for payment attempts, which we don't have yet)
    println!("Concurrent payment test passed with {} successful payments", final_success_count);
}

#[tokio::test]
async fn test_idempotency() {
    let client = Client::new();

    // Create business, customer, and invoice
    let business_id = create_business(&client).await;
    let customer_id = create_customer(&client, business_id).await;
    let invoice_uuid = create_invoice(&client, business_id, customer_id).await;

    // Use the same idempotency key for both requests
    let idempotency_key = Uuid::new_v4();

    // First payment request
    let first_response = client
        .post(format!("{}/invoice/{}/pay", BASE_URL, invoice_uuid))
        .header("Idempotency-Key", idempotency_key.to_string())
        .json(&json!({
            "card_token": "tok_success"
        }))
        .send()
        .await
        .expect("Failed to send first payment request");

    if !first_response.status().is_success() {
        let status = first_response.status();
        let body = first_response.text().await.expect("Failed to read error body");
        panic!("First payment request failed: {} - {}", status, body);
    }
    let first_payment_attempt: serde_json::Value = first_response
        .json()
        .await
        .expect("Failed to parse first payment attempt");
    let first_attempt_id = first_payment_attempt["id"].as_str().expect("Missing attempt id");

    // Second payment request with same idempotency key
    let second_response = client
        .post(format!("{}/invoice/{}/pay", BASE_URL, invoice_uuid))
        .header("Idempotency-Key", idempotency_key.to_string())
        .json(&json!({
            "card_token": "tok_success"
        }))
        .send()
        .await
        .expect("Failed to send second payment request");


    assert!(second_response.status().is_success());
    let second_payment_attempt: serde_json::Value = second_response
        .json()
        .await
        .expect("Failed to parse second payment attempt");
    let second_attempt_id = second_payment_attempt["id"].as_str().expect("Missing attempt id");

    println!("{:?}", second_payment_attempt);

    // Assert same payment attempt is returned (same id)
    assert_eq!(
        first_attempt_id, second_attempt_id,
        "Idempotency failed: different payment attempts returned"
    );

    println!("Idempotency test passed: same payment attempt returned for duplicate requests");
}

#[tokio::test]
async fn test_psp_timeout_failure() {
    let client = Client::new();

    // Create business, customer, and invoice
    let business_id = create_business(&client).await;
    let customer_id = create_customer(&client, business_id).await;
    let invoice_uuid = create_invoice(&client, business_id, customer_id).await;

    // Payment request with TokTimeout (causes PSP to timeout)
    let idempotency_key = Uuid::new_v4();
    let response = client
        .post(format!("{}/invoice/{}/pay", BASE_URL, invoice_uuid))
        .header("Idempotency-Key", idempotency_key.to_string())
        .json(&json!({
            "card_token": "tok_timeout"
        }))
        .send()
        .await
        .expect("Failed to send payment request");

    // The request should fail due to timeout
    assert!(!response.status().is_success());

    // Verify invoice is not stuck in a bad state (should be Open)
    let invoice_response = client
        .get(format!("{}/invoice/{}", BASE_URL, invoice_uuid))
        .send()
        .await
        .expect("Failed to get invoice");

    if !invoice_response.status().is_success() {
        let status = invoice_response.status();
        let body = invoice_response.text().await.expect("Failed to read error body");
        panic!("Failed to get invoice: {} - {}", status, body);
    }
    let final_invoice: serde_json::Value = invoice_response
        .json()
        .await
        .expect("Failed to parse invoice");

    assert_eq!(final_invoice["status"], "Open", "Invoice should remain Open after PSP timeout");

    println!("PSP timeout test passed: invoice not stuck in bad state");
}

#[tokio::test]
async fn test_psp_network_error_failure() {
    let client = Client::new();

    // Create business, customer, and invoice
    let business_id = create_business(&client).await;
    let customer_id = create_customer(&client, business_id).await;
    let invoice_uuid = create_invoice(&client, business_id, customer_id).await;

    // Payment request with TokNetworkError (causes PSP to return 500)
    let idempotency_key = Uuid::new_v4();
    let response = client
        .post(format!("{}/invoice/{}/pay", BASE_URL, invoice_uuid))
        .header("Idempotency-Key", idempotency_key.to_string())
        .json(&json!({
            "card_token": "tok_network_error"
        }))
        .send()
        .await
        .expect("Failed to send payment request");

    // The request should fail due to network error
    assert!(!response.status().is_success());

    // Verify invoice is not stuck in a bad state (should be Open)
    let invoice_response = client
        .get(format!("{}/invoice/{}", BASE_URL, invoice_uuid))
        .send()
        .await
        .expect("Failed to get invoice");

    if !invoice_response.status().is_success() {
        let status = invoice_response.status();
        let body = invoice_response.text().await.expect("Failed to read error body");
        panic!("Failed to get invoice: {} - {}", status, body);
    }
    let final_invoice: serde_json::Value = invoice_response
        .json()
        .await
        .expect("Failed to parse invoice");

    assert_eq!(final_invoice["status"], "Open", "Invoice should remain Open after PSP network error");

    println!("PSP network error test passed: invoice not stuck in bad state");
}
