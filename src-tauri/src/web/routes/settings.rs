use axum::{
    extract::State,
    routing::{get, put},
    Router,
    Json,
};
use std::sync::Arc;
use rusqlite::Connection;
use serde_json::json;

use crate::{
    models::{
        app_state::AppState,
        Settings,
        ApiResponse,
    },
    handlers::ws::WsState,
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/", get(list_settings))
        .route("/", put(update_settings))
        .route("/app-config-path", get(get_app_config_path))
}

fn default_settings() -> Settings {
    Settings {
        show_in_tray: true,
        minimize_to_tray_on_close: true,
        enable_claude_plugin_integration: None,
        skip_claude_onboarding: None,
        launch_on_startup: None,
        silent_startup: None,
        enable_local_proxy: None,
        language: None,
        visible_apps: None,
        claude_config_dir: None,
        codex_config_dir: None,
        gemini_config_dir: None,
        opencode_config_dir: None,
        openclaw_config_dir: None,
        current_provider_claude: None,
        current_provider_codex: None,
        current_provider_gemini: None,
        skill_sync_method: None,
        webdav_sync: None,
        preferred_terminal: None,
    }
}

async fn list_settings(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Settings>> {
    let stored: Option<String> = state.with_db(|db: &Connection| {
        let mut stmt = db
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .ok()?;
        stmt.query_row([&"app_settings"], |row: &rusqlite::Row| row.get::<usize, String>(0))
            .ok()
    });

    if let Some(json_str) = stored {
        match serde_json::from_str::<Settings>(&json_str) {
            Ok(settings) => Json(ApiResponse::success(settings)),
            Err(e) => {
                eprintln!("Failed to parse app_settings from DB: {}. Returning defaults.", e);
                Json(ApiResponse::success(default_settings()))
            }
        }
    } else {
        Json(ApiResponse::success(default_settings()))
    }
}

async fn update_settings(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(settings): Json<Settings>,
) -> Json<ApiResponse<bool>> {
    match serde_json::to_string(&settings) {
        Ok(json_str) => {
            let res = state.with_db(|db: &Connection| {
                db.execute(
                    "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                    rusqlite::params!["app_settings", json_str],
                )
                .map_err(|e| e.to_string())
            });

            match res {
                Ok(_) => {
                    crate::web::handlers::ws::broadcast_event(
                        &ws_state,
                        "settings.changed",
                        json!({}),
                    );
                    Json(ApiResponse::success(true))
                }
                Err(e) => Json(ApiResponse::error(format!("Database error: {}", e))),
            }
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to serialize settings: {}",
            e
        ))),
    }
}

async fn get_app_config_path(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<serde_json::Value>> {
    let path = if let Ok(home_override) = std::env::var("CC_SWITCH_TEST_HOME") {
        let trimmed = home_override.trim();
        if !trimmed.is_empty() {
            std::path::PathBuf::from(trimmed).join(".cc-switch")
        } else {
            dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join(".cc-switch")
        }
    } else {
        dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")).join(".cc-switch")
    };

    Json(ApiResponse::success(json!({ "path": path.to_string_lossy().to_string() })))
}