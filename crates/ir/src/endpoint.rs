//! Endpoint definitions for API configuration
//!
//! This module contains types for defining REST API endpoints and their
//! configuration including CRUD operations, security, and rate limiting.

use imortal_core::{EngineError, EngineResult, Position, Validatable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// EndpointGroup
// ============================================================================

/// A group of endpoints for an entity (CRUD operations)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointGroup {
    /// Unique identifier
    pub id: Uuid,

    /// Reference to the entity this endpoint group belongs to
    pub entity_id: Uuid,

    /// Entity name (for display purposes)
    pub entity_name: String,

    /// Base API path (e.g., "/api/users")
    pub base_path: String,

    /// CRUD operations configuration
    pub operations: Vec<CrudOperation>,

    /// Global security settings (can be overridden per operation)
    pub global_security: EndpointSecurity,

    /// Position on the canvas (for visual representation)
    pub position: Position,

    /// Whether this endpoint group is enabled
    pub enabled: bool,

    /// API version prefix (e.g., "v1")
    pub api_version: Option<String>,

    /// Custom middleware to apply
    pub middleware: Vec<String>,

    /// Tags for API documentation (OpenAPI)
    pub tags: Vec<String>,

    /// Description for API documentation
    pub description: Option<String>,
}

impl EndpointGroup {
    /// Create a new endpoint group for an entity
    pub fn new(entity_id: Uuid, entity_name: impl Into<String>) -> Self {
        let entity_name = entity_name.into();
        let base_path = format!("/api/{}", to_snake_case_plural(&entity_name));

        Self {
            id: Uuid::new_v4(),
            entity_id,
            entity_name: entity_name.clone(),
            base_path,
            operations: CrudOperation::default_all(),
            global_security: EndpointSecurity::default(),
            position: Position::zero(),
            enabled: true,
            api_version: None,
            middleware: Vec::new(),
            tags: vec![entity_name],
            description: None,
        }
    }

    /// Create with default CRUD operations
    pub fn default_crud(entity_id: Uuid, entity_name: impl Into<String>) -> Self {
        Self::new(entity_id, entity_name)
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Set the base path
    pub fn with_base_path(mut self, path: impl Into<String>) -> Self {
        self.base_path = path.into();
        self
    }

    /// Set the API version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }

    /// Set global security
    pub fn with_security(mut self, security: EndpointSecurity) -> Self {
        self.global_security = security;
        self
    }

    /// Require authentication for all endpoints
    pub fn secured(mut self) -> Self {
        self.global_security.auth_required = true;
        self
    }

    /// Require specific roles for all endpoints
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.global_security.auth_required = true;
        self.global_security.roles = roles;
        self
    }

    /// Set position on canvas
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = Position::new(x, y);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add middleware
    pub fn with_middleware(mut self, middleware: impl Into<String>) -> Self {
        self.middleware.push(middleware.into());
        self
    }

    /// Disable the endpoint group
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    // ========================================================================
    // Operation management
    // ========================================================================

    /// Get an operation by type
    pub fn get_operation(&self, op_type: OperationType) -> Option<&CrudOperation> {
        self.operations
            .iter()
            .find(|op| op.operation_type == op_type)
    }

    /// Get a mutable operation by type
    pub fn get_operation_mut(&mut self, op_type: OperationType) -> Option<&mut CrudOperation> {
        self.operations
            .iter_mut()
            .find(|op| op.operation_type == op_type)
    }

    /// Enable a specific operation
    pub fn enable_operation(&mut self, op_type: OperationType) {
        if let Some(op) = self.get_operation_mut(op_type) {
            op.enabled = true;
        }
    }

    /// Disable a specific operation
    pub fn disable_operation(&mut self, op_type: OperationType) {
        if let Some(op) = self.get_operation_mut(op_type) {
            op.enabled = false;
        }
    }

    /// Set security for a specific operation
    pub fn set_operation_security(&mut self, op_type: OperationType, security: EndpointSecurity) {
        if let Some(op) = self.get_operation_mut(op_type) {
            op.security = Some(security);
        }
    }

    /// Get enabled operations
    pub fn enabled_operations(&self) -> Vec<&CrudOperation> {
        self.operations.iter().filter(|op| op.enabled).collect()
    }

    /// Only enable specific operations
    pub fn with_operations(mut self, ops: &[OperationType]) -> Self {
        for operation in &mut self.operations {
            operation.enabled = ops.contains(&operation.operation_type);
        }
        self
    }

    /// Enable only read operations
    pub fn read_only(self) -> Self {
        self.with_operations(&[OperationType::Read, OperationType::ReadAll])
    }

    // ========================================================================
    // Utility methods
    // ========================================================================

    /// Get the full base path including API version
    pub fn full_base_path(&self) -> String {
        match &self.api_version {
            Some(version) => format!(
                "/api/{}{}",
                version,
                self.base_path.trim_start_matches("/api")
            ),
            None => self.base_path.clone(),
        }
    }

    /// Get the effective security for an operation
    pub fn effective_security(&self, op_type: OperationType) -> EndpointSecurity {
        self.get_operation(op_type)
            .and_then(|op| op.security.clone())
            .unwrap_or_else(|| self.global_security.clone())
    }

    /// Check if any operation requires authentication
    pub fn requires_auth(&self) -> bool {
        if self.global_security.auth_required {
            return true;
        }
        self.operations.iter().any(|op| {
            op.enabled
                && op
                    .security
                    .as_ref()
                    .map(|s| s.auth_required)
                    .unwrap_or(false)
        })
    }
}

impl Validatable for EndpointGroup {
    fn validate(&self) -> EngineResult<()> {
        if self.base_path.is_empty() {
            return Err(EngineError::EndpointValidation {
                endpoint: self.entity_name.clone(),
                message: "Base path cannot be empty".to_string(),
            });
        }

        if !self.base_path.starts_with('/') {
            return Err(EngineError::EndpointValidation {
                endpoint: self.entity_name.clone(),
                message: "Base path must start with '/'".to_string(),
            });
        }

        if self.operations.is_empty() {
            return Err(EngineError::EndpointValidation {
                endpoint: self.entity_name.clone(),
                message: "Endpoint group must have at least one operation".to_string(),
            });
        }

        // Validate each operation
        for op in &self.operations {
            op.validate()?;
        }

        Ok(())
    }
}

impl Default for EndpointGroup {
    fn default() -> Self {
        Self::new(Uuid::nil(), "Entity")
    }
}

// ============================================================================
// CrudOperation
// ============================================================================

/// Configuration for a single CRUD operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudOperation {
    /// Operation type
    pub operation_type: OperationType,

    /// Whether this operation is enabled
    pub enabled: bool,

    /// Path suffix (e.g., "/:id" for single item operations)
    pub path_suffix: String,

    /// Security override (uses global if None)
    pub security: Option<EndpointSecurity>,

    /// Rate limiting for this operation
    pub rate_limit: Option<RateLimit>,

    /// Custom handler function name (if not using generated handler)
    pub custom_handler: Option<String>,

    /// Description for API documentation
    pub description: Option<String>,

    /// Operation ID for OpenAPI
    pub operation_id: Option<String>,

    /// Response status code
    pub success_status: u16,

    /// Whether to include in API documentation
    pub documented: bool,
}

impl CrudOperation {
    /// Create a new CRUD operation
    pub fn new(operation_type: OperationType) -> Self {
        let (path_suffix, success_status) = match operation_type {
            OperationType::Create => ("".to_string(), 201),
            OperationType::Read => ("/:id".to_string(), 200),
            OperationType::ReadAll => ("".to_string(), 200),
            OperationType::Update => ("/:id".to_string(), 200),
            OperationType::Delete => ("/:id".to_string(), 204),
        };

        Self {
            operation_type,
            enabled: true,
            path_suffix,
            security: None,
            rate_limit: None,
            custom_handler: None,
            description: None,
            operation_id: None,
            success_status,
            documented: true,
        }
    }

    /// Create default operations for all CRUD types
    pub fn default_all() -> Vec<Self> {
        vec![
            Self::new(OperationType::Create),
            Self::new(OperationType::Read),
            Self::new(OperationType::ReadAll),
            Self::new(OperationType::Update),
            Self::new(OperationType::Delete),
        ]
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Set the security configuration
    pub fn with_security(mut self, security: EndpointSecurity) -> Self {
        self.security = Some(security);
        self
    }

    /// Require authentication
    pub fn secured(mut self) -> Self {
        self.security = Some(EndpointSecurity::authenticated());
        self
    }

    /// Require specific roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.security = Some(EndpointSecurity::with_roles(roles));
        self
    }

    /// Set rate limit
    pub fn with_rate_limit(mut self, requests: u32, window_seconds: u32) -> Self {
        self.rate_limit = Some(RateLimit::new(requests, window_seconds));
        self
    }

    /// Set custom handler
    pub fn with_handler(mut self, handler: impl Into<String>) -> Self {
        self.custom_handler = Some(handler.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set custom path suffix
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path_suffix = path.into();
        self
    }

    /// Disable this operation
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Exclude from documentation
    pub fn undocumented(mut self) -> Self {
        self.documented = false;
        self
    }

    // ========================================================================
    // Utility methods
    // ========================================================================

    /// Get the HTTP method for this operation
    pub fn http_method(&self) -> &'static str {
        self.operation_type.http_method()
    }

    /// Get the full path (base_path + suffix)
    pub fn full_path(&self, base_path: &str) -> String {
        format!("{}{}", base_path, self.path_suffix)
    }

    /// Get the default operation ID
    pub fn default_operation_id(&self, entity_name: &str) -> String {
        self.operation_id.clone().unwrap_or_else(|| {
            format!(
                "{}_{}",
                self.operation_type.to_string().to_lowercase(),
                to_snake_case(entity_name)
            )
        })
    }

    /// Get the handler function name
    pub fn handler_name(&self, entity_name: &str) -> String {
        self.custom_handler.clone().unwrap_or_else(|| {
            let entity_snake = to_snake_case(entity_name);
            match self.operation_type {
                OperationType::Create => format!("create_{}", entity_snake),
                OperationType::Read => format!("get_{}", entity_snake),
                OperationType::ReadAll => format!("list_{}s", entity_snake),
                OperationType::Update => format!("update_{}", entity_snake),
                OperationType::Delete => format!("delete_{}", entity_snake),
            }
        })
    }
}

impl Validatable for CrudOperation {
    fn validate(&self) -> EngineResult<()> {
        // Path suffix validation
        if !self.path_suffix.is_empty()
            && !self.path_suffix.starts_with('/')
            && !self.path_suffix.starts_with(':')
        {
            return Err(EngineError::validation(format!(
                "Path suffix '{}' must start with '/' or ':'",
                self.path_suffix
            )));
        }

        // Rate limit validation
        if let Some(rate_limit) = &self.rate_limit {
            rate_limit.validate()?;
        }

        Ok(())
    }
}

// ============================================================================
// OperationType
// ============================================================================

/// Types of CRUD operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    /// Create a new resource (POST)
    Create,
    /// Read a single resource (GET /:id)
    Read,
    /// Read all resources (GET /)
    ReadAll,
    /// Update a resource (PUT /:id)
    Update,
    /// Delete a resource (DELETE /:id)
    Delete,
}

impl OperationType {
    /// Get the HTTP method
    pub fn http_method(&self) -> &'static str {
        match self {
            OperationType::Create => "POST",
            OperationType::Read => "GET",
            OperationType::ReadAll => "GET",
            OperationType::Update => "PUT",
            OperationType::Delete => "DELETE",
        }
    }

    /// Get a user-friendly display name
    pub fn display_name(&self) -> &'static str {
        match self {
            OperationType::Create => "Create",
            OperationType::Read => "Read",
            OperationType::ReadAll => "List",
            OperationType::Update => "Update",
            OperationType::Delete => "Delete",
        }
    }

    /// Get all operation types
    pub fn all() -> &'static [OperationType] {
        &[
            OperationType::Create,
            OperationType::Read,
            OperationType::ReadAll,
            OperationType::Update,
            OperationType::Delete,
        ]
    }

    /// Check if this is a read operation
    pub fn is_read(&self) -> bool {
        matches!(self, OperationType::Read | OperationType::ReadAll)
    }

    /// Check if this is a write operation
    pub fn is_write(&self) -> bool {
        matches!(
            self,
            OperationType::Create | OperationType::Update | OperationType::Delete
        )
    }

    /// Check if this operation affects a single resource
    pub fn is_single(&self) -> bool {
        matches!(
            self,
            OperationType::Read | OperationType::Update | OperationType::Delete
        )
    }
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// EndpointSecurity
// ============================================================================

/// Security configuration for an endpoint
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EndpointSecurity {
    /// Whether authentication is required
    pub auth_required: bool,

    /// Required roles (any of these roles grants access)
    pub roles: Vec<String>,

    /// Required permissions/scopes
    pub scopes: Vec<String>,

    /// Whether CORS is enabled
    pub cors_enabled: bool,

    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,

    /// Whether to allow public access with limited data
    pub allow_public_preview: bool,
}

impl EndpointSecurity {
    /// Create a new security configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration that requires authentication
    pub fn authenticated() -> Self {
        Self {
            auth_required: true,
            cors_enabled: true,
            ..Self::default()
        }
    }

    /// Create configuration with specific roles
    pub fn with_roles(roles: Vec<String>) -> Self {
        Self {
            auth_required: true,
            roles,
            cors_enabled: true,
            ..Self::default()
        }
    }

    /// Create open (public) configuration
    pub fn open() -> Self {
        Self {
            auth_required: false,
            cors_enabled: true,
            ..Self::default()
        }
    }

    /// Create admin-only configuration
    pub fn admin_only() -> Self {
        Self::with_roles(vec!["admin".to_string()])
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Require authentication
    pub fn require_auth(mut self) -> Self {
        self.auth_required = true;
        self
    }

    /// Add a required role
    pub fn add_role(mut self, role: impl Into<String>) -> Self {
        self.auth_required = true;
        self.roles.push(role.into());
        self
    }

    /// Add a required scope
    pub fn add_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    /// Enable CORS
    pub fn enable_cors(mut self) -> Self {
        self.cors_enabled = true;
        self
    }

    /// Add CORS origin
    pub fn add_cors_origin(mut self, origin: impl Into<String>) -> Self {
        self.cors_enabled = true;
        self.cors_origins.push(origin.into());
        self
    }

    /// Allow public preview
    pub fn allow_preview(mut self) -> Self {
        self.allow_public_preview = true;
        self
    }

    // ========================================================================
    // Utility methods
    // ========================================================================

    /// Check if any roles are required
    pub fn has_roles(&self) -> bool {
        !self.roles.is_empty()
    }

    /// Check if any scopes are required
    pub fn has_scopes(&self) -> bool {
        !self.scopes.is_empty()
    }

    /// Check if the given role is allowed
    pub fn allows_role(&self, role: &str) -> bool {
        self.roles.is_empty() || self.roles.iter().any(|r| r == role)
    }

    /// Check if fully open (no auth, no roles)
    pub fn is_open(&self) -> bool {
        !self.auth_required && self.roles.is_empty()
    }
}

// ============================================================================
// RateLimit
// ============================================================================

/// Rate limiting configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum number of requests allowed
    pub requests: u32,

    /// Time window in seconds
    pub window_seconds: u32,

    /// Whether to apply per-user (true) or globally (false)
    pub per_user: bool,

    /// Custom key for rate limiting (e.g., "ip", "user_id")
    pub key: Option<String>,

    /// Response status code when limit is exceeded
    pub exceeded_status: u16,

    /// Custom message when limit is exceeded
    pub exceeded_message: Option<String>,
}

impl RateLimit {
    /// Create a new rate limit
    pub fn new(requests: u32, window_seconds: u32) -> Self {
        Self {
            requests,
            window_seconds,
            per_user: true,
            key: None,
            exceeded_status: 429,
            exceeded_message: None,
        }
    }

    /// Create a permissive rate limit (100 requests per minute)
    pub fn permissive() -> Self {
        Self::new(100, 60)
    }

    /// Create a strict rate limit (10 requests per minute)
    pub fn strict() -> Self {
        Self::new(10, 60)
    }

    /// Create a burst rate limit (1000 requests per second)
    pub fn burst() -> Self {
        Self::new(1000, 1)
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Apply globally instead of per-user
    pub fn global(mut self) -> Self {
        self.per_user = false;
        self
    }

    /// Set custom key
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set custom exceeded message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.exceeded_message = Some(message.into());
        self
    }

    // ========================================================================
    // Utility methods
    // ========================================================================

    /// Get requests per second
    pub fn requests_per_second(&self) -> f64 {
        if self.window_seconds == 0 {
            return 0.0;
        }
        self.requests as f64 / self.window_seconds as f64
    }

    /// Get the exceeded message
    pub fn get_exceeded_message(&self) -> String {
        self.exceeded_message.clone().unwrap_or_else(|| {
            format!(
                "Rate limit exceeded. Maximum {} requests per {} seconds.",
                self.requests, self.window_seconds
            )
        })
    }
}

impl Validatable for RateLimit {
    fn validate(&self) -> EngineResult<()> {
        if self.requests == 0 {
            return Err(EngineError::validation(
                "Rate limit requests must be greater than 0",
            ));
        }
        if self.window_seconds == 0 {
            return Err(EngineError::validation(
                "Rate limit window must be greater than 0",
            ));
        }
        Ok(())
    }
}

impl Default for RateLimit {
    fn default() -> Self {
        Self::permissive()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_was_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_was_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_was_upper = true;
        } else {
            result.push(c);
            prev_was_upper = false;
        }
    }

    result
}

/// Convert to snake_case plural
fn to_snake_case_plural(s: &str) -> String {
    let snake = to_snake_case(s);

    if snake.ends_with('s')
        || snake.ends_with('x')
        || snake.ends_with("ch")
        || snake.ends_with("sh")
    {
        format!("{}es", snake)
    } else if snake.ends_with('y')
        && !snake.ends_with("ey")
        && !snake.ends_with("ay")
        && !snake.ends_with("oy")
    {
        format!("{}ies", &snake[..snake.len() - 1])
    } else {
        format!("{}s", snake)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_group_new() {
        let entity_id = Uuid::new_v4();
        let group = EndpointGroup::new(entity_id, "User");

        assert_eq!(group.entity_id, entity_id);
        assert_eq!(group.entity_name, "User");
        assert_eq!(group.base_path, "/api/users");
        assert_eq!(group.operations.len(), 5); // All CRUD operations
        assert!(group.enabled);
    }

    #[test]
    fn test_endpoint_group_builder() {
        let group = EndpointGroup::new(Uuid::new_v4(), "BlogPost")
            .with_version("v1")
            .secured()
            .with_description("Blog post endpoints");

        assert_eq!(group.api_version, Some("v1".to_string()));
        assert!(group.global_security.auth_required);
        assert!(group.description.is_some());
    }

    #[test]
    fn test_endpoint_group_full_path() {
        let group = EndpointGroup::new(Uuid::new_v4(), "User").with_version("v1");
        assert_eq!(group.full_base_path(), "/api/v1/users");

        let group_no_version = EndpointGroup::new(Uuid::new_v4(), "User");
        assert_eq!(group_no_version.full_base_path(), "/api/users");
    }

    #[test]
    fn test_endpoint_group_with_operations() {
        let group = EndpointGroup::new(Uuid::new_v4(), "User")
            .with_operations(&[OperationType::Read, OperationType::ReadAll]);

        let enabled: Vec<_> = group
            .enabled_operations()
            .iter()
            .map(|op| op.operation_type)
            .collect();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&OperationType::Read));
        assert!(enabled.contains(&OperationType::ReadAll));
    }

    #[test]
    fn test_crud_operation_new() {
        let op = CrudOperation::new(OperationType::Create);
        assert_eq!(op.operation_type, OperationType::Create);
        assert!(op.enabled);
        assert_eq!(op.http_method(), "POST");
        assert_eq!(op.success_status, 201);
    }

    #[test]
    fn test_crud_operation_handler_name() {
        let op = CrudOperation::new(OperationType::ReadAll);
        assert_eq!(op.handler_name("User"), "list_users");

        let op = CrudOperation::new(OperationType::Create);
        assert_eq!(op.handler_name("BlogPost"), "create_blog_post");
    }

    #[test]
    fn test_operation_type() {
        assert_eq!(OperationType::Create.http_method(), "POST");
        assert_eq!(OperationType::Read.http_method(), "GET");
        assert_eq!(OperationType::Update.http_method(), "PUT");
        assert_eq!(OperationType::Delete.http_method(), "DELETE");

        assert!(OperationType::Read.is_read());
        assert!(OperationType::Create.is_write());
        assert!(OperationType::Read.is_single());
        assert!(!OperationType::ReadAll.is_single());
    }

    #[test]
    fn test_endpoint_security() {
        let open = EndpointSecurity::open();
        assert!(!open.auth_required);
        assert!(open.is_open());

        let authed = EndpointSecurity::authenticated();
        assert!(authed.auth_required);
        assert!(!authed.is_open());

        let admin = EndpointSecurity::admin_only();
        assert!(admin.auth_required);
        assert!(admin.roles.contains(&"admin".to_string()));
    }

    #[test]
    fn test_rate_limit() {
        let rate = RateLimit::new(100, 60);
        assert_eq!(rate.requests, 100);
        assert_eq!(rate.window_seconds, 60);
        assert!(rate.per_user);
        assert!(rate.validate().is_ok());

        let invalid = RateLimit::new(0, 60);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_rate_limit_requests_per_second() {
        let rate = RateLimit::new(60, 60);
        assert!((rate.requests_per_second() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_endpoint_group_validation() {
        let valid = EndpointGroup::new(Uuid::new_v4(), "User");
        assert!(valid.validate().is_ok());

        let mut invalid = EndpointGroup::new(Uuid::new_v4(), "User");
        invalid.base_path = String::new();
        assert!(invalid.validate().is_err());

        let mut no_slash = EndpointGroup::new(Uuid::new_v4(), "User");
        no_slash.base_path = "api/users".to_string();
        assert!(no_slash.validate().is_err());
    }
}
