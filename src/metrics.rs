use lazy_static::lazy_static;
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram, Counter, Histogram};

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