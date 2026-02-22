use axum::{
    extract::State,
    routing::{get, put},
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
        .route("/", get(list_settings))
        .route("/", put(update_settings))
}
async fn list_settings(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::success(serde_json::json!({})))
}
async fn update_settings(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(_settings): Json<serde_json::Value>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

