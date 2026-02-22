use axum::{
    routing::{get, post},
    Router,
    Json,
};
use std::sync::Arc;
use crate::{
    models::{
        app_state::AppState,
        ApiResponse,
    },
    handlers::ws::WsState,
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/status", get(get_proxy_status))
        .route("/start", post(start_proxy))
        .route("/stop", post(stop_proxy))
        .route("/restart", post(restart_proxy))
}

async fn get_proxy_status(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::success(serde_json::json!({
        "running": false,
        "port": 0,
        "takeover_status": {}
    })))
}

async fn start_proxy(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn stop_proxy(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn restart_proxy(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

use axum::extract::State;
