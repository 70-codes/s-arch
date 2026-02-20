//! # Middleware Generator
//!
//! Generates `src/middleware.rs` (or `src/middleware/mod.rs`) for the generated
//! project. The middleware module provides reusable Axum middleware layers for:
//!
//! - **Request logging**: structured tracing of method, path, status, and latency
//! - **Request ID**: injects a unique `X-Request-Id` header into every response
//!
//! These middleware are applied globally via the router in `routes/mod.rs`.
//!
//! ## Usage
//!
//! The middleware is automatically wired into the router by the routes generator.
//! No manual configuration is required.

use crate::context::GenerationContext;
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate the middleware module for the generated project.
///
/// Produces a single file `src/middleware.rs` containing request logging
/// and request-ID middleware.
pub fn generate_middleware(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    let content = build_middleware(ctx);
    vec![GeneratedFile::new(
        "src/middleware.rs",
        content,
        FileType::Rust,
    )]
}

// ============================================================================
// Builder
// ============================================================================

fn build_middleware(ctx: &GenerationContext) -> String {
    let mut out = String::with_capacity(4096);

    out.push_str(&file_header("Custom middleware for request processing."));

    // ── Imports ──────────────────────────────────────────────────────────
    out.push_str(
        "\
use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use uuid::Uuid;

",
    );

    // ── Request logging middleware ────────────────────────────────────────
    if ctx.generate_docs() {
        out.push_str(
            "\
/// Middleware that logs every incoming request and its response status.
///
/// Emits a structured `tracing` event at the `info` level containing:
///
/// - HTTP method
/// - Request path
/// - Response status code
/// - Latency in milliseconds
///
/// # Example output
///
/// ```text
/// INFO request completed method=GET path=/api/users status=200 latency_ms=12
/// ```
",
        );
    }

    out.push_str(
        "\
pub async fn request_logger(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    tracing::debug!(
        method = %method,
        path = %path,
        \"request started\",
    );

    let response = next.run(request).await;

    let latency = start.elapsed();
    let status = response.status().as_u16();

    tracing::info!(
        method = %method,
        path = %path,
        status = status,
        latency_ms = latency.as_millis() as u64,
        \"request completed\",
    );

    response
}

",
    );

    // ── Request ID middleware ─────────────────────────────────────────────
    if ctx.generate_docs() {
        out.push_str(
            "\
/// Middleware that injects a unique `X-Request-Id` header into every response.
///
/// If the incoming request already carries an `X-Request-Id` header, it is
/// preserved. Otherwise a new UUID v4 is generated.
///
/// This is useful for correlating logs across services and for debugging
/// specific requests reported by clients.
",
        );
    }

    out.push_str(
        "\
pub async fn request_id(
    request: Request,
    next: Next,
) -> Response {
    // Reuse existing request ID or generate a new one
    let request_id = request
        .headers()
        .get(\"x-request-id\")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let mut response = next.run(request).await;

    // Attach the request ID to the response
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(\"x-request-id\", value);
    }

    response
}

",
    );

    // ── Request body size limit (optional helper) ────────────────────────
    if ctx.generate_docs() {
        out.push_str(
            "\
/// Maximum allowed request body size in bytes.
///
/// This constant can be used with `axum::extract::DefaultBodyLimit` to
/// prevent excessively large payloads from consuming server resources.
///
/// ```rust,ignore
/// use axum::extract::DefaultBodyLimit;
///
/// Router::new()
///     .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
/// ```
",
        );
    }

    out.push_str(
        "\
pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10 MiB

",
    );

    // ── Tests ────────────────────────────────────────────────────────────
    out.push_str(
        "\
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_body_size() {
        assert_eq!(MAX_BODY_SIZE, 10 * 1024 * 1024);
    }
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
    use imortal_ir::ProjectGraph;

    #[test]
    fn test_generate_middleware_produces_one_file() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path.to_string_lossy(), "src/middleware.rs");
    }

    #[test]
    fn test_middleware_has_request_logger() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub async fn request_logger("));
        assert!(content.contains("request: Request"));
        assert!(content.contains("next: Next"));
        assert!(content.contains("Instant::now()"));
        assert!(content.contains("latency_ms"));
        assert!(content.contains("tracing::info!"));
        assert!(content.contains("request completed"));
    }

    #[test]
    fn test_middleware_has_request_id() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub async fn request_id("));
        assert!(content.contains("x-request-id"));
        assert!(content.contains("Uuid::new_v4()"));
        assert!(content.contains("headers_mut().insert"));
    }

    #[test]
    fn test_middleware_has_max_body_size() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub const MAX_BODY_SIZE: usize"));
        assert!(content.contains("10 * 1024 * 1024"));
    }

    #[test]
    fn test_middleware_has_imports() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        assert!(content.contains("use axum::{"));
        assert!(content.contains("Request"));
        assert!(content.contains("Next"));
        assert!(content.contains("Response"));
        assert!(content.contains("use std::time::Instant;"));
        assert!(content.contains("use uuid::Uuid;"));
        assert!(content.contains("HeaderValue"));
    }

    #[test]
    fn test_middleware_has_file_header() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        assert!(content.contains("Auto-generated by Immortal Engine"));
        assert!(content.contains("DO NOT EDIT"));
    }

    #[test]
    fn test_middleware_with_docs() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        // Doc comments should be present by default
        assert!(content.contains("/// Middleware that logs every incoming request"));
        assert!(content.contains("/// Middleware that injects a unique"));
        assert!(content.contains("/// Maximum allowed request body size"));
    }

    #[test]
    fn test_middleware_without_docs() {
        let project = ProjectGraph::new("test");
        let config = crate::GeneratorConfig::new().without_docs();
        let ctx = GenerationContext::from_project(&project, config);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        // Doc comments should NOT be present
        assert!(!content.contains("/// Middleware that logs every incoming request"));
        assert!(!content.contains("/// Middleware that injects a unique"));
    }

    #[test]
    fn test_middleware_request_logger_logs_method_path_status() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        assert!(content.contains("method = %method"));
        assert!(content.contains("path = %path"));
        assert!(content.contains("status = status"));
    }

    #[test]
    fn test_middleware_request_id_preserves_existing() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        // Should check for existing header before generating a new one
        assert!(content.contains("get(\"x-request-id\")"));
        assert!(content.contains("unwrap_or_else"));
        assert!(content.contains("Uuid::new_v4()"));
    }

    #[test]
    fn test_middleware_has_embedded_tests() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_middleware(&ctx);
        let content = &files[0].content;

        assert!(content.contains("#[cfg(test)]"));
        assert!(content.contains("fn test_max_body_size"));
    }
}
