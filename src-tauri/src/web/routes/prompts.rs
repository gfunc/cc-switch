use axum::{
    extract::State,
    routing::{get, post, put, delete},
    Router,
    Json,
};
use std::sync::Arc;
use crate::web::{
    models::{
        app_state::AppState,
        Prompt,
        ApiResponse,
    },
    handlers::ws::WsState,
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/", get(list_prompts))
        .route("/", post(create_prompt))
        .route("/{id}", get(get_prompt))
        .route("/{id}", put(update_prompt))
        .route("/{id}", delete(delete_prompt))
        .route("/{id}/activate", post(activate_prompt))
}

async fn list_prompts(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<Prompt>>> {
    Json(ApiResponse::success(vec![]))
}

async fn get_prompt(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Option<Prompt>>> {
    Json(ApiResponse::success(None))
}

async fn create_prompt(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(prompt): Json<Prompt>,
) -> Json<ApiResponse<String>> {
    Json(ApiResponse::success(prompt.id))
}

async fn update_prompt(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(_prompt): Json<Prompt>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn delete_prompt(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn activate_prompt(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}
