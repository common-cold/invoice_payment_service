use std::env;

use common::{
    Business, BusinessCustomer, BusinessKey, CreateBusinessArgs, CreateBusinessCustomerArgs,
    CreateInvoiceArgs, Invoice, PspPaymentAttempt, PspPaymentStatus, PaymentAttempt,
    PaymentStatus, Status,
};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions, Transaction, Executor};
use uuid::Uuid;

use dotenv::dotenv;



#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn init_db() -> anyhow::Result<Self> {
        dotenv().ok();

        let DATABASE_URL = env::var("DATABASE_URL")?;

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&DATABASE_URL)
            .await?;

        Ok(Self { pool })
    }

    pub async fn create_business(&self, args: CreateBusinessArgs) -> anyhow::Result<Business> {
        let business = sqlx::query_as!(
            Business,
            r#"
                INSERT INTO businesses (name)
                VALUES ($1)
                RETURNING
                    id,
                    name,
                    created_at,
                    updated_at
            "#,
            args.name,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(business)
    }

    pub async fn create_business_auth_key(
        &self,
        business_id: Uuid,
        key_hash: String,
        label: String,
    ) -> anyhow::Result<BusinessKey> {
        let business_key = sqlx::query_as!(
            BusinessKey,
            r#"
                INSERT INTO business_keys (business_id, key_hash, label, is_active)
                VALUES ($1, $2, $3, true)
                RETURNING
                    id,
                    business_id,
                    key_hash,
                    label,
                    is_active,
                    created_at
            "#,
            business_id,
            key_hash,
            label,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(business_key)
    }

    pub async fn create_business_customer(
        &self,
        args: CreateBusinessCustomerArgs,
    ) -> anyhow::Result<BusinessCustomer> {
        let customer = sqlx::query_as!(
            BusinessCustomer,
            r#"
                INSERT INTO business_customers (business_id, email_id, name)
                VALUES ($1, $2, $3)
                RETURNING
                    id,
                    business_id,
                    email_id,
                    name,
                    created_at,
                    updated_at
            "#,
            args.business_id,
            args.email_id,
            args.name,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(customer)
    }

    pub async fn get_business_customer_by_id(
        &self,
        id: Uuid,
    ) -> anyhow::Result<Option<BusinessCustomer>> {
        let customer = sqlx::query_as!(
            BusinessCustomer,
            r#"
                SELECT
                    id,
                    business_id,
                    email_id,
                    name,
                    created_at,
                    updated_at
                FROM business_customers
                WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(customer)
    }

    pub async fn create_invoice(
        &self,
        args: CreateInvoiceArgs,
    ) -> anyhow::Result<Invoice> {
        let mut tx = self.pool.begin().await?;

        let invoice = sqlx::query_as!(
            Invoice,
            r#"
                INSERT INTO invoices (business_id, customer_id, status, total_cents, due_date)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING
                    id,
                    business_id,
                    customer_id,
                    status AS "status: Status",
                    total_cents,
                    due_date,
                    created_at,
                    updated_at
            "#,
            args.business_id,
            args.customer_id,
            Status::Open as Status,
            args.total_cents.unwrap(),
            args.due_date,
        )
        .fetch_one(&mut *tx)
        .await?;

        for item in args.line_items {
            sqlx::query!(
                r#"
                    INSERT INTO invoice_line_items (invoice_id, description, quantity, unit_price_cents, amount_cents)
                    VALUES ($1, $2, $3, $4, $5)
                "#,
                invoice.id,
                item.description,
                item.quantity as i32,
                item.unit_price_cents,
                item.amount_cents.unwrap(),
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(invoice)
    }

    pub async fn get_invoice_by_id(
        &self,
        id: Uuid,
    ) -> anyhow::Result<Option<Invoice>> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
                SELECT
                    id,
                    business_id,
                    customer_id,
                    status AS "status: Status",
                    total_cents,
                    due_date,
                    created_at,
                    updated_at
                FROM invoices
                WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(invoice)
    }

    pub async fn get_invoice_by_id_with_tx(
        &self,
        id: Uuid,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Option<Invoice>> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
                SELECT
                    id,
                    business_id,
                    customer_id,
                    status AS "status: Status",
                    total_cents,
                    due_date,
                    created_at,
                    updated_at
                FROM invoices
                WHERE id = $1
                FOR UPDATE
            "#,
            id,
        )
        .fetch_optional(tx.as_mut())
        .await?;

        Ok(invoice)
    }

    pub async fn update_invoice_status(
        &self,
        id: Uuid,
        status: Status,
    ) -> anyhow::Result<Invoice> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
                UPDATE invoices
                SET status = $2
                WHERE id = $1
                RETURNING
                    id,
                    business_id,
                    customer_id,
                    status AS "status: Status",
                    total_cents,
                    due_date,
                    created_at,
                    updated_at
            "#,
            id,
            status as Status,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(invoice)
    }

    pub async fn update_invoice_status_with_tx(
        &self,
        id: Uuid,
        status: Status,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Invoice> {
        let invoice = sqlx::query_as!(
            Invoice,
            r#"
                UPDATE invoices
                SET status = $2
                WHERE id = $1
                RETURNING
                    id,
                    business_id,
                    customer_id,
                    status AS "status: Status",
                    total_cents,
                    due_date,
                    created_at,
                    updated_at
            "#,
            id,
            status as Status,
        )
        .fetch_one(tx.as_mut())
        .await?;

        Ok(invoice)
    }

    pub async fn create_payment_attempt(
        &self,
        invoice_id: Uuid,
        status: PaymentStatus,
        idempotency_key: Uuid,
        amount_cents: Option<i32>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> anyhow::Result<PaymentAttempt> {
        let payment_attempt = sqlx::query_as!(
            PaymentAttempt,
            r#"
                INSERT INTO payment_attempts (invoice_id, status, idempotency_key, amount_cents, error_code, error_message)
                VALUES ($1, $2, $3, COALESCE($4, 0), COALESCE($5, ''), COALESCE($6, ''))
                RETURNING
                    id,
                    invoice_id,
                    status AS "status: PaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
            "#,
            invoice_id,
            status as PaymentStatus,
            idempotency_key,
            amount_cents,
            error_code,
            error_message,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(payment_attempt)
    }

    pub async fn create_payment_attempt_with_tx(
        &self,
        invoice_id: Uuid,
        status: PaymentStatus,
        idempotency_key: Uuid,
        amount_cents: Option<i32>,
        error_code: Option<String>,
        error_message: Option<String>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<PaymentAttempt> {
        let payment_attempt = sqlx::query_as!(
            PaymentAttempt,
            r#"
                INSERT INTO payment_attempts (invoice_id, status, idempotency_key, amount_cents, error_code, error_message)
                VALUES ($1, $2, $3, COALESCE($4, 0), COALESCE($5, ''), COALESCE($6, ''))
                RETURNING
                    id,
                    invoice_id,
                    status AS "status: PaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
            "#,
            invoice_id,
            status as PaymentStatus,
            idempotency_key,
            amount_cents,
            error_code,
            error_message,
        )
        .fetch_one(tx.as_mut())
        .await?;

        Ok(payment_attempt)
    }

    pub async fn get_payment_attempt_by_idempotency_key(
        &self,
        idempotency_key: Uuid,
    ) -> anyhow::Result<Option<PaymentAttempt>> {
        let payment_attempt = sqlx::query_as!(
            PaymentAttempt,
            r#"
                SELECT
                    id,
                    invoice_id,
                    status AS "status: PaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
                FROM payment_attempts
                WHERE idempotency_key = $1
            "#,
            idempotency_key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(payment_attempt)
    }

    pub async fn get_payment_attempt_by_idempotency_key_with_tx(
        &self,
        idempotency_key: Uuid,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Option<PaymentAttempt>> {
        let payment_attempt = sqlx::query_as!(
            PaymentAttempt,
            r#"
                SELECT
                    id,
                    invoice_id,
                    status AS "status: PaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
                FROM payment_attempts
                WHERE idempotency_key = $1
            "#,
            idempotency_key,
        )
        .fetch_optional(tx.as_mut())
        .await?;

        Ok(payment_attempt)
    }

    pub async fn update_payment_attempt(
        &self,
        idempotency_key: Uuid,
        status: PaymentStatus,
        amount_cents: Option<i64>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> anyhow::Result<PaymentAttempt> {
        let payment_attempt = sqlx::query_as!(
            PaymentAttempt,
            r#"
                UPDATE payment_attempts
                SET status = $2, amount_cents = COALESCE($3, amount_cents), error_code = COALESCE($4, error_code), error_message = COALESCE($5, error_message)
                WHERE idempotency_key = $1
                RETURNING
                    id,
                    invoice_id,
                    status AS "status: PaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
            "#,
            idempotency_key,
            status as PaymentStatus,
            amount_cents,
            error_code,
            error_message,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(payment_attempt)
    }

    pub async fn update_payment_attempt_with_tx(
        &self,
        idempotency_key: Uuid,
        status: PaymentStatus,
        amount_cents: Option<i64>,
        error_code: Option<String>,
        error_message: Option<String>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<PaymentAttempt> {
        let payment_attempt = sqlx::query_as!(
            PaymentAttempt,
            r#"
                UPDATE payment_attempts
                SET status = $2, amount_cents = COALESCE($3, amount_cents), error_code = COALESCE($4, error_code), error_message = COALESCE($5, error_message)
                WHERE idempotency_key = $1
                RETURNING
                    id,
                    invoice_id,
                    status AS "status: PaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
            "#,
            idempotency_key,
            status as PaymentStatus,
            amount_cents,
            error_code,
            error_message,
        )
        .fetch_one(tx.as_mut())
        .await?;

        Ok(payment_attempt)
    }

    pub async fn get_payment_attempts_by_status(
        &self,
        status: PaymentStatus,
    ) -> anyhow::Result<Vec<PaymentAttempt>> {
        let payment_attempts = sqlx::query_as!(
            PaymentAttempt,
            r#"
                SELECT
                    id,
                    invoice_id,
                    status AS "status: PaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
                FROM payment_attempts
                WHERE status = $1
            "#,
            status as PaymentStatus,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(payment_attempts)
    }

    pub async fn create_psp_payment_attempt(
        &self,
        invoice_id: Uuid,
        status: PspPaymentStatus,
        idempotency_key: Uuid,
        amount_cents: Option<i32>,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> anyhow::Result<PspPaymentAttempt> {
        let psp_payment_attempt = sqlx::query_as!(
            PspPaymentAttempt,
            r#"
                INSERT INTO psp_payment_attempts (invoice_id, status, idempotency_key, amount_cents, error_code, error_message)
                VALUES ($1, $2, $3, COALESCE($4, 0), COALESCE($5, ''), COALESCE($6, ''))
                RETURNING
                    id,
                    invoice_id,
                    status AS "status: PspPaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
            "#,
            invoice_id,
            status as PspPaymentStatus,
            idempotency_key,
            amount_cents,
            error_code,
            error_message,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(psp_payment_attempt)
    }

    pub async fn get_psp_payment_attempt_by_idempotency_key(
        &self,
        idempotency_key: Uuid,
    ) -> anyhow::Result<Option<PspPaymentAttempt>> {
        let psp_payment_attempt = sqlx::query_as!(
            PspPaymentAttempt,
            r#"
                SELECT
                    id,
                    invoice_id,
                    status AS "status: PspPaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
                FROM psp_payment_attempts
                WHERE idempotency_key = $1
            "#,
            idempotency_key,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(psp_payment_attempt)
    }

    pub async fn get_psp_payment_attempts_by_idempotency_keys(
        &self,
        idempotency_keys: Vec<Uuid>,
    ) -> anyhow::Result<Vec<PspPaymentAttempt>> {
        let psp_payment_attempts = sqlx::query_as!(
            PspPaymentAttempt,
            r#"
                SELECT
                    id,
                    invoice_id,
                    status AS "status: PspPaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
                FROM psp_payment_attempts
                WHERE idempotency_key = ANY($1)
            "#,
            &idempotency_keys,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(psp_payment_attempts)
    }

    pub async fn update_psp_payment_attempt_status(
        &self,
        idempotency_key: Uuid,
        status: PspPaymentStatus,
        error_code: Option<String>,
        error_message: Option<String>,
    ) -> anyhow::Result<PspPaymentAttempt> {
        let psp_payment_attempt = sqlx::query_as!(
            PspPaymentAttempt,
            r#"
                UPDATE psp_payment_attempts
                SET status = $2, error_code = COALESCE($3, error_code), error_message = COALESCE($4, error_message)
                WHERE idempotency_key = $1
                RETURNING
                    id,
                    invoice_id,
                    status AS "status: PspPaymentStatus",
                    idempotency_key,
                    amount_cents,
                    error_code,
                    error_message,
                    created_at,
                    updated_at
            "#,
            idempotency_key,
            status as PspPaymentStatus,
            error_code,
            error_message,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(psp_payment_attempt)
    }
}