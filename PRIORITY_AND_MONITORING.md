# Transaction Priority & Monitoring System

## Priority Levels

The transaction processor now supports four priority levels:

- **Low (1)**: Batch processing, non-urgent transfers
- **Normal (2)**: Default priority for standard transfers  
- **High (3)**: Time-sensitive transfers
- **Critical (4)**: Emergency, high-value, or urgent transfers

## API Usage

### Transfer with Priority

```json
POST /transfer
{
  "idempotency_key": "550e8400-e29b-41d4-a716-446655440000",
  "from_account": 1,
  "to_account": 2,
  "amount": 1000,
  "priority": 3
}
```

If priority is not specified, it defaults to Normal (2).

## Performance Monitoring

### Overall Transfer Duration

- **Metric**: `transfer_duration_seconds`
- **Type**: Histogram
- Tracks total end-to-end transfer processing time

### Stage-Specific Latencies

Each transaction processing stage has dedicated timing metrics:

1. **Idempotency Check** (`transfer_stage_idempotency_check_seconds`)
   - Validates that the request hasn't been processed before
   - Critical for exactly-once semantics

2. **Account Validation** (`transfer_stage_account_validation_seconds`)
   - Verifies both accounts exist and sender has sufficient balance
   - Catches validation errors early

3. **Database Transaction** (`transfer_stage_db_transaction_seconds`)
   - Total time for all database operations within the transaction

   3a. **Idempotency Insert** (`transfer_stage_idempotency_insert_seconds`)
       - Records the idempotency key to prevent duplicates
   
   3b. **Transaction Insert** (`transfer_stage_transaction_insert_seconds`)
       - Creates the transaction ledger record
   
   3c. **Balance Update** (`transfer_stage_balance_update_seconds`)
       - Updates sender and receiver account balances
   
   3d. **Commit** (`transfer_stage_commit_seconds`)
       - Commits the database transaction

### Priority-Based Counters

- `transfer_priority_low_total`: Count of low priority transfers
- `transfer_priority_normal_total`: Count of normal priority transfers
- `transfer_priority_high_total`: Count of high priority transfers
- `transfer_priority_critical_total`: Count of critical priority transfers

### Queue Monitoring

- **Metric**: `transfer_queue_depth`
- **Type**: Gauge
- Tracks current number of pending transactions in the priority queue

## Architecture

### Priority Queue

The priority queue is implemented with the following characteristics:

- **Max Heap**: Higher priority transfers are processed first
- **FIFO within Priority**: Transfers with the same priority are processed in order of arrival
- **Thread-Safe**: Uses `Arc<Mutex<>>` for safe concurrent access
- **O(n) Insertion**: Simple insertion sort maintains order

### Processing Pipeline

```
Request → Priority Queue → Idempotency Check → Validation → DB Transaction → Response
   ↓           ↓                 ↓               ↓             ↓          ↓
Priority    Queue Depth    Check Timer    Validation      Stage         Overall
Recorded    Updated        Duration      Timer           Timers        Duration
```

## Metrics Endpoint

View all metrics in Prometheus format:

```bash
curl http://localhost:3000/metrics
```

Example output:
```
transfer_requests_total 100
transfer_success_total 95
transfer_failure_total 5
transfer_priority_high_total 25
transfer_priority_normal_total 70
transfer_priority_low_total 5
transfer_stage_idempotency_check_seconds_sum 0.523
transfer_stage_account_validation_seconds_sum 1.245
transfer_stage_db_transaction_seconds_sum 2.156
transfer_stage_commit_seconds_sum 0.892
```

## Performance Tuning

### Use Case: Identify Bottlenecks

Compare stage durations to find which stages are slowest:

```
Average Idempotency Check:  0.005 seconds (fast)
Average Account Validation: 0.015 seconds (fast)
Average Balance Update:     0.045 seconds (potential bottleneck)
Average Commit:            0.020 seconds (acceptable)
```

In this example, balance updates are the bottleneck. Consider:
- Database indexing on account IDs
- Connection pool size optimization
- Query optimization

### Use Case: Priority-Based SLOs

Set different SLOs for different priorities:

| Priority | P99 Latency Target |
|----------|-------------------|
| Low      | < 500ms           |
| Normal   | < 200ms           |
| High     | < 100ms           |
| Critical | < 50ms            |

Monitor using:
```
transfer_duration_seconds{priority="critical"}.histogram_quantile(0.99)
```

## Testing

### Unit Tests

Run tests for the priority module:

```bash
cargo test priority::tests
```

Tests verify:
- Priority ordering logic
- Queue insertion and removal
- FIFO behavior within same priority
- Empty queue handling

### Integration Tests

Integration tests validate the full pipeline with priorities. See `tests/integration_test.rs`.