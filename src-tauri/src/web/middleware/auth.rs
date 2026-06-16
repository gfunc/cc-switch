use axum::{
    body::Body,
    extract::Request,
    http::{header, Response, StatusCode},
    middleware::Next,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::sync::Mutex;
use std::sync::OnceLock;

const TOKEN_EXPIRATION_SECONDS: usize = 24 * 60 * 60;

/// Cached JWT secret — initialized once on first use.
static JWT_SECRET: OnceLock<String> = OnceLock::new();

/// In-memory blocklist for revoked token JTIs.
static REVOKED_JTIS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn revoked_jtis() -> &'static Mutex<HashSet<String>> {
    REVOKED_JTIS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Revoke a JTI so that any token bearing it is rejected.
pub fn revoke_jti(jti: String) {
    let mut set = revoked_jtis().lock().expect("revoked jti lock poisoned");
    set.insert(jti);
}

/// Check whether a JTI has been revoked.
pub fn is_jti_revoked(jti: &str) -> bool {
    let set = revoked_jtis().lock().expect("revoked jti lock poisoned");
    set.contains(jti)
}

/// Validate a token and revoke its JTI in one step.
pub fn revoke_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let claims = validate_token(token)?;
    revoke_jti(claims.jti.clone());
    Ok(claims)
}

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
    pub jti: String,
}

pub async fn auth_middleware(request: Request, next: Next) -> Response<Body> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(
                    json!({"error": "Missing or invalid authorization header"}).to_string(),
                ))
                .unwrap();
        }
    };

    let claims = match validate_token(token) {
        Ok(c) => c,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(json!({"error": "Invalid token"}).to_string()))
                .unwrap();
        }
    };

    if is_jti_revoked(&claims.jti) {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(json!({"error": "Token revoked"}).to_string()))
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
        jti: uuid::Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_contains_jti() {
        let token = generate_token("admin").expect("token generation failed");
        let claims = validate_token(&token).expect("token validation failed");
        assert!(!claims.jti.is_empty());
    }

    #[test]
    fn revoked_jti_is_rejected() {
        let token = generate_token("admin").expect("token generation failed");
        let jti = validate_token(&token).unwrap().jti;
        assert!(!is_jti_revoked(&jti));
        revoke_jti(jti.clone());
        assert!(is_jti_revoked(&jti));
    }
}
