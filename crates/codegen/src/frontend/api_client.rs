//! # API Client Generator
//!
//! Generates a type-safe HTTP client for the frontend application that
//! communicates with the backend REST API using `reqwest`.
//!
//! ## Generated Files
//!
//! - `frontend/src/api/mod.rs` — module declarations and re-exports
//! - `frontend/src/api/client.rs` — `ApiClient` struct with per-entity CRUD methods
//!
//! ## Architecture
//!
//! The generated `ApiClient` wraps a `reqwest::Client` and provides methods
//! like `list_users()`, `get_user(id)`, `create_user(dto)`, `update_user(id, dto)`,
//! `delete_user(id)` for each entity that has configured endpoints.
//!
//! All methods return `Result<T, ApiError>` where `T` is the expected response
//! type and `ApiError` is a structured error from the `shared` crate.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let client = ApiClient::new();
//! let users = client.list_users(1, 20).await?;
//! let user = client.get_user("some-uuid").await?;
//! let created = client.create_user(&payload).await?;
//! ```

use imortal_ir::OperationType;

use crate::context::{EntityInfo, GenerationContext};
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate the API client module for the frontend.
///
/// Produces:
/// - `frontend/src/api/mod.rs`
/// - `frontend/src/api/client.rs`
pub fn generate_api_client(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    vec![generate_api_mod(ctx), generate_client(ctx)]
}

// ============================================================================
// api/mod.rs
// ============================================================================

fn generate_api_mod(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(512);

    content.push_str(&file_header("API client module for backend communication."));

    content.push_str("pub mod client;\n\n");

    content.push_str("// Re-exports for convenience\n");
    content.push_str("pub use client::ApiClient;\n");

    GeneratedFile::new("frontend/src/api/mod.rs", content, FileType::Rust)
}

// ============================================================================
// api/client.rs
// ============================================================================

fn generate_client(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(16384);

    content.push_str(&file_header(
        "Type-safe API client for communicating with the backend REST API.",
    ));

    // ── Imports ──────────────────────────────────────────────────────────
    content.push_str(
        "\
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use shared::{ApiError, PaginatedResponse};
",
    );

    // Import per-entity DTOs
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        if info.endpoint().is_none() {
            continue;
        }

        let create_dto = GenerationContext::create_dto_name(&entity.name);
        let update_dto = GenerationContext::update_dto_name(&entity.name);
        let response_dto = GenerationContext::response_dto_name(&entity.name);

        content.push_str(&format!(
            "use shared::{{{create_dto}, {update_dto}, {response_dto}}};\n"
        ));
    }

    content.push('\n');

    // ── ClientError ──────────────────────────────────────────────────────
    content.push_str(
        r#"// ============================================================================
// Error Type
// ============================================================================

/// Errors that can occur when making API requests.
#[derive(Debug, Error)]
pub enum ClientError {
    /// HTTP request failed (network error, timeout, etc.).
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// The server returned an error response (4xx or 5xx).
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Machine-readable error code from the response body.
        code: String,
        /// Human-readable error message from the response body.
        message: String,
    },

    /// Failed to deserialise the response body.
    #[error("Failed to parse response: {0}")]
    Parse(String),
}

impl ClientError {
    /// Create an `Api` error from status code and body.
    fn from_api_error(status: u16, api_error: ApiError) -> Self {
        Self::Api {
            status,
            code: api_error.error,
            message: api_error.message,
        }
    }

    /// Whether this is a "not found" (404) error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Api { status: 404, .. })
    }

    /// Whether this is an "unauthorized" (401) error.
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Self::Api { status: 401, .. })
    }

    /// Whether this is a "forbidden" (403) error.
    pub fn is_forbidden(&self) -> bool {
        matches!(self, Self::Api { status: 403, .. })
    }

    /// Whether this is a validation error (422).
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Api { status: 422, .. })
    }

    /// Whether this is a conflict error (409).
    pub fn is_conflict(&self) -> bool {
        matches!(self, Self::Api { status: 409, .. })
    }

    /// Get the user-facing error message.
    pub fn user_message(&self) -> String {
        match self {
            Self::Request(e) => {
                if e.is_timeout() {
                    "Request timed out. Please try again.".to_string()
                } else if e.is_connect() {
                    "Unable to connect to the server. Please check your connection.".to_string()
                } else {
                    "An unexpected network error occurred.".to_string()
                }
            }
            Self::Api { message, .. } => message.clone(),
            Self::Parse(_) => "Received an unexpected response from the server.".to_string(),
        }
    }
}

"#,
    );

    // ── ApiClient struct ─────────────────────────────────────────────────

    let backend_port = ctx.server_port();
    let backend_host = ctx.server_host();

    // In development the frontend and backend typically run on different ports.
    // We default to localhost with the backend's configured port.
    let default_base_url = format!("http://127.0.0.1:{}", backend_port);

    content.push_str(&format!(
        r#"// ============================================================================
// API Client
// ============================================================================

/// Type-safe HTTP client for the backend REST API.
///
/// All methods return `Result<T, ClientError>` where `T` is the expected
/// response type.
///
/// # Example
///
/// ```rust,ignore
/// let client = ApiClient::new();
/// let users = client.list_users(1, 20).await?;
/// ```
#[derive(Debug, Clone)]
pub struct ApiClient {{
    /// The underlying reqwest HTTP client.
    client: Client,
    /// Base URL of the backend API (e.g. `http://127.0.0.1:8080`).
    base_url: String,
    /// Optional JWT token for authenticated requests.
    token: Option<String>,
}}

impl ApiClient {{
    /// Create a new API client with the default base URL.
    ///
    /// The default base URL is `{default_base_url}`. Override it with
    /// [`with_base_url`](ApiClient::with_base_url).
    pub fn new() -> Self {{
        Self {{
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to create HTTP client"),
            base_url: "{default_base_url}".to_string(),
            token: None,
        }}
    }}

    /// Create a client with a custom base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {{
        self.base_url = base_url.into();
        self
    }}

    /// Set the JWT authentication token.
    ///
    /// When set, all requests will include an `Authorization: Bearer <token>`
    /// header.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {{
        self.token = Some(token.into());
        self
    }}

    /// Clear the authentication token.
    pub fn clear_token(&mut self) {{
        self.token = None;
    }}

    /// Build the full URL for an API endpoint path.
    fn url(&self, path: &str) -> String {{
        format!("{{}}{{}}", self.base_url, path)
    }}

"#,
    ));

    // ── Generic request helpers ──────────────────────────────────────────
    content.push_str(
        r#"    // ========================================================================
    // Generic request helpers
    // ========================================================================

    /// Send a GET request and deserialise the response.
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ClientError> {
        let mut req = self.client.get(&self.url(path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;
        self.handle_response(response).await
    }

    /// Send a POST request with a JSON body and deserialise the response.
    async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ClientError> {
        let mut req = self.client.post(&self.url(path)).json(body);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;
        self.handle_response(response).await
    }

    /// Send a PUT request with a JSON body and deserialise the response.
    async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ClientError> {
        let mut req = self.client.put(&self.url(path)).json(body);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;
        self.handle_response(response).await
    }

    /// Send a DELETE request. Returns `Ok(())` on success.
    async fn delete(&self, path: &str) -> Result<(), ClientError> {
        let mut req = self.client.delete(&self.url(path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let body = response
                .json::<ApiError>()
                .await
                .unwrap_or_else(|_| ApiError {
                    error: "unknown".to_string(),
                    message: "An unknown error occurred".to_string(),
                });
            Err(ClientError::from_api_error(status, body))
        }
    }

    /// Handle a response: check for errors and deserialise on success.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, ClientError> {
        let status = response.status();

        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| ClientError::Parse(e.to_string()))
        } else {
            let status_code = status.as_u16();
            let body = response
                .json::<ApiError>()
                .await
                .unwrap_or_else(|_| ApiError {
                    error: "unknown".to_string(),
                    message: format!("Server returned status {}", status_code),
                });
            Err(ClientError::from_api_error(status_code, body))
        }
    }

"#,
    );

    // ── Per-entity CRUD methods ──────────────────────────────────────────

    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);

        let endpoint = match info.endpoint() {
            Some(ep) if ep.enabled => ep,
            _ => continue,
        };

        let enabled_ops: Vec<OperationType> = endpoint
            .enabled_operations()
            .iter()
            .map(|op| op.operation_type)
            .collect();

        let snake = info.snake_name();
        let pascal = info.pascal_name();
        let plural = info.plural_name();
        let base_path = info.base_path();
        let create_dto = GenerationContext::create_dto_name(&entity.name);
        let update_dto = GenerationContext::update_dto_name(&entity.name);
        let response_dto = GenerationContext::response_dto_name(&entity.name);

        content.push_str(&format!(
            "    // ========================================================================\n"
        ));
        content.push_str(&format!("    // {} endpoints\n", pascal));
        content.push_str(&format!(
            "    // ========================================================================\n\n"
        ));

        // ── List (ReadAll) ───────────────────────────────────────────────
        if enabled_ops.contains(&OperationType::ReadAll) {
            content.push_str(&format!(
                r#"    /// List all {plural} with pagination.
    ///
    /// GET {base_path}?page={{page}}&per_page={{per_page}}
    pub async fn list_{plural}(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedResponse<{response_dto}>, ClientError> {{
        let path = format!(
            "{base_path}?page={{}}&per_page={{}}",
            page, per_page,
        );
        self.get(&path).await
    }}

"#,
            ));
        }

        // ── Get (Read) ───────────────────────────────────────────────────
        if enabled_ops.contains(&OperationType::Read) {
            content.push_str(&format!(
                r#"    /// Get a single {snake} by ID.
    ///
    /// GET {base_path}/{{id}}
    pub async fn get_{snake}(
        &self,
        id: &str,
    ) -> Result<{response_dto}, ClientError> {{
        let path = format!("{base_path}/{{}}", id);
        self.get(&path).await
    }}

"#,
            ));
        }

        // ── Create ───────────────────────────────────────────────────────
        if enabled_ops.contains(&OperationType::Create) {
            content.push_str(&format!(
                r#"    /// Create a new {snake}.
    ///
    /// POST {base_path}
    pub async fn create_{snake}(
        &self,
        payload: &{create_dto},
    ) -> Result<{response_dto}, ClientError> {{
        self.post("{base_path}", payload).await
    }}

"#,
            ));
        }

        // ── Update ───────────────────────────────────────────────────────
        if enabled_ops.contains(&OperationType::Update) {
            content.push_str(&format!(
                r#"    /// Update an existing {snake} by ID.
    ///
    /// PUT {base_path}/{{id}}
    pub async fn update_{snake}(
        &self,
        id: &str,
        payload: &{update_dto},
    ) -> Result<{response_dto}, ClientError> {{
        let path = format!("{base_path}/{{}}", id);
        self.put(&path, payload).await
    }}

"#,
            ));
        }

        // ── Delete ───────────────────────────────────────────────────────
        if enabled_ops.contains(&OperationType::Delete) {
            content.push_str(&format!(
                r#"    /// Delete a {snake} by ID.
    ///
    /// DELETE {base_path}/{{id}}
    pub async fn delete_{snake}(
        &self,
        id: &str,
    ) -> Result<(), ClientError> {{
        let path = format!("{base_path}/{{}}", id);
        self.delete(&path).await
    }}

"#,
            ));
        }
    }

    // ── Close impl block ─────────────────────────────────────────────────
    content.push_str("}\n\n");

    // ── Default impl ─────────────────────────────────────────────────────
    content.push_str(
        "\
impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}
",
    );

    GeneratedFile::new("frontend/src/api/client.rs", content, FileType::Rust)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_core::DataType;
    use imortal_ir::{EndpointGroup, Entity, Field, OperationType, ProjectGraph, ProjectType};

    /// Helper: fullstack project with a User entity and full CRUD endpoints.
    fn fullstack_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("my_app");
        project.config.project_type = ProjectType::Fullstack;
        project.config.package_name = "my_app".to_string();
        project.config.server_port = 8080;

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
        project.add_endpoint(EndpointGroup::new(user_id, "User"));

        project
    }

    /// Helper: project with read-only endpoints.
    fn read_only_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("ro_app");
        project.config.project_type = ProjectType::Fullstack;
        project.config.package_name = "ro_app".to_string();

        let mut item = Entity::new("Item");
        item.config.timestamps = false;
        let item_id = item.id;

        let mut name_field = Field::new("name", DataType::String);
        name_field.required = true;
        item.fields.push(name_field);

        project.add_entity(item);

        let endpoint = EndpointGroup::new(item_id, "Item")
            .with_operations(&[OperationType::Read, OperationType::ReadAll]);
        project.add_endpoint(endpoint);

        project
    }

    #[test]
    fn test_generate_api_client_produces_two_files() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        assert_eq!(files.len(), 2);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"frontend/src/api/mod.rs".to_string()));
        assert!(paths.contains(&"frontend/src/api/client.rs".to_string()));
    }

    #[test]
    fn test_api_mod_re_exports() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let mod_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("mod.rs"))
            .unwrap();

        assert!(mod_file.content.contains("pub mod client;"));
        assert!(mod_file.content.contains("pub use client::ApiClient;"));
    }

    #[test]
    fn test_client_has_struct() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("pub struct ApiClient"));
        assert!(content.contains("client: Client"));
        assert!(content.contains("base_url: String"));
        assert!(content.contains("token: Option<String>"));
    }

    #[test]
    fn test_client_has_constructors() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("pub fn new()"));
        assert!(content.contains("pub fn with_base_url("));
        assert!(content.contains("pub fn with_token("));
        assert!(content.contains("pub fn clear_token("));
    }

    #[test]
    fn test_client_uses_configured_port() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        assert!(client_file.content.contains("8080"));
    }

    #[test]
    fn test_client_has_generic_helpers() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("async fn get<T: DeserializeOwned>"));
        assert!(content.contains("async fn post<T: DeserializeOwned, B: Serialize>"));
        assert!(content.contains("async fn put<T: DeserializeOwned, B: Serialize>"));
        assert!(content.contains("async fn delete(&self, path: &str)"));
        assert!(content.contains("async fn handle_response<T: DeserializeOwned>"));
    }

    #[test]
    fn test_client_has_all_crud_methods_for_user() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(
            content.contains("pub async fn list_users("),
            "Missing list_users method"
        );
        assert!(
            content.contains("pub async fn get_user("),
            "Missing get_user method"
        );
        assert!(
            content.contains("pub async fn create_user("),
            "Missing create_user method"
        );
        assert!(
            content.contains("pub async fn update_user("),
            "Missing update_user method"
        );
        assert!(
            content.contains("pub async fn delete_user("),
            "Missing delete_user method"
        );
    }

    #[test]
    fn test_client_list_method_uses_pagination() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("page: u64"));
        assert!(content.contains("per_page: u64"));
        assert!(content.contains("PaginatedResponse<UserResponse>"));
        assert!(content.contains("page="));
        assert!(content.contains("per_page="));
    }

    #[test]
    fn test_client_get_method_takes_id() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("id: &str"));
        assert!(content.contains("-> Result<UserResponse, ClientError>"));
    }

    #[test]
    fn test_client_create_method_takes_dto() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("payload: &CreateUserDto"));
        assert!(content.contains("-> Result<UserResponse, ClientError>"));
    }

    #[test]
    fn test_client_update_method_takes_id_and_dto() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        // update method should take id AND payload
        let update_section = content
            .split("pub async fn update_user(")
            .nth(1)
            .unwrap_or("");

        assert!(update_section.contains("id: &str"));
        assert!(update_section.contains("payload: &UpdateUserDto"));
    }

    #[test]
    fn test_client_delete_method_returns_unit() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        // delete method should return Result<(), ClientError>
        let delete_section = content
            .split("pub async fn delete_user(")
            .nth(1)
            .unwrap_or("");

        assert!(delete_section.contains("-> Result<(), ClientError>"));
    }

    #[test]
    fn test_client_uses_correct_api_paths() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("/api/users"));
    }

    #[test]
    fn test_client_read_only_entity() {
        let project = read_only_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        // Should have list and get methods
        assert!(content.contains("pub async fn list_items("));
        assert!(content.contains("pub async fn get_item("));

        // Should NOT have create, update, delete methods
        assert!(!content.contains("pub async fn create_item("));
        assert!(!content.contains("pub async fn update_item("));
        assert!(!content.contains("pub async fn delete_item("));
    }

    #[test]
    fn test_client_no_methods_for_entity_without_endpoint() {
        let mut project = fullstack_project();

        // Add entity WITHOUT endpoint
        let mut cat = Entity::new("Category");
        cat.config.timestamps = false;
        let mut cname = Field::new("name", DataType::String);
        cname.required = true;
        cat.fields.push(cname);
        project.add_entity(cat);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        // Should have User methods
        assert!(content.contains("list_users"));

        // Should NOT have Category methods
        assert!(!content.contains("list_categories"));
        assert!(!content.contains("get_category"));
    }

    #[test]
    fn test_client_multiple_entities() {
        let mut project = fullstack_project();

        let mut post = Entity::new("Post");
        post.config.timestamps = true;
        let post_id = post.id;

        let mut title = Field::new("title", DataType::String);
        title.required = true;
        post.fields.push(title);

        project.add_entity(post);
        project.add_endpoint(EndpointGroup::new(post_id, "Post"));

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        // Should have methods for both User and Post
        assert!(content.contains("list_users("));
        assert!(content.contains("get_user("));
        assert!(content.contains("create_user("));

        assert!(content.contains("list_posts("));
        assert!(content.contains("get_post("));
        assert!(content.contains("create_post("));

        // Should import both DTOs
        assert!(content.contains("CreateUserDto"));
        assert!(content.contains("CreatePostDto"));
        assert!(content.contains("UserResponse"));
        assert!(content.contains("PostResponse"));
    }

    #[test]
    fn test_client_error_type() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("pub enum ClientError"));
        assert!(content.contains("Request(#[from] reqwest::Error)"));
        assert!(content.contains("Api {"));
        assert!(content.contains("Parse(String)"));
    }

    #[test]
    fn test_client_error_helpers() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("pub fn is_not_found("));
        assert!(content.contains("pub fn is_unauthorized("));
        assert!(content.contains("pub fn is_forbidden("));
        assert!(content.contains("pub fn is_validation("));
        assert!(content.contains("pub fn is_conflict("));
        assert!(content.contains("pub fn user_message("));
    }

    #[test]
    fn test_client_handles_auth_token() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("bearer_auth(token)"));
        assert!(content.contains("token: Option<String>"));
    }

    #[test]
    fn test_client_has_timeout() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("timeout(std::time::Duration::from_secs(30))"));
    }

    #[test]
    fn test_client_error_response_handling() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        // Should parse error responses as ApiError
        assert!(content.contains("json::<ApiError>()"));
        assert!(content.contains("from_api_error"));
        assert!(content.contains("status.is_success()"));
    }

    #[test]
    fn test_client_default_impl() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("impl Default for ApiClient"));
        assert!(content.contains("Self::new()"));
    }

    #[test]
    fn test_client_imports_shared_types() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("use shared::{ApiError, PaginatedResponse}"));
        assert!(content.contains("use shared::{CreateUserDto, UpdateUserDto, UserResponse}"));
    }

    #[test]
    fn test_client_uses_thiserror() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("use thiserror::Error"));
        assert!(content.contains("#[derive(Debug, Error)]"));
        assert!(content.contains("#[error("));
    }

    #[test]
    fn test_client_has_file_header() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        for file in &files {
            assert!(
                file.content.contains("Auto-generated by Immortal Engine"),
                "File {} should have a header",
                file.path.display()
            );
        }
    }

    #[test]
    fn test_client_user_message_method() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        assert!(content.contains("fn user_message(&self) -> String"));
        assert!(content.contains("timed out"));
        assert!(content.contains("Unable to connect"));
        assert!(content.contains("unexpected network error"));
        assert!(content.contains("unexpected response"));
    }

    #[test]
    fn test_client_doc_comments_on_methods() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        let content = &client_file.content;

        // Each CRUD method should have a doc comment with the HTTP method and path
        assert!(content.contains("/// List all users"));
        assert!(content.contains("/// GET /api/users"));

        assert!(content.contains("/// Get a single user"));
        assert!(content.contains("/// GET /api/users/{id}"));

        assert!(content.contains("/// Create a new user"));
        assert!(content.contains("/// POST /api/users"));

        assert!(content.contains("/// Update an existing user"));
        assert!(content.contains("/// PUT /api/users/{id}"));

        assert!(content.contains("/// Delete a user"));
        assert!(content.contains("/// DELETE /api/users/{id}"));
    }

    #[test]
    fn test_client_custom_port() {
        let mut project = fullstack_project();
        project.config.server_port = 3000;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        // The base URL should use the configured port
        assert!(
            client_file.content.contains("3000"),
            "Client should use the configured port 3000"
        );
    }

    #[test]
    fn test_client_entity_section_headers() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        // Should have a section header for User endpoints
        assert!(client_file.content.contains("// User endpoints"));
    }

    #[test]
    fn test_client_disabled_endpoint_excluded() {
        let mut project = ProjectGraph::new("test");
        project.config.project_type = ProjectType::Fullstack;
        project.config.package_name = "test".to_string();

        let mut entity = Entity::new("Widget");
        entity.config.timestamps = false;
        let widget_id = entity.id;

        let mut name_f = Field::new("name", DataType::String);
        name_f.required = true;
        entity.fields.push(name_f);

        project.add_entity(entity);

        // Disabled endpoint
        let endpoint = EndpointGroup::new(widget_id, "Widget").disabled();
        project.add_endpoint(endpoint);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_api_client(&ctx);

        let client_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("client.rs"))
            .unwrap();

        // Disabled endpoint should NOT produce methods
        assert!(!client_file.content.contains("list_widgets"));
        assert!(!client_file.content.contains("get_widget"));
        assert!(!client_file.content.contains("create_widget"));
    }
}
