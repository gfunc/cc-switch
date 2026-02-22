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

const TOKEN_EXPIRATION_SECONDS: usize = 24 * 60 * 60;

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

fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
}

pub fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());
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
