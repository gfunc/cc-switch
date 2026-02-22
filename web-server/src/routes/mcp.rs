use axum::{
    extract::State,
    routing::{get, post, put, delete},
    Router,
    Json,
};
use std::sync::Arc;
use crate::{
    models::{
        app_state::AppState,
        McpServer,
        ApiResponse,
    },
    handlers::ws::WsState,
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/", get(list_mcp_servers))
        .route("/", post(create_mcp_server))
        .route("/{id}", get(get_mcp_server))
        .route("/{id}", put(update_mcp_server))
        .route("/{id}", delete(delete_mcp_server))
        .route("/{id}/toggle", post(toggle_mcp_server))
}

async fn list_mcp_servers(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<McpServer>>> {
    Json(ApiResponse::success(vec![]))
}

async fn get_mcp_server(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Option<McpServer>>> {
    Json(ApiResponse::success(None))
}

async fn create_mcp_server(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(server): Json<McpServer>,
) -> Json<ApiResponse<String>> {
    Json(ApiResponse::success(server.id))
}

async fn update_mcp_server(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(server): Json<McpServer>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn delete_mcp_server(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn toggle_mcp_server(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}
