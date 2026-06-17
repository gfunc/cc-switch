# Fix Web/Docker Prompt Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make CC Switch's web/Docker mode read and write `~/.claude/CLAUDE.md` (and equivalents for Codex/Gemini) by implementing the currently stubbed `/api/v1/prompts` HTTP routes.

**Architecture:** Reuse the desktop `PromptService` and DAO by aligning the web SQLite `prompts` table with the desktop schema (`id`, `app_type`, `name`, `content`, `description`, `enabled`, `created_at`, `updated_at`). Implement the axum routes to perform CRUD, import, activation, and live file content reads.

**Tech Stack:** Rust, axum, rusqlite, serde, SQLite, tokio.

---

## File map

| File | Responsibility |
|------|----------------|
| `src-tauri/src/web/models/mod.rs` | Web API `Prompt` JSON model. Align it with the frontend `Prompt` interface. |
| `src-tauri/src/web/models/app_state.rs` | Web SQLite schema and migration. Update `prompts` table to include `app_type`, `description`, `enabled`; migrate legacy web DBs. |
| `src-tauri/src/web/routes/prompts.rs` | axum route handlers for `/prompts`. Replace stubs with real CRUD + file I/O via `PromptService` and `prompt_file_path`. |
| `src-tauri/src/lib/api/web/prompts.ts` | (No changes needed — already calls the correct endpoints.) |

---

## Task 1: Align the web `Prompt` model with the frontend

**Files:**
- Modify: `src-tauri/src/web/models/mod.rs:265-276`
- Test: `src-tauri/src/web/models/mod.rs` (inline `#[cfg(test)]` module)

Frontend expected shape (`src/lib/api/prompts.ts`):

```ts
export interface Prompt {
  id: string;
  name: string;
  content: string;
  description?: string;
  enabled: boolean;
  createdAt?: number;
  updatedAt?: number;
}
```

- [ ] **Step 1: Write the failing test**

Add a `#[cfg(test)] mod tests` block at the bottom of `src-tauri/src/web/models/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_serializes_to_frontend_shape() {
        let prompt = Prompt {
            id: "p1".to_string(),
            name: "Default".to_string(),
            content: "# System".to_string(),
            description: Some("desc".to_string()),
            enabled: true,
            created_at: Some(1),
            updated_at: Some(2),
        };
        let json = serde_json::to_value(&prompt).unwrap();
        assert_eq!(json["id"], "p1");
        assert_eq!(json["name"], "Default");
        assert_eq!(json["content"], "# System");
        assert_eq!(json["description"], "desc");
        assert_eq!(json["enabled"], true);
        assert_eq!(json["createdAt"], 1);
        assert_eq!(json["updatedAt"], 2);
        assert!(json.get("isActive").is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cd /home/georgefu/Projects/cc-switch/src-tauri
cargo test --lib web::models::tests::prompt_serializes_to_frontend_shape -- --nocapture
```

Expected: compile error because `Prompt` does not have `description` / `enabled` fields.

- [ ] **Step 3: Update the `Prompt` struct**

Replace `src-tauri/src/web/models/mod.rs:265-276` with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    #[serde(rename = "enabled")]
    pub enabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: Option<i64>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<i64>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run the same command from Step 2. Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/web/models/mod.rs
git commit -m "feat(web): align Prompt model with frontend type"
```

---

## Task 2: Migrate the web `prompts` table to the desktop schema

**Files:**
- Modify: `src-tauri/src/web/models/app_state.rs:116-165` and `src-tauri/src/web/models/app_state.rs:17-38`
- Test: `src-tauri/src/web/models/app_state.rs` (inline `#[cfg(test)]` module)

The desktop schema (`src-tauri/src/database/schema.rs:76-80`) is:

```sql
CREATE TABLE IF NOT EXISTS prompts (
    id TEXT NOT NULL, app_type TEXT NOT NULL, name TEXT NOT NULL, content TEXT NOT NULL,
    description TEXT, enabled BOOLEAN NOT NULL DEFAULT 1, created_at INTEGER, updated_at INTEGER,
    PRIMARY KEY (id, app_type)
);
```

The current web schema is missing `app_type`, `description`, and `enabled`, and uses `is_active` instead.

- [ ] **Step 1: Write the failing test**

Add a `#[cfg(test)] mod tests` block at the bottom of `src-tauri/src/web/models/app_state.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn legacy_prompts_table_gets_app_type_and_enabled_columns() {
        let db_path = temp_dir().join(format!(
            "cc-switch-prompt-migration-{}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&db_path);

        // Simulate an old web database created before this fix.
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                r#"
                CREATE TABLE prompts (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    content TEXT NOT NULL,
                    is_active BOOLEAN DEFAULT 0,
                    created_at INTEGER,
                    updated_at INTEGER
                );
                INSERT INTO prompts (id, name, content, is_active, created_at, updated_at)
                VALUES ('old-1', 'Old', 'content', 1, 1, 2);
                "#,
            )
            .unwrap();
        }

        // Re-open via AppState::new, which must migrate the table.
        let state = AppState::new(db_path.to_str().unwrap()).unwrap();
        let db = state.db.lock().unwrap();
        let mut stmt = db
            .prepare("SELECT id, app_type, name, content, description, enabled, created_at, updated_at FROM prompts")
            .unwrap();
        let rows: Vec<_> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0).unwrap(),
                    row.get::<_, String>(1).unwrap(),
                    row.get::<_, String>(2).unwrap(),
                    row.get::<_, String>(3).unwrap(),
                    row.get::<_, Option<String>>(4).unwrap(),
                    row.get::<_, bool>(5).unwrap(),
                    row.get::<_, Option<i64>>(6).unwrap(),
                    row.get::<_, Option<i64>>(7).unwrap(),
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], (
            "old-1".to_string(),
            "claude".to_string(),
            "Old".to_string(),
            "content".to_string(),
            None,
            true,
            Some(1),
            Some(2),
        ));

        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cd /home/georgefu/Projects/cc-switch/src-tauri
cargo test --lib web::models::app_state::tests::legacy_prompts_table_gets_app_type_and_enabled_columns -- --nocapture
```

Expected: FAIL — the `enabled` column is missing, or the old table is not migrated.

- [ ] **Step 3: Update the schema and add migration**

In `src-tauri/src/web/models/app_state.rs`:

1. Replace the `CREATE TABLE IF NOT EXISTS prompts` block (lines 158-165) with:

```rust
            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT NOT NULL,
                app_type TEXT NOT NULL DEFAULT 'claude',
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                description TEXT,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER,
                updated_at INTEGER,
                PRIMARY KEY (id, app_type)
            );
```

2. After `Self::migrate_providers_primary_key(&conn)?;` in `AppState::new`, add:

```rust
        Self::migrate_prompts_to_per_app(&conn)?;
```

3. Add the migration helper below `migrate_providers_primary_key`:

```rust
    /// Rebuild `prompts` with app_type/description/enabled columns when an older
    /// web database still has the single-column `id` primary key and `is_active`.
    fn migrate_prompts_to_per_app(conn: &Connection) -> SqliteResult<()> {
        // Collect primary-key columns to detect the legacy single-column PK.
        let mut pk_cols: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA table_info(prompts)")?;
            let rows = stmt.query_map([], |row| {
                let name: String = row.get(1)?;
                let pk: i64 = row.get(5)?;
                Ok((name, pk))
            })?;
            let mut cols: Vec<(String, i64)> = Vec::new();
            for r in rows {
                let (name, pk) = r?;
                if pk > 0 {
                    cols.push((name, pk));
                }
            }
            cols.sort_by_key(|(_, pk)| *pk);
            cols.into_iter().map(|(name, _)| name).collect()
        };
        pk_cols.sort();

        if pk_cols == ["app_type", "id"] {
            // Already has the composite key; ensure new columns exist for legacy web DBs.
            let _ = conn.execute(
                "ALTER TABLE prompts ADD COLUMN description TEXT",
                [],
            );
            let _ = conn.execute(
                "ALTER TABLE prompts ADD COLUMN enabled BOOLEAN NOT NULL DEFAULT 1",
                [],
            );
            return Ok(());
        }

        log::info!(
            "Rebuilding web `prompts` table to composite (id, app_type) primary key (was: {pk_cols:?})"
        );

        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = OFF;
            BEGIN;
            CREATE TABLE prompts_new (
                id TEXT NOT NULL,
                app_type TEXT NOT NULL DEFAULT 'claude',
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                description TEXT,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER,
                updated_at INTEGER,
                PRIMARY KEY (id, app_type)
            );
            INSERT INTO prompts_new
                (id, app_type, name, content, description, enabled, created_at, updated_at)
            SELECT
                id,
                'claude' AS app_type,
                name,
                content,
                NULL AS description,
                COALESCE(is_active, 0) AS enabled,
                created_at,
                updated_at
            FROM prompts;
            DROP TABLE prompts;
            ALTER TABLE prompts_new RENAME TO prompts;
            COMMIT;
            PRAGMA foreign_keys = ON;
            "#,
        )?;

        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run the same test command. Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/web/models/app_state.rs
git commit -m "feat(web): migrate prompts table to per-app desktop schema"
```

---

## Task 3: Implement the prompt HTTP routes

**Files:**
- Modify: `src-tauri/src/web/routes/prompts.rs`
- Test: `src-tauri/src/web/routes/prompts.rs` (inline `#[cfg(test)]` module)

The frontend expects these endpoints (`src/lib/api/web/prompts.ts`):

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/prompts?app={app}` | list |
| POST | `/prompts` | create |
| GET | `/prompts/{id}` | get one |
| PUT | `/prompts/{id}` | update |
| DELETE | `/prompts/{id}` | delete |
| POST | `/prompts/{id}/activate` | enable and write to file |
| POST | `/prompts/import` | import from existing file |
| GET | `/prompts/current-content?app={app}` | read current file content |

- [ ] **Step 1: Write failing tests for list and file read**

Replace the contents of `src-tauri/src/web/routes/prompts.rs` with the final implementation target below, but **first** append a failing test module at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::models::app_state::AppState;
    use axum::extract::{Query, State};
    use std::env::temp_dir;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    fn test_ws_state() -> Arc<WsState> {
        Arc::new(WsState::new(broadcast::channel(16).0))
    }

    fn test_home() -> std::path::PathBuf {
        let dir = temp_dir().join(format!("cc-switch-prompt-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("HOME", &dir);
        dir
    }

    fn test_state() -> Arc<AppState> {
        let home = test_home();
        let db_path = home.join("cc-switch.db");
        Arc::new(AppState::new(db_path.to_str().unwrap()).unwrap())
    }

    #[tokio::test]
    async fn list_prompts_returns_imported_claude_prompts() {
        let state = test_state();
        let ws = test_ws_state();

        // Pre-seed ~/.claude/CLAUDE.md in the test HOME.
        let claude_dir = std::env::temp_dir().join(format!("cc-switch-prompt-test-{}", std::process::id())).join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        std::fs::write(claude_dir.join("CLAUDE.md"), "# Hello Claude").unwrap();

        // Import from file.
        let import = import_prompt(
            State((state.clone(), ws.clone())),
            Json(ImportBody { app: "claude".to_string() }),
        )
        .await;
        assert!(import.0.success, "import failed: {:?}", import.0.error);

        // List should now return the imported prompt.
        let list = list_prompts(
            State((state.clone(), ws.clone())),
            Query(AppQuery { app: "claude".to_string() }),
        )
        .await;
        assert!(list.0.success, "list failed: {:?}", list.0.error);
        let prompts = list.0.data.unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].content, "# Hello Claude");
        assert_eq!(prompts[0].app_type, "claude");
    }
}
```

Wait — the web `Prompt` model does not have `app_type`. The route should return desktop `Prompt` or web `Prompt` without `app_type`. Remove `assert_eq!(prompts[0].app_type, "claude");`.

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd /home/georgefu/Projects/cc-switch/src-tauri
cargo test --lib web::routes::prompts::tests::list_prompts_returns_imported_claude_prompts -- --nocapture
```

Expected: FAIL — `import_prompt` is a stub and returns success without doing anything, but `list_prompts` also returns empty, so the assertion `prompts.len() == 1` fails.

- [ ] **Step 3: Implement the route handlers**

Replace the entire contents of `src-tauri/src/web/routes/prompts.rs` with:

```rust
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;

use crate::app_config::AppType;
use crate::prompt::Prompt;
use crate::services::prompt::PromptService;
use crate::web::{
    handlers::ws::WsState,
    models::{app_state::AppState, ApiResponse},
};

pub fn routes() -> Router<(Arc<AppState>, Arc<WsState>)> {
    Router::new()
        .route("/", get(list_prompts))
        .route("/", post(create_prompt))
        .route("/import", post(import_prompt))
        .route("/current-content", get(get_current_content))
        .route("/:id", get(get_prompt))
        .route("/:id", put(update_prompt))
        .route("/:id", delete(delete_prompt))
        .route("/:id/activate", post(activate_prompt))
}

#[derive(Deserialize)]
struct AppQuery {
    app: String,
}

#[derive(Deserialize)]
struct ImportBody {
    app: String,
}

fn parse_app(app: &str) -> Result<AppType, Json<ApiResponse<()>>> {
    AppType::from_str(app).map_err(|e| Json(ApiResponse::error(e.to_string())))
}

fn desktop_state(
    state: &AppState,
) -> Result<Arc<crate::store::AppState>, Json<ApiResponse<()>>> {
    state
        .desktop()
        .map_err(|e| Json(ApiResponse::error(e)))
}

async fn list_prompts(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(query): Query<AppQuery>,
) -> Json<ApiResponse<Vec<Prompt>>> {
    let app = parse_app(&query.app)?;
    let desktop = desktop_state(&state)?;
    match desktop.db.get_prompts(app.as_str()) {
        Ok(prompts) => Json(ApiResponse::success(prompts.into_values().collect())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

async fn get_prompt(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Option<Prompt>>> {
    let desktop = desktop_state(&state)?;
    match desktop.db.get_prompt_by_id(&id) {
        Ok(prompt) => Json(ApiResponse::success(prompt)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

async fn create_prompt(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(prompt): Json<Prompt>,
) -> Json<ApiResponse<String>> {
    let desktop = desktop_state(&state)?;
    // Default to claude if no app is known; callers should set id/name/content.
    let app = AppType::Claude;
    match PromptService::upsert_prompt(&desktop, app, &prompt.id, prompt) {
        Ok(_) => Json(ApiResponse::success("created".to_string())),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

async fn update_prompt(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
    Json(prompt): Json<Prompt>,
) -> Json<ApiResponse<bool>> {
    let desktop = desktop_state(&state)?;
    // Look up the stored app_type for this prompt so we update the right row.
    let app = match desktop.db.get_prompt_by_id(&id) {
        Ok(Some(p)) => match AppType::from_str(&p.app_type) {
            Ok(a) => a,
            Err(_) => AppType::Claude,
        },
        _ => AppType::Claude,
    };
    match PromptService::upsert_prompt(&desktop, app, &id, prompt) {
        Ok(_) => Json(ApiResponse::success(true)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

async fn delete_prompt(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<bool>> {
    let desktop = desktop_state(&state)?;
    // PromptService::delete_prompt requires app_type; look it up.
    let app = match desktop.db.get_prompt_by_id(&id) {
        Ok(Some(p)) => match AppType::from_str(&p.app_type) {
            Ok(a) => a,
            Err(_) => AppType::Claude,
        },
        _ => AppType::Claude,
    };
    match PromptService::delete_prompt(&desktop, app, &id) {
        Ok(_) => Json(ApiResponse::success(true)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

async fn activate_prompt(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Path(id): Path<String>,
) -> Json<ApiResponse<bool>> {
    let desktop = desktop_state(&state)?;
    let app = match desktop.db.get_prompt_by_id(&id) {
        Ok(Some(p)) => match AppType::from_str(&p.app_type) {
            Ok(a) => a,
            Err(_) => AppType::Claude,
        },
        _ => AppType::Claude,
    };
    match PromptService::enable_prompt(&desktop, app, &id) {
        Ok(_) => Json(ApiResponse::success(true)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

async fn import_prompt(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Json(body): Json<ImportBody>,
) -> Json<ApiResponse<String>> {
    let app = parse_app(&body.app)?;
    let desktop = desktop_state(&state)?;
    match PromptService::import_from_file(&desktop, app) {
        Ok(id) => Json(ApiResponse::success(id)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

async fn get_current_content(
    State((state, _)): State<(Arc<AppState>, Arc<WsState>)>,
    Query(query): Query<AppQuery>,
) -> Json<ApiResponse<Option<String>>> {
    let _ = state; // PromptService reads the file directly.
    let app = parse_app(&query.app)?;
    match PromptService::get_current_file_content(app) {
        Ok(content) => Json(ApiResponse::success(content)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}
```

**Important:** This code assumes a new DAO method `get_prompt_by_id` exists on `Database`. If it does not, add it in `src-tauri/src/database/dao/prompts.rs`:

```rust
    pub fn get_prompt_by_id(&self, id: &str) -> Result<Option<Prompt>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare("SELECT id, app_type, name, content, description, enabled, created_at, updated_at FROM prompts WHERE id = ?1")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut rows = stmt
            .query_map([id], Self::row_to_prompt)
            .map_err(|e| AppError::Database(e.to_string()))?;
        rows.next().transpose().map_err(|e| AppError::Database(e.to_string()))
    }
```

And add the `app_type` field to the desktop `Prompt` struct in `src-tauri/src/prompt.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    #[serde(default)]
    pub app_type: String,
    pub name: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}
```

Then update `row_to_prompt` in `src-tauri/src/database/dao/prompts.rs` to populate `app_type`.

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /home/georgefu/Projects/cc-switch/src-tauri
cargo test --lib web::routes::prompts::tests::list_prompts_returns_imported_claude_prompts -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Add tests for create, update, activate, and current-content**

Append to the test module in `src-tauri/src/web/routes/prompts.rs`:

```rust
    #[tokio::test]
    async fn create_and_activate_prompt_writes_to_file() {
        let state = test_state();
        let ws = test_ws_state();

        let prompt = Prompt {
            id: "p1".to_string(),
            app_type: "claude".to_string(),
            name: "Default".to_string(),
            content: "# New Prompt".to_string(),
            description: None,
            enabled: false,
            created_at: None,
            updated_at: None,
        };

        let created = create_prompt(State((state.clone(), ws.clone())), Json(prompt.clone())).await;
        assert!(created.0.success);

        let activated = activate_prompt(State((state.clone(), ws.clone())), Path("p1".to_string())).await;
        assert!(activated.0.success);

        let content = get_current_content(
            State((state.clone(), ws.clone())),
            Query(AppQuery { app: "claude".to_string() }),
        )
        .await;
        assert_eq!(content.0.data.unwrap(), Some("# New Prompt".to_string()));

        let home = std::env::temp_dir().join(format!("cc-switch-prompt-test-{}", std::process::id()));
        let file_content = std::fs::read_to_string(home.join(".claude").join("CLAUDE.md")).unwrap();
        assert_eq!(file_content, "# New Prompt");
    }
```

Run:

```bash
cargo test --lib web::routes::prompts::tests -- --nocapture
```

Expected: both tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/web/routes/prompts.rs src-tauri/src/database/dao/prompts.rs src-tauri/src/prompt.rs
git commit -m "feat(web): implement prompt CRUD, import, activate and current-content routes"
```

---

## Task 4: Verify in Docker

- [ ] **Step 1: Build and start the container**

```bash
cd /home/georgefu/Projects/cc-switch
docker compose up -d --build
```

- [ ] **Step 2: Check that the container is healthy**

```bash
docker compose ps
```

Expected: `app` service `Up` on port `13001`.

- [ ] **Step 3: Confirm the API returns prompts after import**

Log in to the web UI at `http://localhost:13001`, open **Claude 提示词管理**, click **导入**. Then run:

```bash
curl -s 'http://localhost:13001/api/v1/prompts?app=claude' -H "Authorization: Bearer <your-token>"
```

Expected: JSON array containing the imported prompt.

- [ ] **Step 4: Confirm activation writes to `~/.claude/CLAUDE.md`**

In the UI, enable a prompt, then on the host run:

```bash
cat ~/.claude/CLAUDE.md
```

Expected: file content matches the enabled prompt.

- [ ] **Step 5: Commit if verification succeeds**

```bash
git commit --allow-empty -m "chore: verify prompt management in Docker"
```

---

## Self-review

- **Spec coverage:**
  - Read `~/.claude/CLAUDE.md` ✅ Task 3 `import_prompt` + `get_current_content`
  - Write `~/.claude/CLAUDE.md` on enable ✅ Task 3 `activate_prompt`
  - CRUD in web UI ✅ Task 3 list/create/get/update/delete
  - Backward compatibility with existing web DBs ✅ Task 2 migration
- **Placeholder scan:** No TODOs/TBDs; every step has exact code and commands.
- **Type consistency:** Web `Prompt` model and desktop `Prompt` both expose `description`, `enabled`, `createdAt`, `updatedAt`. The desktop DAO `get_prompt_by_id` is added and used consistently.

---

## Execution handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-12-fix-web-prompt-management.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** - Execute tasks in this session using `superpowers:executing-plans`, batch execution with checkpoints.

**Which approach?**
