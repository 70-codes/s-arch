//! # Routes Generator (Axum Router)
//!
//! Generates the Axum `Router` configuration that wires together all entity
//! handlers, middleware, and (optionally) authentication layers.
//!
//! ## Generated Files
//!
//! - `src/routes/mod.rs` — module declarations and the top-level `create_router` fn
//! - `src/routes/api.rs` — per-entity route groups, nested under their base paths
//!
//! ## Router Structure
//!
//! ```text
//! Router::new()
//!   .nest("/api/users",   user_routes())
//!   .nest("/api/posts",   post_routes())
//!   …
//!   .layer(TraceLayer)
//!   .layer(CorsLayer)          // if CORS enabled
//!   .with_state(app_state)
//! ```
//!
//! Per-entity route groups honour the endpoint configuration:
//!
//! | Operation | Method & Path            | Condition              |
//! |-----------|--------------------------|------------------------|
//! | Create    | `POST   /`               | operation enabled      |
//! | ReadAll   | `GET    /`               | operation enabled      |
//! | Read      | `GET    /:id`            | operation enabled      |
//! | Update    | `PUT    /:id`            | operation enabled      |
//! | Delete    | `DELETE /:id`            | operation enabled      |
//!
//! When authentication is enabled, secured routes are wrapped with the
//! `require_auth` middleware layer.

use imortal_ir::OperationType;

use crate::context::{EntityInfo, GenerationContext};
use crate::rust::{doc_comment, file_header};
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate all route files (`src/routes/mod.rs` and `src/routes/api.rs`).
pub fn generate_routes(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    vec![generate_routes_mod(ctx), generate_api_routes(ctx)]
}

// ============================================================================
// routes/mod.rs
// ============================================================================

fn generate_routes_mod(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(2048);

    content.push_str(&file_header(
        "Route definitions — top-level router assembly.",
    ));

    // Imports
    content.push_str("pub mod api;\n\n");

    content.push_str("use axum::Router;\n");

    if ctx.config.cors_enabled {
        content.push_str("use tower_http::cors::{Any, CorsLayer};\n");
    }
    content.push_str("use tower_http::trace::TraceLayer;\n");
    content.push_str("use std::time::Duration;\n");
    content.push_str("use tower_http::timeout::TimeoutLayer;\n");
    content.push('\n');
    content.push_str("use crate::state::AppState;\n\n");

    // create_router function
    content.push_str(&doc_comment(
        Some("Build the complete application router with all routes, middleware, and state."),
        ctx,
    ));

    content.push_str("pub fn create_router(state: AppState) -> Router {\n");

    // CORS layer
    if ctx.config.cors_enabled {
        content.push_str(
            "    let cors = CorsLayer::new()\n\
             \x20       .allow_origin(Any)\n\
             \x20       .allow_methods(Any)\n\
             \x20       .allow_headers(Any);\n\n",
        );
    }

    content.push_str("    let api_routes = api::api_routes();\n\n");

    content.push_str("    Router::new()\n");
    content.push_str("        .nest(\"/\", api_routes)\n");

    // Layers
    content.push_str("        .layer(TraceLayer::new_for_http())\n");
    content.push_str("        .layer(TimeoutLayer::new(Duration::from_secs(30)))\n");

    if ctx.config.cors_enabled {
        content.push_str("        .layer(cors)\n");
    }

    content.push_str("        .with_state(state)\n");
    content.push_str("}\n");

    GeneratedFile::new("src/routes/mod.rs", content, FileType::Rust)
}

// ============================================================================
// routes/api.rs — per-entity route groups
// ============================================================================

fn generate_api_routes(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(4096);

    content.push_str(&file_header(
        "API route definitions — entity endpoint groups.",
    ));

    // ── Imports ──────────────────────────────────────────────────────────

    content.push_str("use axum::{\n");
    content.push_str("    Router,\n");
    content.push_str("    routing::{get, post, put, delete},\n");
    content.push_str("};\n");

    if ctx.auth_enabled() {
        content.push_str("use axum::middleware;\n");
    }

    content.push('\n');
    content.push_str("use crate::state::AppState;\n");

    // Import each entity's handler module
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        if has_enabled_handlers(&info) {
            let module = info.module_name();
            content.push_str(&format!("use crate::handlers::{};\n", module,));
        }
    }

    if ctx.auth_enabled() {
        content.push_str("use crate::auth::middleware::require_auth;\n");
    }

    content.push('\n');

    // ── Top-level api_routes function ────────────────────────────────────

    content.push_str(&doc_comment(
        Some("Assemble all API routes.\n\nEach entity's routes are nested under its configured base path."),
        ctx,
    ));

    content.push_str("pub fn api_routes() -> Router<AppState> {\n");
    content.push_str("    Router::new()\n");

    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        if !has_enabled_handlers(&info) {
            continue;
        }

        let base_path = info.base_path();
        let fn_name = format!("{}_routes", info.snake_name());

        content.push_str(&format!(
            "        .nest(\"{}\", {}())\n",
            base_path, fn_name,
        ));
    }

    content.push_str("}\n\n");

    // ── Per-entity route functions ───────────────────────────────────────

    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        if !has_enabled_handlers(&info) {
            continue;
        }

        content.push_str(&generate_entity_routes(&info, ctx));
        content.push('\n');
    }

    GeneratedFile::new("src/routes/api.rs", content, FileType::Rust)
}

// ============================================================================
// Per-entity route builder function
// ============================================================================

/// Generate the `fn {entity}_routes() -> Router<AppState>` function for one
/// entity, respecting which CRUD operations are enabled and whether
/// individual operations require authentication.
fn generate_entity_routes(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let endpoint = match info.endpoint() {
        Some(ep) if ep.enabled => ep,
        _ => return String::new(),
    };

    let fn_name = format!("{}_routes", info.snake_name());
    let module = info.module_name();
    let pascal = info.pascal_name();

    let mut out = String::with_capacity(1024);

    out.push_str(&doc_comment(
        Some(&format!("Routes for {} endpoints.", pascal)),
        ctx,
    ));

    out.push_str(&format!("fn {}() -> Router<AppState> {{\n", fn_name,));

    // Determine which operations are enabled and whether they need auth.
    let enabled_ops = endpoint.enabled_operations();

    // We split into "public" routes and "secured" routes so we can layer
    // the auth middleware only on the secured ones.
    let auth_enabled = ctx.auth_enabled();

    // Collect operations grouped by whether they require auth
    let mut public_ops: Vec<&imortal_ir::CrudOperation> = Vec::new();
    let mut secured_ops: Vec<&imortal_ir::CrudOperation> = Vec::new();

    for op in &enabled_ops {
        let effective_security = op.security.as_ref().unwrap_or(&endpoint.global_security);

        if auth_enabled && effective_security.auth_required {
            secured_ops.push(op);
        } else {
            public_ops.push(op);
        }
    }

    let has_public = !public_ops.is_empty();
    let has_secured = !secured_ops.is_empty();

    // Build the public router
    if has_public && has_secured {
        // Need two routers merged
        out.push_str("    let public = Router::new()\n");
        for op in &public_ops {
            out.push_str(&route_line(op, &module, "        "));
        }
        out.push_str("    ;\n\n");

        out.push_str("    let secured = Router::new()\n");
        for op in &secured_ops {
            out.push_str(&route_line(op, &module, "        "));
        }
        out.push_str("        .route_layer(middleware::from_fn(require_auth))\n");
        out.push_str("    ;\n\n");

        out.push_str("    public.merge(secured)\n");
    } else if has_secured {
        // All routes are secured
        out.push_str("    Router::new()\n");
        for op in &secured_ops {
            out.push_str(&route_line(op, &module, "        "));
        }
        out.push_str("        .route_layer(middleware::from_fn(require_auth))\n");
    } else {
        // All routes are public
        out.push_str("    Router::new()\n");
        for op in &public_ops {
            out.push_str(&route_line(op, &module, "        "));
        }
    }

    out.push_str("}\n");
    out
}

/// Produce a single `.route(…)` line for a CRUD operation.
///
/// For example:
/// ```text
///     .route("/", post(user::create_user))
///     .route("/", get(user::list_users))
///     .route("/:id", get(user::get_user))
/// ```
///
/// Operations that share the same path pattern are combined into a single
/// `.route()` call using method chaining (e.g. `get(…).post(…)`).
fn route_line(op: &imortal_ir::CrudOperation, handler_module: &str, indent: &str) -> String {
    let handler_name = op.handler_name(handler_module);
    let handler_ref = format!("{}::{}", handler_module, handler_name);

    let (path, method_fn) = match op.operation_type {
        OperationType::Create => ("/", "post"),
        OperationType::ReadAll => ("/", "get"),
        OperationType::Read => ("/:id", "get"),
        OperationType::Update => ("/:id", "put"),
        OperationType::Delete => ("/:id", "delete"),
    };

    format!("{indent}.route(\"{path}\", {method_fn}({handler_ref}))\n",)
}

// ============================================================================
// Helpers
// ============================================================================

/// Check whether an entity has at least one enabled handler that should
/// appear in the router.
fn has_enabled_handlers(info: &EntityInfo) -> bool {
    info.endpoint()
        .map(|ep| ep.enabled && !ep.enabled_operations().is_empty())
        .unwrap_or(false)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_core::DataType;
    use imortal_ir::{
        AuthConfig, CrudOperation, EndpointGroup, EndpointSecurity, Entity, Field, OperationType,
        ProjectGraph,
    };
    use uuid::Uuid;

    /// Helper: project with a User entity and full CRUD endpoints.
    fn setup_full_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("test_api");

        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;

        let mut email = Field::new("email", DataType::String);
        email.required = true;
        user.fields.push(email);

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        user.fields.push(name);

        project.add_entity(user);

        let endpoint = EndpointGroup::new(user_id, "User");
        project.add_endpoint(endpoint);

        project
    }

    /// Helper: project with two entities.
    fn setup_multi_entity_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("blog_api");

        // User
        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;
        let mut email = Field::new("email", DataType::String);
        email.required = true;
        user.fields.push(email);
        project.add_entity(user);
        project.add_endpoint(EndpointGroup::new(user_id, "User"));

        // Post
        let mut post = Entity::new("Post");
        post.config.timestamps = true;
        let post_id = post.id;
        let mut title = Field::new("title", DataType::String);
        title.required = true;
        post.fields.push(title);
        project.add_entity(post);
        project.add_endpoint(EndpointGroup::new(post_id, "Post"));

        project
    }

    #[test]
    fn test_generate_routes_produces_two_files() {
        let project = setup_full_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        assert_eq!(files.len(), 2);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();
        assert!(paths.contains(&"src/routes/mod.rs".to_string()));
        assert!(paths.contains(&"src/routes/api.rs".to_string()));
    }

    #[test]
    fn test_routes_mod_creates_router() {
        let project = setup_full_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let mod_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/mod.rs")
            .unwrap();

        let content = &mod_file.content;
        assert!(content.contains("pub fn create_router("));
        assert!(content.contains("Router::new()"));
        assert!(content.contains("api::api_routes()"));
        assert!(content.contains(".with_state(state)"));
        assert!(content.contains("TraceLayer"));
        assert!(content.contains("TimeoutLayer"));
    }

    #[test]
    fn test_routes_mod_with_cors() {
        let mut project = setup_full_project();
        project.config.cors_enabled = true;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let mod_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/mod.rs")
            .unwrap();

        let content = &mod_file.content;
        assert!(content.contains("CorsLayer"));
        assert!(content.contains(".layer(cors)"));
    }

    #[test]
    fn test_routes_mod_without_cors() {
        let mut project = setup_full_project();
        project.config.cors_enabled = false;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let mod_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/mod.rs")
            .unwrap();

        let content = &mod_file.content;
        assert!(!content.contains("CorsLayer"));
    }

    #[test]
    fn test_api_routes_nests_entity_routes() {
        let project = setup_full_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;
        assert!(content.contains("pub fn api_routes()"));
        assert!(content.contains(".nest("));
        assert!(content.contains("/api/users"));
        assert!(content.contains("user_routes()"));
    }

    #[test]
    fn test_api_routes_multiple_entities() {
        let project = setup_multi_entity_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;
        assert!(content.contains("/api/users"));
        assert!(content.contains("/api/posts"));
        assert!(content.contains("user_routes()"));
        assert!(content.contains("post_routes()"));
    }

    #[test]
    fn test_entity_routes_all_crud() {
        let project = setup_full_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;

        // Should have route entries for all 5 CRUD operations
        assert!(content.contains("post(user::create_user)"));
        assert!(content.contains("get(user::list_users)"));
        assert!(content.contains("get(user::get_user)"));
        assert!(content.contains("put(user::update_user)"));
        assert!(content.contains("delete(user::delete_user)"));
    }

    #[test]
    fn test_entity_routes_read_only() {
        let mut project = ProjectGraph::new("ro_api");

        let mut item = Entity::new("Item");
        item.config.timestamps = false;
        let item_id = item.id;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        item.fields.push(name);
        project.add_entity(item);

        let endpoint = EndpointGroup::new(item_id, "Item")
            .with_operations(&[OperationType::Read, OperationType::ReadAll]);
        project.add_endpoint(endpoint);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;

        // Should have get routes
        assert!(content.contains("get(item::list_items)"));
        assert!(content.contains("get(item::get_item)"));

        // Should NOT have write routes
        assert!(!content.contains("post(item::create_item)"));
        assert!(!content.contains("put(item::update_item)"));
        assert!(!content.contains("delete(item::delete_item)"));
    }

    #[test]
    fn test_entity_routes_with_auth() {
        let mut project = ProjectGraph::new("auth_api");
        project.config.auth = AuthConfig::jwt();

        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;
        let mut email = Field::new("email", DataType::String);
        email.required = true;
        user.fields.push(email);
        project.add_entity(user);

        // All operations require auth
        let endpoint = EndpointGroup::new(user_id, "User").secured();
        project.add_endpoint(endpoint);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;

        // Should import middleware
        assert!(content.contains("use axum::middleware;"));
        assert!(content.contains("use crate::auth::middleware::require_auth;"));

        // Should have route_layer for auth
        assert!(content.contains("route_layer(middleware::from_fn(require_auth))"));
    }

    #[test]
    fn test_entity_routes_mixed_security() {
        let mut project = ProjectGraph::new("mixed_api");
        project.config.auth = AuthConfig::jwt();

        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;
        let mut email = Field::new("email", DataType::String);
        email.required = true;
        user.fields.push(email);
        project.add_entity(user);

        // ReadAll is public, everything else requires auth
        let mut endpoint = EndpointGroup::new(user_id, "User");
        endpoint.global_security.auth_required = true;
        endpoint.set_operation_security(OperationType::ReadAll, EndpointSecurity::open());
        project.add_endpoint(endpoint);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;

        // Should have both public and secured sections
        assert!(content.contains("let public = Router::new()"));
        assert!(content.contains("let secured = Router::new()"));
        assert!(content.contains("public.merge(secured)"));
    }

    #[test]
    fn test_no_routes_for_entity_without_endpoint() {
        let mut project = ProjectGraph::new("no_ep_api");

        let mut entity = Entity::new("Internal");
        entity.config.timestamps = false;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);
        project.add_entity(entity);

        // No endpoint group added!

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;

        // Should NOT reference this entity
        assert!(!content.contains("internal"));
    }

    #[test]
    fn test_no_routes_for_disabled_endpoint() {
        let mut project = ProjectGraph::new("disabled_api");

        let mut entity = Entity::new("Widget");
        entity.config.timestamps = false;
        let widget_id = entity.id;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);
        project.add_entity(entity);

        let endpoint = EndpointGroup::new(widget_id, "Widget").disabled();
        project.add_endpoint(endpoint);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;
        assert!(!content.contains("widget"));
    }

    #[test]
    fn test_route_line_paths() {
        let create_op = CrudOperation::new(OperationType::Create);
        let read_op = CrudOperation::new(OperationType::Read);
        let read_all_op = CrudOperation::new(OperationType::ReadAll);
        let update_op = CrudOperation::new(OperationType::Update);
        let delete_op = CrudOperation::new(OperationType::Delete);

        let create_line = route_line(&create_op, "user", "        ");
        assert!(create_line.contains("route(\"/\""));
        assert!(create_line.contains("post(user::create_user)"));

        let read_line = route_line(&read_op, "user", "        ");
        assert!(read_line.contains("route(\"/:id\""));
        assert!(read_line.contains("get(user::get_user)"));

        let read_all_line = route_line(&read_all_op, "user", "        ");
        assert!(read_all_line.contains("route(\"/\""));
        assert!(read_all_line.contains("get(user::list_users)"));

        let update_line = route_line(&update_op, "user", "        ");
        assert!(update_line.contains("route(\"/:id\""));
        assert!(update_line.contains("put(user::update_user)"));

        let delete_line = route_line(&delete_op, "user", "        ");
        assert!(delete_line.contains("route(\"/:id\""));
        assert!(delete_line.contains("delete(user::delete_user)"));
    }

    #[test]
    fn test_handler_imports_present() {
        let project = setup_multi_entity_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;

        assert!(content.contains("use crate::handlers::user;"));
        assert!(content.contains("use crate::handlers::post;"));
        assert!(content.contains("use crate::state::AppState;"));
    }

    #[test]
    fn test_routes_import_routing_methods() {
        let project = setup_full_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_routes(&ctx);

        let api_file = files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .unwrap();

        let content = &api_file.content;

        assert!(content.contains("routing::{get, post, put, delete}"));
    }
}
