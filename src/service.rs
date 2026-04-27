use crate::db::{get_account, check_idempotency_key};
use crate::models::TransferRequest;
use crate::errors::AppError;
use sqlx::{Transaction, Postgres, Executor};

pub async fn process_transfer(
    pool: &sqlx::PgPool,
    request: TransferRequest,
) -> Result<(), AppError> {
    // Check idempotency
    if check_idempotency_key(pool, request.idempotency_key).await? {
        return Err(AppError::IdempotencyConflict);
    }

    // Validate accounts and balance
    let from_account = get_account(pool, request.from_account).await?;
    let _to_account = get_account(pool, request.to_account).await?;

    if from_account.balance < request.amount {
        return Err(AppError::InsufficientFunds);
    }

    if request.amount <= 0 {
        return Err(AppError::Validation("Amount must be positive".to_string()));
    }

    // Start transaction
    let mut tx: Transaction<Postgres> = pool.begin().await?;

    // Insert idempotency key
    let q = sqlx::query("INSERT INTO idempotency_keys (key) VALUES ($1)")
        .bind(request.idempotency_key);
    tx.execute(q).await?;

    // Insert transaction
    let q = sqlx::query("INSERT INTO transactions (from_account, to_account, amount) VALUES ($1, $2, $3)")
        .bind(request.from_account)
        .bind(request.to_account)
        .bind(request.amount);
    tx.execute(q).await?;

    // Update balances
    let q = sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE id = $2")
        .bind(request.amount)
        .bind(request.from_account);
    tx.execute(q).await?;

    let q = sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE id = $2")
        .bind(request.amount)
        .bind(request.to_account);
    tx.execute(q).await?;

    // Commit
    tx.commit().await?;

    Ok(())
}