//! # Immortal IR (Intermediate Representation)
//!
//! This crate provides the intermediate representation for Immortal Engine projects.
//! It contains all the data structures needed to represent a project's schema,
//! including entities, fields, relationships, and endpoints.
//!
//! ## Core Concepts
//!
//! - **Entity**: A data model that maps to a database table (e.g., User, Post)
//! - **Field**: A property of an entity that maps to a column (e.g., email, title)
//! - **Relationship**: A connection between two entities (one-to-one, one-to-many, etc.)
//! - **Endpoint**: An API endpoint configuration for CRUD operations
//! - **ProjectGraph**: The root container that holds all project data
//!

// Module declarations
pub mod endpoint;
pub mod entity;
pub mod field;
pub mod project;
pub mod relationship;
pub mod relationship_helpers;
pub mod serialization;
pub mod validation;

// Re-export commonly used types at crate root
pub use endpoint::{CrudOperation, EndpointGroup, EndpointSecurity, OperationType, RateLimit};
pub use entity::{Entity, EntityConfig};
pub use field::{DefaultValue, Field, ForeignKeyRef, UiHints, WidgetType};
pub use project::{
    AuthConfig, AuthStrategy, CanvasState, DatabaseConfig, ProjectConfig, ProjectGraph,
    ProjectMeta, ProjectType,
};
pub use relationship::{PortPosition, Relationship};
pub use relationship_helpers::{
    FkFieldInfo, add_fk_field_for_relationship, calculate_fk_info, create_fk_field,
    create_fk_field_default, determine_fk_entity, generate_fk_field_name, generate_inverse_name,
    generate_junction_table_name, generate_relationship_name, has_fk_to_entity,
};
pub use serialization::{load_project, save_project};
pub use validation::{ValidationResult, ValidationRule, Validator};

// Re-export core types that are commonly used with IR
pub use imortal_core::{
    DataType, DatabaseType, EngineError, EngineResult, IdType, Position, Rect, ReferentialAction,
    RelationType, Size, Validation,
};

/// Current schema version for project files
pub const SCHEMA_VERSION: u32 = 1;

/// File extension for Immortal Engine project files
pub const PROJECT_FILE_EXTENSION: &str = "ieng";

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// Prelude Module
// ============================================================================

/// Convenient re-exports for common usage
pub mod prelude {
    pub use crate::{
        AuthStrategy,
        CrudOperation,
        // Re-exported from core
        DataType,
        DatabaseType,
        EndpointGroup,
        EngineError,
        EngineResult,
        // Core types
        Entity,
        EntityConfig,
        Field,
        IdType,
        // Operations
        OperationType,
        Position,
        ProjectConfig,
        ProjectGraph,
        ProjectMeta,
        ProjectType,
        RelationType,
        Relationship,
        Size,
        // Relationship helpers
        add_fk_field_for_relationship,
        create_fk_field,
        generate_fk_field_name,
        generate_relationship_name,
        has_fk_to_entity,
    };
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        assert_eq!(SCHEMA_VERSION, 1);
    }

    #[test]
    fn test_file_extension() {
        assert_eq!(PROJECT_FILE_EXTENSION, "ieng");
    }
}
