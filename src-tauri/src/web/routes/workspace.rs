use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::services::workspace::WorkspaceService;
use crate::web::{
    handlers::ws::WsState,
    models::{app_state::AppState, ApiResponse},
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/file/:filename", get(read_workspace_file).put(write_workspace_file))
        .route("/daily-memory", get(list_daily_memory_files))
        .route("/daily-memory/search", get(search_daily_memory_files))
        .route(
            "/daily-memory/:filename",
            get(read_daily_memory_file)
                .put(write_daily_memory_file)
                .delete(delete_daily_memory_file),
        )
        .route("/directory", get(get_directory_path))
}

#[derive(Deserialize)]
struct WriteBody {
    content: String,
}

#[derive(Deserialize)]
struct SubdirQuery {
    subdir: String,
}

async fn read_workspace_file(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(filename): Path<String>,
) -> Json<ApiResponse<Option<String>>> {
    match WorkspaceService::read_workspace_file(&filename).await {
        Ok(v) => Json(ApiResponse::success(v)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn write_workspace_file(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(filename): Path<String>,
    Json(body): Json<WriteBody>,
) -> Json<ApiResponse<bool>> {
    match WorkspaceService::write_workspace_file(&filename, &body.content).await {
        Ok(_) => Json(ApiResponse::success(true)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn list_daily_memory_files(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<crate::services::workspace::DailyMemoryFileInfo>>> {
    match WorkspaceService::list_daily_memory_files().await {
        Ok(v) => Json(ApiResponse::success(v)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn read_daily_memory_file(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(filename): Path<String>,
) -> Json<ApiResponse<Option<String>>> {
    match WorkspaceService::read_daily_memory_file(&filename).await {
        Ok(v) => Json(ApiResponse::success(v)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn write_daily_memory_file(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(filename): Path<String>,
    Json(body): Json<WriteBody>,
) -> Json<ApiResponse<bool>> {
    match WorkspaceService::write_daily_memory_file(&filename, &body.content).await {
        Ok(_) => Json(ApiResponse::success(true)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn delete_daily_memory_file(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(filename): Path<String>,
) -> Json<ApiResponse<bool>> {
    match WorkspaceService::delete_daily_memory_file(&filename).await {
        Ok(_) => Json(ApiResponse::success(true)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    query: Option<String>,
}

async fn search_daily_memory_files(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(query): Query<SearchQuery>,
) -> Json<ApiResponse<Vec<crate::services::workspace::DailyMemorySearchResult>>> {
    let q = query.query.as_deref().unwrap_or("");
    match WorkspaceService::search_daily_memory_files(q).await {
        Ok(v) => Json(ApiResponse::success(v)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_directory_path(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(query): Query<SubdirQuery>,
) -> Json<ApiResponse<String>> {
    let path = match query.subdir.as_str() {
        "memory" => WorkspaceService::memory_directory(),
        _ => WorkspaceService::workspace_directory(),
    };
    Json(ApiResponse::success(path.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::models::app_state::AppState;
    use std::env::temp_dir;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    fn test_ws_state() -> Arc<WsState> {
        Arc::new(WsState::new(broadcast::channel(16).0))
    }

    fn test_state() -> Arc<AppState> {
        let home = temp_dir().join(format!("cc-switch-web-ws-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("CC_SWITCH_TEST_HOME", &home);
        std::env::set_var("HOME", &home);
        let db_path = home.join("cc-switch.db");
        Arc::new(AppState::new(db_path.to_str().unwrap()).unwrap())
    }

    #[tokio::test]
    async fn workspace_web_routes_read_and_write_workspace_file() {
        let state = test_state();
        let ws = test_ws_state();

        let written = write_workspace_file(
            State((state.clone(), ws.clone())),
            Path("AGENTS.md".to_string()),
            Json(WriteBody {
                content: "# agents".to_string(),
            }),
        )
        .await;
        assert!(written.0.success, "write failed: {:?}", written.0.error);

        let read = read_workspace_file(
            State((state.clone(), ws.clone())),
            Path("AGENTS.md".to_string()),
        )
        .await;
        assert!(read.0.success, "read failed: {:?}", read.0.error);
        assert_eq!(read.0.data.unwrap(), Some("# agents".to_string()));
    }
}
