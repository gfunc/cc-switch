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
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

const TOKEN_EXPIRATION_SECONDS: usize = 24 * 60 * 60;

const AUTH_TOKEN_FILE: &str = "auth_token";

/// Cached AUTH_TOKEN — initialized lazily and resettable by `rotate_auth_token`.
static AUTH_TOKEN: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn auth_token_cache() -> &'static Mutex<Option<String>> {
    AUTH_TOKEN.get_or_init(|| Mutex::new(None))
}

fn load_auth_token_from_sources() -> String {
    if let Ok(token) = env::var("AUTH_TOKEN") {
        if !token.is_empty() {
            log::info!("Using AUTH_TOKEN from environment");
            return token;
        }
    }
    let path = auth_token_path();
    if let Ok(token) = fs::read_to_string(&path) {
        let token = token.trim().to_string();
        if !token.is_empty() {
            log::info!("Loaded AUTH_TOKEN from {}", path.display());
            return token;
        }
    }
    let token = uuid::Uuid::new_v4().to_string() + &uuid::Uuid::new_v4().to_string();
    if let Err(e) = fs::write(&path, &token) {
        log::error!("Failed to persist AUTH_TOKEN to {}: {}", path.display(), e);
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
    }
    log::info!(
        "Generated new AUTH_TOKEN and persisted to {}. New token: {}",
        path.display(),
        token
    );
    token
}

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

fn auth_token_path() -> PathBuf {
    let mut dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cc-switch");
    let _ = fs::create_dir_all(&dir);
    dir.push(AUTH_TOKEN_FILE);
    dir
}

/// Get or load the AUTH_TOKEN.
/// Priority: AUTH_TOKEN env var > file at `auth_token_path()` > generate and persist a new one.
pub fn get_auth_token() -> String {
    let mut guard = auth_token_cache()
        .lock()
        .expect("AUTH_TOKEN cache lock poisoned");
    if let Some(token) = guard.as_ref() {
        return token.clone();
    }
    let token = load_auth_token_from_sources();
    *guard = Some(token.clone());
    token
}

/// Rotate the AUTH_TOKEN: generate a new random value, persist to disk, and
/// update the in-memory cache so existing JWTs signed with the previous secret
/// immediately fail validation.
pub fn rotate_auth_token() -> String {
    let new_token = uuid::Uuid::new_v4().to_string() + &uuid::Uuid::new_v4().to_string();
    let path = auth_token_path();
    if let Err(e) = fs::write(&path, &new_token) {
        log::error!("Failed to persist rotated AUTH_TOKEN: {}", e);
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
    }
    let mut guard = auth_token_cache()
        .lock()
        .expect("AUTH_TOKEN cache lock poisoned");
    *guard = Some(new_token.clone());
    log::info!("Rotated AUTH_TOKEN. New token: {}", new_token);
    new_token
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
    let secret = get_auth_token();
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
}

pub fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = get_auth_token();
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
    use serial_test::serial;
    use std::env;

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

    #[test]
    #[serial]
    fn auth_token_loaded_from_env() {
        unsafe { env::set_var("AUTH_TOKEN", "test-secret-from-env") };
        let token = get_auth_token();
        assert_eq!(token, "test-secret-from-env");
        unsafe { env::remove_var("AUTH_TOKEN") };
    }

    #[test]
    #[serial]
    fn auth_token_returns_non_empty_string() {
        unsafe { env::remove_var("AUTH_TOKEN") };
        let token = get_auth_token();
        assert!(!token.is_empty());
    }

    #[test]
    #[serial]
    fn rotate_auth_token_invalidates_old_tokens() {
        unsafe { env::set_var("AUTH_TOKEN", "initial-rotation-secret") };
        // Prime the cache.
        let _ = get_auth_token();

        let old_token = generate_token("admin").expect("token generation failed");
        assert!(validate_token(&old_token).is_ok());

        let new_token = rotate_auth_token();
        assert_ne!(new_token, "initial-rotation-secret");

        // Old JWT now fails signature validation.
        assert!(validate_token(&old_token).is_err());

        unsafe { env::remove_var("AUTH_TOKEN") };
    }
}
