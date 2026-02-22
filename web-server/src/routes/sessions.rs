use axum::{
    routing::{get, post, delete},
    Router,
    Json,
};
use std::sync::Arc;
use crate::{
    models::{
        app_state::AppState,
        Session,
        ApiResponse,
    },
    handlers::ws::WsState,
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/", get(list_sessions))
        .route("/{id}", get(get_session))
        .route("/{id}/resume", post(resume_session))
        .route("/{id}", delete(delete_session))
}

async fn list_sessions(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<Session>>> {
    Json(ApiResponse::success(vec![]))
}

async fn get_session(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Option<Session>>> {
    Json(ApiResponse::success(None))
}

async fn resume_session(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn delete_session(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

use axum::extract::State;
