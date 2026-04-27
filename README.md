# Fast Transaction Processor

A single-node transaction service built with Rust, Axum, and PostgreSQL.

## Features

- HTTP/2 API with Axum
- Transaction validation and idempotency
- PostgreSQL with connection pooling
- Prometheus metrics
- Integration tests

## Architecture

- **API Layer**: Axum HTTP server
- **Service Layer**: Business logic, validation, idempotency
- **Storage Layer**: SQLx with PostgreSQL

## Database Schema

- `accounts`: Account balances
- `transactions`: Transaction ledger
- `idempotency_keys`: Prevents duplicate requests

## Setup

1. Install PostgreSQL
2. Create database: `createdb fast_tx`
3. Set environment variables:
   - `DATABASE_URL=postgresql://user:pass@localhost/fast_tx`
   - `SERVER_ADDR=127.0.0.1`
   - `SERVER_PORT=3000`

## Run

```bash
cargo run
```

## API

### POST /transfer

Transfer money between accounts.

Request:
```json
{
  "idempotency_key": "uuid",
  "from_account": 1,
  "to_account": 2,
  "amount": 100
}
```

Response:
```json
{
  "success": true,
  "message": "Transfer successful"
}
```

### GET /metrics

Prometheus metrics.

## Tests

```bash
cargo test
```

## Troubleshooting

- Ensure PostgreSQL is running
- Check DATABASE_URL
- Run migrations manually if needed