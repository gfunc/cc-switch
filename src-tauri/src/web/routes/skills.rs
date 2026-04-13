use axum::{
    extract::{State, Path, Query},
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
        Skill,
        ApiResponse,
    },
    handlers::ws::WsState,
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/", get(list_skills))
        .route("/installed", get(get_installed_skills))
        .route("/discover", get(discover_skills))
        .route("/unmanaged", get(scan_unmanaged_skills))
        .route("/repos", get(get_skill_repos))
        .route("/repos", post(add_skill_repo))
    .route("/repos/:owner/:name", delete(remove_skill_repo))
    .route("/:id/install", post(install_skill))
    .route("/:id/uninstall", delete(uninstall_skill))
    .route("/:id/toggle", post(toggle_skill_app))
        .route("/import", post(import_skills))
}

async fn list_skills(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<Skill>>> {
    let app = params.get("app").cloned().unwrap_or_else(|| "claude".to_string());
    
    let result: Vec<Skill> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT id, name, description, installed_at, updated_at, source, version 
             FROM skills 
             WHERE app_type = ?1 OR app_type = 'all'
             ORDER BY installed_at DESC"
        ).ok()?;
        
        let rows = stmt.query_map([&app], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                installed_at: row.get(3)?,
                updated_at: row.get(4)?,
                source: row.get(5)?,
                version: row.get(6)?,
            })
        }).ok()?;
        
        let skills: Vec<Skill> = rows.filter_map(|r| r.ok()).collect();
        Some(skills)
    }).unwrap_or_default();
    
    Json(ApiResponse::success(result))
}

async fn get_installed_skills(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<Skill>>> {
    let result: Vec<Skill> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT id, name, description, installed_at, updated_at, source, version 
             FROM skills 
             ORDER BY installed_at DESC"
        ).ok()?;
        
        let rows = stmt.query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                installed_at: row.get(3)?,
                updated_at: row.get(4)?,
                source: row.get(5)?,
                version: row.get(6)?,
            })
        }).ok()?;
        
        let skills: Vec<Skill> = rows.filter_map(|r| r.ok()).collect();
        Some(skills)
    }).unwrap_or_default();
    
    Json(ApiResponse::success(result))
}

async fn discover_skills(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<Skill>>> {
    // Web-server cannot directly access GitHub repos
    // This would need to be populated via configuration or client-side discovery
    Json(ApiResponse::success(vec![]))
}

async fn scan_unmanaged_skills(
    State((_state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    // Web-server cannot access local filesystem
    // This would need to be done client-side
    Json(ApiResponse::success(vec![]))
}

async fn get_skill_repos(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let result: Vec<serde_json::Value> = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT value FROM settings WHERE key = 'skill_repos'"
        ).ok()?;
        
        let repos_str: String = stmt.query_row([], |row| row.get(0)).ok()?;
        serde_json::from_str(&repos_str).ok()
    }).unwrap_or_default();
    
    Json(ApiResponse::success(result))
}

async fn add_skill_repo(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(repo): Json<serde_json::Value>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        // Get existing repos
        let mut stmt = db.prepare(
            "SELECT value FROM settings WHERE key = 'skill_repos'"
        ).ok()?;
        
        let repos_str: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
        let mut repos: Vec<serde_json::Value> = repos_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        // Add new repo
        repos.push(repo);
        
        // Save back
        let repos_str = serde_json::to_string(&repos).ok()?;
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('skill_repos', ?1)",
            [&repos_str],
        ).ok()?;
        
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "skill.repo_added",
            json!({}),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn remove_skill_repo(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path((owner, name)): Path<(String, String)>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        let mut stmt = db.prepare(
            "SELECT value FROM settings WHERE key = 'skill_repos'"
        ).ok()?;
        
        let repos_str: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
        let repos: Vec<serde_json::Value> = repos_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        // Filter out the repo to remove
        let filtered: Vec<serde_json::Value> = repos.into_iter()
            .filter(|r| {
                let repo_owner = r.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo_name = r.get("name").and_then(|v| v.as_str()).unwrap_or("");
                !(repo_owner == owner && repo_name == name)
            })
            .collect();
        
        let repos_str = serde_json::to_string(&filtered).ok()?;
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('skill_repos', ?1)",
            [&repos_str],
        ).ok()?;
        
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "skill.repo_removed",
            json!({ "owner": owner, "name": name }),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn install_skill(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
    Json(skill): Json<Skill>,
) -> Json<ApiResponse<bool>> {
    let now = chrono::Utc::now().timestamp();
    
    let result = state.with_db(|db: &Connection| {
        db.execute(
            "INSERT OR REPLACE INTO skills (id, name, description, installed_at, updated_at, source, version) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                &id,
                &skill.name,
                &skill.description,
                skill.installed_at.or(Some(now)),
                Some(now),
                &skill.source,
                &skill.version,
            ],
        ).map_err(|e| e.to_string()).ok()?;
        
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "skill.installed",
            json!({ "id": id }),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn uninstall_skill(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<bool>> {
    let result = state.with_db(|db: &Connection| {
        db.execute(
            "DELETE FROM skills WHERE id = ?1",
            [&id],
        ).map_err(|e| e.to_string()).ok()?;
        
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "skill.uninstalled",
            json!({ "id": id }),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn toggle_skill_app(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Json<ApiResponse<bool>> {
    let app = payload.get("app").and_then(|v| v.as_str()).unwrap_or("");
    let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    
    // Store enabled state in a separate table or as JSON in settings
    let result = state.with_db(|db: &Connection| {
        // For simplicity, store in settings table as JSON
        let key = format!("skill_{}_apps", id);
        
        let mut stmt = db.prepare(
            "SELECT value FROM settings WHERE key = ?1"
        ).ok()?;
        
        let apps_str: Option<String> = stmt.query_row([&key], |row| row.get(0)).ok();
        let mut apps: serde_json::Value = apps_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}));
        
        // Update the app's enabled state
        if let Some(obj) = apps.as_object_mut() {
            obj.insert(app.to_string(), json!(enabled));
        }
        
        let apps_str = serde_json::to_string(&apps).ok()?;
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [&key, &apps_str],
        ).ok()?;
        
        Some(true)
    }).unwrap_or(false);
    
    if result {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "skill.toggled",
            json!({ "id": id, "app": app, "enabled": enabled }),
        );
    }
    
    Json(ApiResponse::success(result))
}

async fn import_skills(
    State((state, ws_state)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(payload): Json<serde_json::Value>,
) -> Json<ApiResponse<Vec<Skill>>> {
    let directories = payload.get("directories")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
        .unwrap_or_default();
    
    let now = chrono::Utc::now().timestamp();
    let mut imported = vec![];
    
    for dir in directories {
        let skill = Skill {
            id: dir.clone(),
            name: dir.split('/').last().unwrap_or(&dir).to_string(),
            description: None,
            installed_at: Some(now),
            updated_at: Some(now),
            source: Some("imported".to_string()),
            version: None,
        };
        
        let result = state.with_db(|db: &Connection| {
            db.execute(
                "INSERT OR REPLACE INTO skills (id, name, description, installed_at, updated_at, source, version) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    &skill.id,
                    &skill.name,
                    &skill.description,
                    skill.installed_at,
                    skill.updated_at,
                    &skill.source,
                    &skill.version,
                ],
            ).ok()?;
            
            Some(true)
        }).unwrap_or(false);
        
        if result {
            imported.push(skill);
        }
    }
    
    if !imported.is_empty() {
        crate::web::handlers::ws::broadcast_event(
            &ws_state,
            "skill.imported",
            json!({ "count": imported.len() }),
        );
    }
    
    Json(ApiResponse::success(imported))
}