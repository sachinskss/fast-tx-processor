use fast_tx_processor::db::create_pool;
use fast_tx_processor::models::TransferRequest;
use fast_tx_processor::service::process_transfer;
use uuid::Uuid;

async fn setup_test_db(pool: &sqlx::PgPool) {
    // Clean up
    sqlx::query!("DELETE FROM idempotency_keys").execute(pool).await.unwrap();
    sqlx::query!("DELETE FROM transactions").execute(pool).await.unwrap();
    sqlx::query!("UPDATE accounts SET balance = 1000 WHERE id IN (1, 2)").execute(pool).await.unwrap();
}

#[tokio::test]
async fn test_transfer_success() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    let request = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 1,
        to_account: 2,
        amount: 100,
        priority: 2,
    };

    process_transfer(&pool, request).await.unwrap();

    // Check balances
    let from_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 1").fetch_one(&pool).await.unwrap().balance;
    let to_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 2").fetch_one(&pool).await.unwrap().balance;

    assert_eq!(from_balance, 900);
    assert_eq!(to_balance, 1100);

    // Check transaction recorded
    let tx_count = sqlx::query!("SELECT COUNT(*) as count FROM transactions").fetch_one(&pool).await.unwrap().count.unwrap_or(0);
    assert_eq!(tx_count, 1);
}

#[tokio::test]
async fn test_idempotency() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    let key = Uuid::new_v4();
    let request = TransferRequest {
        idempotency_key: key,
        from_account: 1,
        to_account: 2,
        amount: 100,
        priority: 2,
    };

    // First transfer
    process_transfer(&pool, request.clone()).await.unwrap();

    // Second transfer with same key should fail
    let result = process_transfer(&pool, request).await;
    assert!(result.is_err());

    // Balances should not change again
    let from_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 1").fetch_one(&pool).await.unwrap().balance;
    let to_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 2").fetch_one(&pool).await.unwrap().balance;

    assert_eq!(from_balance, 900);
    assert_eq!(to_balance, 1100);

    // Only one transaction
    let tx_count = sqlx::query!("SELECT COUNT(*) as count FROM transactions").fetch_one(&pool).await.unwrap().count.unwrap_or(0);
    assert_eq!(tx_count, 1);
}

#[tokio::test]
async fn test_insufficient_funds() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    let request = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 1,
        to_account: 2,
        amount: 2000, // More than balance
        priority: 2,
    };

    let result = process_transfer(&pool, request).await;
    assert!(result.is_err());

    // Balances unchanged
    let from_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 1").fetch_one(&pool).await.unwrap().balance;
    let to_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 2").fetch_one(&pool).await.unwrap().balance;

    assert_eq!(from_balance, 1000);
    assert_eq!(to_balance, 1000);

    // No transaction
    let tx_count = sqlx::query!("SELECT COUNT(*) as count FROM transactions").fetch_one(&pool).await.unwrap().count.unwrap_or(0);
    assert_eq!(tx_count, 0);
}

#[tokio::test]
async fn test_invalid_amount_zero() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    let request = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 1,
        to_account: 2,
        amount: 0,
        priority: 2,
    };

    let result = process_transfer(&pool, request).await;
    assert!(result.is_err());

    // Balances unchanged
    let from_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 1").fetch_one(&pool).await.unwrap().balance;
    let to_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 2").fetch_one(&pool).await.unwrap().balance;

    assert_eq!(from_balance, 1000);
    assert_eq!(to_balance, 1000);
}

#[tokio::test]
async fn test_invalid_amount_negative() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    let request = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 1,
        to_account: 2,
        amount: -100,
        priority: 2,
    };

    let result = process_transfer(&pool, request).await;
    assert!(result.is_err());

    // Balances unchanged
    let from_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 1").fetch_one(&pool).await.unwrap().balance;
    let to_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 2").fetch_one(&pool).await.unwrap().balance;

    assert_eq!(from_balance, 1000);
    assert_eq!(to_balance, 1000);
}

#[tokio::test]
async fn test_non_existent_from_account() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    let request = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 999, // Non-existent
        to_account: 2,
        amount: 100,
        priority: 2,
    };

    let result = process_transfer(&pool, request).await;
    assert!(result.is_err());

    // Balances unchanged
    let to_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 2").fetch_one(&pool).await.unwrap().balance;
    assert_eq!(to_balance, 1000);
}

#[tokio::test]
async fn test_non_existent_to_account() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    let request = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 1,
        to_account: 999, // Non-existent
        amount: 100,
        priority: 2,
    };

    let result = process_transfer(&pool, request).await;
    assert!(result.is_err());

    // Balances unchanged
    let from_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 1").fetch_one(&pool).await.unwrap().balance;
    assert_eq!(from_balance, 1000);
}

#[tokio::test]
async fn test_multiple_transfers() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    // First transfer
    let request1 = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 1,
        to_account: 2,
        amount: 100,
        priority: 2,
    };
    process_transfer(&pool, request1).await.unwrap();

    // Second transfer
    let request2 = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 1,
        to_account: 2,
        amount: 200,
        priority: 2,
    };
    process_transfer(&pool, request2).await.unwrap();

    // Check balances
    let from_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 1").fetch_one(&pool).await.unwrap().balance;
    let to_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 2").fetch_one(&pool).await.unwrap().balance;

    assert_eq!(from_balance, 700);
    assert_eq!(to_balance, 1300);

    // Two transactions
    let tx_count = sqlx::query!("SELECT COUNT(*) as count FROM transactions").fetch_one(&pool).await.unwrap().count.unwrap_or(0);
    assert_eq!(tx_count, 2);
}

#[tokio::test]
async fn test_transfer_exact_balance() {
    let pool = create_pool("postgres://user:pass@localhost/fast_tx_test").await.unwrap();
    setup_test_db(&pool).await;

    let request = TransferRequest {
        idempotency_key: Uuid::new_v4(),
        from_account: 1,
        to_account: 2,
        amount: 1000, // Exact balance
        priority: 2,
    };

    process_transfer(&pool, request).await.unwrap();

    // Check balances
    let from_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 1").fetch_one(&pool).await.unwrap().balance;
    let to_balance = sqlx::query!("SELECT balance FROM accounts WHERE id = 2").fetch_one(&pool).await.unwrap().balance;

    assert_eq!(from_balance, 0);
    assert_eq!(to_balance, 2000);
}