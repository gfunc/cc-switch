use axum::{
    extract::{State, Query},
    routing::{get, post},
    Router,
    Json,
};
use std::sync::Arc;
use std::collections::HashMap;
use rusqlite::Connection;
use serde_json::json;

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
        .route("/takeover", get(get_takeover_status))
        .route("/takeover", post(set_takeover))
        .route("/config", get(get_proxy_config))
        .route("/config", post(update_proxy_config))
        .route("/config/global", get(get_global_config))
        .route("/config/global", post(update_global_config))
        .route("/config/app", get(get_app_config))
        .route("/config/app", post(update_app_config))
}

fn get_setting_string(db: &Connection, key: &str) -> Option<String> {
    db.prepare("SELECT value FROM settings WHERE key = ?1")
        .ok()?
        .query_row([key], |row| row.get::<_, String>(0))
        .ok()
}

async fn get_proxy_status(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<serde_json::Value>> {
    let running: bool = state.with_db(|db: &Connection| {
        get_setting_string(db, "proxy_running")
            .map(|v| v == "1")
            .unwrap_or(false)
    });
    
    let port: i64 = state.with_db(|db: &Connection| {
        get_setting_string(db, "proxy_port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    });
    
    let takeover_status: HashMap<String, bool> = state.with_db(|db: &Connection| {
        get_setting_string(db, "proxy_takeover_status")
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default()
    });
    
    Json(ApiResponse::success(json!({
        "running": running,
        "port": port,
        "takeover_status": takeover_status
    })))
}

async fn start_proxy(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('proxy_running', '1')",
            [],
        ).ok()?;
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "proxy.started",
            json!({}),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn stop_proxy(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('proxy_running', '0')",
            [],
        ).ok()?;
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "proxy.stopped",
            json!({}),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn restart_proxy(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('proxy_running', '1')",
            [],
        ).ok()?;
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "proxy.restarted",
            json!({}),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn get_takeover_status(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<HashMap<String, bool>>> {
    let status: HashMap<String, bool> = state.with_db(|db: &Connection| {
        get_setting_string(db, "proxy_takeover_status")
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default()
    });
    
    Json(ApiResponse::success(status))
}

async fn set_takeover(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(payload): Json<serde_json::Value>,
) -> Json<ApiResponse<bool>> {
    let app = payload.get("app").and_then(|v| v.as_str()).unwrap_or("");
    let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    
    let result = state.with_db(|db: &Connection| {
        let status_str = get_setting_string(db, "proxy_takeover_status");
        let mut status: HashMap<String, bool> = status_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        status.insert(app.to_string(), enabled);
        
        let status_str = serde_json::to_string(&status).ok()?;
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('proxy_takeover_status', ?1)",
            [&status_str],
        ).ok()?;
        
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "proxy.takeover_changed",
            json!({ "app": app, "enabled": enabled }),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn get_proxy_config(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<serde_json::Value>> {
    let config: serde_json::Value = state.with_db(|db: &Connection| {
        get_setting_string(db, "proxy_config")
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_else(|| json!({}))
    });
    
    Json(ApiResponse::success(config))
}

async fn update_proxy_config(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(config): Json<serde_json::Value>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        let config_str = serde_json::to_string(&config).ok()?;
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('proxy_config', ?1)",
            [&config_str],
        ).ok()?;
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "proxy.config_updated",
            json!({}),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn get_global_config(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<serde_json::Value>> {
    let config: serde_json::Value = state.with_db(|db: &Connection| {
        get_setting_string(db, "proxy_global_config")
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_else(|| json!({}))
    });
    
    Json(ApiResponse::success(config))
}

async fn update_global_config(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(config): Json<serde_json::Value>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        let config_str = serde_json::to_string(&config).ok()?;
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('proxy_global_config', ?1)",
            [&config_str],
        ).ok()?;
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "proxy.global_config_updated",
            json!({}),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn get_app_config(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let app = params.get("app").cloned().unwrap_or_default();
    
    let config: serde_json::Value = state.with_db(|db: &Connection| {
        let key = format!("proxy_app_config_{}", app);
        match db.prepare("SELECT value FROM settings WHERE key = ?1") {
            Ok(mut stmt) => {
                stmt.query_row([&key], |row| row.get::<_, String>(0))
                    .ok()
                    .and_then(|v| serde_json::from_str(&v).ok())
                    .unwrap_or_else(|| json!({}))
            }
            Err(_) => json!({})
        }
    });
    
    Json(ApiResponse::success(config))
}

async fn update_app_config(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(params): Query<HashMap<String, String>>,
    Json(config): Json<serde_json::Value>,
) -> Json<ApiResponse<bool>> {
    let app = params.get("app").cloned().unwrap_or_default();
    
    let result = state.with_db(|db: &Connection| {
        let key = format!("proxy_app_config_{}", app);
        let config_str = serde_json::to_string(&config).ok()?;
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, config_str],
        ).ok()?;
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "proxy.app_config_updated",
            json!({ "app": app }),
        );
    }
    
    Json(ApiResponse::success(result))
}
