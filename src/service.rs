use crate::db::{get_account, check_idempotency_key};
use crate::models::TransferRequest;
use crate::errors::AppError;
use crate::priority::Priority;
use crate::metrics::{
    STAGE_IDEMPOTENCY_CHECK, STAGE_ACCOUNT_VALIDATION, STAGE_DB_TRANSACTION,
    STAGE_IDEMPOTENCY_INSERT, STAGE_TRANSACTION_INSERT, STAGE_BALANCE_UPDATE,
    STAGE_COMMIT, PRIORITY_LOW_COUNT, PRIORITY_NORMAL_COUNT, PRIORITY_HIGH_COUNT,
    PRIORITY_CRITICAL_COUNT,
};
use sqlx::{Transaction, Postgres, Executor};

pub async fn process_transfer(
    pool: &sqlx::PgPool,
    request: TransferRequest,
) -> Result<(), AppError> {
    // Record priority
    let priority = Priority::from_u8(request.priority).unwrap_or(Priority::Normal);
    match priority {
        Priority::Low => PRIORITY_LOW_COUNT.inc(),
        Priority::Normal => PRIORITY_NORMAL_COUNT.inc(),
        Priority::High => PRIORITY_HIGH_COUNT.inc(),
        Priority::Critical => PRIORITY_CRITICAL_COUNT.inc(),
    }

    // Stage 1: Check idempotency
    let timer = STAGE_IDEMPOTENCY_CHECK.start_timer();
    if check_idempotency_key(pool, request.idempotency_key).await? {
        return Err(AppError::IdempotencyConflict);
    }
    timer.observe_duration();

    // Stage 2: Validate accounts and balance
    let timer = STAGE_ACCOUNT_VALIDATION.start_timer();
    let from_account = get_account(pool, request.from_account).await?;
    let _to_account = get_account(pool, request.to_account).await?;

    if from_account.balance < request.amount {
        return Err(AppError::InsufficientFunds);
    }

    if request.amount <= 0 {
        return Err(AppError::Validation("Amount must be positive".to_string()));
    }
    timer.observe_duration();

    // Stage 3: Start transaction and perform updates
    let timer = STAGE_DB_TRANSACTION.start_timer();
    let mut tx: Transaction<Postgres> = pool.begin().await?;

    // Stage 3a: Insert idempotency key
    let idempotency_timer = STAGE_IDEMPOTENCY_INSERT.start_timer();
    let q = sqlx::query("INSERT INTO idempotency_keys (key) VALUES ($1)")
        .bind(request.idempotency_key);
    tx.execute(q).await?;
    idempotency_timer.observe_duration();

    // Stage 3b: Insert transaction record
    let tx_insert_timer = STAGE_TRANSACTION_INSERT.start_timer();
    let q = sqlx::query("INSERT INTO transactions (from_account, to_account, amount) VALUES ($1, $2, $3)")
        .bind(request.from_account)
        .bind(request.to_account)
        .bind(request.amount);
    tx.execute(q).await?;
    tx_insert_timer.observe_duration();

    // Stage 3c: Update balances
    let balance_timer = STAGE_BALANCE_UPDATE.start_timer();
    let q = sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE id = $2")
        .bind(request.amount)
        .bind(request.from_account);
    tx.execute(q).await?;

    let q = sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE id = $2")
        .bind(request.amount)
        .bind(request.to_account);
    tx.execute(q).await?;
    balance_timer.observe_duration();

    // Stage 3d: Commit transaction
    let commit_timer = STAGE_COMMIT.start_timer();
    tx.commit().await?;
    commit_timer.observe_duration();

    timer.observe_duration();

    Ok(())
}