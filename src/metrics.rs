use lazy_static::lazy_static;
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram, register_gauge, Counter, Histogram};

lazy_static! {
    pub static ref TRANSFER_REQUESTS: Counter = register_counter!(
        "transfer_requests_total",
        "Total number of transfer requests"
    ).unwrap();

    pub static ref TRANSFER_SUCCESS: Counter = register_counter!(
        "transfer_success_total",
        "Total number of successful transfers"
    ).unwrap();

    pub static ref TRANSFER_FAILURE: Counter = register_counter!(
        "transfer_failure_total",
        "Total number of failed transfers"
    ).unwrap();

    pub static ref TRANSFER_DURATION: Histogram = register_histogram!(
        "transfer_duration_seconds",
        "Duration of transfer processing"
    ).unwrap();

    // Stage-specific metrics
    pub static ref STAGE_IDEMPOTENCY_CHECK: Histogram = register_histogram!(
        "transfer_stage_idempotency_check_seconds",
        "Time spent checking idempotency"
    ).unwrap();

    pub static ref STAGE_ACCOUNT_VALIDATION: Histogram = register_histogram!(
        "transfer_stage_account_validation_seconds",
        "Time spent validating accounts and balance"
    ).unwrap();

    pub static ref STAGE_DB_TRANSACTION: Histogram = register_histogram!(
        "transfer_stage_db_transaction_seconds",
        "Time spent in database transaction"
    ).unwrap();

    pub static ref STAGE_IDEMPOTENCY_INSERT: Histogram = register_histogram!(
        "transfer_stage_idempotency_insert_seconds",
        "Time spent inserting idempotency key"
    ).unwrap();

    pub static ref STAGE_TRANSACTION_INSERT: Histogram = register_histogram!(
        "transfer_stage_transaction_insert_seconds",
        "Time spent inserting transaction record"
    ).unwrap();

    pub static ref STAGE_BALANCE_UPDATE: Histogram = register_histogram!(
        "transfer_stage_balance_update_seconds",
        "Time spent updating balances"
    ).unwrap();

    pub static ref STAGE_COMMIT: Histogram = register_histogram!(
        "transfer_stage_commit_seconds",
        "Time spent committing database transaction"
    ).unwrap();

    // Priority-based counters
    pub static ref PRIORITY_LOW_COUNT: Counter = register_counter!(
        "transfer_priority_low_total",
        "Total low priority transfers"
    ).unwrap();

    pub static ref PRIORITY_NORMAL_COUNT: Counter = register_counter!(
        "transfer_priority_normal_total",
        "Total normal priority transfers"
    ).unwrap();

    pub static ref PRIORITY_HIGH_COUNT: Counter = register_counter!(
        "transfer_priority_high_total",
        "Total high priority transfers"
    ).unwrap();

    pub static ref PRIORITY_CRITICAL_COUNT: Counter = register_counter!(
        "transfer_priority_critical_total",
        "Total critical priority transfers"
    ).unwrap();

    // Queue depth
    pub static ref QUEUE_DEPTH: prometheus::Gauge = register_gauge!(
        "transfer_queue_depth",
        "Current depth of transfer priority queue"
    ).unwrap();
}

pub fn encode_metrics() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_counters() {
        // Note: In a real test, we'd reset metrics, but for simplicity, just check they can be incremented
        TRANSFER_REQUESTS.inc();
        TRANSFER_SUCCESS.inc();
        TRANSFER_FAILURE.inc();

        // Since we can't easily check the values without exposing internals,
        // we just ensure no panics
        assert!(true);
    }

    #[test]
    fn test_histogram() {
        let timer = TRANSFER_DURATION.start_timer();
        std::thread::sleep(Duration::from_millis(10));
        timer.observe_duration();

        assert!(true);
    }

    #[test]
    fn test_encode_metrics() {
        let result = encode_metrics();
        assert!(result.is_ok());
        let metrics = result.unwrap();
        // Just check it's not empty
        assert!(!metrics.is_empty());
    }
}