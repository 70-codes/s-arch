//! # Error Module Generator
//!
//! Generates `src/error.rs` for the generated project. The error module
//! defines an `AppError` enum that serves as the unified error type across
//! all Axum handlers. It implements `IntoResponse` so that errors are
//! automatically converted into structured JSON responses with appropriate
//! HTTP status codes.
//!
//! ## Generated Error Variants
//!
//! | Variant          | Status Code | Trigger                            |
//! |------------------|-------------|------------------------------------|
//! | `NotFound`       | 404         | Entity not found by ID             |
//! | `BadRequest`     | 400         | Malformed request body / params    |
//! | `Validation`     | 422         | `validator` crate validation fails |
//! | `Unauthorized`   | 401         | Missing or invalid auth token      |
//! | `Forbidden`      | 403         | Insufficient roles / permissions   |
//! | `Conflict`       | 409         | Unique constraint violation        |
//! | `Database`       | 500         | SeaORM / SQLx errors               |
//! | `Internal`       | 500         | Catch-all for unexpected errors    |
//!
//! ## `From` Implementations
//!
//! - `From<sea_orm::DbErr>` — maps database errors, with special handling
//!   for unique-constraint violations (→ `Conflict`)
//! - `From<validator::ValidationErrors>` — maps validation failures
//! - `From<std::io::Error>` — maps I/O errors
//! - `From<anyhow::Error>` — maps generic errors
//!
//! ## `IntoResponse` Implementation
//!
//! Every variant is serialised as:
//!
//! ```json
//! {
//!   "error": "not_found",
//!   "message": "The requested resource was not found"
//! }
//! ```

use crate::context::GenerationContext;
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate the `src/error.rs` file for the generated project.
pub fn generate_error(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    let content = build_error(ctx);
    vec![GeneratedFile::new("src/error.rs", content, FileType::Rust)]
}

// ============================================================================
// Builder
// ============================================================================

fn build_error(ctx: &GenerationContext) -> String {
    let auth_enabled = ctx.auth_enabled();

    let mut out = String::with_capacity(8192);

    out.push_str(&file_header(
        "Application error types with automatic HTTP response conversion.",
    ));

    // ── Imports ──────────────────────────────────────────────────────────
    out.push_str(
        "\
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

",
    );

    // ── AppError enum ────────────────────────────────────────────────────
    out.push_str(
        "\
// ============================================================================
// AppError
// ============================================================================

/// Unified application error type.
///
/// Every handler returns `Result<T, AppError>`. The `IntoResponse`
/// implementation converts each variant into a JSON response with the
/// appropriate HTTP status code.
#[derive(Debug, Error)]
pub enum AppError {
    /// The requested resource was not found (404).
    #[error(\"The requested resource was not found\")]
    NotFound,

    /// The request was malformed or contained invalid data (400).
    #[error(\"Bad request: {0}\")]
    BadRequest(String),

    /// One or more fields failed validation (422).
    #[error(\"Validation error: {0}\")]
    Validation(String),

",
    );

    if auth_enabled {
        out.push_str(
            "\
    /// The request lacks valid authentication credentials (401).
    #[error(\"Unauthorized: {0}\")]
    Unauthorized(String),

    /// The authenticated user does not have sufficient permissions (403).
    #[error(\"Forbidden: {0}\")]
    Forbidden(String),

",
        );
    }

    out.push_str(
        "\
    /// A unique constraint was violated (409).
    #[error(\"Conflict: {0}\")]
    Conflict(String),

    /// A database error occurred (500).
    #[error(\"Database error: {0}\")]
    Database(String),

    /// An unexpected internal error occurred (500).
    #[error(\"Internal error: {0}\")]
    Internal(String),
}

",
    );

    // ── IntoResponse ─────────────────────────────────────────────────────
    out.push_str(
        "\
// ============================================================================
// IntoResponse — convert AppError into an HTTP response
// ============================================================================

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                \"not_found\",
                self.to_string(),
            ),
            AppError::BadRequest(_) => (
                StatusCode::BAD_REQUEST,
                \"bad_request\",
                self.to_string(),
            ),
            AppError::Validation(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                \"validation_error\",
                self.to_string(),
            ),
",
    );

    if auth_enabled {
        out.push_str(
            "\
            AppError::Unauthorized(_) => (
                StatusCode::UNAUTHORIZED,
                \"unauthorized\",
                self.to_string(),
            ),
            AppError::Forbidden(_) => (
                StatusCode::FORBIDDEN,
                \"forbidden\",
                self.to_string(),
            ),
",
        );
    }

    out.push_str(
        "\
            AppError::Conflict(_) => (
                StatusCode::CONFLICT,
                \"conflict\",
                self.to_string(),
            ),
            AppError::Database(msg) => {
                // Log the full database error but return a generic message
                tracing::error!(\"Database error: {}\", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    \"database_error\",
                    \"An internal database error occurred\".to_string(),
                )
            }
            AppError::Internal(msg) => {
                tracing::error!(\"Internal error: {}\", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    \"internal_error\",
                    \"An unexpected error occurred\".to_string(),
                )
            }
        };

        let body = json!({
            \"error\": error_code,
            \"message\": message,
        });

        (status, Json(body)).into_response()
    }
}

",
    );

    // ── From<sea_orm::DbErr> ─────────────────────────────────────────────
    out.push_str(
        "\
// ============================================================================
// From implementations
// ============================================================================

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        let msg = err.to_string();

        // Detect unique constraint violations for a friendlier error
        if msg.contains(\"duplicate key\")
            || msg.contains(\"UNIQUE constraint failed\")
            || msg.contains(\"Duplicate entry\")
        {
            return AppError::Conflict(
                \"A record with the given unique field(s) already exists\".to_string(),
            );
        }

        // Detect foreign key violations
        if msg.contains(\"foreign key constraint\")
            || msg.contains(\"FOREIGN KEY constraint failed\")
        {
            return AppError::BadRequest(
                \"Referenced record does not exist\".to_string(),
            );
        }

        AppError::Database(msg)
    }
}

",
    );

    // ── From<validator::ValidationErrors> ─────────────────────────────────
    out.push_str(
        "\
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        // Build a human-readable summary of all validation failures
        let mut messages: Vec<String> = Vec::new();

        for (field, errors) in err.field_errors() {
            for error in errors {
                let msg = error
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| {
                        format!(\"Validation failed on field '{}': {:?}\", field, error.code)
                    });
                messages.push(msg);
            }
        }

        AppError::Validation(messages.join(\"; \"))
    }
}

",
    );

    // ── From<std::io::Error> ─────────────────────────────────────────────
    out.push_str(
        "\
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(format!(\"IO error: {}\", err))
    }
}

",
    );

    // ── From<anyhow::Error> ──────────────────────────────────────────────
    out.push_str(
        "\
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(format!(\"{:#}\", err))
    }
}

",
    );

    // ── ErrorResponse (typed response structure) ─────────────────────────
    out.push_str(
        "\
// ============================================================================
// Typed error response (for documentation / OpenAPI)
// ============================================================================

/// JSON structure returned for all error responses.
///
/// This struct exists primarily for OpenAPI schema generation (via `utoipa`).
/// The actual serialisation is handled inline by the `IntoResponse`
/// implementation above.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g. `\"not_found\"`, `\"validation_error\"`).
    pub error: String,

    /// Human-readable error description.
    pub message: String,
}

",
    );

    // ── Convenience constructors ─────────────────────────────────────────
    out.push_str(
        "\
// ============================================================================
// Convenience constructors
// ============================================================================

impl AppError {
    /// Create a `BadRequest` error with a formatted message.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    /// Create a `Validation` error with a formatted message.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a `Conflict` error with a formatted message.
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Create an `Internal` error with a formatted message.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
",
    );

    if auth_enabled {
        out.push_str(
            "\
\n    /// Create an `Unauthorized` error with a formatted message.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// Create a `Forbidden` error with a formatted message.
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }
",
        );
    }

    out.push_str("}\n\n");

    // ── Tests ────────────────────────────────────────────────────────────
    out.push_str(
        "\
// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_message() {
        let err = AppError::NotFound;
        assert_eq!(err.to_string(), \"The requested resource was not found\");
    }

    #[test]
    fn test_bad_request_message() {
        let err = AppError::bad_request(\"missing field\");
        assert_eq!(err.to_string(), \"Bad request: missing field\");
    }

    #[test]
    fn test_validation_message() {
        let err = AppError::validation(\"email is invalid\");
        assert_eq!(err.to_string(), \"Validation error: email is invalid\");
    }

    #[test]
    fn test_conflict_message() {
        let err = AppError::conflict(\"email already exists\");
        assert_eq!(err.to_string(), \"Conflict: email already exists\");
    }

    #[test]
    fn test_internal_message() {
        let err = AppError::internal(\"something broke\");
        assert_eq!(err.to_string(), \"Internal error: something broke\");
    }

    #[test]
    fn test_from_db_err_duplicate_key() {
        let db_err = sea_orm::DbErr::Query(sea_orm::RuntimeErr::Internal(
            \"duplicate key value violates unique constraint\".to_string(),
        ));
        let app_err = AppError::from(db_err);
        assert!(matches!(app_err, AppError::Conflict(_)));
    }

    #[test]
    fn test_from_db_err_generic() {
        let db_err = sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
            \"connection refused\".to_string(),
        ));
        let app_err = AppError::from(db_err);
        assert!(matches!(app_err, AppError::Database(_)));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, \"file missing\");
        let app_err = AppError::from(io_err);
        assert!(matches!(app_err, AppError::Internal(_)));
        assert!(app_err.to_string().contains(\"IO error\"));
    }
",
    );

    if auth_enabled {
        out.push_str(
            "\
\n    #[test]
    fn test_unauthorized_message() {
        let err = AppError::unauthorized(\"token expired\");
        assert_eq!(err.to_string(), \"Unauthorized: token expired\");
    }

    #[test]
    fn test_forbidden_message() {
        let err = AppError::forbidden(\"admin only\");
        assert_eq!(err.to_string(), \"Forbidden: admin only\");
    }
",
        );
    }

    out.push_str(
        "\
}
",
    );

    out
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_ir::{AuthConfig, ProjectGraph};

    #[test]
    fn test_generate_error_produces_one_file() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path.to_string_lossy(), "src/error.rs");
    }

    #[test]
    fn test_error_has_app_error_enum() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub enum AppError"));
        assert!(content.contains("NotFound"));
        assert!(content.contains("BadRequest"));
        assert!(content.contains("Validation"));
        assert!(content.contains("Conflict"));
        assert!(content.contains("Database"));
        assert!(content.contains("Internal"));
    }

    #[test]
    fn test_error_has_into_response() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("impl IntoResponse for AppError"));
        assert!(content.contains("StatusCode::NOT_FOUND"));
        assert!(content.contains("StatusCode::BAD_REQUEST"));
        assert!(content.contains("StatusCode::UNPROCESSABLE_ENTITY"));
        assert!(content.contains("StatusCode::CONFLICT"));
        assert!(content.contains("StatusCode::INTERNAL_SERVER_ERROR"));
    }

    #[test]
    fn test_error_has_from_impls() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("impl From<sea_orm::DbErr> for AppError"));
        assert!(content.contains("impl From<validator::ValidationErrors> for AppError"));
        assert!(content.contains("impl From<std::io::Error> for AppError"));
        assert!(content.contains("impl From<anyhow::Error> for AppError"));
    }

    #[test]
    fn test_error_has_convenience_constructors() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub fn bad_request("));
        assert!(content.contains("pub fn validation("));
        assert!(content.contains("pub fn conflict("));
        assert!(content.contains("pub fn internal("));
    }

    #[test]
    fn test_error_has_error_response_struct() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub struct ErrorResponse"));
        assert!(content.contains("pub error: String"));
        assert!(content.contains("pub message: String"));
    }

    #[test]
    fn test_error_with_auth_has_unauthorized_forbidden() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("Unauthorized"));
        assert!(content.contains("Forbidden"));
        assert!(content.contains("StatusCode::UNAUTHORIZED"));
        assert!(content.contains("StatusCode::FORBIDDEN"));
        assert!(content.contains("pub fn unauthorized("));
        assert!(content.contains("pub fn forbidden("));
    }

    #[test]
    fn test_error_without_auth_no_unauthorized_forbidden() {
        let mut project = ProjectGraph::new("test");
        project.config.auth.enabled = false;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(!content.contains("Unauthorized"));
        assert!(!content.contains("Forbidden"));
        assert!(!content.contains("StatusCode::UNAUTHORIZED"));
        assert!(!content.contains("StatusCode::FORBIDDEN"));
    }

    #[test]
    fn test_error_db_duplicate_key_detection() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("duplicate key"));
        assert!(content.contains("UNIQUE constraint failed"));
        assert!(content.contains("Duplicate entry"));
        assert!(content.contains("AppError::Conflict"));
    }

    #[test]
    fn test_error_db_foreign_key_detection() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("foreign key constraint"));
        assert!(content.contains("FOREIGN KEY constraint failed"));
        assert!(content.contains("AppError::BadRequest"));
    }

    #[test]
    fn test_error_hides_internal_details() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        // Database and internal errors should log details but return generic messages
        assert!(content.contains("tracing::error!(\"Database error:"));
        assert!(content.contains("tracing::error!(\"Internal error:"));
        assert!(content.contains("\"An internal database error occurred\""));
        assert!(content.contains("\"An unexpected error occurred\""));
    }

    #[test]
    fn test_error_json_response_structure() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        // Responses should be structured JSON with error and message fields
        assert!(content.contains("json!({"));
        assert!(content.contains("\"error\": error_code"));
        assert!(content.contains("\"message\": message"));
    }

    #[test]
    fn test_error_validation_from_impl_details() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        // The From<ValidationErrors> impl should iterate field errors
        assert!(content.contains("field_errors()"));
        assert!(content.contains("messages.join"));
    }

    #[test]
    fn test_error_uses_thiserror() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("use thiserror::Error;"));
        assert!(content.contains("#[derive(Debug, Error)]"));
        assert!(content.contains("#[error("));
    }

    #[test]
    fn test_error_has_file_header() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("Auto-generated by Immortal Engine"));
        assert!(content.contains("DO NOT EDIT"));
    }

    #[test]
    fn test_error_has_tests() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("#[cfg(test)]"));
        assert!(content.contains("fn test_not_found_message"));
        assert!(content.contains("fn test_bad_request_message"));
        assert!(content.contains("fn test_validation_message"));
        assert!(content.contains("fn test_conflict_message"));
        assert!(content.contains("fn test_internal_message"));
        assert!(content.contains("fn test_from_db_err_duplicate_key"));
        assert!(content.contains("fn test_from_db_err_generic"));
        assert!(content.contains("fn test_from_io_error"));
    }

    #[test]
    fn test_error_with_auth_has_auth_tests() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(content.contains("fn test_unauthorized_message"));
        assert!(content.contains("fn test_forbidden_message"));
    }

    #[test]
    fn test_error_without_auth_no_auth_tests() {
        let mut project = ProjectGraph::new("test");
        project.config.auth.enabled = false;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_error(&ctx);
        let content = &files[0].content;

        assert!(!content.contains("fn test_unauthorized_message"));
        assert!(!content.contains("fn test_forbidden_message"));
    }
}
