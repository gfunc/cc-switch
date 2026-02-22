use rusqlite::{Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

impl AppState {
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;"
        )?;
        
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
        })
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
