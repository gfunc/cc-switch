use axum::{
    routing::{get, post, put, delete},
    Router,
    Json,
    extract::{State, Path, Query},
};
use std::sync::Arc;
use crate::{
    models::{
        app_state::AppState,
        Provider,
        ApiResponse,
    },
    handlers::ws::WsState,
};
use indexmap::IndexMap;
use rusqlite::{Connection, Result as SqliteResult};
use serde_json::json;

const DEFAULT_APP_TYPE: &str = "claude";

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/", get(list_providers))
        .route("/", post(create_provider))
        .route("/{id}", get(get_provider))
        .route("/{id}", put(update_provider))
        .route("/{id}", delete(delete_provider))
        .route("/{id}/switch", post(switch_provider))
        .route("/{id}/endpoints", get(get_custom_endpoints))
        .route("/{id}/endpoints", post(add_custom_endpoint))
        .route("/{id}/endpoints/{url}", delete(remove_custom_endpoint))
        .route("/current", get(get_current_provider))
        .route("/sort", post(update_sort_order))
        .route("/import-default", post(import_default_config))
}

async fn import_default_config(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<bool>> {
    let app = params.get("app").cloned().unwrap_or_else(|| DEFAULT_APP_TYPE.to_string());
    
    // In web mode, there's no local filesystem config to import from
    // This feature requires the desktop app with access to local config files
    Json(ApiResponse::error(format!(
        "Import from live config is not available in web mode. \
        Please use the desktop app to import your {} configuration, \
        or manually add providers using the 'Add Provider' button.",
        app
    )))
}

fn row_to_provider(row: &rusqlite::Row) -> SqliteResult<Provider> {
    let settings_config: String = row.get(2)?;
    let meta: Option<String> = row.get(9)?;
    
    Ok(Provider {
        id: row.get(0)?,
        name: row.get(1)?,
        settings_config: serde_json::from_str(&settings_config).unwrap_or_default(),
        website_url: row.get(3)?,
        category: row.get(4)?,
        created_at: row.get(5)?,
        sort_index: row.get(6)?,
        notes: row.get(7)?,
        is_partner: row.get(8)?,
        meta: meta.and_then(|m: String| serde_json::from_str(&m).ok()),
        icon: row.get(10)?,
        icon_color: row.get(11)?,
        in_failover_queue: row.get(12)?,
    })
}

async fn list_providers(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<IndexMap<String, Provider>>> {
    let app = params.get("app").cloned().unwrap_or_default();
    
    let result: Result<IndexMap<String, Provider>, String> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT id, name, settings_config, website_url, category, created_at, sort_index, notes, is_partner, meta, icon, icon_color, in_failover_queue 
             FROM providers 
             WHERE app_type = ?1 
             ORDER BY sort_index ASC, created_at ASC"
        ).map_err(|e| e.to_string())?;
        
        let rows = stmt.query_map([&app], row_to_provider)
            .map_err(|e| e.to_string())?;
        
        let mut providers = IndexMap::new();
        for row in rows {
            if let Ok(provider) = row {
                providers.insert(provider.id.clone(), provider);
            }
        }
        
        Ok(providers)
    });
    
    match result {
        Ok(providers) => Json(ApiResponse::success(providers)),
        Err(e) => Json(ApiResponse::error(format!("Database error: {}", e))),
    }
}

async fn get_provider(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Option<Provider>>> {
    let result: Option<Provider> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT id, name, settings_config, website_url, category, created_at, sort_index, notes, is_partner, meta, icon, icon_color, in_failover_queue 
             FROM providers 
             WHERE id = ?1"
        ).ok()?;
        
        stmt.query_row([&id], row_to_provider).ok()
    });
    
    Json(ApiResponse::success(result))
}

async fn create_provider(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(provider): Json<Provider>,
) -> Json<ApiResponse<String>> {
    let result: Result<(), String> = state.with_db(|db: &Connection| {
        let is_partner_str: String = provider.is_partner.map(|b| if b { "1".to_string() } else { "0".to_string() }).unwrap_or_default();
        let in_failover_str: String = provider.in_failover_queue.map(|b| if b { "1".to_string() } else { "0".to_string() }).unwrap_or_default();
        
        db.execute(
            "INSERT INTO providers (id, name, settings_config, website_url, category, created_at, sort_index, notes, is_partner, meta, icon, icon_color, in_failover_queue, app_type) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            [
                &provider.id,
                &provider.name,
                &serde_json::to_string(&provider.settings_config).unwrap_or_default(),
                &provider.website_url.unwrap_or_default(),
                &provider.category.unwrap_or_default(),
                &provider.created_at.map(|t| t.to_string()).unwrap_or_default(),
                &provider.sort_index.map(|i| i.to_string()).unwrap_or_default(),
                &provider.notes.unwrap_or_default(),
                &is_partner_str,
                &serde_json::to_string(&provider.meta).unwrap_or_default(),
                &provider.icon.unwrap_or_default(),
                &provider.icon_color.unwrap_or_default(),
                &in_failover_str,
                &DEFAULT_APP_TYPE.to_string(),
            ]
        ).map_err(|e| e.to_string())?;
        
        Ok(())
    });
    
    match result {
        Ok(_) => {
            crate::handlers::ws::broadcast_event(
                &ws_state,
                "provider.created",
                json!({ "id": provider.id })
            );
            Json(ApiResponse::success(provider.id))
        }
        Err(e) => Json(ApiResponse::error(format!("Failed to create provider: {}", e))),
    }
}

async fn update_provider(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
    Json(provider): Json<Provider>,
) -> Json<ApiResponse<bool>> {
    let result: Result<(), String> = state.with_db(|db: &Connection| {
        let is_partner_str: String = provider.is_partner.map(|b| if b { "1".to_string() } else { "0".to_string() }).unwrap_or_default();
        let in_failover_str: String = provider.in_failover_queue.map(|b| if b { "1".to_string() } else { "0".to_string() }).unwrap_or_default();
        
        db.execute(
            "UPDATE providers SET 
                name = ?2, settings_config = ?3, website_url = ?4, category = ?5, 
                sort_index = ?6, notes = ?7, is_partner = ?8, meta = ?9, 
                icon = ?10, icon_color = ?11, in_failover_queue = ?12 
             WHERE id = ?1",
            [
                &id,
                &provider.name,
                &serde_json::to_string(&provider.settings_config).unwrap_or_default(),
                &provider.website_url.unwrap_or_default(),
                &provider.category.unwrap_or_default(),
                &provider.sort_index.map(|i| i.to_string()).unwrap_or_default(),
                &provider.notes.unwrap_or_default(),
                &is_partner_str,
                &serde_json::to_string(&provider.meta).unwrap_or_default(),
                &provider.icon.unwrap_or_default(),
                &provider.icon_color.unwrap_or_default(),
                &in_failover_str,
            ]
        ).map_err(|e| e.to_string())?;
        
        Ok(())
    });
    
    match result {
        Ok(_) => {
            crate::handlers::ws::broadcast_event(
                &ws_state,
                "provider.updated",
                json!({ "id": id })
            );
            Json(ApiResponse::success(true))
        }
        Err(e) => Json(ApiResponse::error(format!("Failed to update provider: {}", e))),
    }
}

async fn delete_provider(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<bool>> {
    let result: Result<(), String> = state.with_db(|db: &Connection| {
        db.execute("DELETE FROM providers WHERE id = ?1", [&id
        ]).map_err(|e| e.to_string())?;
        Ok(())
    });
    
    match result {
        Ok(_) => {
            crate::handlers::ws::broadcast_event(
                &ws_state,
                "provider.deleted",
                json!({ "id": id })
            );
            Json(ApiResponse::success(true))
        }
        Err(e) => Json(ApiResponse::error(format!("Failed to delete provider: {}", e))),
    }
}

async fn switch_provider(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<bool>> {
    let app = params.get("app").cloned().unwrap_or_else(|| DEFAULT_APP_TYPE.to_string());
    
    let result: Result<(), String> = state.with_db(|db: &Connection| {
        db.execute("BEGIN TRANSACTION", []).ok();
        
        let res: Result<usize, rusqlite::Error> = db.execute(
            "UPDATE providers SET is_current = 0 WHERE app_type = ?1",
            [&app
            ]
        ).and_then(|_| {
            db.execute(
                "UPDATE providers SET is_current = 1 WHERE id = ?1 AND app_type = ?2",
                [&id, &app
                ]
            )
        });
        
        match res {
            Ok(_) => {
                db.execute("COMMIT", []).ok();
                Ok(())
            }
            Err(e) => {
                db.execute("ROLLBACK", []).ok();
                Err(e.to_string())
            }
        }
    });
    
    match result {
        Ok(_) => {
            crate::handlers::ws::broadcast_event(
                &ws_state,
                "provider.switched",
                json!({ "id": id, "app": app })
            );
            Json(ApiResponse::success(true))
        }
        Err(e) => Json(ApiResponse::error(format!("Failed to switch provider: {}", e))),
    }
}

async fn get_current_provider(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Option<String>>> {
    let app = params.get("app").cloned().unwrap_or_default();
    
    let result: Option<String> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT id FROM providers WHERE app_type = ?1 AND is_current = 1"
        ).ok()?;
        
        stmt.query_row([&app
        ], |row: &rusqlite::Row| {
            row.get::<usize, String>(0)
        }).ok()
    });
    
    Json(ApiResponse::success(result))
}

async fn get_custom_endpoints(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Vec<crate::models::CustomEndpoint>>> {
    let result: Vec<crate::models::CustomEndpoint> = state.with_db(|db: &Connection| {
        let mut stmt = match db.prepare(
            "SELECT meta FROM providers WHERE id = ?1"
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        
        let meta_str: String = match stmt.query_row([&id
        ], |row: &rusqlite::Row| row.get(0)) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        
        let meta: Option<crate::models::ProviderMeta> = serde_json::from_str(&meta_str).ok();
        
        meta.and_then(|m| m.custom_endpoints).map(|endpoints| {
            endpoints.into_values().collect::<Vec<_>>()
        }).unwrap_or_default()
    });
    
    Json(ApiResponse::success(result))
}

async fn add_custom_endpoint(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(_id): Path<String>,
    Json(_endpoint): Json<crate::models::CustomEndpoint>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn remove_custom_endpoint(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path((_id, _url)): Path<(String, String)>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}

async fn update_sort_order(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(_updates): Json<Vec<serde_json::Value>>,
) -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(true))
}
