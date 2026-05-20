use std::env;

use common::{
    Business, BusinessCustomer, BusinessKey, CreateBusinessArgs, CreateBusinessCustomerArgs,
    CreateInvoiceArgs, Invoice, InvoiceLineItem, Status,
};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
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
            Status::Pending as Status,
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
}