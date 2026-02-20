//! Project definitions for Immortal Engine
//!
//! This module contains the root project structures including `ProjectGraph`,
//! which is the main container for all project data, and related configuration types.

use crate::relationship_helpers::{
    add_fk_field_for_relationship, calculate_fk_info, determine_fk_entity, generate_fk_field_name,
    generate_relationship_name,
};
use crate::{EndpointGroup, Entity, Relationship};
use chrono::{DateTime, Utc};
use imortal_core::{DatabaseType, EngineError, EngineResult, Position, Validatable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// ProjectGraph
// ============================================================================

/// Root container for an Immortal Engine project
///
/// The `ProjectGraph` holds all entities, relationships, endpoints, and
/// configuration for a single project. It can be serialized to JSON for
/// persistence and loaded back when opening a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGraph {
    /// Project metadata
    pub meta: ProjectMeta,

    /// Project configuration
    pub config: ProjectConfig,

    /// All entities in the project, keyed by ID
    pub entities: HashMap<Uuid, Entity>,

    /// All relationships between entities, keyed by ID
    pub relationships: HashMap<Uuid, Relationship>,

    /// API endpoint configurations, keyed by ID
    pub endpoints: HashMap<Uuid, EndpointGroup>,

    /// Canvas state (pan, zoom, etc.)
    pub canvas: CanvasState,

    /// Currently selected entity IDs
    #[serde(default)]
    pub selected_entities: Vec<Uuid>,

    /// Currently selected relationship IDs
    #[serde(default)]
    pub selected_relationships: Vec<Uuid>,

    /// Schema version for migration purposes
    pub schema_version: u32,
}

impl ProjectGraph {
    /// Create a new project with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            meta: ProjectMeta::new(name),
            config: ProjectConfig::default(),
            entities: HashMap::new(),
            relationships: HashMap::new(),
            endpoints: HashMap::new(),
            canvas: CanvasState::default(),
            selected_entities: Vec::new(),
            selected_relationships: Vec::new(),
            schema_version: crate::SCHEMA_VERSION,
        }
    }

    /// Create a new project with configuration
    pub fn with_config(name: impl Into<String>, config: ProjectConfig) -> Self {
        let mut project = Self::new(name);
        project.config = config;
        project
    }

    // ========================================================================
    // Entity Management
    // ========================================================================

    /// Add an entity to the project
    pub fn add_entity(&mut self, entity: Entity) -> Uuid {
        let id = entity.id;
        self.entities.insert(id, entity);
        self.touch();
        id
    }

    /// Remove an entity by ID
    pub fn remove_entity(&mut self, id: Uuid) -> Option<Entity> {
        // Also remove any relationships involving this entity
        let relationships_to_remove: Vec<Uuid> = self
            .relationships
            .values()
            .filter(|r| r.from_entity_id == id || r.to_entity_id == id)
            .map(|r| r.id)
            .collect();

        for rel_id in relationships_to_remove {
            self.relationships.remove(&rel_id);
        }
        let endpoints_to_remove: Vec<Uuid> = self
            .endpoints
            .values()
            .filter(|e| e.entity_id == id)
            .map(|e| e.id)
            .collect();

        for endpoint_id in endpoints_to_remove {
            self.endpoints.remove(&endpoint_id);
        }
        self.selected_entities.retain(|&eid| eid != id);

        self.touch();
        self.entities.remove(&id)
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: Uuid) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// Get a mutable entity by ID
    pub fn get_entity_mut(&mut self, id: Uuid) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    /// Get an entity by name
    pub fn get_entity_by_name(&self, name: &str) -> Option<&Entity> {
        self.entities.values().find(|e| e.name == name)
    }

    /// Get all entities
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// Get the number of entities
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    // ========================================================================
    // Relationship Management
    // ========================================================================

    /// Add a relationship to the project
    pub fn add_relationship(&mut self, relationship: Relationship) -> Uuid {
        let id = relationship.id;
        self.relationships.insert(id, relationship);
        self.touch();
        id
    }

    /// Add a relationship and auto-generate the FK field on the appropriate entity
    ///
    /// This is the recommended way to create relationships as it ensures:
    /// - The FK field is created on the correct entity based on relationship type
    /// - The FK field name is generated consistently
    /// - The relationship's from_field is set correctly
    ///
    /// Returns `Ok((relationship_id, Some(fk_field_id)))` on success,
    /// or `Ok((relationship_id, None))` for many-to-many relationships (no direct FK)
    pub fn create_relationship_with_fk(
        &mut self,
        mut relationship: Relationship,
    ) -> EngineResult<(Uuid, Option<Uuid>)> {
        let from_entity = self
            .entities
            .get(&relationship.from_entity_id)
            .ok_or_else(|| EngineError::EntityNotFound(relationship.from_entity_id.to_string()))?
            .clone();

        let to_entity = self
            .entities
            .get(&relationship.to_entity_id)
            .ok_or_else(|| EngineError::EntityNotFound(relationship.to_entity_id.to_string()))?
            .clone();

        // Auto-generate relationship name if empty
        if relationship.name.is_empty() {
            relationship.name = generate_relationship_name(&from_entity.name, &to_entity.name);
        }

        // Determine which entity needs the FK field
        let fk_entity_id = determine_fk_entity(&relationship);

        let fk_field_id = if let Some(fk_entity_id) = fk_entity_id {
            // Calculate FK info
            let fk_info = calculate_fk_info(&relationship, &from_entity.name, &to_entity.name);

            if let Some(info) = fk_info {
                // Get the target entity (the one being referenced)
                let target_entity = if info.referenced_entity_id == from_entity.id {
                    &from_entity
                } else {
                    &to_entity
                };

                // Update relationship's from_field with the FK field name
                relationship.from_field = info.field_name.clone();

                // Get mutable reference to the entity that needs the FK
                let entity_for_fk = self
                    .entities
                    .get_mut(&fk_entity_id)
                    .ok_or_else(|| EngineError::EntityNotFound(fk_entity_id.to_string()))?;

                // Check if FK field already exists
                let fk_name = &info.field_name;
                if entity_for_fk.has_field(fk_name) {
                    // Field already exists, just use it
                    let existing_field_id = entity_for_fk.get_field_by_name(fk_name).map(|f| f.id);
                    existing_field_id
                } else {
                    // Create and add the FK field
                    let fk_field_id =
                        add_fk_field_for_relationship(entity_for_fk, target_entity, &relationship)?;
                    Some(fk_field_id)
                }
            } else {
                None
            }
        } else {
            // Many-to-many: no direct FK (uses junction table)
            None
        };
        let rel_id = self.add_relationship(relationship);

        Ok((rel_id, fk_field_id))
    }

    /// Generate a FK field name for a relationship
    pub fn suggest_fk_field_name(&self, target_entity_id: Uuid) -> Option<String> {
        self.entities
            .get(&target_entity_id)
            .map(|e| generate_fk_field_name(&e.name))
    }

    /// Remove a relationship by ID
    pub fn remove_relationship(&mut self, id: Uuid) -> Option<Relationship> {
        self.selected_relationships.retain(|&rid| rid != id);
        self.touch();
        self.relationships.remove(&id)
    }

    /// Get a relationship by ID
    pub fn get_relationship(&self, id: Uuid) -> Option<&Relationship> {
        self.relationships.get(&id)
    }

    /// Get a mutable relationship by ID
    pub fn get_relationship_mut(&mut self, id: Uuid) -> Option<&mut Relationship> {
        self.relationships.get_mut(&id)
    }

    /// Get all relationships for an entity
    pub fn relationships_for_entity(&self, entity_id: Uuid) -> Vec<&Relationship> {
        self.relationships
            .values()
            .filter(|r| r.from_entity_id == entity_id || r.to_entity_id == entity_id)
            .collect()
    }

    /// Get all relationships
    pub fn relationships(&self) -> impl Iterator<Item = &Relationship> {
        self.relationships.values()
    }

    /// Get the number of relationships
    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }

    // ========================================================================
    // Endpoint Management
    // ========================================================================

    /// Add an endpoint group to the project
    pub fn add_endpoint(&mut self, endpoint: EndpointGroup) -> Uuid {
        let id = endpoint.id;
        self.endpoints.insert(id, endpoint);
        self.touch();
        id
    }

    /// Remove an endpoint group by ID
    pub fn remove_endpoint(&mut self, id: Uuid) -> Option<EndpointGroup> {
        self.touch();
        self.endpoints.remove(&id)
    }

    /// Get an endpoint group by ID
    pub fn get_endpoint(&self, id: Uuid) -> Option<&EndpointGroup> {
        self.endpoints.get(&id)
    }

    /// Get a mutable endpoint group by ID
    pub fn get_endpoint_mut(&mut self, id: Uuid) -> Option<&mut EndpointGroup> {
        self.endpoints.get_mut(&id)
    }

    /// Get endpoint group for an entity
    pub fn endpoint_for_entity(&self, entity_id: Uuid) -> Option<&EndpointGroup> {
        self.endpoints.values().find(|e| e.entity_id == entity_id)
    }

    /// Get all endpoints
    pub fn endpoints(&self) -> impl Iterator<Item = &EndpointGroup> {
        self.endpoints.values()
    }

    /// Get the number of endpoint groups
    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    // ========================================================================
    // Selection Management
    // ========================================================================

    /// Select an entity
    pub fn select_entity(&mut self, id: Uuid) {
        if !self.selected_entities.contains(&id) {
            self.selected_entities.push(id);
            if let Some(entity) = self.entities.get_mut(&id) {
                entity.selected = true;
            }
        }
    }

    /// Deselect an entity
    pub fn deselect_entity(&mut self, id: Uuid) {
        self.selected_entities.retain(|&eid| eid != id);
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.selected = false;
        }
    }

    /// Toggle entity selection
    pub fn toggle_entity_selection(&mut self, id: Uuid) {
        if self.selected_entities.contains(&id) {
            self.deselect_entity(id);
        } else {
            self.select_entity(id);
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        for id in &self.selected_entities {
            if let Some(entity) = self.entities.get_mut(id) {
                entity.selected = false;
            }
        }
        self.selected_entities.clear();
        self.selected_relationships.clear();
    }

    /// Select all entities
    pub fn select_all_entities(&mut self) {
        self.selected_entities = self.entities.keys().cloned().collect();
        for entity in self.entities.values_mut() {
            entity.selected = true;
        }
    }

    /// Get selected entities
    pub fn selected_entities(&self) -> Vec<&Entity> {
        self.selected_entities
            .iter()
            .filter_map(|id| self.entities.get(id))
            .collect()
    }

    /// Check if there's any selection
    pub fn has_selection(&self) -> bool {
        !self.selected_entities.is_empty() || !self.selected_relationships.is_empty()
    }

    /// Get the count of selected items
    pub fn selection_count(&self) -> usize {
        self.selected_entities.len() + self.selected_relationships.len()
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.meta.modified_at = Utc::now();
    }

    /// Check if the project is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Clear all project data (keep metadata)
    pub fn clear(&mut self) {
        self.entities.clear();
        self.relationships.clear();
        self.endpoints.clear();
        self.selected_entities.clear();
        self.selected_relationships.clear();
        self.canvas = CanvasState::default();
        self.touch();
    }

    /// Get the maximum z-index among entities
    pub fn max_z_index(&self) -> i32 {
        self.entities.values().map(|e| e.z_index).max().unwrap_or(0)
    }

    /// Get the minimum z-index among entities
    pub fn min_z_index(&self) -> i32 {
        self.entities.values().map(|e| e.z_index).min().unwrap_or(0)
    }

    /// Delete all selected items
    pub fn delete_selected(&mut self) {
        let entities_to_remove: Vec<Uuid> = self.selected_entities.clone();
        let relationships_to_remove: Vec<Uuid> = self.selected_relationships.clone();

        for id in entities_to_remove {
            self.remove_entity(id);
        }

        for id in relationships_to_remove {
            self.remove_relationship(id);
        }

        self.clear_selection();
    }

    /// Move all selected entities by a delta
    pub fn move_selected(&mut self, dx: f32, dy: f32) {
        for id in &self.selected_entities {
            if let Some(entity) = self.entities.get_mut(id) {
                entity.translate(dx, dy);
            }
        }
        self.touch();
    }

    /// Find entity at a given position
    pub fn entity_at(&self, position: Position) -> Option<&Entity> {
        self.entities
            .values()
            .filter(|e| e.contains(position))
            .max_by_key(|e| e.z_index)
    }

    /// Find entity ID at a given position
    pub fn entity_id_at(&self, position: Position) -> Option<Uuid> {
        self.entity_at(position).map(|e| e.id)
    }
}

impl Validatable for ProjectGraph {
    fn validate(&self) -> EngineResult<()> {
        // Validate metadata
        self.meta.validate()?;

        // Validate all entities
        for entity in self.entities.values() {
            entity.validate()?;
        }

        // Check for duplicate entity names
        let mut names = std::collections::HashSet::new();
        for entity in self.entities.values() {
            if !names.insert(&entity.name) {
                return Err(EngineError::DuplicateEntity(entity.name.clone()));
            }
        }

        // Validate all relationships
        for relationship in self.relationships.values() {
            if !self.entities.contains_key(&relationship.from_entity_id) {
                return Err(EngineError::RelationshipValidation(format!(
                    "Relationship '{}' references non-existent source entity",
                    relationship.name
                )));
            }
            if !self.entities.contains_key(&relationship.to_entity_id) {
                return Err(EngineError::RelationshipValidation(format!(
                    "Relationship '{}' references non-existent target entity",
                    relationship.name
                )));
            }
        }

        // Validate all endpoints
        for endpoint in self.endpoints.values() {
            if !self.entities.contains_key(&endpoint.entity_id) {
                return Err(EngineError::EndpointValidation {
                    endpoint: endpoint.base_path.clone(),
                    message: "References non-existent entity".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Default for ProjectGraph {
    fn default() -> Self {
        Self::new("Untitled Project")
    }
}

// ============================================================================
// ProjectMeta
// ============================================================================

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    /// Unique project identifier
    pub id: Uuid,

    /// Project name
    pub name: String,

    /// Project description
    pub description: Option<String>,

    /// Project author
    pub author: Option<String>,

    /// Project version
    pub version: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub modified_at: DateTime<Utc>,

    /// Path where the project was last saved
    pub file_path: Option<String>,
}

impl ProjectMeta {
    /// Create new metadata with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            author: None,
            version: "0.1.0".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            file_path: None,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
}

impl Validatable for ProjectMeta {
    fn validate(&self) -> EngineResult<()> {
        if self.name.is_empty() {
            return Err(EngineError::validation("Project name cannot be empty"));
        }
        if self.name.len() > 100 {
            return Err(EngineError::validation(
                "Project name too long (max 100 characters)",
            ));
        }
        Ok(())
    }
}

// ============================================================================
// ProjectConfig
// ============================================================================

/// Project configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project type (REST API or Fullstack)
    pub project_type: ProjectType,

    /// Target database
    pub database: DatabaseType,

    /// Database connection configuration
    pub db_config: DatabaseConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Package name for generated Cargo.toml
    pub package_name: String,

    /// Rust edition for generated code
    pub rust_edition: String,

    /// Enable OpenAPI/Swagger documentation
    pub openapi_enabled: bool,

    /// Enable CORS
    pub cors_enabled: bool,

    /// Server host for generated project
    pub server_host: String,

    /// Server port for generated project
    pub server_port: u16,

    /// Custom configuration options
    pub custom_options: HashMap<String, String>,
}

impl ProjectConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create config for REST API project
    pub fn rest_api() -> Self {
        Self {
            project_type: ProjectType::RestApi,
            ..Default::default()
        }
    }

    /// Create config for fullstack project
    pub fn fullstack() -> Self {
        Self {
            project_type: ProjectType::Fullstack,
            ..Default::default()
        }
    }

    /// Set the database type
    pub fn with_database(mut self, database: DatabaseType) -> Self {
        self.database = database;
        self
    }

    /// Set the authentication config
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    /// Set the package name
    pub fn with_package_name(mut self, name: impl Into<String>) -> Self {
        self.package_name = name.into();
        self
    }

    /// Enable OpenAPI documentation
    pub fn with_openapi(mut self) -> Self {
        self.openapi_enabled = true;
        self
    }

    /// Set a custom option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_options.insert(key.into(), value.into());
        self
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_type: ProjectType::RestApi,
            database: DatabaseType::PostgreSQL,
            db_config: DatabaseConfig::default(),
            auth: AuthConfig::default(),
            package_name: "my_app".to_string(),
            rust_edition: "2024".to_string(),
            openapi_enabled: true,
            cors_enabled: true,
            server_host: "0.0.0.0".to_string(),
            server_port: 8080,
            custom_options: HashMap::new(),
        }
    }
}

// ============================================================================
// DatabaseConfig
// ============================================================================

/// Database connection configuration.
///
/// Holds the connection details (host, port, credentials, database name)
/// and pool settings used to generate the `.env.example` and config module
/// in the generated project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseConfig {
    /// Database server hostname or IP address.
    pub host: String,

    /// Database server port.
    pub port: u16,

    /// Database username for authentication.
    pub username: String,

    /// Database password for authentication.
    pub password: String,

    /// Name of the database to connect to.
    pub database_name: String,

    /// Maximum number of connections in the pool.
    pub max_connections: u32,

    /// Minimum number of idle connections in the pool.
    pub min_connections: u32,

    /// Connection timeout in seconds.
    pub connect_timeout_secs: u32,

    /// Idle connection timeout in seconds.
    pub idle_timeout_secs: u32,

    /// Whether to enable SSL/TLS for the connection.
    pub ssl_enabled: bool,
}

impl DatabaseConfig {
    /// Create a new database config with default values for the given database type.
    pub fn for_database(db_type: DatabaseType) -> Self {
        let (default_port, default_user) = match db_type {
            DatabaseType::PostgreSQL => (5432, "postgres"),
            DatabaseType::MySQL => (3306, "root"),
            DatabaseType::SQLite => (0, ""),
        };

        Self {
            host: if matches!(db_type, DatabaseType::SQLite) {
                String::new()
            } else {
                "localhost".to_string()
            },
            port: default_port,
            username: default_user.to_string(),
            password: String::new(),
            database_name: "my_app".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            ssl_enabled: false,
        }
    }

    /// Build the full connection URL string for this database configuration.
    ///
    /// # Examples
    ///
    /// - PostgreSQL: `postgres://user:pass@localhost:5432/my_app`
    /// - MySQL: `mysql://user:pass@localhost:3306/my_app`
    /// - SQLite: `sqlite://./my_app.db?mode=rwc`
    pub fn connection_url(&self, db_type: DatabaseType) -> String {
        match db_type {
            DatabaseType::PostgreSQL => {
                if self.password.is_empty() {
                    format!(
                        "postgres://{}@{}:{}/{}",
                        self.username, self.host, self.port, self.database_name
                    )
                } else {
                    format!(
                        "postgres://{}:{}@{}:{}/{}",
                        self.username, self.password, self.host, self.port, self.database_name
                    )
                }
            }
            DatabaseType::MySQL => {
                if self.password.is_empty() {
                    format!(
                        "mysql://{}@{}:{}/{}",
                        self.username, self.host, self.port, self.database_name
                    )
                } else {
                    format!(
                        "mysql://{}:{}@{}:{}/{}",
                        self.username, self.password, self.host, self.port, self.database_name
                    )
                }
            }
            DatabaseType::SQLite => {
                format!("sqlite://./{}.db?mode=rwc", self.database_name)
            }
        }
    }

    /// Get a display-safe connection URL (password masked).
    pub fn display_url(&self, db_type: DatabaseType) -> String {
        match db_type {
            DatabaseType::PostgreSQL => {
                let pass = if self.password.is_empty() {
                    String::new()
                } else {
                    ":****".to_string()
                };
                format!(
                    "postgres://{}{}@{}:{}/{}",
                    self.username, pass, self.host, self.port, self.database_name
                )
            }
            DatabaseType::MySQL => {
                let pass = if self.password.is_empty() {
                    String::new()
                } else {
                    ":****".to_string()
                };
                format!(
                    "mysql://{}{}@{}:{}/{}",
                    self.username, pass, self.host, self.port, self.database_name
                )
            }
            DatabaseType::SQLite => {
                format!("sqlite://./{}.db?mode=rwc", self.database_name)
            }
        }
    }

    /// Check whether the configuration has enough info for a connection attempt.
    pub fn is_configured(&self, db_type: DatabaseType) -> bool {
        match db_type {
            DatabaseType::SQLite => !self.database_name.is_empty(),
            _ => {
                !self.host.is_empty()
                    && self.port > 0
                    && !self.username.is_empty()
                    && !self.database_name.is_empty()
            }
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::for_database(DatabaseType::PostgreSQL)
    }
}

// ============================================================================
// ProjectType
// ============================================================================

/// Type of project to generate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    /// REST API only (Axum backend)
    #[default]
    RestApi,
    /// Fullstack (Axum backend + Dioxus frontend)
    Fullstack,
}

impl ProjectType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            ProjectType::RestApi => "REST API",
            ProjectType::Fullstack => "Fullstack",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            ProjectType::RestApi => "Backend-only REST API with Axum",
            ProjectType::Fullstack => "Full application with Axum backend and Dioxus frontend",
        }
    }

    /// Get all project types
    pub fn all() -> &'static [ProjectType] {
        &[ProjectType::RestApi, ProjectType::Fullstack]
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// AuthConfig
// ============================================================================

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is enabled
    pub enabled: bool,

    /// Authentication strategy
    pub strategy: AuthStrategy,

    /// Environment variable name for JWT secret
    pub jwt_secret_env: String,

    /// Token expiration time in hours
    pub token_expiry_hours: u32,

    /// Whether to generate user registration endpoint
    pub enable_registration: bool,

    /// Whether to generate password reset endpoints
    pub enable_password_reset: bool,

    /// Default roles for new users
    pub default_roles: Vec<String>,

    /// Available roles in the system
    pub available_roles: Vec<String>,
}

impl AuthConfig {
    /// Create a new auth config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create config with JWT authentication
    pub fn jwt() -> Self {
        Self {
            enabled: true,
            strategy: AuthStrategy::Jwt,
            ..Default::default()
        }
    }

    /// Create config with session authentication
    pub fn session() -> Self {
        Self {
            enabled: true,
            strategy: AuthStrategy::Session,
            ..Default::default()
        }
    }

    /// Create config with no authentication
    pub fn none() -> Self {
        Self {
            enabled: false,
            strategy: AuthStrategy::None,
            ..Default::default()
        }
    }

    /// Enable authentication
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Set the strategy
    pub fn with_strategy(mut self, strategy: AuthStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set token expiry
    pub fn with_expiry_hours(mut self, hours: u32) -> Self {
        self.token_expiry_hours = hours;
        self
    }

    /// Add an available role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        let role = role.into();
        if !self.available_roles.contains(&role) {
            self.available_roles.push(role);
        }
        self
    }

    /// Enable registration
    pub fn with_registration(mut self) -> Self {
        self.enable_registration = true;
        self
    }

    /// Enable password reset
    pub fn with_password_reset(mut self) -> Self {
        self.enable_password_reset = true;
        self
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: AuthStrategy::Jwt,
            jwt_secret_env: "JWT_SECRET".to_string(),
            token_expiry_hours: 24,
            enable_registration: true,
            enable_password_reset: false,
            default_roles: vec!["user".to_string()],
            available_roles: vec!["user".to_string(), "admin".to_string()],
        }
    }
}

// ============================================================================
// AuthStrategy
// ============================================================================

/// Authentication strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthStrategy {
    /// No authentication
    None,
    /// JWT token-based authentication
    #[default]
    Jwt,
    /// Session-based authentication
    Session,
    /// API Key authentication
    ApiKey,
}

impl AuthStrategy {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AuthStrategy::None => "None",
            AuthStrategy::Jwt => "JWT",
            AuthStrategy::Session => "Session",
            AuthStrategy::ApiKey => "API Key",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            AuthStrategy::None => "No authentication (public endpoints)",
            AuthStrategy::Jwt => "JSON Web Token based stateless authentication",
            AuthStrategy::Session => "Server-side session based authentication",
            AuthStrategy::ApiKey => "API Key based authentication",
        }
    }

    /// Get all strategies
    pub fn all() -> &'static [AuthStrategy] {
        &[
            AuthStrategy::None,
            AuthStrategy::Jwt,
            AuthStrategy::Session,
            AuthStrategy::ApiKey,
        ]
    }
}

impl std::fmt::Display for AuthStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// CanvasState
// ============================================================================

/// State of the canvas (pan, zoom, etc.)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CanvasState {
    /// Horizontal pan offset
    pub pan_x: f32,

    /// Vertical pan offset
    pub pan_y: f32,

    /// Zoom level (1.0 = 100%)
    pub zoom: f32,

    /// Grid size in pixels
    pub grid_size: f32,

    /// Whether to show the grid
    pub show_grid: bool,

    /// Whether to snap to grid
    pub snap_to_grid: bool,
}

impl CanvasState {
    /// Create new canvas state
    pub fn new() -> Self {
        Self::default()
    }

    /// Pan by a delta
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    /// Set the pan position
    pub fn set_pan(&mut self, x: f32, y: f32) {
        self.pan_x = x;
        self.pan_y = y;
    }

    /// Zoom by a factor
    pub fn zoom_by(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.1, 5.0);
    }

    /// Set the zoom level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 5.0);
    }

    /// Reset to default view
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen_x: f32, screen_y: f32) -> Position {
        Position::new(
            (screen_x - self.pan_x) / self.zoom,
            (screen_y - self.pan_y) / self.zoom,
        )
    }

    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas_pos: Position) -> Position {
        Position::new(
            canvas_pos.x * self.zoom + self.pan_x,
            canvas_pos.y * self.zoom + self.pan_y,
        )
    }

    /// Snap a position to the grid
    pub fn snap_position(&self, pos: Position) -> Position {
        if self.snap_to_grid {
            Position::new(
                (pos.x / self.grid_size).round() * self.grid_size,
                (pos.y / self.grid_size).round() * self.grid_size,
            )
        } else {
            pos
        }
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            grid_size: 20.0,
            show_grid: true,
            snap_to_grid: true,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Entity;

    #[test]
    fn test_project_new() {
        let project = ProjectGraph::new("Test Project");
        assert_eq!(project.meta.name, "Test Project");
        assert!(project.is_empty());
        assert_eq!(project.schema_version, crate::SCHEMA_VERSION);
    }

    #[test]
    fn test_project_add_entity() {
        let mut project = ProjectGraph::new("Test");
        let entity = Entity::new("User");
        let id = entity.id;

        project.add_entity(entity);

        assert_eq!(project.entity_count(), 1);
        assert!(project.get_entity(id).is_some());
        assert_eq!(project.get_entity(id).unwrap().name, "User");
    }

    #[test]
    fn test_project_remove_entity() {
        let mut project = ProjectGraph::new("Test");
        let entity = Entity::new("User");
        let id = entity.id;
        project.add_entity(entity);

        let removed = project.remove_entity(id);

        assert!(removed.is_some());
        assert!(project.is_empty());
    }

    #[test]
    fn test_project_selection() {
        let mut project = ProjectGraph::new("Test");
        let entity = Entity::new("User");
        let id = entity.id;
        project.add_entity(entity);

        project.select_entity(id);
        assert!(project.has_selection());
        assert_eq!(project.selection_count(), 1);

        project.deselect_entity(id);
        assert!(!project.has_selection());

        project.select_entity(id);
        project.clear_selection();
        assert!(!project.has_selection());
    }

    #[test]
    fn test_project_validation() {
        let project = ProjectGraph::new("Valid Project");
        assert!(project.validate().is_ok());

        let mut invalid = ProjectGraph::new("");
        invalid.meta.name = String::new();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_project_config() {
        let config = ProjectConfig::rest_api()
            .with_database(DatabaseType::PostgreSQL)
            .with_auth(AuthConfig::jwt());

        assert_eq!(config.project_type, ProjectType::RestApi);
        assert_eq!(config.database, DatabaseType::PostgreSQL);
        assert!(config.auth.enabled);
        assert_eq!(config.auth.strategy, AuthStrategy::Jwt);
    }

    #[test]
    fn test_canvas_state() {
        let mut canvas = CanvasState::default();

        canvas.pan(10.0, 20.0);
        assert_eq!(canvas.pan_x, 10.0);
        assert_eq!(canvas.pan_y, 20.0);

        canvas.zoom_by(1.5);
        assert!((canvas.zoom - 1.5).abs() < 0.001);

        canvas.reset();
        assert_eq!(canvas.pan_x, 0.0);
        assert_eq!(canvas.zoom, 1.0);
    }

    #[test]
    fn test_canvas_snap_to_grid() {
        let canvas = CanvasState {
            snap_to_grid: true,
            grid_size: 20.0,
            ..Default::default()
        };

        let pos = Position::new(15.0, 27.0);
        let snapped = canvas.snap_position(pos);
        assert_eq!(snapped.x, 20.0);
        assert_eq!(snapped.y, 20.0);

        let canvas_no_snap = CanvasState {
            snap_to_grid: false,
            ..Default::default()
        };
        let not_snapped = canvas_no_snap.snap_position(pos);
        assert_eq!(not_snapped.x, 15.0);
        assert_eq!(not_snapped.y, 27.0);
    }

    #[test]
    fn test_canvas_coordinate_conversion() {
        let canvas = CanvasState {
            pan_x: 100.0,
            pan_y: 50.0,
            zoom: 2.0,
            ..Default::default()
        };

        // Canvas to screen
        let canvas_pos = Position::new(10.0, 20.0);
        let screen_pos = canvas.canvas_to_screen(canvas_pos);
        assert_eq!(screen_pos.x, 120.0); // 10 * 2 + 100
        assert_eq!(screen_pos.y, 90.0); // 20 * 2 + 50

        // Screen to canvas
        let screen_pos = Position::new(120.0, 90.0);
        let canvas_pos = canvas.screen_to_canvas(screen_pos.x, screen_pos.y);
        assert_eq!(canvas_pos.x, 10.0);
        assert_eq!(canvas_pos.y, 20.0);
    }

    #[test]
    fn test_project_type() {
        assert_eq!(ProjectType::RestApi.display_name(), "REST API");
        assert_eq!(ProjectType::Fullstack.display_name(), "Fullstack");
    }

    #[test]
    fn test_auth_config() {
        let config = AuthConfig::jwt()
            .with_expiry_hours(48)
            .with_role("editor")
            .with_registration();

        assert!(config.enabled);
        assert_eq!(config.strategy, AuthStrategy::Jwt);
        assert_eq!(config.token_expiry_hours, 48);
        assert!(config.available_roles.contains(&"editor".to_string()));
        assert!(config.enable_registration);
    }

    #[test]
    fn test_auth_strategy() {
        assert_eq!(AuthStrategy::Jwt.display_name(), "JWT");
        assert_eq!(AuthStrategy::Session.display_name(), "Session");
        assert_eq!(AuthStrategy::None.display_name(), "None");
    }

    #[test]
    fn test_project_meta() {
        let meta = ProjectMeta::new("My Project")
            .with_author("John Doe")
            .with_description("A test project")
            .with_version("1.0.0");

        assert_eq!(meta.name, "My Project");
        assert_eq!(meta.author, Some("John Doe".to_string()));
        assert_eq!(meta.description, Some("A test project".to_string()));
        assert_eq!(meta.version, "1.0.0");
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_project_entity_at() {
        let mut project = ProjectGraph::new("Test");

        let entity1 = Entity::new("User").at(0.0, 0.0);
        let entity2 = Entity::new("Post").at(300.0, 0.0);

        let id1 = entity1.id;
        project.add_entity(entity1);
        project.add_entity(entity2);

        // Find entity at position
        let found = project.entity_at(Position::new(50.0, 50.0));
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id1);

        // No entity at this position
        let not_found = project.entity_at(Position::new(600.0, 600.0));
        assert!(not_found.is_none());
    }

    #[test]
    fn test_project_move_selected() {
        let mut project = ProjectGraph::new("Test");

        let entity = Entity::new("User").at(100.0, 100.0);
        let id = entity.id;
        project.add_entity(entity);
        project.select_entity(id);

        project.move_selected(50.0, 25.0);

        let moved = project.get_entity(id).unwrap();
        assert_eq!(moved.position.x, 150.0);
        assert_eq!(moved.position.y, 125.0);
    }

    #[test]
    fn test_project_delete_selected() {
        let mut project = ProjectGraph::new("Test");

        let entity = Entity::new("User");
        let id = entity.id;
        project.add_entity(entity);
        project.select_entity(id);

        assert_eq!(project.entity_count(), 1);
        project.delete_selected();
        assert_eq!(project.entity_count(), 0);
    }

    #[test]
    fn test_create_relationship_with_fk() {
        use imortal_core::RelationType;

        let mut project = ProjectGraph::new("Test");

        // Create two entities
        let user = Entity::new("User");
        let post = Entity::new("Post");
        let user_id = user.id;
        let post_id = post.id;

        project.add_entity(user);
        project.add_entity(post);

        // Create a One-to-Many relationship (User has many Posts)
        let relationship = Relationship::new(user_id, post_id, RelationType::OneToMany);

        // Should auto-generate FK field on Post entity
        let result = project.create_relationship_with_fk(relationship);
        assert!(result.is_ok());

        let (rel_id, fk_field_id) = result.unwrap();

        // Relationship should be created
        assert!(project.relationships.contains_key(&rel_id));

        // FK field should be created on Post (the "many" side)
        assert!(fk_field_id.is_some());
        let fk_id = fk_field_id.unwrap();

        let post_entity = project.get_entity(post_id).unwrap();
        let fk_field = post_entity.get_field(fk_id);
        assert!(fk_field.is_some());

        let fk = fk_field.unwrap();
        assert_eq!(fk.name, "user_id");
        assert!(fk.is_foreign_key);
        assert!(fk.indexed);
    }

    #[test]
    fn test_create_relationship_with_fk_many_to_many() {
        use imortal_core::RelationType;

        let mut project = ProjectGraph::new("Test");

        // Create two entities
        let user = Entity::new("User");
        let role = Entity::new("Role");
        let user_id = user.id;
        let role_id = role.id;

        project.add_entity(user);
        project.add_entity(role);

        // Create a Many-to-Many relationship
        let relationship = Relationship::many_to_many(user_id, role_id, "user_roles");

        // Should NOT create FK field (M:N uses junction table)
        let result = project.create_relationship_with_fk(relationship);
        assert!(result.is_ok());

        let (rel_id, fk_field_id) = result.unwrap();

        // Relationship should be created
        assert!(project.relationships.contains_key(&rel_id));

        // No FK field for M:N
        assert!(fk_field_id.is_none());
    }

    #[test]
    fn test_suggest_fk_field_name() {
        let mut project = ProjectGraph::new("Test");

        let user = Entity::new("User");
        let user_id = user.id;
        project.add_entity(user);

        let suggested = project.suggest_fk_field_name(user_id);
        assert_eq!(suggested, Some("user_id".to_string()));
    }
}
