use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferRequest {
    pub idempotency_key: Uuid,
    pub from_account: i64,
    pub to_account: i64,
    pub amount: i64,
    #[serde(default = "default_priority")]
    pub priority: u8, // 1=Low, 2=Normal, 3=High, 4=Critical
}

fn default_priority() -> u8 {
    2 // Normal priority
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: i64,
    pub balance: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i64,
    pub from_account: i64,
    pub to_account: i64,
    pub amount: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct IdempotencyKey {
    pub key: Uuid,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_transfer_request_creation() {
        let key = Uuid::new_v4();
        let request = TransferRequest {
            idempotency_key: key,
            from_account: 1,
            to_account: 2,
            amount: 100,
            priority: 2,
        };

        assert_eq!(request.idempotency_key, key);
        assert_eq!(request.from_account, 1);
        assert_eq!(request.to_account, 2);
        assert_eq!(request.amount, 100);
        assert_eq!(request.priority, 2);
    }

    #[test]
    fn test_transfer_request_serialization() {
        let key = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let request = TransferRequest {
            idempotency_key: key,
            from_account: 1,
            to_account: 2,
            amount: 100,
            priority: 2,
        };

        let json = serde_json::to_string(&request).unwrap();
        let expected = format!(r#"{{"idempotency_key":"{}","from_account":1,"to_account":2,"amount":100,"priority":2}}"#, key);
        assert_eq!(json, expected);
    }

    #[test]
    fn test_transfer_request_deserialization() {
        let key = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let json = format!(r#"{{"idempotency_key":"{}","from_account":1,"to_account":2,"amount":100,"priority":3}}"#, key);
        let request: TransferRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.idempotency_key, key);
        assert_eq!(request.from_account, 1);
        assert_eq!(request.to_account, 2);
        assert_eq!(request.amount, 100);
        assert_eq!(request.priority, 3);
    }

    #[test]
    fn test_account_creation() {
        let account = Account {
            id: 1,
            balance: 1000,
        };

        assert_eq!(account.id, 1);
        assert_eq!(account.balance, 1000);
    }

    #[test]
    fn test_transaction_creation() {
        let created_at = Utc::now();
        let transaction = Transaction {
            id: 1,
            from_account: 1,
            to_account: 2,
            amount: 100,
            created_at,
        };

        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.from_account, 1);
        assert_eq!(transaction.to_account, 2);
        assert_eq!(transaction.amount, 100);
        assert_eq!(transaction.created_at, created_at);
    }

    #[test]
    fn test_idempotency_key_creation() {
        let key = Uuid::new_v4();
        let created_at = Utc::now();
        let idempotency_key = IdempotencyKey {
            key,
            created_at,
        };

        assert_eq!(idempotency_key.key, key);
        assert_eq!(idempotency_key.created_at, created_at);
    }

    #[test]
    fn test_struct_equality() {
        let key1 = Uuid::new_v4();
        let key2 = key1.clone();

        let request1 = TransferRequest {
            idempotency_key: key1,
            from_account: 1,
            to_account: 2,
            amount: 100,
            priority: 2,
        };

        let request2 = TransferRequest {
            idempotency_key: key2,
            from_account: 1,
            to_account: 2,
            amount: 100,
            priority: 2,
        };

        assert_eq!(request1.from_account, request2.from_account);
        assert_eq!(request1.to_account, request2.to_account);
        assert_eq!(request1.amount, request2.amount);
        assert_eq!(request1.idempotency_key, request2.idempotency_key);
        assert_eq!(request1.priority, request2.priority);
    }
}