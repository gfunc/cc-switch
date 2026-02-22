use axum::{
    routing::{get, post, delete},
    Router,
    Json,
};
use std::sync::Arc;
use crate::{
    models::{
        app_state::AppState,
        Skill,
        ApiResponse,
    },
    handlers::ws::WsState,
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/", get(list_skills))
        .route("/discover", get(discover_skills))
        .route("/{id}/install", post(install_skill))
        .route("/{id}/uninstall", delete(uninstall_skill))
}

async fn list_skills(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<Skill>>> {
    Json(ApiResponse::success(vec![]))
}

async fn discover_skills(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<Skill>>> {
    Json(ApiResponse::success(vec![]))
}

async fn install_skill(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn uninstall_skill(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

use axum::extract::State;
