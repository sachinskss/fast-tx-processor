use axum::{routing::post, Router};
use crate::api::{transfer_handler, metrics_handler};
use crate::db::DbPool;

pub fn create_router(pool: DbPool) -> Router {
    Router::new()
        .route("/transfer", post(transfer_handler))
        .route("/metrics", axum::routing::get(metrics_handler))
        .with_state(pool)
}