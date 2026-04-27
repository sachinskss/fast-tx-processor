use sqlx::{PgPool, postgres::PgPoolOptions, Row, Executor};
use crate::models::Account;
use crate::errors::AppError;
use uuid::Uuid;

pub type DbPool = PgPool;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn get_account<E>(executor: E, id: i64) -> Result<Account, AppError>
where
    E: for<'e> Executor<'e, Database = sqlx::Postgres>,
{
    let row = sqlx::query("SELECT id, balance FROM accounts WHERE id = $1")
        .bind(id)
        .fetch_one(executor)
        .await?;
    Ok(Account {
        id: row.get(0),
        balance: row.get(1),
    })
}

pub async fn check_idempotency_key<E>(executor: E, key: Uuid) -> Result<bool, AppError>
where
    E: for<'e> Executor<'e, Database = sqlx::Postgres>,
{
    let row = sqlx::query("SELECT COUNT(*) as count FROM idempotency_keys WHERE key = $1")
        .bind(key)
        .fetch_one(executor)
        .await?;
    Ok(row.get::<i64, _>(0) > 0)
}