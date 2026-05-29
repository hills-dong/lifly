use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::AppError;

// ── JWT claims ──────────────────────────────────────────────────────────────

/// Claims embedded in every JWT token issued by the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — the user's UUID.
    pub sub: Uuid,
    /// Issued-at (epoch seconds).
    pub iat: i64,
    /// Expiration (epoch seconds).
    pub exp: i64,
}

// ── Token helpers ───────────────────────────────────────────────────────────

/// Default token lifetime: 7 days.
const TOKEN_DURATION_DAYS: i64 = 7;

/// Create a signed JWT for the given `user_id`.
pub fn create_token(user_id: Uuid, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp: (now + Duration::days(TOKEN_DURATION_DAYS)).timestamp(),
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("failed to create token: {e}")))
}

/// Verify a JWT and return the embedded claims.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Unauthorized(format!("invalid token: {e}")))
}

// ── Axum extractor ──────────────────────────────────────────────────────────

/// Authenticated user extracted from the `Authorization: Bearer <token>` header.
///
/// Use this as a handler parameter to require authentication:
///
/// ```ignore
/// async fn me(auth: AuthUser) -> impl IntoResponse {
///     format!("user {}", auth.user_id)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Pull JWT_SECRET from request extensions (added by middleware / layer).
        let secret = parts
            .extensions
            .get::<JwtSecret>()
            .map(|s| s.0.clone())
            .ok_or_else(|| AppError::Internal("JWT secret not configured".to_string()))?;

        // Try Authorization header first, then fall back to ?token= query param.
        // The query param is needed for <img src="..."> tags which cannot set headers.
        let token = if let Some(header_value) = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            header_value
                .strip_prefix("Bearer ")
                .ok_or_else(|| {
                    AppError::Unauthorized("invalid authorization scheme".into())
                })?
                .to_string()
        } else {
            // Fall back to ?token= query parameter
            let query = parts.uri.query().unwrap_or("");
            query
                .split('&')
                .find_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    if key == "token" {
                        Some(value.to_string())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| AppError::Unauthorized("missing authorization".into()))?
        };

        let claims = verify_token(&token, &secret)?;

        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}

// ── Shared JWT secret type ──────────────────────────────────────────────────

/// New-type wrapper so we can store the JWT secret in Axum's request extensions.
///
/// Insert it via a middleware layer:
///
/// ```ignore
/// use axum::middleware;
///
/// async fn inject_jwt_secret(
///     mut req: axum::extract::Request,
///     next: middleware::Next,
/// ) -> axum::response::Response {
///     req.extensions_mut().insert(JwtSecret("my-secret".to_string()));
///     next.run(req).await
/// }
///
/// let app = Router::new()
///     .route("/me", get(me))
///     .layer(middleware::from_fn(inject_jwt_secret));
/// ```
#[derive(Debug, Clone)]
pub struct JwtSecret(pub String);

// ── Admin panel auth ──────────────────────────────────────────────────────────
//
// The operations admin panel uses config-based credentials that are completely
// independent of the `users` table. Its tokens are deliberately NOT interchangeable
// with user tokens: an `AdminClaims` carries `scope: "admin"` and a string `sub`
// (the admin username, not a user UUID). A user token (UUID `sub`, no `scope`) fails
// to decode as `AdminClaims`, and an admin token (`sub = "admin"`) fails to decode
// as user `Claims` because the UUID parse rejects it.

/// Scope marker embedded in every admin token.
const ADMIN_SCOPE: &str = "admin";

/// Claims embedded in admin-panel JWTs.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminClaims {
    /// Subject — the admin username (from config), not a user UUID.
    pub sub: String,
    /// Token scope; always `"admin"` for admin tokens.
    pub scope: String,
    /// Issued-at (epoch seconds).
    pub iat: i64,
    /// Expiration (epoch seconds).
    pub exp: i64,
}

/// Create a signed admin JWT for the given `username`.
pub fn create_admin_token(username: &str, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = AdminClaims {
        sub: username.to_string(),
        scope: ADMIN_SCOPE.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::days(TOKEN_DURATION_DAYS)).timestamp(),
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("failed to create admin token: {e}")))
}

/// Verify an admin JWT and return its claims, rejecting non-admin scopes.
pub fn verify_admin_token(token: &str, secret: &str) -> Result<AdminClaims, AppError> {
    let claims = jsonwebtoken::decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Unauthorized(format!("invalid admin token: {e}")))?;

    if claims.scope != ADMIN_SCOPE {
        return Err(AppError::Unauthorized("not an admin token".into()));
    }

    Ok(claims)
}

/// Authenticated admin extracted from the `Authorization: Bearer <token>` header.
///
/// Use this as a handler parameter to require admin privileges.
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub username: String,
}

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret = parts
            .extensions
            .get::<JwtSecret>()
            .map(|s| s.0.clone())
            .ok_or_else(|| AppError::Internal("JWT secret not configured".to_string()))?;

        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("missing admin authorization".into()))?;

        let claims = verify_admin_token(token, &secret)?;

        Ok(AdminUser {
            username: claims.sub,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-for-unit-tests";

    #[test]
    fn create_and_verify_token_roundtrip() {
        let user_id = Uuid::new_v4();
        let token = create_token(user_id, TEST_SECRET).expect("should create token");
        let claims = verify_token(&token, TEST_SECRET).expect("should verify token");

        assert_eq!(claims.sub, user_id);
        assert!(claims.exp > claims.iat);
        // Token should expire 7 days from now.
        let expected_duration = Duration::days(TOKEN_DURATION_DAYS).num_seconds();
        assert_eq!(claims.exp - claims.iat, expected_duration);
    }

    #[test]
    fn verify_token_with_wrong_secret_fails() {
        let user_id = Uuid::new_v4();
        let token = create_token(user_id, TEST_SECRET).expect("should create token");
        let result = verify_token(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn verify_invalid_token_string_fails() {
        let result = verify_token("not.a.valid.jwt.token", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn verify_empty_token_fails() {
        let result = verify_token("", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn expired_token_fails_verification() {
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        // Create a token that expired 1 hour ago.
        let claims = Claims {
            sub: user_id,
            iat: (now - Duration::hours(2)).timestamp(),
            exp: (now - Duration::hours(1)).timestamp(),
        };

        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .expect("should encode token");

        let result = verify_token(&token, TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn different_users_get_different_tokens() {
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let token1 = create_token(user1, TEST_SECRET).expect("token1");
        let token2 = create_token(user2, TEST_SECRET).expect("token2");
        assert_ne!(token1, token2);

        let claims1 = verify_token(&token1, TEST_SECRET).expect("verify1");
        let claims2 = verify_token(&token2, TEST_SECRET).expect("verify2");
        assert_eq!(claims1.sub, user1);
        assert_eq!(claims2.sub, user2);
    }

    #[test]
    fn admin_token_roundtrip_carries_scope() {
        let token = create_admin_token("admin", TEST_SECRET).expect("should create admin token");
        let claims = verify_admin_token(&token, TEST_SECRET).expect("should verify admin token");
        assert_eq!(claims.sub, "admin");
        assert_eq!(claims.scope, "admin");
    }

    #[test]
    fn user_token_is_rejected_as_admin() {
        // A normal user token (UUID sub, no scope) must not pass admin verification.
        let token = create_token(Uuid::new_v4(), TEST_SECRET).expect("user token");
        assert!(verify_admin_token(&token, TEST_SECRET).is_err());
    }

    #[test]
    fn admin_token_is_rejected_as_user() {
        // An admin token (string sub) must not pass user verification — the UUID
        // `sub` parse fails.
        let token = create_admin_token("admin", TEST_SECRET).expect("admin token");
        assert!(verify_token(&token, TEST_SECRET).is_err());
    }

    #[test]
    fn admin_token_with_wrong_secret_fails() {
        let token = create_admin_token("admin", TEST_SECRET).expect("admin token");
        assert!(verify_admin_token(&token, "wrong-secret").is_err());
    }
}
