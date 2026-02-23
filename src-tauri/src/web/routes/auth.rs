use axum::{
    routing::{post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::web::{
    models::ApiResponse,
    middleware::auth::validate_token,
};

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyTokenResponse {
    pub valid: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    #[allow(dead_code)]
    pub username: String,
    #[allow(dead_code)]
    pub password: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/verify", post(verify_token))
        .route("/login", post(login_deprecated))
}

async fn verify_token(
    Json(req): Json<VerifyTokenRequest>,
) -> Json<ApiResponse<VerifyTokenResponse>> {
    if req.token.is_empty() {
        return Json(ApiResponse::error("Token is required".to_string()));
    }

    match validate_token(&req.token) {
        Ok(_claims) => {
            Json(ApiResponse::success(VerifyTokenResponse { valid: true }))
        }
        Err(_) => {
            Json(ApiResponse::success(VerifyTokenResponse { valid: false }))
        }
    }
}

async fn login_deprecated(
    Json(_req): Json<LoginRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::error(
        "Password login is no longer supported. Please generate a token using the CLI: cc-switch-web generate-token".to_string()
    ))
}
