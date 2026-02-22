use axum::{
    routing::post,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::{
    models::ApiResponse,
    middleware::auth::generate_token,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login))
}

async fn login(
    Json(req): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    let expected_password = std::env::var("CC_SWITCH_PASSWORD")
        .unwrap_or_else(|_| "admin".to_string());
    
    if req.password != expected_password {
        return Json(ApiResponse::error("Invalid credentials".to_string()));
    }
    
    match generate_token(&req.username) {
        Ok(token) => {
            let response = LoginResponse {
                token,
                user: UserInfo {
                    id: req.username.clone(),
                    username: req.username,
                },
            };
            Json(ApiResponse::success(response))
        }
        Err(_) => Json(ApiResponse::error("Failed to generate token".to_string())),
    }
}
