use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{info, error};

mod handlers;
mod middleware;
mod models;
mod routes;

use handlers::ws::WsState;
use models::app_state::AppState;
use routes::{providers, settings, mcp, prompts, skills, sessions, proxy, auth};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "cc_switch_web=debug,tower_http=debug".into()),
        )
        .init();

    let db_path = std::env::var("CC_SWITCH_DB_PATH")
        .unwrap_or_else(|_| {
            let home = dirs::home_dir().expect("Failed to get home directory");
            home.join(".cc-switch/cc-switch.db")
                .to_str()
                .unwrap()
                .to_string()
        });

    let state = Arc::new(AppState::new(&db_path).expect("Failed to initialize app state"));
    
    let (tx, _rx) = broadcast::channel(100);
    let ws_state = Arc::new(WsState::new(tx));

    let app = create_router(state.clone(), ws_state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000u16);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    info!("CC Switch Web Server starting on http://{}", addr);
    info!("Database path: {}", db_path);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }
}

fn create_router(state: Arc<AppState>, ws_state: Arc<WsState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let shared_state = (state, ws_state);

    let protected_routes = Router::new()
        .nest("/providers", providers::routes())
        .nest("/settings", settings::routes())
        .nest("/mcp", mcp::routes())
        .nest("/prompts", prompts::routes())
        .nest("/skills", skills::routes())
        .nest("/sessions", sessions::routes())
        .nest("/proxy", proxy::routes())
        .route("/ws", get(handlers::ws::ws_handler))
        .layer(axum::middleware::from_fn(middleware::auth_middleware))
        .with_state(shared_state.clone());

    let api_routes = Router::new()
        .nest("/auth", auth::routes())
        .merge(protected_routes);

    Router::new()
        .nest("/api/v1", api_routes)
        .route("/health", get(health_check))
        .nest_service("/assets", ServeDir::new("dist/assets"))
        .fallback_service(ServeFile::new("dist/index.html"))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    })))
}
