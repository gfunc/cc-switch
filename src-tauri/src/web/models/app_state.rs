use rusqlite::{Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

impl AppState {
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let mut conn = Connection::open(db_path)?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        Self::init_schema(&mut conn)?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
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
}
