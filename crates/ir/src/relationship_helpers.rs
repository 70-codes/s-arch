//! Relationship Helper Functions
//!
//! This module provides helper functions for managing relationships between entities,
//! including automatic foreign key field generation.
//!
//! ## FK Auto-Generation
//!
//! When a relationship is created between entities, the appropriate foreign key
//! field should be added to the correct entity:
//!
//! - **One-to-One**: FK can be on either side (typically the "from" side)
//! - **One-to-Many**: FK is on the "many" side (the "to" entity)
//! - **Many-to-One**: FK is on the "from" entity
//! - **Many-to-Many**: A junction table is created with FKs to both entities

use crate::entity::Entity;
use crate::field::{Field, ForeignKeyRef};
use crate::relationship::Relationship;
use imortal_core::{DataType, EngineError, EngineResult, ReferentialAction, RelationType};
use uuid::Uuid;

// ============================================================================
// FK Field Name Generation
// ============================================================================

/// Generate a foreign key field name from an entity name
///
/// Converts the entity name to snake_case and appends "_id"
///
/// # Examples
///
/// - "User" -> "user_id"
/// - "BlogPost" -> "blog_post_id"
/// - "user" -> "user_id"
pub fn generate_fk_field_name(entity_name: &str) -> String {
    let snake = to_snake_case(entity_name);
    format!("{}_id", snake)
}

/// Generate an inverse relationship name
///
/// Creates a pluralized, snake_case version for the inverse side
///
/// # Examples
///
/// - "User" -> "users"
/// - "BlogPost" -> "blog_posts"
pub fn generate_inverse_name(entity_name: &str) -> String {
    let snake = to_snake_case(entity_name);
    pluralize(&snake)
}

/// Generate a relationship name from two entity names
///
/// # Examples
///
/// - ("User", "Post") -> "UserPosts"
/// - ("Author", "Book") -> "AuthorBooks"
pub fn generate_relationship_name(from_entity: &str, to_entity: &str) -> String {
    let to_plural = capitalize(&pluralize(&to_snake_case(to_entity)));
    format!("{}{}", from_entity, to_plural)
}

/// Generate a junction table name for many-to-many relationships
///
/// # Examples
///
/// - ("User", "Role") -> "user_roles"
/// - ("Student", "Course") -> "student_courses"
pub fn generate_junction_table_name(entity1: &str, entity2: &str) -> String {
    let snake1 = to_snake_case(entity1);
    let snake2 = to_snake_case(entity2);

    // Sort alphabetically for consistency
    let (first, second) = if snake1 < snake2 {
        (snake1, snake2)
    } else {
        (snake2, snake1)
    };

    format!("{}_{}", first, pluralize(&second))
}

// ============================================================================
// FK Field Creation
// ============================================================================

/// Create a foreign key field for a relationship
///
/// # Arguments
///
/// * `target_entity` - The entity being referenced
/// * `field_name` - Optional custom field name (defaults to generated name)
/// * `required` - Whether the FK is required (NOT NULL)
/// * `on_delete` - Referential action on delete
/// * `on_update` - Referential action on update
pub fn create_fk_field(
    target_entity: &Entity,
    field_name: Option<&str>,
    required: bool,
    on_delete: ReferentialAction,
    on_update: ReferentialAction,
) -> Field {
    let name = field_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| generate_fk_field_name(&target_entity.name));

    let mut field = Field::new(&name, DataType::Uuid);
    field.is_foreign_key = true;
    field.indexed = true;
    field.required = required;
    field.foreign_key_ref = Some(ForeignKeyRef {
        entity_id: target_entity.id,
        entity_name: target_entity.name.clone(),
        field_name: "id".to_string(),
        on_delete,
        on_update,
    });
    field.description = Some(format!("Foreign key to {}", target_entity.name));
    field.ui_hints.label = Some(format!("{} ID", target_entity.name));

    field
}

/// Create a foreign key field with default settings
pub fn create_fk_field_default(target_entity: &Entity) -> Field {
    create_fk_field(
        target_entity,
        None,
        true,
        ReferentialAction::Restrict,
        ReferentialAction::Cascade,
    )
}

// ============================================================================
// Relationship FK Auto-Generation
// ============================================================================

/// Determines which entity should have the FK field for a given relationship type
///
/// Returns the entity ID that should receive the FK field, or None for M:N
/// (since M:N uses a junction table instead of a direct FK)
pub fn determine_fk_entity(relationship: &Relationship) -> Option<Uuid> {
    match relationship.relation_type {
        // One-to-One: FK on the "from" side (configurable, but this is default)
        RelationType::OneToOne => Some(relationship.from_entity_id),

        // One-to-Many: FK on the "many" side (to_entity)
        RelationType::OneToMany => Some(relationship.to_entity_id),

        // Many-to-One: FK on the "from" side
        RelationType::ManyToOne => Some(relationship.from_entity_id),

        // Many-to-Many: Uses junction table, no direct FK
        RelationType::ManyToMany { .. } => None,
    }
}

/// Information about the FK field to be created
#[derive(Debug, Clone)]
pub struct FkFieldInfo {
    /// Entity that should have the FK field added
    pub entity_id: Uuid,
    /// Entity that is being referenced
    pub referenced_entity_id: Uuid,
    /// Suggested field name
    pub field_name: String,
    /// Whether the FK should be required
    pub required: bool,
    /// On delete action
    pub on_delete: ReferentialAction,
    /// On update action
    pub on_update: ReferentialAction,
}

/// Calculate FK field information for a relationship
///
/// Returns None for many-to-many relationships (they use junction tables)
pub fn calculate_fk_info(
    relationship: &Relationship,
    from_entity_name: &str,
    to_entity_name: &str,
) -> Option<FkFieldInfo> {
    let fk_entity_id = determine_fk_entity(relationship)?;

    // Determine which entity is referenced (the other one)
    let (referenced_entity_id, referenced_name) = if fk_entity_id == relationship.from_entity_id {
        (relationship.to_entity_id, to_entity_name)
    } else {
        (relationship.from_entity_id, from_entity_name)
    };

    // Use from_field if set, otherwise generate
    let field_name = if !relationship.from_field.is_empty() {
        relationship.from_field.clone()
    } else {
        generate_fk_field_name(referenced_name)
    };

    Some(FkFieldInfo {
        entity_id: fk_entity_id,
        referenced_entity_id,
        field_name,
        required: relationship.required,
        on_delete: relationship.on_delete,
        on_update: relationship.on_update,
    })
}

/// Add FK field to an entity based on relationship
///
/// # Arguments
///
/// * `entity` - The entity to add the FK field to (mutable)
/// * `target_entity` - The entity being referenced
/// * `relationship` - The relationship definition
///
/// # Returns
///
/// The UUID of the newly created field, or an error if the field already exists
pub fn add_fk_field_for_relationship(
    entity: &mut Entity,
    target_entity: &Entity,
    relationship: &Relationship,
) -> EngineResult<Uuid> {
    // Check if FK field already exists
    let fk_name = if !relationship.from_field.is_empty() {
        relationship.from_field.clone()
    } else {
        generate_fk_field_name(&target_entity.name)
    };

    if entity.has_field(&fk_name) {
        return Err(EngineError::DuplicateField {
            entity: entity.name.clone(),
            field: fk_name,
        });
    }

    // Create the FK field
    let field = create_fk_field(
        target_entity,
        Some(&fk_name),
        relationship.required,
        relationship.on_delete,
        relationship.on_update,
    );

    let field_id = field.id;
    entity.add_field(field);

    Ok(field_id)
}

/// Check if an entity already has an FK field referencing another entity
pub fn has_fk_to_entity(entity: &Entity, target_entity_id: Uuid) -> bool {
    entity.fields.iter().any(|f| {
        f.is_foreign_key
            && f.foreign_key_ref
                .as_ref()
                .map(|fk| fk.entity_id == target_entity_id)
                .unwrap_or(false)
    })
}

/// Get existing FK field that references a specific entity
pub fn get_fk_field_to_entity(entity: &Entity, target_entity_id: Uuid) -> Option<&Field> {
    entity.fields.iter().find(|f| {
        f.is_foreign_key
            && f.foreign_key_ref
                .as_ref()
                .map(|fk| fk.entity_id == target_entity_id)
                .unwrap_or(false)
    })
}

/// Remove FK field that references a specific entity
///
/// Returns the removed field, if any
pub fn remove_fk_field_to_entity(entity: &mut Entity, target_entity_id: Uuid) -> Option<Field> {
    let field_id = entity
        .fields
        .iter()
        .find(|f| {
            f.is_foreign_key
                && f.foreign_key_ref
                    .as_ref()
                    .map(|fk| fk.entity_id == target_entity_id)
                    .unwrap_or(false)
        })
        .map(|f| f.id)?;

    entity.remove_field(field_id)
}

// ============================================================================
// Relationship Validation
// ============================================================================

/// Validate that a relationship can be created between two entities
pub fn validate_relationship_creation(
    from_entity: &Entity,
    to_entity: &Entity,
    relation_type: &RelationType,
) -> EngineResult<()> {
    // Can't create relationship to self (except for some special cases)
    if from_entity.id == to_entity.id {
        return Err(EngineError::RelationshipValidation(
            "Cannot create a self-referencing relationship".to_string(),
        ));
    }

    // Check for existing FK (for types that use direct FKs)
    match relation_type {
        RelationType::OneToOne | RelationType::ManyToOne => {
            if has_fk_to_entity(from_entity, to_entity.id) {
                return Err(EngineError::RelationshipValidation(format!(
                    "Entity '{}' already has a foreign key to '{}'",
                    from_entity.name, to_entity.name
                )));
            }
        }
        RelationType::OneToMany => {
            if has_fk_to_entity(to_entity, from_entity.id) {
                return Err(EngineError::RelationshipValidation(format!(
                    "Entity '{}' already has a foreign key to '{}'",
                    to_entity.name, from_entity.name
                )));
            }
        }
        RelationType::ManyToMany { .. } => {
            // M:N can have multiple relationships between same entities
        }
    }

    Ok(())
}

// ============================================================================
// String Utilities
// ============================================================================

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_is_upper = false;
    let mut prev_is_underscore = true;

    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == '-' || c == ' ' {
            if !prev_is_underscore {
                result.push('_');
            }
            prev_is_underscore = true;
            prev_is_upper = false;
        } else if c.is_uppercase() {
            if i > 0 && !prev_is_upper && !prev_is_underscore {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_is_upper = true;
            prev_is_underscore = false;
        } else {
            result.push(c.to_ascii_lowercase());
            prev_is_upper = false;
            prev_is_underscore = false;
        }
    }

    result
}

/// Capitalize the first letter of a string
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Simple pluralization (English)
fn pluralize(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    // Handle common irregular plurals
    match s {
        "person" => return "people".to_string(),
        "child" => return "children".to_string(),
        "man" => return "men".to_string(),
        "woman" => return "women".to_string(),
        "foot" => return "feet".to_string(),
        "tooth" => return "teeth".to_string(),
        "goose" => return "geese".to_string(),
        "mouse" => return "mice".to_string(),
        _ => {}
    }

    // Handle words ending in 's', 'x', 'z', 'ch', 'sh'
    if s.ends_with('s')
        || s.ends_with('x')
        || s.ends_with('z')
        || s.ends_with("ch")
        || s.ends_with("sh")
    {
        return format!("{}es", s);
    }

    // Handle words ending in consonant + 'y'
    if s.ends_with('y') {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() >= 2 {
            let second_last = chars[chars.len() - 2];
            if !"aeiou".contains(second_last) {
                return format!("{}ies", &s[..s.len() - 1]);
            }
        }
    }

    // Handle words ending in 'f' or 'fe'
    if s.ends_with("fe") {
        return format!("{}ves", &s[..s.len() - 2]);
    }
    if s.ends_with('f') {
        return format!("{}ves", &s[..s.len() - 1]);
    }

    // Default: just add 's'
    format!("{}s", s)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_fk_field_name() {
        assert_eq!(generate_fk_field_name("User"), "user_id");
        assert_eq!(generate_fk_field_name("BlogPost"), "blog_post_id");
        assert_eq!(generate_fk_field_name("user"), "user_id");
        assert_eq!(generate_fk_field_name("API_Key"), "api_key_id");
    }

    #[test]
    fn test_generate_inverse_name() {
        assert_eq!(generate_inverse_name("User"), "users");
        assert_eq!(generate_inverse_name("BlogPost"), "blog_posts");
        assert_eq!(generate_inverse_name("Category"), "categories");
    }

    #[test]
    fn test_generate_relationship_name() {
        assert_eq!(generate_relationship_name("User", "Post"), "UserPosts");
        assert_eq!(generate_relationship_name("Author", "Book"), "AuthorBooks");
    }

    #[test]
    fn test_generate_junction_table_name() {
        assert_eq!(generate_junction_table_name("User", "Role"), "role_users");
        assert_eq!(
            generate_junction_table_name("Student", "Course"),
            "course_students"
        );
        // Should be consistent regardless of order
        assert_eq!(generate_junction_table_name("Role", "User"), "role_users");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("BlogPost"), "blog_post");
        // Note: consecutive uppercase letters are treated as one word
        assert_eq!(to_snake_case("APIKey"), "apikey");
        assert_eq!(to_snake_case("Api_Key"), "api_key");
        assert_eq!(to_snake_case("userID"), "user_id");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize("user"), "users");
        assert_eq!(pluralize("post"), "posts");
        assert_eq!(pluralize("category"), "categories");
        assert_eq!(pluralize("box"), "boxes");
        assert_eq!(pluralize("bus"), "buses");
        assert_eq!(pluralize("leaf"), "leaves");
        assert_eq!(pluralize("person"), "people");
        assert_eq!(pluralize("child"), "children");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("user"), "User");
        assert_eq!(capitalize("post"), "Post");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_determine_fk_entity() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // One-to-One: FK on from
        let one_to_one = Relationship::one_to_one(from_id, to_id);
        assert_eq!(determine_fk_entity(&one_to_one), Some(from_id));

        // One-to-Many: FK on to
        let one_to_many = Relationship::one_to_many(from_id, to_id);
        assert_eq!(determine_fk_entity(&one_to_many), Some(to_id));

        // Many-to-One: FK on from
        let many_to_one = Relationship::many_to_one(from_id, to_id);
        assert_eq!(determine_fk_entity(&many_to_one), Some(from_id));

        // Many-to-Many: No direct FK
        let many_to_many = Relationship::many_to_many(from_id, to_id, "users_roles");
        assert_eq!(determine_fk_entity(&many_to_many), None);
    }

    #[test]
    fn test_create_fk_field_default() {
        let target = Entity::new("User");
        let fk_field = create_fk_field_default(&target);

        assert_eq!(fk_field.name, "user_id");
        assert!(fk_field.is_foreign_key);
        assert!(fk_field.indexed);
        assert!(fk_field.required);
        assert!(fk_field.foreign_key_ref.is_some());

        let fk_ref = fk_field.foreign_key_ref.unwrap();
        assert_eq!(fk_ref.entity_id, target.id);
        assert_eq!(fk_ref.entity_name, "User");
        assert_eq!(fk_ref.field_name, "id");
    }

    #[test]
    fn test_has_fk_to_entity() {
        let target = Entity::new("User");
        let mut source = Entity::new("Post");

        assert!(!has_fk_to_entity(&source, target.id));

        let fk_field = create_fk_field_default(&target);
        source.add_field(fk_field);

        assert!(has_fk_to_entity(&source, target.id));
    }

    #[test]
    fn test_calculate_fk_info() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // One-to-Many: FK info should point to "to" entity needing the FK
        let rel = Relationship::one_to_many(from_id, to_id);
        let info = calculate_fk_info(&rel, "User", "Post").unwrap();

        assert_eq!(info.entity_id, to_id); // Post gets the FK
        assert_eq!(info.referenced_entity_id, from_id); // References User
        assert_eq!(info.field_name, "user_id");
    }
}
