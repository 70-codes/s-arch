//! # Config Module Generator
//!
//! Generates `src/config.rs` for the generated project. The config module
//! reads configuration values from environment variables (loaded via `dotenvy`)
//! and exposes them through a typed `Config` struct.
//!
//! ## Generated Structure
//!
//! ```rust,ignore
//! pub struct Config {
//!     pub database_url: String,
//!     pub server_host: String,
//!     pub server_port: u16,
//!     pub jwt_secret: String,          // if auth enabled
//!     pub jwt_expiry_hours: u64,       // if auth enabled
//!     pub database_max_connections: u32,
//!     pub database_min_connections: u32,
//! }
//! ```

use crate::context::GenerationContext;
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate the `src/config.rs` file for the generated project.
pub fn generate_config(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    let content = build_config(ctx);
    vec![GeneratedFile::new("src/config.rs", content, FileType::Rust)]
}

// ============================================================================
// Builder
// ============================================================================

fn build_config(ctx: &GenerationContext) -> String {
    let host = ctx.server_host();
    let port = ctx.server_port();
    let auth_enabled = ctx.auth_enabled();
    let expiry_hours = ctx.auth_config().token_expiry_hours;

    let mut out = String::with_capacity(4096);

    out.push_str(&file_header(
        "Application configuration loaded from environment variables.",
    ));

    // ── Imports ──────────────────────────────────────────────────────────
    out.push_str("use std::env;\n\n");

    // ── Config struct ────────────────────────────────────────────────────
    out.push_str(
        "\
/// Application configuration.
///
/// All values are loaded from environment variables at startup.
/// A `.env` file is supported via the `dotenvy` crate.
#[derive(Debug, Clone)]
pub struct Config {
    // ── Server ───────────────────────────────────────────────────────
    /// Host to bind the HTTP server to (e.g. `0.0.0.0`).
    pub server_host: String,

    /// Port to bind the HTTP server to (e.g. `8080`).
    pub server_port: u16,

    /// Log level filter (e.g. `info`, `debug,tower_http=trace`).
    pub rust_log: String,

    // ── Database ─────────────────────────────────────────────────────
    /// Database connection URL.
    pub database_url: String,

    /// Maximum number of connections in the pool.
    pub database_max_connections: u32,

    /// Minimum number of idle connections in the pool.
    pub database_min_connections: u32,
",
    );

    if auth_enabled {
        out.push_str(
            "\
    // ── Authentication ───────────────────────────────────────────────
    /// Secret key used to sign and verify JWT tokens.
    ///
    /// **Must** be a long, cryptographically random string in production.
    pub jwt_secret: String,

    /// JWT token lifetime in hours.
    pub jwt_expiry_hours: u64,
",
        );
    }

    out.push_str("}\n\n");

    // ── Config::from_env ─────────────────────────────────────────────────
    out.push_str("impl Config {\n");

    out.push_str(&format!(
        "\
    /// Load configuration from environment variables.
    ///
    /// This function reads from the process environment (and any `.env` file
    /// loaded by `dotenvy`). Missing required variables cause a panic with
    /// a descriptive message.
    ///
    /// ## Required Variables
    ///
    /// - `DATABASE_URL`
    ///
    /// ## Optional Variables (with defaults)
    ///
    /// - `SERVER_HOST` (default: `{host}`)
    /// - `SERVER_PORT` (default: `{port}`)
    /// - `RUST_LOG` (default: `info`)
    /// - `DATABASE_MAX_CONNECTIONS` (default: `10`)
    /// - `DATABASE_MIN_CONNECTIONS` (default: `1`)
"
    ));

    if auth_enabled {
        out.push_str(&format!(
            "\
    /// - `JWT_SECRET` (**required** when auth is enabled)
    /// - `JWT_EXPIRY_HOURS` (default: `{expiry_hours}`)
",
        ));
    }

    out.push_str("    pub fn from_env() -> Self {\n");

    out.push_str(&format!(
        "\
        let server_host = env::var(\"SERVER_HOST\")
            .unwrap_or_else(|_| \"{host}\".to_string());

        let server_port = env::var(\"SERVER_PORT\")
            .unwrap_or_else(|_| \"{port}\".to_string())
            .parse::<u16>()
            .expect(\"SERVER_PORT must be a valid u16\");

        let rust_log = env::var(\"RUST_LOG\")
            .unwrap_or_else(|_| \"info\".to_string());

        let database_url = env::var(\"DATABASE_URL\")
            .expect(\"DATABASE_URL environment variable is required\");

        let database_max_connections = env::var(\"DATABASE_MAX_CONNECTIONS\")
            .unwrap_or_else(|_| \"10\".to_string())
            .parse::<u32>()
            .expect(\"DATABASE_MAX_CONNECTIONS must be a valid u32\");

        let database_min_connections = env::var(\"DATABASE_MIN_CONNECTIONS\")
            .unwrap_or_else(|_| \"1\".to_string())
            .parse::<u32>()
            .expect(\"DATABASE_MIN_CONNECTIONS must be a valid u32\");

"
    ));

    if auth_enabled {
        out.push_str(&format!(
            "\
        let jwt_secret = env::var(\"JWT_SECRET\")
            .expect(\"JWT_SECRET environment variable is required for authentication\");

        let jwt_expiry_hours = env::var(\"JWT_EXPIRY_HOURS\")
            .unwrap_or_else(|_| \"{expiry_hours}\".to_string())
            .parse::<u64>()
            .expect(\"JWT_EXPIRY_HOURS must be a valid u64\");

"
        ));
    }

    // Construct Self
    out.push_str("        Self {\n");
    out.push_str("            server_host,\n");
    out.push_str("            server_port,\n");
    out.push_str("            rust_log,\n");
    out.push_str("            database_url,\n");
    out.push_str("            database_max_connections,\n");
    out.push_str("            database_min_connections,\n");

    if auth_enabled {
        out.push_str("            jwt_secret,\n");
        out.push_str("            jwt_expiry_hours,\n");
    }

    out.push_str("        }\n");
    out.push_str("    }\n\n");

    // ── bind_address helper ──────────────────────────────────────────────
    out.push_str(
        "\
    /// Return the `host:port` string suitable for binding a TCP listener.
    pub fn bind_address(&self) -> String {
        format!(\"{}:{}\", self.server_host, self.server_port)
    }
",
    );

    // ── database_connect_options helper ──────────────────────────────────
    out.push_str(
        "\
\n    /// Build SeaORM `ConnectOptions` from the loaded configuration.
    pub fn database_connect_options(&self) -> sea_orm::ConnectOptions {
        let mut opt = sea_orm::ConnectOptions::new(&self.database_url);
        opt.max_connections(self.database_max_connections)
            .min_connections(self.database_min_connections)
            .sqlx_logging(true)
            .sqlx_logging_level(tracing::log::LevelFilter::Debug);
        opt
    }
",
    );

    out.push_str("}\n\n");

    // ── Default impl (using from_env) ────────────────────────────────────
    out.push_str(
        "\
impl Default for Config {
    /// Create a Config by reading from the environment.
    ///
    /// This is equivalent to calling [`Config::from_env()`].
    fn default() -> Self {
        Self::from_env()
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
    use imortal_ir::{AuthConfig, ProjectGraph};

    #[test]
    fn test_generate_config_produces_one_file() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path.to_string_lossy(), "src/config.rs");
    }

    #[test]
    fn test_config_has_struct() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub struct Config"));
        assert!(content.contains("pub server_host: String"));
        assert!(content.contains("pub server_port: u16"));
        assert!(content.contains("pub database_url: String"));
        assert!(content.contains("pub database_max_connections: u32"));
        assert!(content.contains("pub database_min_connections: u32"));
        assert!(content.contains("pub rust_log: String"));
    }

    #[test]
    fn test_config_has_from_env() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub fn from_env()"));
        assert!(content.contains("env::var(\"DATABASE_URL\")"));
        assert!(content.contains("env::var(\"SERVER_HOST\")"));
        assert!(content.contains("env::var(\"SERVER_PORT\")"));
        assert!(content.contains("env::var(\"RUST_LOG\")"));
    }

    #[test]
    fn test_config_has_helpers() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub fn bind_address("));
        assert!(content.contains("pub fn database_connect_options("));
        assert!(content.contains("ConnectOptions"));
    }

    #[test]
    fn test_config_with_auth() {
        let mut project = ProjectGraph::new("test");
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("pub jwt_secret: String"));
        assert!(content.contains("pub jwt_expiry_hours: u64"));
        // Uses the configurable env variable name from AuthConfig.jwt_secret_env
        assert!(content.contains("env::var(\"JWT_SECRET\")"));
        assert!(content.contains("env::var(\"JWT_EXPIRY_HOURS\")"));
        assert!(content.contains("jwt_secret,"));
        assert!(content.contains("jwt_expiry_hours,"));
    }

    #[test]
    fn test_config_without_auth() {
        let mut project = ProjectGraph::new("test");
        project.config.auth.enabled = false;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(!content.contains("jwt_secret"));
        assert!(!content.contains("JWT_SECRET"));
        assert!(!content.contains("jwt_expiry_hours"));
    }

    #[test]
    fn test_config_uses_project_host_port() {
        let mut project = ProjectGraph::new("test");
        project.config.server_host = "127.0.0.1".to_string();
        project.config.server_port = 3000;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("127.0.0.1"));
        assert!(content.contains("3000"));
    }

    #[test]
    fn test_config_has_default_impl() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("impl Default for Config"));
        assert!(content.contains("Self::from_env()"));
    }

    #[test]
    fn test_config_custom_expiry() {
        let mut project = ProjectGraph::new("test");
        let mut auth = AuthConfig::jwt();
        auth.token_expiry_hours = 72;
        project.config.auth = auth;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("72"));
    }

    #[test]
    fn test_config_database_connect_options() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("max_connections(self.database_max_connections)"));
        assert!(content.contains("min_connections(self.database_min_connections)"));
        assert!(content.contains("sqlx_logging(true)"));
    }

    #[test]
    fn test_config_file_header() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_config(&ctx);
        let content = &files[0].content;

        assert!(content.contains("Auto-generated by Immortal Engine"));
        assert!(content.contains("DO NOT EDIT"));
    }
}
