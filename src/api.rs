use axum::{extract::State, http::StatusCode, Json};
use crate::models::TransferRequest;
use crate::service::process_transfer;
use crate::metrics::{TRANSFER_REQUESTS, TRANSFER_SUCCESS, TRANSFER_FAILURE, TRANSFER_DURATION};
use crate::errors::AppError;
use crate::db::DbPool;
use serde::Serialize;

#[derive(Serialize)]
pub struct TransferResponse {
    pub success: bool,
    pub message: String,
}

pub async fn transfer_handler(
    State(pool): State<DbPool>,
    Json(request): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, AppError> {
    TRANSFER_REQUESTS.inc();

    let timer = TRANSFER_DURATION.start_timer();

    let result = process_transfer(&pool, request).await;

    timer.observe_duration();

    match result {
        Ok(_) => {
            TRANSFER_SUCCESS.inc();
            Ok(Json(TransferResponse {
                success: true,
                message: "Transfer successful".to_string(),
            }))
        }
        Err(e) => {
            TRANSFER_FAILURE.inc();
            Err(e)
        }
    }
}

pub async fn metrics_handler() -> Result<String, StatusCode> {
    match crate::metrics::encode_metrics() {
        Ok(metrics) => Ok(metrics),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}