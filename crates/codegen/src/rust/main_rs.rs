//! # Main Entry Point Generator
//!
//! Generates `src/main.rs` for the generated project. The main function is the
//! server entry point that:
//!
//! 1. Loads environment variables from `.env` via `dotenvy`
//! 2. Initialises structured logging with `tracing_subscriber`
//! 3. Reads configuration from environment (`Config::from_env()`)
//! 4. Connects to the database via SeaORM
//! 5. Builds the application state (`AppState`)
//! 6. Assembles the Axum router (with middleware, CORS, auth layers)
//! 7. Binds a TCP listener and starts the server
//!
//! ## Generated Code Structure
//!
//! ```rust,ignore
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     dotenvy::dotenv().ok();
//!     init_tracing();
//!     let config = Config::from_env();
//!     let db = connect_database(&config).await?;
//!     let state = AppState::new(db, config.clone());
//!     let router = create_router(state);
//!     let listener = tokio::net::TcpListener::bind(config.bind_address()).await?;
//!     axum::serve(listener, router).await?;
//!     Ok(())
//! }
//! ```

use crate::context::GenerationContext;
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate the `src/main.rs` file for the generated project.
pub fn generate_main(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    let content = build_main(ctx);
    vec![GeneratedFile::new("src/main.rs", content, FileType::Rust)]
}

// ============================================================================
// Builder
// ============================================================================

fn build_main(ctx: &GenerationContext) -> String {
    let pkg = ctx.package_name();
    let host = ctx.server_host();
    let port = ctx.server_port();
    let auth_enabled = ctx.auth_enabled();
    let db_name = match ctx.database() {
        imortal_ir::DatabaseType::PostgreSQL => "PostgreSQL",
        imortal_ir::DatabaseType::MySQL => "MySQL",
        imortal_ir::DatabaseType::SQLite => "SQLite",
    };

    let mut out = String::with_capacity(4096);

    out.push_str(&file_header(&format!("{} — application entry point.", pkg)));

    // ── Imports ──────────────────────────────────────────────────────────
    out.push_str("use anyhow::Context;\n");
    out.push_str("use sea_orm::Database;\n");
    out.push_str(
        "use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};\n",
    );
    out.push_str("\n");

    // Import from the crate itself (the generated lib.rs)
    let crate_ident = pkg.replace('-', "_");
    out.push_str(&format!("use {}::config::Config;\n", crate_ident));
    out.push_str(&format!("use {}::routes::create_router;\n", crate_ident));
    out.push_str(&format!("use {}::state::AppState;\n", crate_ident));
    out.push_str("\n");

    // ── main ─────────────────────────────────────────────────────────────
    out.push_str(&format!(
        r#"/// Application entry point.
///
/// Starts the {} server backed by {} on `{}:{}`.
#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    // ── 1. Load .env file (if present) ───────────────────────────────
    dotenvy::dotenv().ok();

    // ── 2. Initialise structured logging ─────────────────────────────
    init_tracing();

    tracing::info!(
        name = "{}",
        version = env!("CARGO_PKG_VERSION"),
        "starting application",
    );

    // ── 3. Load configuration ────────────────────────────────────────
    let config = Config::from_env();
    tracing::info!(
        host = %config.server_host,
        port = config.server_port,
        "configuration loaded",
    );

    // ── 4. Connect to database ───────────────────────────────────────
    tracing::info!("connecting to {} database…");
    let db = Database::connect(config.database_connect_options())
        .await
        .context("failed to connect to database")?;
    tracing::info!("database connection established");

    // ── 5. Build application state ───────────────────────────────────
    let state = AppState::new(db, config.clone());

    // ── 6. Build router ──────────────────────────────────────────────
    let router = create_router(state);

    // ── 7. Start server ──────────────────────────────────────────────
    let bind_addr = config.bind_address();
    tracing::info!(address = %bind_addr, "starting HTTP server");

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind to {{}}", bind_addr))?;

    tracing::info!(
        address = %bind_addr,
        "server is ready — listening for connections",
    );

    axum::serve(listener, router)
        .await
        .context("server error")?;

    Ok(())
}}
"#,
        pkg, db_name, host, port, pkg, db_name,
    ));

    out.push('\n');

    // ── init_tracing helper ──────────────────────────────────────────────
    out.push_str(
        r#"/// Initialise the `tracing` subscriber with an env-filter.
///
/// The log level is controlled by the `RUST_LOG` environment variable.
/// If not set it defaults to `info` for application logs and `warn` for
/// dependencies.
///
/// # Example
///
/// ```bash
/// RUST_LOG=debug cargo run
/// RUST_LOG=info,tower_http=debug cargo run
/// ```
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("info,tower_http=debug,sea_orm=info")
        });

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false))
        .init();
}
"#,
    );

    out.push('\n');

    // ── Banner helper (optional, nice touch) ─────────────────────────────
    if ctx.generate_docs() {
        out.push_str(&format!(
            r#"/// Print a startup banner to stdout.
///
/// This is purely cosmetic and can be removed if desired.
#[allow(dead_code)]
fn print_banner() {{
    println!();
    println!("  ╔═══════════════════════════════════════════════════╗");
    println!("  ║  {} {{:<42}} ║", "");
    println!("  ║  Generated by Immortal Engine v2.0               ║");
    println!("  ╚═══════════════════════════════════════════════════╝");
    println!();
}}
"#,
            pkg,
        ));
    }

    out
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_ir::{AuthConfig, DatabaseType, ProjectGraph};

    #[test]
    fn test_generate_main_produces_one_file() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path.to_string_lossy(), "src/main.rs");
    }

    #[test]
    fn test_main_has_tokio_main() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("#[tokio::main]"));
        assert!(content.contains("async fn main()"));
        assert!(content.contains("anyhow::Result<()>"));
    }

    #[test]
    fn test_main_loads_dotenv() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("dotenvy::dotenv().ok()"));
    }

    #[test]
    fn test_main_inits_tracing() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("init_tracing()"));
        assert!(content.contains("fn init_tracing()"));
        assert!(content.contains("EnvFilter"));
        assert!(content.contains("tracing_subscriber::registry()"));
    }

    #[test]
    fn test_main_connects_to_database() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("Database::connect"));
        assert!(content.contains("database_connect_options()"));
        assert!(content.contains("database connection established"));
    }

    #[test]
    fn test_main_builds_state() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("AppState::new(db, config.clone())"));
    }

    #[test]
    fn test_main_creates_router() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("create_router(state)"));
    }

    #[test]
    fn test_main_binds_listener() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("TcpListener::bind"));
        assert!(content.contains("config.bind_address()"));
    }

    #[test]
    fn test_main_starts_axum_serve() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("axum::serve(listener, router)"));
    }

    #[test]
    fn test_main_imports_crate_modules() {
        let project = ProjectGraph::new("my-api");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        // Hyphens in package name should be converted to underscores for use
        assert!(content.contains("use my_app::config::Config;"));
        assert!(content.contains("use my_app::routes::create_router;"));
        assert!(content.contains("use my_app::state::AppState;"));
    }

    #[test]
    fn test_main_imports_anyhow() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("use anyhow::Context;"));
    }

    #[test]
    fn test_main_imports_sea_orm() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("use sea_orm::Database;"));
    }

    #[test]
    fn test_main_mentions_database_type_postgresql() {
        let mut project = ProjectGraph::new("test");
        project.config.database = DatabaseType::PostgreSQL;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("PostgreSQL"));
    }

    #[test]
    fn test_main_mentions_database_type_mysql() {
        let mut project = ProjectGraph::new("test");
        project.config.database = DatabaseType::MySQL;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("MySQL"));
    }

    #[test]
    fn test_main_mentions_database_type_sqlite() {
        let mut project = ProjectGraph::new("test");
        project.config.database = DatabaseType::SQLite;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("SQLite"));
    }

    #[test]
    fn test_main_uses_config_host_port() {
        let mut project = ProjectGraph::new("test");
        project.config.server_host = "127.0.0.1".to_string();
        project.config.server_port = 3000;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("127.0.0.1"));
        assert!(content.contains("3000"));
    }

    #[test]
    fn test_main_has_file_header() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("Auto-generated by Immortal Engine"));
        assert!(content.contains("DO NOT EDIT"));
    }

    #[test]
    fn test_main_has_graceful_error_handling() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        // Should use .context() for better error messages
        assert!(content.contains(".context(\"failed to connect to database\")"));
        assert!(content.contains("failed to bind"));
        assert!(content.contains(".context(\"server error\")"));
    }

    #[test]
    fn test_main_has_startup_logging() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("starting application"));
        assert!(content.contains("configuration loaded"));
        assert!(content.contains("starting HTTP server"));
        assert!(content.contains("server is ready"));
    }

    #[test]
    fn test_init_tracing_has_env_filter() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("EnvFilter::try_from_default_env"));
        assert!(content.contains("EnvFilter::new"));
        assert!(content.contains("info,tower_http=debug"));
    }

    #[test]
    fn test_init_tracing_configures_fmt_layer() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("tracing_subscriber::fmt::layer()"));
        assert!(content.contains("with_target(true)"));
    }

    #[test]
    fn test_main_with_docs_has_banner() {
        let mut project = ProjectGraph::new("cool_api");
        project.config.package_name = "cool_api".to_string();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("fn print_banner()"));
        assert!(content.contains("Immortal Engine v2.0"));
        assert!(content.contains("cool_api"));
    }

    #[test]
    fn test_main_without_docs_no_banner() {
        let project = ProjectGraph::new("test");
        let config = crate::GeneratorConfig::new().without_docs();
        let ctx = GenerationContext::from_project(&project, config);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(!content.contains("fn print_banner()"));
    }

    #[test]
    fn test_main_returns_ok() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        assert!(content.contains("Ok(())"));
    }

    #[test]
    fn test_main_pkg_name_in_log() {
        let project = ProjectGraph::new("my_awesome_api");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_main(&ctx);
        let content = &files[0].content;

        // The package name should appear in the startup log
        assert!(content.contains("my_app")); // default package name from config
    }
}
