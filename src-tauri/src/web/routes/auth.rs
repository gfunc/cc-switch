use crate::web::{
    middleware::auth::{generate_token, revoke_token, validate_token},
    models::ApiResponse,
};
use axum::{
    extract::Request,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::env;

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
        .route("/logout", post(logout_route))
        .route("/token-reveal-enabled", get(token_reveal_enabled_route))
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
        Ok(_claims) => Json(ApiResponse::success(VerifyTokenResponse { valid: true })),
        Err(_) => Json(ApiResponse::success(VerifyTokenResponse { valid: false })),
    }
}

async fn login_deprecated(Json(_req): Json<LoginRequest>) -> Json<ApiResponse<serde_json::Value>> {
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
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to generate token: {}",
            e
        ))),
    }
}

async fn logout_route(request: Request) -> Json<ApiResponse<()>> {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Json(ApiResponse::error(
                "Missing or invalid authorization header".to_string(),
            ));
        }
    };

    match revoke_token(token) {
        Ok(_) => Json(ApiResponse::success(())),
        Err(_) => Json(ApiResponse::error("Invalid token".to_string())),
    }
}

async fn token_reveal_enabled_route() -> Json<ApiResponse<bool>> {
    Json(ApiResponse::success(token_reveal_enabled()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::middleware::auth::{generate_token, is_jti_revoked, validate_token};
    use serial_test::serial;

    #[test]
    #[serial]
    fn token_reveal_enabled_reflects_env() {
        assert_eq!(token_reveal_enabled(), false);
    }

    #[tokio::test]
    #[serial]
    async fn logout_route_revokes_token() {
        let token = generate_token("admin").unwrap();
        let jti = validate_token(&token).unwrap().jti;

        let request = axum::http::Request::builder()
            .uri("/auth/logout")
            .method("POST")
            .header("Authorization", format!("Bearer {}", token))
            .body(axum::body::Body::empty())
            .unwrap();

        let response = logout_route(axum::extract::Request::from(request)).await;
        assert!(response.0.success);
        assert!(is_jti_revoked(&jti));
    }
}
