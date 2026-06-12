use rusqlite::{Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    /// Absolute path to the shared SQLite file. Used to lazily build a full
    /// desktop [`crate::store::AppState`] for features that need the migrated
    /// schema, DAOs and services (skills, claude-desktop, etc.).
    db_path: String,
    /// Lazily-constructed desktop application state. Shares the same on-disk
    /// database file as `db` (a second connection), so advanced features reuse
    /// the exact desktop logic instead of being reimplemented here.
    desktop_state: Mutex<Option<Arc<crate::store::AppState>>>,
}

impl AppState {
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let mut conn = Connection::open(db_path)?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        Self::init_schema(&mut conn)?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            db_path: db_path.to_string(),
            desktop_state: Mutex::new(None),
        })
    }

    fn init_schema(conn: &mut Connection) -> SqliteResult<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                settings_config TEXT NOT NULL,
                website_url TEXT,
                category TEXT,
                created_at INTEGER,
                sort_index INTEGER,
                notes TEXT,
                is_partner BOOLEAN,
                meta TEXT,
                icon TEXT,
                icon_color TEXT,
                in_failover_queue BOOLEAN,
                app_type TEXT DEFAULT 'claude',
                is_current BOOLEAN DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            );

            CREATE TABLE IF NOT EXISTS mcp_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                server_config TEXT NOT NULL,
                description TEXT,
                homepage TEXT,
                docs TEXT,
                tags TEXT,
                enabled_claude BOOLEAN DEFAULT 0,
                enabled_codex BOOLEAN DEFAULT 0,
                enabled_gemini BOOLEAN DEFAULT 0,
                enabled_opencode BOOLEAN DEFAULT 0,
                enabled_openclaw BOOLEAN DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                is_active BOOLEAN DEFAULT 0,
                created_at INTEGER,
                updated_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                installed_at INTEGER,
                updated_at INTEGER,
                source TEXT,
                version TEXT
            );

            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                provider_id TEXT,
                title TEXT,
                summary TEXT,
                project_dir TEXT,
                created_at INTEGER,
                last_active_at INTEGER,
                source_path TEXT,
                resume_command TEXT
            );

            CREATE TABLE IF NOT EXISTS session_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                messages TEXT,
                FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS proxy_settings (
                id TEXT PRIMARY KEY,
                enabled BOOLEAN DEFAULT 0,
                config TEXT
            );
            "#,
        )?;

        Ok(())
    }

    pub fn with_db<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Connection) -> T,
    {
        let db = self.db.lock().unwrap();
        f(&db)
    }

    pub fn with_db_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut Connection) -> T,
    {
        let mut db = self.db.lock().unwrap();
        f(&mut db)
    }

    /// Lazily build (and cache) a full desktop [`crate::store::AppState`] backed
    /// by the same SQLite file as this web state.
    ///
    /// This opens a second connection and runs the desktop schema migrations via
    /// [`crate::database::Database::init_at_path`]. In the embedded Tauri web
    /// server the file is already migrated by the desktop app, so this is a
    /// no-op migration-wise; it simply unlocks the desktop DAOs and services for
    /// feature parity (skills, claude-desktop, live imports, etc.).
    pub fn desktop(&self) -> Result<Arc<crate::store::AppState>, String> {
        let mut guard = self
            .desktop_state
            .lock()
            .map_err(|e| format!("desktop state lock poisoned: {e}"))?;

        if let Some(existing) = guard.as_ref() {
            return Ok(existing.clone());
        }

        let db = crate::database::Database::init_at_path(std::path::Path::new(&self.db_path))
            .map_err(|e| e.to_string())?;
        let state = Arc::new(crate::store::AppState::new(Arc::new(db)));
        *guard = Some(state.clone());
        Ok(state)
    }
}
