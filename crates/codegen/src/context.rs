//! # Generation Context
//!
//! The `GenerationContext` holds all the information needed by code generators
//! to produce output files. It is built from a `ProjectGraph` and provides
//! convenient accessors and helper methods for:
//!
//! - Project metadata (name, package name, database, auth strategy)
//! - Sorted entity lists with dependency ordering
//! - Relationship lookups per entity
//! - Endpoint lookups per entity
//! - Case conversion utilities (snake_case, PascalCase, plural forms)
//! - Data type mapping (Rust types, SQL types, SeaORM types)
//!

use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use imortal_core::{DataType, IdType};
use imortal_ir::{
    AuthConfig, AuthStrategy, DatabaseType, EndpointGroup, Entity, Field, ProjectConfig,
    ProjectGraph, ProjectMeta, ProjectType, Relationship,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::GeneratorConfig;

// ============================================================================
// GenerationContext
// ============================================================================

/// Context carrying all information needed for code generation.
///
/// Built once from a `ProjectGraph` and shared (by reference) with every
/// individual generator module.
#[derive(Debug, Clone)]
pub struct GenerationContext {
    // ── project-level ────────────────────────────────────────────────────
    /// Project metadata (name, author, description, …)
    pub meta: ProjectMeta,

    /// Project configuration (database, auth, edition, …)
    pub config: ProjectConfig,

    /// Generator configuration (output dir, flags, …)
    pub generator_config: GeneratorConfig,

    // ── entities ─────────────────────────────────────────────────────────
    /// All entities, sorted by dependency order (referenced first).
    entities: Vec<Entity>,

    /// Lookup: entity id → index into `entities`
    entity_index: HashMap<Uuid, usize>,

    // ── relationships ────────────────────────────────────────────────────
    /// All relationships
    relationships: Vec<Relationship>,

    /// Lookup: entity id → relationships where the entity is the *source*
    outgoing: HashMap<Uuid, Vec<usize>>,

    /// Lookup: entity id → relationships where the entity is the *target*
    incoming: HashMap<Uuid, Vec<usize>>,

    // ── endpoints ────────────────────────────────────────────────────────
    /// All endpoint groups
    endpoints: Vec<EndpointGroup>,

    /// Lookup: entity id → endpoint group index
    endpoint_by_entity: HashMap<Uuid, usize>,

    // ── derived ──────────────────────────────────────────────────────────
    /// Timestamp prefix for migration files (YYYYMMDD)
    pub migration_date_prefix: String,
}

impl GenerationContext {
    // ====================================================================
    // Construction
    // ====================================================================

    /// Build a `GenerationContext` from a `ProjectGraph` and generator config.
    pub fn from_project(project: &ProjectGraph, generator_config: GeneratorConfig) -> Self {
        // Collect and sort entities by dependency order
        let entities = Self::dependency_sorted_entities(project);

        let entity_index: HashMap<Uuid, usize> = entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id, i))
            .collect();

        // Collect relationships
        let relationships: Vec<Relationship> = project.relationships.values().cloned().collect();

        let mut outgoing: HashMap<Uuid, Vec<usize>> = HashMap::new();
        let mut incoming: HashMap<Uuid, Vec<usize>> = HashMap::new();
        for (i, rel) in relationships.iter().enumerate() {
            outgoing.entry(rel.from_entity_id).or_default().push(i);
            incoming.entry(rel.to_entity_id).or_default().push(i);
        }

        // Collect endpoints
        let endpoints: Vec<EndpointGroup> = project.endpoints.values().cloned().collect();
        let endpoint_by_entity: HashMap<Uuid, usize> = endpoints
            .iter()
            .enumerate()
            .map(|(i, ep)| (ep.entity_id, i))
            .collect();

        // Build migration date prefix from current time
        let now = chrono::Utc::now();
        let migration_date_prefix = now.format("%Y%m%d").to_string();

        Self {
            meta: project.meta.clone(),
            config: project.config.clone(),
            generator_config,
            entities,
            entity_index,
            relationships,
            outgoing,
            incoming,
            endpoints,
            endpoint_by_entity,
            migration_date_prefix,
        }
    }

    /// Build with default generator config (convenience for tests).
    pub fn from_project_default(project: &ProjectGraph) -> Self {
        Self::from_project(project, GeneratorConfig::default())
    }

    // ====================================================================
    // Entity accessors
    // ====================================================================

    /// All entities in dependency order (referenced tables first).
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Number of entities.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get an entity by id.
    pub fn entity_by_id(&self, id: Uuid) -> Option<&Entity> {
        self.entity_index.get(&id).map(|&i| &self.entities[i])
    }

    /// Get an entity by name (case-insensitive).
    pub fn entity_by_name(&self, name: &str) -> Option<&Entity> {
        let lower = name.to_lowercase();
        self.entities
            .iter()
            .find(|e| e.name.to_lowercase() == lower)
    }

    /// Get the non-PK, non-generated fields that the user should provide
    /// when *creating* a resource.
    pub fn create_fields<'a>(&self, entity: &'a Entity) -> Vec<&'a Field> {
        entity
            .fields
            .iter()
            .filter(|f| {
                !f.is_primary_key
                    && !f.readonly
                    && f.name != "created_at"
                    && f.name != "updated_at"
                    && f.name != "deleted_at"
            })
            .collect()
    }

    /// Get the fields that the user may update (non-PK, non-FK, non-readonly,
    /// non-timestamp). All wrapped in `Option<T>` for partial updates.
    pub fn update_fields<'a>(&self, entity: &'a Entity) -> Vec<&'a Field> {
        entity
            .fields
            .iter()
            .filter(|f| {
                !f.is_primary_key
                    && !f.is_foreign_key
                    && !f.readonly
                    && !f.secret
                    && f.name != "created_at"
                    && f.name != "updated_at"
                    && f.name != "deleted_at"
            })
            .collect()
    }

    /// Get the fields to include in a *response* DTO (everything except
    /// secrets like password hashes).
    pub fn response_fields<'a>(&self, entity: &'a Entity) -> Vec<&'a Field> {
        entity.fields.iter().filter(|f| !f.secret).collect()
    }

    /// Get the primary key field, or `None`.
    pub fn primary_key_field<'a>(&self, entity: &'a Entity) -> Option<&'a Field> {
        entity.fields.iter().find(|f| f.is_primary_key)
    }

    /// Get all foreign-key fields in an entity.
    pub fn foreign_key_fields<'a>(&self, entity: &'a Entity) -> Vec<&'a Field> {
        entity.fields.iter().filter(|f| f.is_foreign_key).collect()
    }

    // ====================================================================
    // Relationship accessors
    // ====================================================================

    /// All relationships.
    pub fn relationships(&self) -> &[Relationship] {
        &self.relationships
    }

    /// Relationships where `entity_id` is the *source* (from).
    pub fn outgoing_relationships(&self, entity_id: Uuid) -> Vec<&Relationship> {
        self.outgoing
            .get(&entity_id)
            .map(|indices| indices.iter().map(|&i| &self.relationships[i]).collect())
            .unwrap_or_default()
    }

    /// Relationships where `entity_id` is the *target* (to).
    pub fn incoming_relationships(&self, entity_id: Uuid) -> Vec<&Relationship> {
        self.incoming
            .get(&entity_id)
            .map(|indices| indices.iter().map(|&i| &self.relationships[i]).collect())
            .unwrap_or_default()
    }

    // ====================================================================
    // Endpoint accessors
    // ====================================================================

    /// All endpoint groups.
    pub fn endpoints(&self) -> &[EndpointGroup] {
        &self.endpoints
    }

    /// Get the endpoint group for a specific entity.
    pub fn endpoint_for_entity(&self, entity_id: Uuid) -> Option<&EndpointGroup> {
        self.endpoint_by_entity
            .get(&entity_id)
            .map(|&i| &self.endpoints[i])
    }

    // ====================================================================
    // Project-level helpers
    // ====================================================================

    /// The Cargo package name for the generated project.
    pub fn package_name(&self) -> &str {
        &self.config.package_name
    }

    /// Rust edition string (e.g. "2024").
    pub fn rust_edition(&self) -> &str {
        &self.config.rust_edition
    }

    /// Target database type.
    pub fn database(&self) -> DatabaseType {
        self.config.database
    }

    /// Whether auth is enabled.
    pub fn auth_enabled(&self) -> bool {
        self.config.auth.enabled
    }

    /// Auth strategy.
    pub fn auth_strategy(&self) -> AuthStrategy {
        self.config.auth.strategy
    }

    /// Auth configuration.
    pub fn auth_config(&self) -> &AuthConfig {
        &self.config.auth
    }

    /// Is this a fullstack project?
    pub fn is_fullstack(&self) -> bool {
        matches!(self.config.project_type, ProjectType::Fullstack)
    }

    /// Is OpenAPI generation enabled?
    pub fn openapi_enabled(&self) -> bool {
        self.config.openapi_enabled
    }

    /// Server host.
    pub fn server_host(&self) -> &str {
        &self.config.server_host
    }

    /// Server port.
    pub fn server_port(&self) -> u16 {
        self.config.server_port
    }

    /// Whether to generate tests.
    pub fn generate_tests(&self) -> bool {
        self.generator_config.generate_tests
    }

    /// Whether to generate doc comments.
    pub fn generate_docs(&self) -> bool {
        self.generator_config.generate_docs
    }

    /// Whether to generate migrations.
    pub fn generate_migrations(&self) -> bool {
        self.generator_config.generate_migrations
    }

    // ====================================================================
    // Naming helpers
    // ====================================================================

    /// Convert a name to `snake_case` (e.g. "BlogPost" → "blog_post").
    pub fn snake(name: &str) -> String {
        name.to_snake_case()
    }

    /// Convert a name to `PascalCase` (e.g. "blog_post" → "BlogPost").
    pub fn pascal(name: &str) -> String {
        name.to_pascal_case()
    }

    /// Convert a name to `camelCase` (e.g. "blog_post" → "blogPost").
    pub fn camel(name: &str) -> String {
        name.to_lower_camel_case()
    }

    /// Pluralise a snake_case word with simple English heuristics.
    pub fn pluralize(word: &str) -> String {
        let s = word.to_snake_case();
        if s.ends_with('s')
            || s.ends_with('x')
            || s.ends_with("ch")
            || s.ends_with("sh")
            || s.ends_with("ss")
        {
            format!("{}es", s)
        } else if s.ends_with('y')
            && !s.ends_with("ey")
            && !s.ends_with("ay")
            && !s.ends_with("oy")
            && !s.ends_with("uy")
        {
            format!("{}ies", &s[..s.len() - 1])
        } else {
            format!("{}s", s)
        }
    }

    /// Entity name → module file name (snake_case, e.g. "User" → "user").
    pub fn module_name(entity_name: &str) -> String {
        Self::snake(entity_name)
    }

    /// Entity name → table name (snake_case plural, e.g. "User" → "users").
    pub fn table_name(entity_name: &str) -> String {
        Self::pluralize(&Self::snake(entity_name))
    }

    /// Entity name → API base path (e.g. "BlogPost" → "/api/blog_posts").
    pub fn default_base_path(entity_name: &str) -> String {
        format!("/api/{}", Self::pluralize(&Self::snake(entity_name)))
    }

    /// Entity name → DTO struct names.
    pub fn create_dto_name(entity_name: &str) -> String {
        format!("Create{}Dto", Self::pascal(entity_name))
    }

    pub fn update_dto_name(entity_name: &str) -> String {
        format!("Update{}Dto", Self::pascal(entity_name))
    }

    pub fn response_dto_name(entity_name: &str) -> String {
        format!("{}Response", Self::pascal(entity_name))
    }

    // ====================================================================
    // Type mapping helpers
    // ====================================================================

    /// Map a `DataType` to a Rust type string suitable for use in
    /// generated source code.
    pub fn rust_type(dt: &DataType) -> String {
        dt.to_rust_type()
    }

    /// Map a `DataType` to its SeaORM `ColumnType` variant string.
    pub fn sea_orm_type(dt: &DataType) -> String {
        dt.to_sea_orm_type()
    }

    /// Map a `DataType` to a SQL column type for the given database.
    pub fn sql_type(dt: &DataType, db: DatabaseType) -> String {
        match dt {
            DataType::String => match db {
                DatabaseType::PostgreSQL | DatabaseType::MySQL => "VARCHAR(255)".into(),
                DatabaseType::SQLite => "TEXT".into(),
            },
            DataType::Text => match db {
                DatabaseType::PostgreSQL => "TEXT".into(),
                DatabaseType::MySQL => "LONGTEXT".into(),
                DatabaseType::SQLite => "TEXT".into(),
            },
            DataType::Int32 => match db {
                DatabaseType::PostgreSQL => "INTEGER".into(),
                DatabaseType::MySQL => "INT".into(),
                DatabaseType::SQLite => "INTEGER".into(),
            },
            DataType::Int64 => match db {
                DatabaseType::PostgreSQL => "BIGINT".into(),
                DatabaseType::MySQL => "BIGINT".into(),
                DatabaseType::SQLite => "INTEGER".into(),
            },
            DataType::Float32 => match db {
                DatabaseType::PostgreSQL => "REAL".into(),
                DatabaseType::MySQL => "FLOAT".into(),
                DatabaseType::SQLite => "REAL".into(),
            },
            DataType::Float64 => match db {
                DatabaseType::PostgreSQL => "DOUBLE PRECISION".into(),
                DatabaseType::MySQL => "DOUBLE".into(),
                DatabaseType::SQLite => "REAL".into(),
            },
            DataType::Bool => match db {
                DatabaseType::PostgreSQL => "BOOLEAN".into(),
                DatabaseType::MySQL => "TINYINT(1)".into(),
                DatabaseType::SQLite => "INTEGER".into(),
            },
            DataType::Uuid => match db {
                DatabaseType::PostgreSQL => "UUID".into(),
                DatabaseType::MySQL => "CHAR(36)".into(),
                DatabaseType::SQLite => "TEXT".into(),
            },
            DataType::DateTime => match db {
                DatabaseType::PostgreSQL => "TIMESTAMP WITH TIME ZONE".into(),
                DatabaseType::MySQL => "DATETIME".into(),
                DatabaseType::SQLite => "TEXT".into(),
            },
            DataType::Date => match db {
                DatabaseType::PostgreSQL | DatabaseType::MySQL => "DATE".into(),
                DatabaseType::SQLite => "TEXT".into(),
            },
            DataType::Time => match db {
                DatabaseType::PostgreSQL | DatabaseType::MySQL => "TIME".into(),
                DatabaseType::SQLite => "TEXT".into(),
            },
            DataType::Bytes => match db {
                DatabaseType::PostgreSQL => "BYTEA".into(),
                DatabaseType::MySQL | DatabaseType::SQLite => "BLOB".into(),
            },
            DataType::Json => match db {
                DatabaseType::PostgreSQL => "JSONB".into(),
                DatabaseType::MySQL => "JSON".into(),
                DatabaseType::SQLite => "TEXT".into(),
            },
            DataType::Optional(inner) => Self::sql_type(inner, db),
            DataType::Array(inner) => {
                // PostgreSQL has native arrays; others fall back to JSON
                match db {
                    DatabaseType::PostgreSQL => {
                        format!("{}[]", Self::sql_type(inner, db))
                    }
                    _ => "JSON".into(),
                }
            }
            DataType::Reference { .. } => Self::sql_type(&DataType::Uuid, db),
            DataType::Enum { name, variants } => match db {
                DatabaseType::PostgreSQL => format!("VARCHAR(50) /* enum {} */", name),
                DatabaseType::MySQL => {
                    let opts = variants
                        .iter()
                        .map(|v| format!("'{}'", v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("ENUM({})", opts)
                }
                DatabaseType::SQLite => "TEXT".into(),
            },
        }
    }

    /// Return the SQL `DEFAULT` clause string for a field, if any.
    pub fn sql_default(field: &Field, db: DatabaseType) -> Option<String> {
        use imortal_ir::DefaultValue;
        field.default_value.as_ref().map(|dv| match dv {
            DefaultValue::Null => "DEFAULT NULL".into(),
            DefaultValue::Bool(b) => {
                if *b {
                    "DEFAULT TRUE".into()
                } else {
                    "DEFAULT FALSE".into()
                }
            }
            DefaultValue::Int(i) => format!("DEFAULT {}", i),
            DefaultValue::Float(f) => format!("DEFAULT {}", f),
            DefaultValue::String(s) => format!("DEFAULT '{}'", s.replace('\'', "''")),
            DefaultValue::Now => match db {
                DatabaseType::PostgreSQL => "DEFAULT CURRENT_TIMESTAMP".into(),
                DatabaseType::MySQL => "DEFAULT CURRENT_TIMESTAMP".into(),
                DatabaseType::SQLite => "DEFAULT CURRENT_TIMESTAMP".into(),
            },
            DefaultValue::Uuid => match db {
                DatabaseType::PostgreSQL => "DEFAULT gen_random_uuid()".into(),
                _ => String::new(),
            },
            DefaultValue::Expression(expr) => format!("DEFAULT {}", expr),
            DefaultValue::EmptyArray => match db {
                DatabaseType::PostgreSQL => "DEFAULT '{}'".into(),
                _ => "DEFAULT '[]'".into(),
            },
            DefaultValue::EmptyObject => "DEFAULT '{}'".into(),
        })
    }

    /// Map `IdType` to the SQL column type for the primary key.
    pub fn pk_sql_type(id_type: IdType, db: DatabaseType) -> String {
        match id_type {
            IdType::Uuid => Self::sql_type(&DataType::Uuid, db),
            IdType::Serial => match db {
                DatabaseType::PostgreSQL => "SERIAL".into(),
                DatabaseType::MySQL => "INT AUTO_INCREMENT".into(),
                DatabaseType::SQLite => "INTEGER".into(),
            },
            IdType::Cuid => "VARCHAR(30)".into(),
            IdType::Ulid => "VARCHAR(26)".into(),
        }
    }

    /// Map `IdType` to the Rust type used for the primary key.
    pub fn pk_rust_type(id_type: IdType) -> &'static str {
        match id_type {
            IdType::Uuid => "Uuid",
            IdType::Serial => "i32",
            IdType::Cuid => "String",
            IdType::Ulid => "String",
        }
    }

    /// Database connection string environment variable name.
    pub fn database_url_env(&self) -> &'static str {
        "DATABASE_URL"
    }

    /// Connection string for `.env.example`.
    ///
    /// Uses the configured `DatabaseConfig` (host, port, username, password,
    /// database name) to build the URL. If the user filled in connection
    /// details in the Project Setup page, those values are used; otherwise
    /// the defaults for the chosen database type apply.
    pub fn example_database_url(&self) -> String {
        self.config.db_config.connection_url(self.database())
    }

    /// Database pool max connections from config.
    pub fn db_max_connections(&self) -> u32 {
        self.config.db_config.max_connections
    }

    /// Database pool min connections from config.
    pub fn db_min_connections(&self) -> u32 {
        self.config.db_config.min_connections
    }

    // ====================================================================
    // Migration ordering
    // ====================================================================

    /// Generate a migration file name with ordering index.
    pub fn migration_filename(&self, index: usize, table_name: &str) -> String {
        format!(
            "{}{:06}_create_{}.sql",
            self.migration_date_prefix, index, table_name
        )
    }

    // ====================================================================
    // Internal: dependency-ordered entity sort
    // ====================================================================

    /// Topological sort of entities so that referenced tables come first.
    fn dependency_sorted_entities(project: &ProjectGraph) -> Vec<Entity> {
        let entities: Vec<Entity> = project.entities.values().cloned().collect();
        if entities.len() <= 1 {
            return entities;
        }

        // Build adjacency: entity depends on other entity if it has an FK
        // pointing to the other.
        let id_to_idx: HashMap<Uuid, usize> = entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id, i))
            .collect();

        let n = entities.len();
        let mut in_degree = vec![0u32; n];
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];

        for rel in project.relationships.values() {
            // to_entity depends on from_entity for OneToMany etc.
            if let (Some(&from_idx), Some(&to_idx)) = (
                id_to_idx.get(&rel.from_entity_id),
                id_to_idx.get(&rel.to_entity_id),
            ) {
                if from_idx != to_idx {
                    adj[from_idx].push(to_idx);
                    in_degree[to_idx] += 1;
                }
            }
        }

        // Also check FK fields directly
        for (i, entity) in entities.iter().enumerate() {
            for field in &entity.fields {
                if let Some(fk) = &field.foreign_key_ref {
                    if let Some(&dep_idx) = id_to_idx.get(&fk.entity_id) {
                        if dep_idx != i && !adj[dep_idx].contains(&i) {
                            adj[dep_idx].push(i);
                            in_degree[i] += 1;
                        }
                    }
                }
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        let mut sorted_indices: Vec<usize> = Vec::with_capacity(n);

        while let Some(node) = queue.pop() {
            sorted_indices.push(node);
            for &neighbor in &adj[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push(neighbor);
                }
            }
        }

        // If we couldn't sort everything (cycles), append remaining
        if sorted_indices.len() < n {
            for i in 0..n {
                if !sorted_indices.contains(&i) {
                    sorted_indices.push(i);
                }
            }
        }

        sorted_indices
            .into_iter()
            .map(|i| entities[i].clone())
            .collect()
    }
}

// ============================================================================
// EntityInfo — convenience wrapper for a single entity during generation
// ============================================================================

/// Lightweight wrapper for generating code for one entity at a time.
#[derive(Debug, Clone)]
pub struct EntityInfo<'a> {
    pub entity: &'a Entity,
    pub ctx: &'a GenerationContext,
}

impl<'a> EntityInfo<'a> {
    pub fn new(entity: &'a Entity, ctx: &'a GenerationContext) -> Self {
        Self { entity, ctx }
    }

    /// The Rust module name (snake_case).
    pub fn module_name(&self) -> String {
        GenerationContext::module_name(&self.entity.name)
    }

    /// The database table name.
    pub fn table_name(&self) -> String {
        if self.entity.table_name.is_empty() {
            GenerationContext::table_name(&self.entity.name)
        } else {
            self.entity.table_name.clone()
        }
    }

    /// PascalCase entity name.
    pub fn pascal_name(&self) -> String {
        GenerationContext::pascal(&self.entity.name)
    }

    /// snake_case entity name.
    pub fn snake_name(&self) -> String {
        GenerationContext::snake(&self.entity.name)
    }

    /// Plural snake_case (for collections / routes).
    pub fn plural_name(&self) -> String {
        GenerationContext::pluralize(&self.snake_name())
    }

    /// The primary-key field (or None).
    pub fn pk(&self) -> Option<&'a Field> {
        self.ctx.primary_key_field(self.entity)
    }

    /// PK Rust type.
    pub fn pk_rust_type(&self) -> String {
        self.pk()
            .map(|f| GenerationContext::rust_type(&f.data_type))
            .unwrap_or_else(|| "Uuid".to_string())
    }

    /// Fields suitable for a Create DTO.
    pub fn create_fields(&self) -> Vec<&'a Field> {
        self.ctx.create_fields(self.entity)
    }

    /// Fields suitable for an Update DTO.
    pub fn update_fields(&self) -> Vec<&'a Field> {
        self.ctx.update_fields(self.entity)
    }

    /// Fields suitable for a Response DTO.
    pub fn response_fields(&self) -> Vec<&'a Field> {
        self.ctx.response_fields(self.entity)
    }

    /// Whether this entity has timestamps (created_at / updated_at).
    pub fn has_timestamps(&self) -> bool {
        self.entity.config.timestamps
    }

    /// Whether soft-delete is enabled.
    pub fn has_soft_delete(&self) -> bool {
        self.entity.config.soft_delete
    }

    /// The ID type for the entity.
    pub fn id_type(&self) -> IdType {
        self.entity.config.id_type
    }

    /// Outgoing relationships from this entity.
    pub fn outgoing_relationships(&self) -> Vec<&'a Relationship> {
        self.ctx.outgoing_relationships(self.entity.id)
    }

    /// Incoming relationships to this entity.
    pub fn incoming_relationships(&self) -> Vec<&'a Relationship> {
        self.ctx.incoming_relationships(self.entity.id)
    }

    /// Endpoint group for this entity (if configured).
    pub fn endpoint(&self) -> Option<&'a EndpointGroup> {
        self.ctx.endpoint_for_entity(self.entity.id)
    }

    /// The default API base path for this entity.
    pub fn base_path(&self) -> String {
        self.endpoint()
            .map(|ep| ep.full_base_path())
            .unwrap_or_else(|| GenerationContext::default_base_path(&self.entity.name))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake() {
        assert_eq!(GenerationContext::snake("BlogPost"), "blog_post");
        assert_eq!(GenerationContext::snake("User"), "user");
        assert_eq!(GenerationContext::snake("HTTPRequest"), "http_request");
    }

    #[test]
    fn test_pascal() {
        assert_eq!(GenerationContext::pascal("blog_post"), "BlogPost");
        assert_eq!(GenerationContext::pascal("user"), "User");
    }

    #[test]
    fn test_camel() {
        assert_eq!(GenerationContext::camel("blog_post"), "blogPost");
        assert_eq!(GenerationContext::camel("User"), "user");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(GenerationContext::pluralize("user"), "users");
        assert_eq!(GenerationContext::pluralize("post"), "posts");
        assert_eq!(GenerationContext::pluralize("category"), "categories");
        assert_eq!(GenerationContext::pluralize("address"), "addresses");
        assert_eq!(GenerationContext::pluralize("bus"), "buses");
        assert_eq!(GenerationContext::pluralize("key"), "keys");
        assert_eq!(GenerationContext::pluralize("status"), "statuses");
    }

    #[test]
    fn test_table_name() {
        assert_eq!(GenerationContext::table_name("User"), "users");
        assert_eq!(GenerationContext::table_name("BlogPost"), "blog_posts");
        assert_eq!(GenerationContext::table_name("Category"), "categories");
    }

    #[test]
    fn test_default_base_path() {
        assert_eq!(GenerationContext::default_base_path("User"), "/api/users");
        assert_eq!(
            GenerationContext::default_base_path("BlogPost"),
            "/api/blog_posts"
        );
    }

    #[test]
    fn test_dto_names() {
        assert_eq!(GenerationContext::create_dto_name("User"), "CreateUserDto");
        assert_eq!(GenerationContext::update_dto_name("User"), "UpdateUserDto");
        assert_eq!(GenerationContext::response_dto_name("User"), "UserResponse");
    }

    #[test]
    fn test_sql_type_postgresql() {
        let db = DatabaseType::PostgreSQL;
        assert_eq!(
            GenerationContext::sql_type(&DataType::String, db),
            "VARCHAR(255)"
        );
        assert_eq!(GenerationContext::sql_type(&DataType::Int32, db), "INTEGER");
        assert_eq!(GenerationContext::sql_type(&DataType::Uuid, db), "UUID");
        assert_eq!(
            GenerationContext::sql_type(&DataType::DateTime, db),
            "TIMESTAMP WITH TIME ZONE"
        );
        assert_eq!(GenerationContext::sql_type(&DataType::Bool, db), "BOOLEAN");
        assert_eq!(GenerationContext::sql_type(&DataType::Json, db), "JSONB");
    }

    #[test]
    fn test_sql_type_mysql() {
        let db = DatabaseType::MySQL;
        assert_eq!(
            GenerationContext::sql_type(&DataType::Bool, db),
            "TINYINT(1)"
        );
        assert_eq!(GenerationContext::sql_type(&DataType::Uuid, db), "CHAR(36)");
        assert_eq!(GenerationContext::sql_type(&DataType::Text, db), "LONGTEXT");
    }

    #[test]
    fn test_sql_type_sqlite() {
        let db = DatabaseType::SQLite;
        assert_eq!(GenerationContext::sql_type(&DataType::String, db), "TEXT");
        assert_eq!(GenerationContext::sql_type(&DataType::DateTime, db), "TEXT");
        assert_eq!(GenerationContext::sql_type(&DataType::Int64, db), "INTEGER");
    }

    #[test]
    fn test_pk_types() {
        assert_eq!(
            GenerationContext::pk_sql_type(IdType::Uuid, DatabaseType::PostgreSQL),
            "UUID"
        );
        assert_eq!(
            GenerationContext::pk_sql_type(IdType::Serial, DatabaseType::PostgreSQL),
            "SERIAL"
        );
        assert_eq!(GenerationContext::pk_rust_type(IdType::Uuid), "Uuid");
        assert_eq!(GenerationContext::pk_rust_type(IdType::Serial), "i32");
    }

    #[test]
    fn test_example_database_url() {
        let project = ProjectGraph::new("my_app");
        let ctx = GenerationContext::from_project_default(&project);
        let url = ctx.example_database_url();
        assert!(url.contains("my_app"));
        assert!(url.starts_with("postgres://"));
    }

    #[test]
    fn test_context_from_empty_project() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        assert_eq!(ctx.entity_count(), 0);
        assert!(ctx.entities().is_empty());
        assert!(ctx.relationships().is_empty());
        assert!(ctx.endpoints().is_empty());
        assert_eq!(ctx.package_name(), "my_app");
    }

    #[test]
    fn test_context_with_entities() {
        use imortal_ir::Entity;

        let mut project = ProjectGraph::new("blog");
        let user = Entity::new("User");
        let post = Entity::new("Post");
        project.add_entity(user);
        project.add_entity(post);

        let ctx = GenerationContext::from_project_default(&project);
        assert_eq!(ctx.entity_count(), 2);
    }

    #[test]
    fn test_entity_info() {
        use imortal_ir::Entity;

        let mut project = ProjectGraph::new("shop");
        let entity = Entity::new("Product");
        let eid = entity.id;
        project.add_entity(entity);

        let ctx = GenerationContext::from_project_default(&project);
        let e = ctx.entity_by_id(eid).unwrap();
        let info = EntityInfo::new(e, &ctx);

        assert_eq!(info.module_name(), "product");
        assert_eq!(info.pascal_name(), "Product");
        assert_eq!(info.snake_name(), "product");
        assert_eq!(info.plural_name(), "products");
        assert_eq!(info.base_path(), "/api/products");
    }

    #[test]
    fn test_migration_filename() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let name = ctx.migration_filename(1, "users");
        assert!(name.ends_with("_create_users.sql"));
        assert!(name.contains("000001"));
    }
}
