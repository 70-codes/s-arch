//! # Test File Generator
//!
//! Generates `tests/api_tests.rs` for the generated project. The test file
//! contains integration tests that verify the generated API endpoints work
//! correctly end-to-end.
//!
//! ## Generated Tests
//!
//! For each entity with enabled endpoints, the generator produces:
//!
//! - Health check / server startup test
//! - CRUD operation tests (create, read, list, update, delete)
//! - Validation failure tests (if validator is used)
//! - Not-found tests (404 for missing resources)
//!
//! ## Test Infrastructure
//!
//! The generated tests use:
//!
//! - `tokio::test` for async test execution
//! - `reqwest` for HTTP client requests
//! - `serial_test` for tests that share database state
//!
//! ## Usage
//!
//! Tests are only generated when `ctx.generate_tests()` returns `true`
//! (controlled by the `GeneratorConfig`).
//!
//! ```bash
//! # Run the generated tests
//! cargo test
//!
//! # Run with output
//! cargo test -- --nocapture
//! ```

use imortal_ir::OperationType;

use crate::context::{EntityInfo, GenerationContext};
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate all test files for the generated project.
///
/// Currently produces a single `tests/api_tests.rs` file. Returns an empty
/// `Vec` if test generation is disabled.
pub fn generate_tests(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    if !ctx.generate_tests() {
        return Vec::new();
    }

    vec![generate_api_tests(ctx)]
}

// ============================================================================
// tests/api_tests.rs
// ============================================================================

fn generate_api_tests(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(8192);

    content.push_str(&file_header(
        "Integration tests for the generated API endpoints.",
    ));

    // ── Imports ──────────────────────────────────────────────────────────
    content.push_str(&generate_test_imports(ctx));
    content.push('\n');

    // ── Test helpers ─────────────────────────────────────────────────────
    content.push_str(&generate_test_helpers(ctx));
    content.push('\n');

    // ── Health check test ────────────────────────────────────────────────
    content.push_str(&generate_health_check_test(ctx));
    content.push('\n');

    // ── Per-entity tests ─────────────────────────────────────────────────
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);

        // Only generate tests for entities with enabled endpoints
        let endpoint = match info.endpoint() {
            Some(ep) if ep.enabled && !ep.enabled_operations().is_empty() => ep,
            _ => continue,
        };

        let enabled_ops: Vec<OperationType> = endpoint
            .enabled_operations()
            .iter()
            .map(|op| op.operation_type)
            .collect();

        content.push_str(&generate_entity_test_module(&info, ctx, &enabled_ops));
        content.push('\n');
    }

    GeneratedFile::new("tests/api_tests.rs", content, FileType::Rust)
}

// ============================================================================
// Imports
// ============================================================================

fn generate_test_imports(ctx: &GenerationContext) -> String {
    let crate_ident = ctx.package_name().replace('-', "_");

    let mut out = String::with_capacity(1024);

    out.push_str("#![allow(unused_imports, dead_code)]\n\n");

    out.push_str("use reqwest::StatusCode;\n");
    out.push_str("use serde_json::{json, Value};\n");
    out.push_str("use std::net::TcpListener;\n");
    out.push('\n');

    // Import from the generated crate
    out.push_str(&format!("use {}::config::Config;\n", crate_ident));
    out.push_str(&format!("use {}::routes::create_router;\n", crate_ident));
    out.push_str(&format!("use {}::state::AppState;\n", crate_ident));
    out.push('\n');

    out
}

// ============================================================================
// Test helpers
// ============================================================================

fn generate_test_helpers(ctx: &GenerationContext) -> String {
    let mut out = String::with_capacity(4096);

    out.push_str(
        "\
// ============================================================================
// Test Helpers
// ============================================================================

/// Base URL for the test server.
///
/// The test server is started on a random available port to avoid conflicts
/// with other running instances.
struct TestServer {
    /// The base URL including the random port (e.g. `http://127.0.0.1:54321`).
    base_url: String,
}

impl TestServer {
    /// Start a test server on a random port.
    ///
    /// This spins up a full Axum server in a background tokio task and
    /// returns a `TestServer` handle with the base URL for making requests.
    async fn start() -> Self {
        // Load .env for test database URL
        dotenvy::dotenv().ok();

        let config = Config::from_env();

        // Bind to a random port
        let listener = TcpListener::bind(\"127.0.0.1:0\")
            .expect(\"failed to bind to random port\");
        let port = listener.local_addr().unwrap().port();
        let base_url = format!(\"http://127.0.0.1:{}\", port);

        // Connect to database
        let db = sea_orm::Database::connect(config.database_connect_options())
            .await
            .expect(\"failed to connect to test database\");

        let state = AppState::new(db, config);
        let router = create_router(state);

        // Convert std TcpListener to tokio TcpListener
        listener.set_nonblocking(true).unwrap();
        let tokio_listener = tokio::net::TcpListener::from_std(listener).unwrap();

        // Start server in background
        tokio::spawn(async move {
            axum::serve(tokio_listener, router)
                .await
                .expect(\"server error\");
        });

        // Give the server a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Self { base_url }
    }

    /// Build a URL for the given API path.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let url = server.url(\"/api/users\");
    /// // => \"http://127.0.0.1:54321/api/users\"
    /// ```
    fn url(&self, path: &str) -> String {
        format!(\"{}{}\", self.base_url, path)
    }

    /// Create a reqwest client for making test requests.
    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

/// Create a test HTTP client (convenience function).
fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect(\"failed to create HTTP client\")
}

",
    );

    // Add auth helper if auth is enabled
    if ctx.auth_enabled() {
        let crate_ident = ctx.package_name().replace('-', "_");
        out.push_str(&format!(
            "\
/// Create a JWT token for testing authenticated endpoints.
///
/// This creates a token with the given user ID and roles, signed with the
/// JWT secret from the environment.
fn create_test_token(user_id: &str, roles: Vec<String>) -> String {{
    use {crate_ident}::auth::jwt::{{Claims, create_token}};

    let secret = std::env::var(\"JWT_SECRET\")
        .unwrap_or_else(|_| \"test-secret-for-integration-tests\".to_string());

    let claims = Claims::new(user_id, \"test@example.com\", roles, 1);
    create_token(&claims, &secret).expect(\"failed to create test token\")
}}

/// Create an admin token for testing.
fn admin_token() -> String {{
    create_test_token(\"test-admin-id\", vec![\"admin\".to_string()])
}}

/// Create a regular user token for testing.
fn user_token() -> String {{
    create_test_token(\"test-user-id\", vec![\"user\".to_string()])
}}

"
        ));
    }

    out
}

// ============================================================================
// Health check test
// ============================================================================

fn generate_health_check_test(ctx: &GenerationContext) -> String {
    let mut out = String::with_capacity(512);

    out.push_str(
        "\
// ============================================================================
// Server Health Check
// ============================================================================

/// Verify that the test server starts up and is reachable.
///
/// This test ensures the basic infrastructure (database connection, router
/// assembly, TCP binding) works correctly before running endpoint-specific
/// tests.
#[tokio::test]
async fn test_server_starts() {
    let server = TestServer::start().await;
    let client = test_client();

    // The root path may or may not return 200 depending on whether a
    // root handler is configured. We just verify the server responds.
    let result = client.get(&server.url(\"/\")).send().await;

    // The server should respond (even if it's a 404 for the root path)
    assert!(result.is_ok(), \"Server should respond to requests\");
}

",
    );

    out
}

// ============================================================================
// Per-entity test module
// ============================================================================

fn generate_entity_test_module(
    info: &EntityInfo,
    ctx: &GenerationContext,
    enabled_ops: &[OperationType],
) -> String {
    let module_name = format!("{}_tests", info.snake_name());
    let pascal = info.pascal_name();
    let base_path = info.base_path();
    let create_dto = GenerationContext::create_dto_name(&info.entity.name);
    let response_dto = GenerationContext::response_dto_name(&info.entity.name);

    let create_fields = info.create_fields();

    let mut out = String::with_capacity(4096);

    out.push_str(&format!(
        "\
// ============================================================================
// {} Tests
// ============================================================================

/// Integration tests for {} CRUD endpoints.
mod {} {{
    use super::*;

",
        pascal, pascal, module_name,
    ));

    // ── Build a sample JSON payload for creating an entity ────────────────
    let sample_json = build_sample_create_json(info, ctx);

    // ── List test (ReadAll) ──────────────────────────────────────────────
    if enabled_ops.contains(&OperationType::ReadAll) {
        out.push_str(&format!(
            "\
    /// Test listing all {plural}.
    ///
    /// GET {base_path}
    #[tokio::test]
    async fn test_list_{plural}() {{
        let server = TestServer::start().await;
        let client = test_client();

        let response = client
            .get(&server.url(\"{base_path}\"))
            .send()
            .await
            .expect(\"request failed\");

        assert_eq!(
            response.status(),
            StatusCode::OK,
            \"List endpoint should return 200\"
        );

        let body: Value = response.json().await.expect(\"invalid JSON\");
        assert!(body.get(\"items\").is_some(), \"Response should have 'items' field\");
        assert!(body.get(\"total\").is_some(), \"Response should have 'total' field\");
        assert!(body.get(\"page\").is_some(), \"Response should have 'page' field\");
    }}

",
            plural = info.plural_name(),
            base_path = base_path,
        ));

        // Pagination test
        out.push_str(&format!(
            "\
    /// Test list endpoint with pagination parameters.
    ///
    /// GET {base_path}?page=1&per_page=5
    #[tokio::test]
    async fn test_list_{plural}_with_pagination() {{
        let server = TestServer::start().await;
        let client = test_client();

        let response = client
            .get(&server.url(\"{base_path}?page=1&per_page=5\"))
            .send()
            .await
            .expect(\"request failed\");

        assert_eq!(response.status(), StatusCode::OK);

        let body: Value = response.json().await.expect(\"invalid JSON\");
        let per_page = body.get(\"per_page\").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(per_page <= 5, \"per_page should respect the requested limit\");
    }}

",
            plural = info.plural_name(),
            base_path = base_path,
        ));
    }

    // ── Create test ──────────────────────────────────────────────────────
    if enabled_ops.contains(&OperationType::Create) {
        out.push_str(&format!(
            "\
    /// Test creating a new {snake}.
    ///
    /// POST {base_path}
    #[tokio::test]
    async fn test_create_{snake}() {{
        let server = TestServer::start().await;
        let client = test_client();

        let payload = {sample_json};

        let response = client
            .post(&server.url(\"{base_path}\"))
            .json(&payload)
            .send()
            .await
            .expect(\"request failed\");

        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            \"Create endpoint should return 201\"
        );

        let body: Value = response.json().await.expect(\"invalid JSON\");
        assert!(body.get(\"id\").is_some(), \"Response should contain an 'id' field\");
    }}

",
            snake = info.snake_name(),
            base_path = base_path,
            sample_json = sample_json,
        ));

        // Validation failure test
        out.push_str(&format!(
            "\
    /// Test that creating with an empty body returns a validation error.
    ///
    /// POST {base_path} with empty JSON {{}}
    #[tokio::test]
    async fn test_create_{snake}_validation_error() {{
        let server = TestServer::start().await;
        let client = test_client();

        let response = client
            .post(&server.url(\"{base_path}\"))
            .json(&json!({{}}))
            .send()
            .await
            .expect(\"request failed\");

        // Should fail with 400 or 422
        let status = response.status().as_u16();
        assert!(
            status == 400 || status == 422,
            \"Empty payload should fail validation, got {{}}\",
            status,
        );
    }}

",
            snake = info.snake_name(),
            base_path = base_path,
        ));
    }

    // ── Get test (Read) ──────────────────────────────────────────────────
    if enabled_ops.contains(&OperationType::Read) {
        let pk_type = info.pk_rust_type();

        out.push_str(&format!(
            "\
    /// Test getting a non-existent {snake} returns 404.
    ///
    /// GET {base_path}/{{non_existent_id}}
    #[tokio::test]
    async fn test_get_{snake}_not_found() {{
        let server = TestServer::start().await;
        let client = test_client();

        // Use a random UUID that almost certainly doesn't exist
        let fake_id = uuid::Uuid::new_v4();

        let response = client
            .get(&server.url(&format!(\"{base_path}/{{}}\", fake_id)))
            .send()
            .await
            .expect(\"request failed\");

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            \"Non-existent resource should return 404\"
        );

        let body: Value = response.json().await.expect(\"invalid JSON\");
        assert!(
            body.get(\"error\").is_some(),
            \"Error response should have an 'error' field\"
        );
    }}

",
            snake = info.snake_name(),
            base_path = base_path,
        ));
    }

    // ── Delete not-found test ────────────────────────────────────────────
    if enabled_ops.contains(&OperationType::Delete) {
        out.push_str(&format!(
            "\
    /// Test deleting a non-existent {snake} returns 404.
    ///
    /// DELETE {base_path}/{{non_existent_id}}
    #[tokio::test]
    async fn test_delete_{snake}_not_found() {{
        let server = TestServer::start().await;
        let client = test_client();

        let fake_id = uuid::Uuid::new_v4();

        let response = client
            .delete(&server.url(&format!(\"{base_path}/{{}}\", fake_id)))
            .send()
            .await
            .expect(\"request failed\");

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            \"Deleting non-existent resource should return 404\"
        );
    }}

",
            snake = info.snake_name(),
            base_path = base_path,
        ));
    }

    // ── Full CRUD lifecycle test ─────────────────────────────────────────
    if enabled_ops.contains(&OperationType::Create)
        && enabled_ops.contains(&OperationType::Read)
        && enabled_ops.contains(&OperationType::Delete)
    {
        out.push_str(&format!(
            "\
    /// Test the full create → read → delete lifecycle for a {snake}.
    ///
    /// This test verifies that:
    /// 1. Creating a resource returns 201 with an ID
    /// 2. Reading that resource by ID returns 200 with correct data
    /// 3. Deleting that resource returns 204
    /// 4. Reading the deleted resource returns 404
    #[tokio::test]
    async fn test_{snake}_crud_lifecycle() {{
        let server = TestServer::start().await;
        let client = test_client();

        // 1. Create
        let payload = {sample_json};

        let create_resp = client
            .post(&server.url(\"{base_path}\"))
            .json(&payload)
            .send()
            .await
            .expect(\"create request failed\");

        assert_eq!(create_resp.status(), StatusCode::CREATED);

        let created: Value = create_resp.json().await.expect(\"invalid JSON\");
        let id = created
            .get(\"id\")
            .and_then(|v| v.as_str())
            .expect(\"created resource should have an 'id'\");

        // 2. Read
        let get_resp = client
            .get(&server.url(&format!(\"{base_path}/{{}}\", id)))
            .send()
            .await
            .expect(\"get request failed\");

        assert_eq!(get_resp.status(), StatusCode::OK);

        let fetched: Value = get_resp.json().await.expect(\"invalid JSON\");
        assert_eq!(
            fetched.get(\"id\").and_then(|v| v.as_str()),
            Some(id),
            \"Fetched resource should have the same ID\"
        );

        // 3. Delete
        let delete_resp = client
            .delete(&server.url(&format!(\"{base_path}/{{}}\", id)))
            .send()
            .await
            .expect(\"delete request failed\");

        assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

        // 4. Verify deleted
        let verify_resp = client
            .get(&server.url(&format!(\"{base_path}/{{}}\", id)))
            .send()
            .await
            .expect(\"verify request failed\");

        assert_eq!(
            verify_resp.status(),
            StatusCode::NOT_FOUND,
            \"Deleted resource should return 404\"
        );
    }}

",
            snake = info.snake_name(),
            base_path = base_path,
            sample_json = sample_json,
        ));
    }

    // ── Update test ──────────────────────────────────────────────────────
    if enabled_ops.contains(&OperationType::Create) && enabled_ops.contains(&OperationType::Update)
    {
        let update_fields = info.update_fields();
        let update_json = build_sample_update_json(&update_fields);

        out.push_str(&format!(
            "\
    /// Test updating an existing {snake}.
    ///
    /// POST {base_path} (create) → PUT {base_path}/{{id}} (update) → GET (verify)
    #[tokio::test]
    async fn test_update_{snake}() {{
        let server = TestServer::start().await;
        let client = test_client();

        // Create first
        let payload = {sample_json};

        let create_resp = client
            .post(&server.url(\"{base_path}\"))
            .json(&payload)
            .send()
            .await
            .expect(\"create request failed\");

        assert_eq!(create_resp.status(), StatusCode::CREATED);

        let created: Value = create_resp.json().await.expect(\"invalid JSON\");
        let id = created
            .get(\"id\")
            .and_then(|v| v.as_str())
            .expect(\"created resource should have an 'id'\");

        // Update
        let update_payload = {update_json};

        let update_resp = client
            .put(&server.url(&format!(\"{base_path}/{{}}\", id)))
            .json(&update_payload)
            .send()
            .await
            .expect(\"update request failed\");

        assert_eq!(
            update_resp.status(),
            StatusCode::OK,
            \"Update should return 200\"
        );

        let updated: Value = update_resp.json().await.expect(\"invalid JSON\");
        assert_eq!(
            updated.get(\"id\").and_then(|v| v.as_str()),
            Some(id),
            \"Updated resource should have the same ID\"
        );
    }}

",
            snake = info.snake_name(),
            base_path = base_path,
            sample_json = sample_json,
            update_json = update_json,
        ));
    }

    // ── Update not-found test ────────────────────────────────────────────
    if enabled_ops.contains(&OperationType::Update) {
        out.push_str(&format!(
            "\
    /// Test updating a non-existent {snake} returns 404.
    ///
    /// PUT {base_path}/{{non_existent_id}}
    #[tokio::test]
    async fn test_update_{snake}_not_found() {{
        let server = TestServer::start().await;
        let client = test_client();

        let fake_id = uuid::Uuid::new_v4();
        let payload = json!({{  }});

        let response = client
            .put(&server.url(&format!(\"{base_path}/{{}}\", fake_id)))
            .json(&payload)
            .send()
            .await
            .expect(\"request failed\");

        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            \"Updating non-existent resource should return 404\"
        );
    }}

",
            snake = info.snake_name(),
            base_path = base_path,
        ));
    }

    // Close module
    out.push_str("}\n");

    out
}

// ============================================================================
// Sample data helpers
// ============================================================================

/// Build a sample JSON object for creating an entity, with sensible test values
/// based on field types.
fn build_sample_create_json(info: &EntityInfo, _ctx: &GenerationContext) -> String {
    let fields = info.create_fields();

    let mut entries: Vec<String> = Vec::new();

    for field in &fields {
        let name = GenerationContext::snake(&field.name);
        let value = sample_value_for_type(&field.data_type, &name);
        entries.push(format!("            \"{}\": {}", name, value));
    }

    if entries.is_empty() {
        return "json!({})".to_string();
    }

    format!("json!({{\n{}\n        }})", entries.join(",\n"),)
}

/// Build a sample JSON object for updating an entity (partial update).
fn build_sample_update_json(fields: &[&imortal_ir::Field]) -> String {
    let mut entries: Vec<String> = Vec::new();

    // Only include the first field (partial update)
    if let Some(field) = fields.first() {
        let name = GenerationContext::snake(&field.name);
        let value = sample_value_for_type(&field.data_type, &format!("updated_{}", name));
        entries.push(format!("            \"{}\": {}", name, value));
    }

    if entries.is_empty() {
        return "json!({})".to_string();
    }

    format!("json!({{\n{}\n        }})", entries.join(",\n"),)
}

/// Generate a sensible sample value for a given data type.
///
/// These are used to build test request payloads.
fn sample_value_for_type(dt: &imortal_core::DataType, field_name: &str) -> String {
    use imortal_core::DataType;

    match dt {
        DataType::String | DataType::Text => {
            // Try to generate contextually appropriate values
            if field_name.contains("email") {
                "\"test@example.com\"".to_string()
            } else if field_name.contains("url") || field_name.contains("website") {
                "\"https://example.com\"".to_string()
            } else if field_name.contains("phone") {
                "\"+1234567890\"".to_string()
            } else if field_name.contains("password") {
                "\"TestPassword123!\"".to_string()
            } else {
                format!("\"test_{}\"", field_name)
            }
        }
        DataType::Int32 | DataType::Int64 => "42".to_string(),
        DataType::Float32 | DataType::Float64 => "3.14".to_string(),
        DataType::Bool => "true".to_string(),
        DataType::Uuid => format!(
            "\"{}\"",
            uuid::Uuid::nil() // Use nil UUID for predictable test data
        ),
        DataType::DateTime => "\"2026-01-29T12:00:00Z\"".to_string(),
        DataType::Date => "\"2026-01-29\"".to_string(),
        DataType::Time => "\"12:00:00\"".to_string(),
        DataType::Json => "{}".to_string(),
        DataType::Bytes => "\"dGVzdA==\"".to_string(), // base64 of "test"
        DataType::Optional(inner) => sample_value_for_type(inner, field_name),
        DataType::Array(inner) => {
            format!("[{}]", sample_value_for_type(inner, field_name))
        }
        DataType::Reference { .. } => format!("\"{}\"", uuid::Uuid::nil()),
        DataType::Enum { variants, .. } => {
            if let Some(first) = variants.first() {
                format!("\"{}\"", first)
            } else {
                "\"variant\"".to_string()
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_core::DataType;
    use imortal_ir::{
        AuthConfig, CrudOperation, EndpointGroup, Entity, Field, OperationType, ProjectGraph,
    };
    use uuid::Uuid;

    /// Helper: create a project with a User entity and full CRUD endpoints.
    fn setup_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("test_api");

        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;

        let mut email = Field::new("email", DataType::String);
        email.required = true;
        email.unique = true;
        user.fields.push(email);

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        user.fields.push(name);

        project.add_entity(user);

        let endpoint = EndpointGroup::new(user_id, "User");
        project.add_endpoint(endpoint);

        project
    }

    /// Helper: project with read-only endpoints.
    fn setup_read_only_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("read_api");

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

        project
    }

    #[test]
    fn test_generate_tests_produces_one_file() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path.to_string_lossy(), "tests/api_tests.rs");
    }

    #[test]
    fn test_generate_tests_disabled() {
        let project = setup_project();
        let config = crate::GeneratorConfig::new().without_tests();
        let ctx = GenerationContext::from_project(&project, config);
        let files = generate_tests(&ctx);

        assert!(files.is_empty());
    }

    #[test]
    fn test_test_file_has_imports() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("use reqwest::StatusCode;"));
        assert!(content.contains("use serde_json::{json, Value};"));
        assert!(content.contains("use std::net::TcpListener;"));
    }

    #[test]
    fn test_test_file_has_test_server() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("struct TestServer"));
        assert!(content.contains("async fn start()"));
        assert!(content.contains("fn url(&self"));
        assert!(content.contains("fn client(&self"));
        assert!(content.contains("fn test_client()"));
    }

    #[test]
    fn test_test_file_has_health_check() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("#[tokio::test]"));
        assert!(content.contains("async fn test_server_starts()"));
    }

    #[test]
    fn test_test_file_has_entity_tests() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        // Should have a test module for users
        assert!(content.contains("mod user_tests"));

        // Should have all CRUD tests
        assert!(content.contains("test_list_users"), "Missing list test");
        assert!(content.contains("test_create_user"), "Missing create test");
        assert!(
            content.contains("test_get_user_not_found"),
            "Missing get not-found test"
        );
        assert!(
            content.contains("test_delete_user_not_found"),
            "Missing delete not-found test"
        );
        assert!(
            content.contains("test_user_crud_lifecycle"),
            "Missing CRUD lifecycle test"
        );
        assert!(content.contains("test_update_user"), "Missing update test");
    }

    #[test]
    fn test_test_file_list_test_checks_pagination() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("test_list_users_with_pagination"));
        assert!(content.contains("page=1&per_page=5"));
    }

    #[test]
    fn test_test_file_create_validation_error() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("test_create_user_validation_error"));
        assert!(content.contains("json!({})"));
        assert!(content.contains("400 || status == 422"));
    }

    #[test]
    fn test_test_file_crud_lifecycle() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("test_user_crud_lifecycle"));
        // Should create, read, delete, then verify 404
        assert!(content.contains("StatusCode::CREATED"));
        assert!(content.contains("StatusCode::OK"));
        assert!(content.contains("StatusCode::NO_CONTENT"));
        assert!(content.contains("StatusCode::NOT_FOUND"));
    }

    #[test]
    fn test_test_file_update_not_found() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("test_update_user_not_found"));
    }

    #[test]
    fn test_test_file_uses_correct_paths() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("/api/users"));
    }

    #[test]
    fn test_test_file_read_only_entity() {
        let project = setup_read_only_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        // Should have list and get tests
        assert!(content.contains("test_list_items"));
        assert!(content.contains("test_get_item_not_found"));

        // Should NOT have create, update, delete tests
        assert!(!content.contains("test_create_item"));
        assert!(!content.contains("test_update_item"));
        assert!(!content.contains("test_delete_item"));
        assert!(!content.contains("test_item_crud_lifecycle"));
    }

    #[test]
    fn test_test_file_with_auth_has_token_helpers() {
        let mut project = setup_project();
        project.config.auth = AuthConfig::jwt();

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("fn create_test_token("));
        assert!(content.contains("fn admin_token()"));
        assert!(content.contains("fn user_token()"));
        assert!(content.contains("JWT_SECRET"));
    }

    #[test]
    fn test_test_file_without_auth_no_token_helpers() {
        let mut project = setup_project();
        project.config.auth.enabled = false;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(!content.contains("fn create_test_token("));
        assert!(!content.contains("fn admin_token()"));
    }

    #[test]
    fn test_sample_values() {
        assert!(sample_value_for_type(&DataType::String, "email").contains("@example.com"));
        assert!(sample_value_for_type(&DataType::String, "name").contains("test_name"));
        assert_eq!(sample_value_for_type(&DataType::Int32, "count"), "42");
        assert_eq!(sample_value_for_type(&DataType::Float64, "price"), "3.14");
        assert_eq!(sample_value_for_type(&DataType::Bool, "active"), "true");
        assert!(sample_value_for_type(&DataType::Uuid, "ref_id").contains("00000000"));
        assert!(sample_value_for_type(&DataType::DateTime, "created_at").contains("2026"));
        assert_eq!(
            sample_value_for_type(&DataType::Optional(Box::new(DataType::String)), "bio"),
            "\"test_bio\""
        );
        assert!(
            sample_value_for_type(&DataType::Array(Box::new(DataType::Int32)), "scores")
                .starts_with("[")
        );
    }

    #[test]
    fn test_sample_create_json() {
        let mut project = ProjectGraph::new("test");

        let mut entity = Entity::new("Product");
        entity.config.timestamps = false;

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);

        let mut price = Field::new("price", DataType::Float64);
        price.required = true;
        entity.fields.push(price);

        project.add_entity(entity);

        let ctx = GenerationContext::from_project_default(&project);
        let e = ctx.entities().first().unwrap();
        let info = EntityInfo::new(e, &ctx);

        let json = build_sample_create_json(&info, &ctx);

        assert!(json.contains("json!"));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"price\""));
        assert!(json.contains("test_name"));
        assert!(json.contains("3.14"));
    }

    #[test]
    fn test_sample_update_json() {
        let name_field = Field::new("name", DataType::String);
        let price_field = Field::new("price", DataType::Float64);
        let fields: Vec<&Field> = vec![&name_field, &price_field];

        let json = build_sample_update_json(&fields);

        assert!(json.contains("json!"));
        // Should only include the first field for a partial update
        assert!(json.contains("\"name\""));
    }

    #[test]
    fn test_sample_create_json_empty_entity() {
        let mut project = ProjectGraph::new("test");
        let entity = Entity::new("Empty");
        project.add_entity(entity);

        let ctx = GenerationContext::from_project_default(&project);
        let e = ctx.entities().first().unwrap();
        let info = EntityInfo::new(e, &ctx);

        let json = build_sample_create_json(&info, &ctx);
        assert_eq!(json, "json!({})");
    }

    #[test]
    fn test_no_entity_tests_without_endpoints() {
        let mut project = ProjectGraph::new("no_ep");

        let mut entity = Entity::new("Internal");
        entity.config.timestamps = false;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);
        project.add_entity(entity);

        // No endpoints configured
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        // Should NOT have a test module for this entity
        assert!(!content.contains("mod internal_tests"));
    }

    #[test]
    fn test_email_field_gets_email_sample() {
        let mut project = ProjectGraph::new("test");

        let mut entity = Entity::new("User");
        entity.config.timestamps = false;
        let entity_id = entity.id;

        let mut email = Field::new("email", DataType::String);
        email.required = true;
        entity.fields.push(email);

        project.add_entity(entity);
        project.add_endpoint(EndpointGroup::new(entity_id, "User"));

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        // The email field should use an email-like test value
        assert!(content.contains("test@example.com"));
    }

    #[test]
    fn test_password_field_gets_password_sample() {
        let mut project = ProjectGraph::new("test");

        let mut entity = Entity::new("User");
        entity.config.timestamps = false;
        let entity_id = entity.id;

        let mut pw = Field::new("password", DataType::String);
        pw.required = true;
        entity.fields.push(pw);

        project.add_entity(entity);
        project.add_endpoint(EndpointGroup::new(entity_id, "User"));

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("TestPassword123!"));
    }

    #[test]
    fn test_test_server_uses_random_port() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("127.0.0.1:0"));
        assert!(content.contains("local_addr().unwrap().port()"));
    }

    #[test]
    fn test_test_server_connects_to_database() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("sea_orm::Database::connect"));
        assert!(content.contains("database_connect_options()"));
    }

    #[test]
    fn test_file_header_present() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_tests(&ctx);
        let content = &files[0].content;

        assert!(content.contains("Auto-generated by Immortal Engine"));
        assert!(content.contains("DO NOT EDIT"));
    }

    #[test]
    fn test_enum_sample_value() {
        let dt = DataType::Enum {
            name: "Status".to_string(),
            variants: vec!["Active".to_string(), "Inactive".to_string()],
        };
        let value = sample_value_for_type(&dt, "status");
        assert_eq!(value, "\"Active\"");
    }

    #[test]
    fn test_enum_sample_value_empty_variants() {
        let dt = DataType::Enum {
            name: "Empty".to_string(),
            variants: vec![],
        };
        let value = sample_value_for_type(&dt, "status");
        assert_eq!(value, "\"variant\"");
    }
}
