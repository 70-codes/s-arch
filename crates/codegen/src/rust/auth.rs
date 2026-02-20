//! # Auth Generator (JWT)
//!
//! Generates JWT authentication code for the generated project when
//! authentication is enabled in the project configuration.
//!
//! ## Generated Files
//!
//! - `src/auth/mod.rs` — module declarations and re-exports
//! - `src/auth/jwt.rs` — `Claims` struct, `create_token`, `verify_token`
//! - `src/auth/middleware.rs` — Axum `require_auth` middleware and role checks
//!
//! ## Usage
//!
//! The generated auth module is only produced when `ctx.auth_enabled()` is
//! `true`. The router generator (`routes.rs`) will wrap secured routes with
//! the `require_auth` middleware layer.
//!
//! ## Token Flow
//!
//! ```text
//! Client → Authorization: Bearer <token> → require_auth middleware
//!          → verify_token(token, secret) → Claims inserted into request extensions
//!          → Handler reads Claims from extensions
//! ```

use imortal_ir::AuthStrategy;

use crate::context::GenerationContext;
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate all authentication files (`src/auth/mod.rs`, `src/auth/jwt.rs`,
/// `src/auth/middleware.rs`).
///
/// Returns an empty `Vec` if authentication is not enabled.
pub fn generate_auth(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    if !ctx.auth_enabled() {
        return Vec::new();
    }

    vec![
        generate_auth_mod(ctx),
        generate_jwt(ctx),
        generate_auth_middleware(ctx),
    ]
}

// ============================================================================
// auth/mod.rs
// ============================================================================

fn generate_auth_mod(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(512);

    content.push_str(&file_header("Authentication module."));

    content.push_str("pub mod jwt;\n");
    content.push_str("pub mod middleware;\n\n");

    content.push_str("// Re-exports for convenience\n");
    content.push_str("pub use jwt::{Claims, create_token, verify_token};\n");
    content.push_str("pub use middleware::require_auth;\n");

    GeneratedFile::new("src/auth/mod.rs", content, FileType::Rust)
}

// ============================================================================
// auth/jwt.rs — Claims, create_token, verify_token
// ============================================================================

fn generate_jwt(ctx: &GenerationContext) -> GeneratedFile {
    let expiry_hours = ctx.auth_config().token_expiry_hours;
    let secret_env = &ctx.auth_config().jwt_secret_env;

    let mut content = String::with_capacity(4096);

    content.push_str(&file_header("JWT token creation and verification."));

    // ── Imports ──────────────────────────────────────────────────────────
    content.push_str(
        "\
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

",
    );

    // ── Claims ───────────────────────────────────────────────────────────
    content.push_str(
        "\
/// JWT claims payload.
///
/// This struct is encoded into (and decoded from) every JWT issued by the
/// application. It is inserted into Axum request extensions by the
/// `require_auth` middleware so that handlers can access the authenticated
/// user's identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — typically the user's primary key (UUID or ID).
    pub sub: String,

    /// User email address.
    pub email: String,

    /// Roles assigned to the user (e.g. `[\"admin\", \"editor\"]`).
    pub roles: Vec<String>,

    /// Expiration time (seconds since UNIX epoch).
    pub exp: u64,

    /// Issued-at time (seconds since UNIX epoch).
    pub iat: u64,
}

impl Claims {
    /// Create a new set of claims.
    ///
    /// # Arguments
    ///
    /// * `user_id`      — unique identifier for the user (UUID string)
    /// * `email`        — user's email address
    /// * `roles`        — list of role strings
",
    );

    content.push_str(&format!(
        "    /// * `expiry_hours` — token lifetime in hours (default: {})\n",
        expiry_hours,
    ));

    content.push_str(
        "\
    pub fn new(
        user_id: impl Into<String>,
        email: impl Into<String>,
        roles: Vec<String>,
        expiry_hours: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect(\"system clock before UNIX epoch\")
            .as_secs();

        Self {
            sub: user_id.into(),
            email: email.into(),
            roles,
            iat: now,
            exp: now + (expiry_hours * 3600),
        }
    }

",
    );

    content.push_str(&format!(
        "\
    /// Create claims with the default expiry ({expiry_hours} hours).
    pub fn with_default_expiry(
        user_id: impl Into<String>,
        email: impl Into<String>,
        roles: Vec<String>,
    ) -> Self {{
        Self::new(user_id, email, roles, {expiry_hours})
    }}

    /// Check whether this token has a specific role.
    pub fn has_role(&self, role: &str) -> bool {{
        self.roles.iter().any(|r| r == role)
    }}

    /// Check whether this token has *any* of the given roles.
    pub fn has_any_role(&self, roles: &[&str]) -> bool {{
        roles.iter().any(|role| self.has_role(role))
    }}

    /// Check whether the token has expired (based on current system time).
    pub fn is_expired(&self) -> bool {{
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= self.exp
    }}
}}

",
    ));

    // ── create_token ─────────────────────────────────────────────────────
    content.push_str(
        "\
/// Encode a `Claims` value into a signed JWT string.
///
/// # Arguments
///
/// * `claims` — the claims to encode
/// * `secret` — the HMAC-SHA256 secret key (from environment)
///
/// # Errors
///
/// Returns a `jsonwebtoken::errors::Error` if encoding fails.
pub fn create_token(
    claims: &Claims,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

",
    );

    // ── verify_token ─────────────────────────────────────────────────────
    content.push_str(
        "\
/// Decode and verify a JWT string, returning the contained `Claims`.
///
/// # Arguments
///
/// * `token`  — the raw JWT string (without the \"Bearer \" prefix)
/// * `secret` — the HMAC-SHA256 secret key (must match the one used to sign)
///
/// # Errors
///
/// Returns a `jsonwebtoken::errors::Error` if:
/// - The token is malformed
/// - The signature is invalid
/// - The token has expired
pub fn verify_token(
    token: &str,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

",
    );

    // ── Helper: hash / verify password ───────────────────────────────────
    content.push_str(
        "\
// ============================================================================
// Password Hashing Utilities
// ============================================================================

/// Hash a plain-text password using bcrypt.
///
/// # Errors
///
/// Returns an error if bcrypt hashing fails.
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

/// Verify a plain-text password against a bcrypt hash.
///
/// Returns `true` if the password matches the hash.
pub fn verify_password(
    password: &str,
    hash: &str,
) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}

",
    );

    // ── Tests ────────────────────────────────────────────────────────────
    content.push_str(
        "\
#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = \"test-secret-key-at-least-32-chars!!\";

    #[test]
    fn test_create_and_verify_token() {
        let claims = Claims::new(\"user-123\", \"test@example.com\", vec![\"admin\".into()], 1);
        let token = create_token(&claims, TEST_SECRET).expect(\"should create token\");

        let decoded = verify_token(&token, TEST_SECRET).expect(\"should verify token\");
        assert_eq!(decoded.sub, \"user-123\");
        assert_eq!(decoded.email, \"test@example.com\");
        assert_eq!(decoded.roles, vec![\"admin\"]);
    }

    #[test]
    fn test_invalid_secret_fails() {
        let claims = Claims::new(\"user-1\", \"a@b.com\", vec![], 1);
        let token = create_token(&claims, TEST_SECRET).unwrap();

        let result = verify_token(&token, \"wrong-secret\");
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_has_role() {
        let claims = Claims::new(\"u\", \"e\", vec![\"admin\".into(), \"editor\".into()], 1);
        assert!(claims.has_role(\"admin\"));
        assert!(claims.has_role(\"editor\"));
        assert!(!claims.has_role(\"viewer\"));
    }

    #[test]
    fn test_claims_has_any_role() {
        let claims = Claims::new(\"u\", \"e\", vec![\"editor\".into()], 1);
        assert!(claims.has_any_role(&[\"admin\", \"editor\"]));
        assert!(!claims.has_any_role(&[\"admin\", \"superuser\"]));
    }

    #[test]
    fn test_claims_with_default_expiry() {
        let claims = Claims::with_default_expiry(\"u\", \"e\", vec![]);
        assert!(!claims.is_expired());
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_claims_is_expired() {
        let mut claims = Claims::new(\"u\", \"e\", vec![], 1);
        // Force expiration to the past
        claims.exp = claims.iat.saturating_sub(1);
        assert!(claims.is_expired());
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = \"my_secure_password\";
        let hash = hash_password(password).expect(\"should hash\");

        assert!(verify_password(password, &hash).expect(\"should verify\"));
        assert!(!verify_password(\"wrong_password\", &hash).expect(\"should verify\"));
    }
}
",
    );

    GeneratedFile::new("src/auth/jwt.rs", content, FileType::Rust)
}

// ============================================================================
// auth/middleware.rs — require_auth, require_roles
// ============================================================================

fn generate_auth_middleware(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(4096);

    content.push_str(&file_header("Authentication middleware for Axum routes."));

    // ── Imports ──────────────────────────────────────────────────────────
    content.push_str(
        "\
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde_json::json;

use crate::auth::jwt::{verify_token, Claims};
use crate::state::AppState;

",
    );

    // ── require_auth ─────────────────────────────────────────────────────
    content.push_str(
        "\
/// Middleware that requires a valid JWT in the `Authorization: Bearer <token>`
/// header.
///
/// On success the decoded [`Claims`] are inserted into the request's
/// extensions map so downstream handlers can access them via:
///
/// ```rust,ignore
/// let claims = request.extensions().get::<Claims>().unwrap();
/// ```
///
/// On failure a `401 Unauthorized` JSON response is returned.
pub async fn require_auth(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let token = auth.token();

    let claims = verify_token(token, &state.config.jwt_secret)
        .map_err(|e| {
            tracing::warn!(\"JWT verification failed: {}\", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    \"error\": \"unauthorized\",
                    \"message\": \"Invalid or expired authentication token\"
                })),
            )
                .into_response()
        })?;

    // Make claims available to handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

",
    );

    // ── require_roles ────────────────────────────────────────────────────
    content.push_str(
        "\
/// Extract the authenticated user's [`Claims`] from a request's extensions.
///
/// This is a convenience function for handlers that need to inspect the
/// claims after the `require_auth` middleware has run.
///
/// # Panics
///
/// Panics if called on a request that has not passed through `require_auth`.
/// In practice this never happens because the route layer ensures it.
pub fn extract_claims(request: &Request) -> &Claims {
    request
        .extensions()
        .get::<Claims>()
        .expect(\"Claims not found — did the request pass through require_auth?\")
}

/// Check whether the authenticated user has *any* of the required roles.
///
/// Returns `Ok(())` if the user has at least one matching role, or an
/// error response with `403 Forbidden` otherwise.
///
/// # Usage
///
/// ```rust,ignore
/// pub async fn admin_only_handler(request: Request) -> Result<…, Response> {
///     let claims = extract_claims(&request);
///     check_roles(claims, &[\"admin\"])?;
///     // … handler logic …
/// }
/// ```
pub fn check_roles(claims: &Claims, required_roles: &[&str]) -> Result<(), Response> {
    if required_roles.is_empty() {
        return Ok(());
    }

    if claims.has_any_role(required_roles) {
        Ok(())
    } else {
        tracing::warn!(
            \"Access denied for user '{}': required roles {:?}, has {:?}\",
            claims.sub,
            required_roles,
            claims.roles,
        );
        Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                \"error\": \"forbidden\",
                \"message\": \"Insufficient permissions\"
            })),
        )
            .into_response())
    }
}

",
    );

    // ── Optional: role-checking middleware factory ────────────────────────
    content.push_str(
        "\
/// Create a middleware layer that requires the authenticated user to have
/// at least one of the specified roles.
///
/// This is intended to be used as a route layer **after** `require_auth`:
///
/// ```rust,ignore
/// Router::new()
///     .route(\"/admin\", get(admin_handler))
///     .route_layer(middleware::from_fn(require_auth))
///     // Note: role checking is typically done inside the handler
///     // using check_roles() for more flexibility.
/// ```
///
/// For fine-grained per-handler role checks, prefer calling
/// [`check_roles`] directly inside the handler function.

",
    );

    // ── Helper: extract claims from extensions in handlers ────────────────
    content.push_str(
        "\
/// Extension trait for easily extracting [`Claims`] in Axum handlers.
///
/// # Example
///
/// ```rust,ignore
/// use crate::auth::middleware::AuthExt;
///
/// pub async fn my_handler(
///     Extension(claims): Extension<Claims>,
/// ) -> impl IntoResponse {
///     format!(\"Hello, {}!\", claims.email)
/// }
/// ```
///
/// Alternatively, just use `request.extensions().get::<Claims>()`.
pub type AuthUser = axum::Extension<Claims>;

",
    );

    // ── Tests ────────────────────────────────────────────────────────────
    content.push_str(
        "\
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::Claims;

    #[test]
    fn test_check_roles_passes_with_matching_role() {
        let claims = Claims::new(\"u1\", \"a@b.com\", vec![\"admin\".into()], 1);
        let result = check_roles(&claims, &[\"admin\"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_roles_passes_with_any_matching_role() {
        let claims = Claims::new(\"u1\", \"a@b.com\", vec![\"editor\".into()], 1);
        let result = check_roles(&claims, &[\"admin\", \"editor\"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_roles_fails_without_matching_role() {
        let claims = Claims::new(\"u1\", \"a@b.com\", vec![\"viewer\".into()], 1);
        let result = check_roles(&claims, &[\"admin\"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_roles_passes_with_empty_required() {
        let claims = Claims::new(\"u1\", \"a@b.com\", vec![], 1);
        let result = check_roles(&claims, &[]);
        assert!(result.is_ok());
    }
}
",
    );

    GeneratedFile::new("src/auth/middleware.rs", content, FileType::Rust)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_ir::{AuthConfig, ProjectGraph};

    #[test]
    fn test_generate_auth_disabled() {
        let mut project = ProjectGraph::new("no_auth");
        project.config.auth.enabled = false;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        assert!(files.is_empty());
    }

    #[test]
    fn test_generate_auth_enabled() {
        let mut project = ProjectGraph::new("auth_app");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        assert_eq!(files.len(), 3);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"src/auth/mod.rs".to_string()));
        assert!(paths.contains(&"src/auth/jwt.rs".to_string()));
        assert!(paths.contains(&"src/auth/middleware.rs".to_string()));
    }

    #[test]
    fn test_auth_mod_re_exports() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let mod_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/mod.rs")
            .unwrap();

        let content = &mod_file.content;
        assert!(content.contains("pub mod jwt;"));
        assert!(content.contains("pub mod middleware;"));
        assert!(content.contains("pub use jwt::{Claims, create_token, verify_token};"));
        assert!(content.contains("pub use middleware::require_auth;"));
    }

    #[test]
    fn test_jwt_contains_claims_struct() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let jwt_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/jwt.rs")
            .unwrap();

        let content = &jwt_file.content;

        assert!(content.contains("pub struct Claims"));
        assert!(content.contains("pub sub: String"));
        assert!(content.contains("pub email: String"));
        assert!(content.contains("pub roles: Vec<String>"));
        assert!(content.contains("pub exp: u64"));
        assert!(content.contains("pub iat: u64"));
    }

    #[test]
    fn test_jwt_contains_create_verify_functions() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let jwt_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/jwt.rs")
            .unwrap();

        let content = &jwt_file.content;

        assert!(content.contains("pub fn create_token("));
        assert!(content.contains("pub fn verify_token("));
        assert!(content.contains("EncodingKey::from_secret"));
        assert!(content.contains("DecodingKey::from_secret"));
        assert!(content.contains("Header::default()"));
        assert!(content.contains("Validation::default()"));
    }

    #[test]
    fn test_jwt_contains_password_hashing() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let jwt_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/jwt.rs")
            .unwrap();

        let content = &jwt_file.content;

        assert!(content.contains("pub fn hash_password("));
        assert!(content.contains("pub fn verify_password("));
        assert!(content.contains("bcrypt::hash"));
        assert!(content.contains("bcrypt::verify"));
    }

    #[test]
    fn test_jwt_contains_role_helpers() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let jwt_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/jwt.rs")
            .unwrap();

        let content = &jwt_file.content;

        assert!(content.contains("pub fn has_role("));
        assert!(content.contains("pub fn has_any_role("));
        assert!(content.contains("pub fn is_expired("));
    }

    #[test]
    fn test_middleware_contains_require_auth() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let mw_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/middleware.rs")
            .unwrap();

        let content = &mw_file.content;

        assert!(content.contains("pub async fn require_auth("));
        assert!(content.contains("State(state): State<AppState>"));
        assert!(content.contains("TypedHeader(auth): TypedHeader<Authorization<Bearer>>"));
        assert!(content.contains("verify_token(token, &state.config.jwt_secret)"));
        assert!(content.contains("request.extensions_mut().insert(claims)"));
        assert!(content.contains("next.run(request).await"));
    }

    #[test]
    fn test_middleware_contains_check_roles() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let mw_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/middleware.rs")
            .unwrap();

        let content = &mw_file.content;

        assert!(content.contains("pub fn check_roles("));
        assert!(content.contains("StatusCode::FORBIDDEN"));
        assert!(content.contains("Insufficient permissions"));
    }

    #[test]
    fn test_middleware_contains_extract_claims() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let mw_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/middleware.rs")
            .unwrap();

        let content = &mw_file.content;

        assert!(content.contains("pub fn extract_claims("));
        assert!(content.contains("pub type AuthUser"));
    }

    #[test]
    fn test_middleware_returns_json_errors() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let mw_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/middleware.rs")
            .unwrap();

        let content = &mw_file.content;

        // Should return structured JSON error responses, not bare status codes
        assert!(content.contains("json!({"));
        assert!(content.contains("\"error\""));
        assert!(content.contains("\"message\""));
        assert!(content.contains("StatusCode::UNAUTHORIZED"));
        assert!(content.contains("StatusCode::FORBIDDEN"));
    }

    #[test]
    fn test_jwt_expiry_from_config() {
        let mut project = ProjectGraph::new("test");
        let mut auth = AuthConfig::jwt();
        auth.token_expiry_hours = 48;
        project.config.auth = auth;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let jwt_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/jwt.rs")
            .unwrap();

        let content = &jwt_file.content;

        // The default expiry constant should reflect the config
        assert!(content.contains("48"));
    }

    #[test]
    fn test_jwt_file_has_tests() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let jwt_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/jwt.rs")
            .unwrap();

        assert!(jwt_file.content.contains("#[cfg(test)]"));
        assert!(jwt_file.content.contains("fn test_create_and_verify_token"));
        assert!(jwt_file.content.contains("fn test_invalid_secret_fails"));
        assert!(jwt_file.content.contains("fn test_claims_has_role"));
        assert!(
            jwt_file
                .content
                .contains("fn test_hash_and_verify_password")
        );
    }

    #[test]
    fn test_middleware_file_has_tests() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_auth(&ctx);

        let mw_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/auth/middleware.rs")
            .unwrap();

        assert!(mw_file.content.contains("#[cfg(test)]"));
        assert!(
            mw_file
                .content
                .contains("fn test_check_roles_passes_with_matching_role")
        );
        assert!(
            mw_file
                .content
                .contains("fn test_check_roles_fails_without_matching_role")
        );
    }
}
