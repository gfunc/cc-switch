use axum::{
    extract::{State, Path},
    routing::{get, post, delete},
    Router,
    Json,
};
use std::sync::Arc;
use rusqlite::Connection;
use serde_json::json;

use crate::web::{
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
    .route("/:id", get(get_session))
    .route("/:id/resume", post(resume_session))
    .route("/:id", delete(delete_session))
    .route("/:id/messages", get(get_session_messages))
}

async fn list_sessions(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<Session>>> {
    let result: Vec<Session> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT session_id, provider_id, title, summary, project_dir, created_at, last_active_at, source_path, resume_command 
             FROM sessions 
             ORDER BY last_active_at DESC"
        ).ok()?;
        
        let rows = stmt.query_map([], |row| {
            Ok(Session {
                session_id: row.get(0)?,
                provider_id: row.get(1)?,
                title: row.get(2)?,
                summary: row.get(3)?,
                project_dir: row.get(4)?,
                created_at: row.get(5)?,
                last_active_at: row.get(6)?,
                source_path: row.get(7)?,
                resume_command: row.get(8)?,
            })
        }).ok()?;
        
        let sessions: Vec<Session> = rows.filter_map(|r| r.ok()).collect();
        Some(sessions)
    }).unwrap_or_default();
    
    Json(ApiResponse::success(result))
}

async fn get_session(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Option<Session>>> {
    let result: Option<Session> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT session_id, provider_id, title, summary, project_dir, created_at, last_active_at, source_path, resume_command 
             FROM sessions 
             WHERE session_id = ?1"
        ).ok()?;
        
        stmt.query_row([&id], |row| {
            Ok(Session {
                session_id: row.get(0)?,
                provider_id: row.get(1)?,
                title: row.get(2)?,
                summary: row.get(3)?,
                project_dir: row.get(4)?,
                created_at: row.get(5)?,
                last_active_at: row.get(6)?,
                source_path: row.get(7)?,
                resume_command: row.get(8)?,
            })
        }).ok()
    });
    
    Json(ApiResponse::success(result))
}

async fn get_session_messages(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let result: Vec<serde_json::Value> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT messages FROM session_messages WHERE session_id = ?1 ORDER BY created_at ASC"
        ).ok()?;
        
        let rows = stmt.query_map([&id], |row| {
            let msg_str: String = row.get(0)?;
            serde_json::from_str(&msg_str).map_err(|_| rusqlite::Error::InvalidQuery)
        }).ok()?;
        
        let messages: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
        Some(messages)
    }).unwrap_or_default();
    
    Json(ApiResponse::success(result))
}

async fn resume_session(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Json<ApiResponse<bool>> {
    let command = payload.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let cwd = payload.get("cwd").and_then(|v| v.as_str());
    
    let result = state.with_db(|db: &Connection| {
        let now = chrono::Utc::now().timestamp();
        db.execute(
            "UPDATE sessions SET last_active_at = ?1, resume_command = ?2 WHERE session_id = ?3",
            rusqlite::params![now, command, &id],
        ).ok()?;
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "session.resumed",
            json!({ "id": id, "command": command, "cwd": cwd }),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn delete_session(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        db.execute(
            "DELETE FROM sessions WHERE session_id = ?1",
            [&id],
        ).ok()?;
        
        db.execute(
            "DELETE FROM session_messages WHERE session_id = ?1",
            [&id],
        ).ok()?;
        
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "session.deleted",
            json!({ "id": id }),
        );
    }
    
    Json(ApiResponse::success(result))
}