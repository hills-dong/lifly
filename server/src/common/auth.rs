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

        // Extract the Bearer token from the Authorization header.
        let header_value = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing authorization header".to_string()))?;

        let token = header_value
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("invalid authorization scheme".to_string()))?;

        let claims = verify_token(token, &secret)?;

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
