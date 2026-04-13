use axum::{
    routing::{post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::env;
use crate::web::{
    models::ApiResponse,
    middleware::auth::{validate_token, generate_token},
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
        .route("/generate", post(generate_token_route))
        .route("/login", post(login_deprecated))
}

fn token_reveal_enabled() -> bool {
    env::var("CC_SWITCH_ENABLE_TOKEN_REVEAL")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
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

async fn generate_token_route() -> Json<ApiResponse<String>> {
    if !token_reveal_enabled() {
        return Json(ApiResponse::error(
            "Token reveal is disabled. Set CC_SWITCH_ENABLE_TOKEN_REVEAL=true to enable this endpoint.".to_string(),
        ));
    }

    match generate_token("admin") {
        Ok(token) => Json(ApiResponse::success(token)),
        Err(e) => Json(ApiResponse::error(format!("Failed to generate token: {}", e))),
    }
}
