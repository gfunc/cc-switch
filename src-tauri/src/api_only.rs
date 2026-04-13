//! API-only mode: runs the embedded web server without any Tauri/GTK initialization.
//! Activated at build time via `--features api-only` (used in Docker).
//! No display server (Xvfb) required.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use rusqlite::params;

use crate::web::{create_router, handlers::ws::WsState, models::app_state::AppState};

const FIRST_TOKEN_PRINTED_KEY: &str = "web_admin_token_printed";

fn print_first_login_token_once(state: &AppState) {
    let already_printed = state.with_db(|db| {
        db.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [FIRST_TOKEN_PRINTED_KEY],
            |row| row.get::<usize, String>(0),
        )
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false)
    });

    if already_printed {
        return;
    }

    match crate::web::middleware::auth::generate_token("admin") {
        Ok(token) => {
            // Print once on first startup so browser users can sign in immediately.
            println!("[cc-switch] First-time web login token (save this now): {token}");
            println!("[cc-switch] This token will not be auto-printed again.");

            state.with_db_mut(|db| {
                let _ = db.execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![FIRST_TOKEN_PRINTED_KEY, "1"],
                );
            });
        }
        Err(err) => {
            log::error!("Failed to generate first-time web login token: {}", err);
        }
    }
}

pub fn run() -> ! {
    // Initialize logging from RUST_LOG env var
    env_logger::init();

    let port: u16 = std::env::var("CC_SWITCH_WEB_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let bind_all = std::env::var("CC_SWITCH_WEB_BIND_ALL")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let bind_addr: [u8; 4] = if bind_all { [0, 0, 0, 0] } else { [127, 0, 0, 1] };
    let addr = SocketAddr::from((bind_addr, port));

    // Resolve database path: env override or default app config dir
    let db_path = std::env::var("CC_SWITCH_DB_PATH").unwrap_or_else(|_| {
        let dir = crate::config::get_app_config_dir();
        std::fs::create_dir_all(&dir).expect("Failed to create app config directory");
        dir.join("cc-switch.db")
            .to_str()
            .expect("db path is not valid UTF-8")
            .to_owned()
    });

    log::info!("=== cc-switch API-only mode ===");
    log::info!("Listening on http://{}", addr);
    log::info!("Database: {}", db_path);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async move {
        let web_state = Arc::new(
            AppState::new(&db_path).expect("Failed to initialize database"),
        );

        print_first_login_token_once(&web_state);

        let (tx, _rx) = broadcast::channel(100);
        let ws_state = Arc::new(WsState::new(tx));

        let router = create_router(web_state, ws_state);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

        log::info!("Web server ready");

        axum::serve(listener, router)
            .await
            .expect("Web server exited with error");
    });

    std::process::exit(0);
}
