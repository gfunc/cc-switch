use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode, header},
    middleware::Next,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::sync::OnceLock;

const TOKEN_EXPIRATION_SECONDS: usize = 24 * 60 * 60;

/// Cached JWT secret — initialized once on first use.
static JWT_SECRET: OnceLock<String> = OnceLock::new();

/// Get or generate the JWT secret.
/// Priority: JWT_SECRET env var > generate a random one (persisted in env for the process lifetime).
fn get_jwt_secret() -> &'static str {
    JWT_SECRET.get_or_init(|| {
        if let Ok(secret) = env::var("JWT_SECRET") {
            if !secret.is_empty() {
                return secret;
            }
        }
        // Generate a random secret for this process lifetime
        let secret = uuid::Uuid::new_v4().to_string();
        log::info!("Generated random JWT secret for this session");
        secret
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Response<Body> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(json!({"error": "Missing or invalid authorization header"}).to_string()))
                .unwrap();
        }
    };

    if validate_token(token).is_err() {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(json!({"error": "Invalid token"}).to_string()))
            .unwrap();
    }

    next.run(request).await
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
}

pub fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = get_jwt_secret();
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + TOKEN_EXPIRATION_SECONDS;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
