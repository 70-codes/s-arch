//! Entity definitions for data models
//!
//! This module contains the `Entity` struct and related types for defining
//! data models (tables) in the Immortal Engine IR.

use crate::field::Field;
use chrono::{DateTime, Utc};
use imortal_core::{DataType, EngineError, EngineResult, IdType, Position, Size, Validatable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Entity
// ============================================================================

/// Represents a data entity (maps to a database table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier for this entity
    pub id: Uuid,

    /// Entity name (PascalCase, e.g., "User", "BlogPost")
    pub name: String,

    /// Database table name (snake_case, e.g., "users", "blog_posts")
    pub table_name: String,

    /// Human-readable description
    pub description: Option<String>,

    /// Fields (columns) in this entity
    pub fields: Vec<Field>,

    /// Position on the canvas
    pub position: Position,

    /// Size on the canvas
    pub size: Size,

    /// Entity configuration options
    pub config: EntityConfig,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Whether the entity is collapsed in the UI
    pub collapsed: bool,

    /// Whether the entity is selected in the UI
    pub selected: bool,

    /// Z-index for layering on canvas
    pub z_index: i32,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub modified_at: DateTime<Utc>,
}

impl Entity {
    /// Create a new entity with the given name
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let table_name = to_snake_case_plural(&name);

        let mut entity = Self {
            id: Uuid::new_v4(),
            name,
            table_name,
            description: None,
            fields: Vec::new(),
            position: Position::zero(),
            size: Size::default_entity(),
            config: EntityConfig::default(),
            tags: Vec::new(),
            collapsed: false,
            selected: false,
            z_index: 0,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };
        entity.fields.push(Field::primary_key());

        entity
    }

    /// Create a new entity with timestamps enabled
    pub fn with_timestamps(name: impl Into<String>) -> Self {
        let mut entity = Self::new(name);
        entity.config.timestamps = true;
        entity.fields.push(Field::created_at());
        entity.fields.push(Field::updated_at());
        entity
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Set the table name
    pub fn with_table_name(mut self, table_name: impl Into<String>) -> Self {
        self.table_name = table_name.into();
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the position on the canvas
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Set the position using x, y coordinates
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = Position::new(x, y);
        self
    }

    /// Set the size
    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set the configuration
    pub fn with_config(mut self, config: EntityConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Enable soft delete
    pub fn soft_delete(mut self) -> Self {
        self.config.soft_delete = true;
        // Add deleted_at field
        let deleted_at = Field::new(
            "deleted_at",
            DataType::Optional(Box::new(DataType::DateTime)),
        )
        .with_label("Deleted At")
        .readonly()
        .hidden();
        self.fields.push(deleted_at);
        self
    }

    /// Enable audit logging
    pub fn auditable(mut self) -> Self {
        self.config.auditable = true;
        self
    }

    // ========================================================================
    // Field management
    // ========================================================================

    /// Add a field to the entity
    pub fn add_field(&mut self, field: Field) {
        // Update display order to be at the end
        let mut field = field;
        if field.display_order == 0 {
            field.display_order = self.fields.len() as i32;
        }
        self.fields.push(field);
        self.touch();
    }

    /// Add a field using builder pattern
    pub fn with_field(mut self, field: Field) -> Self {
        self.add_field(field);
        self
    }

    /// Remove a field by ID
    pub fn remove_field(&mut self, field_id: Uuid) -> Option<Field> {
        if let Some(pos) = self.fields.iter().position(|f| f.id == field_id) {
            self.touch();
            Some(self.fields.remove(pos))
        } else {
            None
        }
    }

    /// Get a field by ID
    pub fn get_field(&self, field_id: Uuid) -> Option<&Field> {
        self.fields.iter().find(|f| f.id == field_id)
    }

    /// Get a mutable field by ID
    pub fn get_field_mut(&mut self, field_id: Uuid) -> Option<&mut Field> {
        self.fields.iter_mut().find(|f| f.id == field_id)
    }

    /// Get a field by name
    pub fn get_field_by_name(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Get a mutable field by name
    pub fn get_field_by_name_mut(&mut self, name: &str) -> Option<&mut Field> {
        self.fields.iter_mut().find(|f| f.name == name)
    }

    /// Move a field to a new position (reorder)
    pub fn move_field(&mut self, field_id: Uuid, new_index: usize) -> bool {
        if let Some(current_index) = self.fields.iter().position(|f| f.id == field_id) {
            if new_index < self.fields.len() {
                let field = self.fields.remove(current_index);
                self.fields.insert(new_index, field);

                for (i, field) in self.fields.iter_mut().enumerate() {
                    field.display_order = i as i32;
                }
                self.touch();
                return true;
            }
        }
        false
    }

    /// Get fields sorted by display order
    pub fn sorted_fields(&self) -> Vec<&Field> {
        let mut fields: Vec<&Field> = self.fields.iter().collect();
        fields.sort_by_key(|f| f.display_order);
        fields
    }

    // ========================================================================
    // Query methods
    // ========================================================================

    /// Get the primary key field
    pub fn primary_key(&self) -> Option<&Field> {
        self.fields.iter().find(|f| f.is_primary_key)
    }

    /// Get all foreign key fields
    pub fn foreign_keys(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.is_foreign_key).collect()
    }

    /// Get all required fields
    pub fn required_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.required).collect()
    }

    /// Get all unique fields
    pub fn unique_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.unique).collect()
    }

    /// Get all indexed fields
    pub fn indexed_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.indexed).collect()
    }

    /// Get fields for create DTO (excludes PK, readonly, fields with defaults)
    pub fn create_dto_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.in_create_dto()).collect()
    }

    /// Get fields for update DTO (excludes PK, readonly)
    pub fn update_dto_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.in_update_dto()).collect()
    }

    /// Get fields for response DTO (excludes secrets)
    pub fn response_dto_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.in_response_dto()).collect()
    }

    /// Get visible fields (not hidden)
    pub fn visible_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| !f.hidden).collect()
    }

    /// Check if entity has a specific field name
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.iter().any(|f| f.name == name)
    }

    /// Get the number of fields
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    // ========================================================================
    // Canvas methods
    // ========================================================================

    /// Get the bounding rectangle
    pub fn bounds(&self) -> imortal_core::Rect {
        imortal_core::Rect::new(self.position, self.size)
    }

    /// Check if a point is inside the entity bounds
    pub fn contains(&self, point: Position) -> bool {
        self.bounds().contains(point)
    }

    /// Get the center position
    pub fn center(&self) -> Position {
        self.bounds().center()
    }

    /// Move the entity by a delta
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position = self.position.offset(dx, dy);
        self.touch();
    }

    /// Set the position
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
        self.touch();
    }

    /// Select this entity
    pub fn select(&mut self) {
        self.selected = true;
    }

    /// Deselect this entity
    pub fn deselect(&mut self) {
        self.selected = false;
    }

    /// Toggle selection
    pub fn toggle_selection(&mut self) {
        self.selected = !self.selected;
    }

    /// Toggle collapsed state
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Bring to front (increase z-index)
    pub fn bring_to_front(&mut self, max_z: i32) {
        self.z_index = max_z + 1;
    }

    /// Send to back (decrease z-index)
    pub fn send_to_back(&mut self, min_z: i32) {
        self.z_index = min_z - 1;
    }

    // ========================================================================
    // Utility methods
    // ========================================================================

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Calculate the height based on number of fields
    pub fn calculate_height(&self) -> f32 {
        let header_height = 48.0;
        let field_height = 28.0;
        let padding = 16.0;
        let visible_fields = if self.collapsed {
            0
        } else {
            self.visible_fields().len()
        };

        header_height + (visible_fields as f32 * field_height) + padding
    }

    /// Update size based on content
    pub fn fit_content(&mut self) {
        self.size.height = self.calculate_height();
    }

    /// Duplicate this entity with a new ID and modified name
    pub fn duplicate(&self) -> Self {
        let mut new_entity = self.clone();
        new_entity.id = Uuid::new_v4();
        new_entity.name = format!("{}_copy", self.name);
        new_entity.table_name = format!("{}_copy", self.table_name);
        new_entity.position = self.position.offset(30.0, 30.0);
        new_entity.selected = false;
        new_entity.created_at = Utc::now();
        new_entity.modified_at = Utc::now();

        // Regenerate field IDs
        for field in &mut new_entity.fields {
            field.id = Uuid::new_v4();
        }

        new_entity
    }

    /// Get the Rust struct name (PascalCase)
    pub fn struct_name(&self) -> &str {
        &self.name
    }

    /// Get the Rust module name (snake_case)
    pub fn module_name(&self) -> String {
        to_snake_case(&self.name)
    }
}

impl Validatable for Entity {
    fn validate(&self) -> EngineResult<()> {
        // Entity name must not be empty
        if self.name.is_empty() {
            return Err(EngineError::entity_validation(
                &self.name,
                "Entity name cannot be empty",
            ));
        }

        // Entity name must be valid identifier
        if !is_valid_identifier(&self.name) {
            return Err(EngineError::entity_validation(
                &self.name,
                format!("Entity name '{}' is not a valid identifier", self.name),
            ));
        }

        // Entity name should be PascalCase
        if !self
            .name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
        {
            return Err(EngineError::entity_validation(
                &self.name,
                "Entity name should start with an uppercase letter",
            ));
        }

        // Table name must not be empty
        if self.table_name.is_empty() {
            return Err(EngineError::entity_validation(
                &self.name,
                "Table name cannot be empty",
            ));
        }

        // Must have at least one field
        if self.fields.is_empty() {
            return Err(EngineError::entity_validation(
                &self.name,
                "Entity must have at least one field",
            ));
        }

        // Must have a primary key
        if self.primary_key().is_none() {
            return Err(EngineError::entity_validation(
                &self.name,
                "Entity must have a primary key",
            ));
        }

        // Validate all fields
        for field in &self.fields {
            field.validate().map_err(|e| {
                EngineError::field_validation(&self.name, &field.name, e.to_string())
            })?;
        }

        // Check for duplicate field names
        let mut field_names = std::collections::HashSet::new();
        for field in &self.fields {
            if !field_names.insert(&field.name) {
                return Err(EngineError::DuplicateField {
                    entity: self.name.clone(),
                    field: field.name.clone(),
                });
            }
        }

        Ok(())
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::new("Entity")
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Entity {}

impl std::hash::Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// ============================================================================
// EntityConfig
// ============================================================================

/// Configuration options for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConfig {
    /// Auto-generate created_at/updated_at fields
    pub timestamps: bool,

    /// Use soft delete (deleted_at) instead of hard delete
    pub soft_delete: bool,

    /// Primary key type
    pub id_type: IdType,

    /// Enable audit logging for this entity
    pub auditable: bool,

    /// Generate API endpoints for this entity
    pub generate_api: bool,

    /// Custom attributes for the model (e.g., SeaORM attributes)
    pub model_attributes: Vec<String>,

    /// Custom table options (e.g., PostgreSQL schema)
    pub table_options: std::collections::HashMap<String, String>,
}

impl EntityConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable timestamps
    pub fn with_timestamps(mut self) -> Self {
        self.timestamps = true;
        self
    }

    /// Enable soft delete
    pub fn with_soft_delete(mut self) -> Self {
        self.soft_delete = true;
        self
    }

    /// Set the ID type
    pub fn with_id_type(mut self, id_type: IdType) -> Self {
        self.id_type = id_type;
        self
    }

    /// Enable audit logging
    pub fn with_audit(mut self) -> Self {
        self.auditable = true;
        self
    }

    /// Disable API generation
    pub fn without_api(mut self) -> Self {
        self.generate_api = false;
        self
    }

    /// Add a model attribute
    pub fn with_attribute(mut self, attr: impl Into<String>) -> Self {
        self.model_attributes.push(attr.into());
        self
    }

    /// Add a table option
    pub fn with_table_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.table_options.insert(key.into(), value.into());
        self
    }
}

impl Default for EntityConfig {
    fn default() -> Self {
        Self {
            timestamps: true,
            soft_delete: false,
            id_type: IdType::Uuid,
            auditable: false,
            generate_api: true,
            model_attributes: Vec::new(),
            table_options: std::collections::HashMap::new(),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a string to snake_case
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

/// Convert a string to snake_case plural (simple English pluralization)
fn to_snake_case_plural(s: &str) -> String {
    let snake = to_snake_case(s);

    // Simple pluralization rules
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

/// Check if a string is a valid Rust identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // First character must be letter or underscore
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_new() {
        let entity = Entity::new("User");
        assert_eq!(entity.name, "User");
        assert_eq!(entity.table_name, "users");
        assert!(!entity.fields.is_empty()); // Has primary key
        assert!(entity.primary_key().is_some());
    }

    #[test]
    fn test_entity_with_timestamps() {
        let entity = Entity::with_timestamps("Post");
        assert!(entity.config.timestamps);
        assert!(entity.has_field("created_at"));
        assert!(entity.has_field("updated_at"));
    }

    #[test]
    fn test_entity_builder() {
        let entity = Entity::new("BlogPost")
            .with_description("A blog post")
            .at(100.0, 200.0)
            .with_tag("content")
            .soft_delete();

        assert_eq!(entity.description, Some("A blog post".to_string()));
        assert_eq!(entity.position.x, 100.0);
        assert_eq!(entity.position.y, 200.0);
        assert!(entity.tags.contains(&"content".to_string()));
        assert!(entity.config.soft_delete);
        assert!(entity.has_field("deleted_at"));
    }

    #[test]
    fn test_entity_add_field() {
        let mut entity = Entity::new("User");
        let initial_count = entity.field_count();

        entity.add_field(Field::new("email", DataType::String).required().unique());

        assert_eq!(entity.field_count(), initial_count + 1);
        assert!(entity.has_field("email"));

        let email_field = entity.get_field_by_name("email").unwrap();
        assert!(email_field.required);
        assert!(email_field.unique);
    }

    #[test]
    fn test_entity_remove_field() {
        let mut entity = Entity::new("User");
        let field = Field::new("temp", DataType::String);
        let field_id = field.id;
        entity.add_field(field);

        assert!(entity.has_field("temp"));

        let removed = entity.remove_field(field_id);
        assert!(removed.is_some());
        assert!(!entity.has_field("temp"));
    }

    #[test]
    fn test_entity_validation() {
        let entity = Entity::new("User");
        assert!(entity.validate().is_ok());

        // Empty name
        let mut invalid = Entity::new("User");
        invalid.name = String::new();
        assert!(invalid.validate().is_err());

        // No primary key
        let mut no_pk = Entity::new("User");
        no_pk.fields.clear();
        no_pk.fields.push(Field::new("name", DataType::String));
        assert!(no_pk.validate().is_err());
    }

    #[test]
    fn test_entity_duplicate() {
        let original = Entity::new("User").at(100.0, 100.0);
        let copy = original.duplicate();

        assert_ne!(original.id, copy.id);
        assert_eq!(copy.name, "User_copy");
        assert_ne!(original.position, copy.position);
    }

    #[test]
    fn test_entity_dto_fields() {
        let entity = Entity::with_timestamps("User")
            .with_field(Field::new("email", DataType::String).required())
            .with_field(Field::new("password", DataType::String).required().secret());

        // Create DTO should not include id, created_at, updated_at
        let create_fields = entity.create_dto_fields();
        assert!(create_fields.iter().any(|f| f.name == "email"));
        assert!(create_fields.iter().any(|f| f.name == "password"));
        assert!(!create_fields.iter().any(|f| f.name == "id"));
        assert!(!create_fields.iter().any(|f| f.name == "created_at"));

        // Response DTO should not include password (secret)
        let response_fields = entity.response_dto_fields();
        assert!(response_fields.iter().any(|f| f.name == "email"));
        assert!(!response_fields.iter().any(|f| f.name == "password"));
    }

    #[test]
    fn test_to_snake_case_plural() {
        assert_eq!(to_snake_case_plural("User"), "users");
        assert_eq!(to_snake_case_plural("BlogPost"), "blog_posts");
        assert_eq!(to_snake_case_plural("Category"), "categories");
        assert_eq!(to_snake_case_plural("Box"), "boxes");
        assert_eq!(to_snake_case_plural("Key"), "keys");
    }

    #[test]
    fn test_entity_bounds() {
        let entity = Entity::new("User").at(50.0, 100.0);
        let bounds = entity.bounds();

        assert_eq!(bounds.position.x, 50.0);
        assert_eq!(bounds.position.y, 100.0);
        assert!(entity.contains(Position::new(60.0, 110.0)));
        assert!(!entity.contains(Position::new(0.0, 0.0)));
    }

    #[test]
    fn test_entity_translate() {
        let mut entity = Entity::new("User").at(100.0, 100.0);
        entity.translate(50.0, -25.0);

        assert_eq!(entity.position.x, 150.0);
        assert_eq!(entity.position.y, 75.0);
    }

    #[test]
    fn test_entity_config() {
        let config = EntityConfig::new()
            .with_timestamps()
            .with_soft_delete()
            .with_id_type(IdType::Uuid)
            .with_audit();

        assert!(config.timestamps);
        assert!(config.soft_delete);
        assert!(config.auditable);
        assert_eq!(config.id_type, IdType::Uuid);
    }
}
