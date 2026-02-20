//! Error types for Immortal Engine
//!
//! This module provides unified error handling across the entire engine,
//! including validation errors, IO errors, serialization errors, and more.

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for Immortal Engine
#[derive(Debug, Error)]
pub enum EngineError {
    // ========================================================================
    // Validation Errors
    // ========================================================================
    /// General validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Entity validation failed
    #[error("Entity validation failed for '{entity}': {message}")]
    EntityValidation { entity: String, message: String },

    /// Field validation failed
    #[error("Field validation failed for '{entity}.{field}': {message}")]
    FieldValidation {
        entity: String,
        field: String,
        message: String,
    },

    /// Relationship validation failed
    #[error("Relationship validation failed: {0}")]
    RelationshipValidation(String),

    /// Endpoint validation failed
    #[error("Endpoint validation failed for '{endpoint}': {message}")]
    EndpointValidation { endpoint: String, message: String },

    // ========================================================================
    // Not Found Errors
    // ========================================================================
    /// Entity not found
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// Field not found
    #[error("Field '{field}' not found in entity '{entity}'")]
    FieldNotFound { entity: String, field: String },

    /// Relationship not found
    #[error("Relationship not found: {0}")]
    RelationshipNotFound(String),

    /// Endpoint not found
    #[error("Endpoint not found: {0}")]
    EndpointNotFound(String),

    /// Project not found
    #[error("Project not found at path: {0}")]
    ProjectNotFound(PathBuf),

    // ========================================================================
    // Duplicate Errors
    // ========================================================================
    /// Duplicate entity name
    #[error("Duplicate entity name: '{0}' already exists")]
    DuplicateEntity(String),

    /// Duplicate field name
    #[error("Duplicate field name: '{field}' already exists in entity '{entity}'")]
    DuplicateField { entity: String, field: String },

    /// Duplicate relationship
    #[error("Duplicate relationship between '{from}' and '{to}'")]
    DuplicateRelationship { from: String, to: String },

    // ========================================================================
    // Code Generation Errors
    // ========================================================================
    /// Code generation failed
    #[error("Code generation failed: {0}")]
    CodeGeneration(String),

    /// Template rendering failed
    #[error("Template rendering failed for '{template}': {message}")]
    TemplateRender { template: String, message: String },

    /// Invalid output path
    #[error("Invalid output path: {0}")]
    InvalidOutputPath(PathBuf),

    /// Output directory already exists
    #[error("Output directory already exists: {0}")]
    OutputExists(PathBuf),

    // ========================================================================
    // IO Errors
    // ========================================================================
    /// File IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// File read error
    #[error("Failed to read file '{path}': {message}")]
    FileRead { path: PathBuf, message: String },

    /// File write error
    #[error("Failed to write file '{path}': {message}")]
    FileWrite { path: PathBuf, message: String },

    /// Directory creation failed
    #[error("Failed to create directory '{path}': {message}")]
    DirectoryCreate { path: PathBuf, message: String },

    // ========================================================================
    // Serialization Errors
    // ========================================================================
    /// JSON serialization error
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    /// Invalid project file format
    #[error("Invalid project file format: {0}")]
    InvalidProjectFormat(String),

    /// Schema version mismatch
    #[error("Schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: u32, found: u32 },

    // ========================================================================
    // Configuration Errors
    // ========================================================================
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    // ========================================================================
    // UI Errors
    // ========================================================================
    /// UI state error
    #[error("UI state error: {0}")]
    UiState(String),

    /// Canvas operation error
    #[error("Canvas operation failed: {0}")]
    CanvasOperation(String),

    // ========================================================================
    // Generic Errors
    // ========================================================================
    /// Internal error (should not happen)
    #[error("Internal error: {0}")]
    Internal(String),

    /// Operation cancelled by user
    #[error("Operation cancelled")]
    Cancelled,

    /// Feature not implemented
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    /// Generic error with context
    #[error("{context}: {message}")]
    WithContext { context: String, message: String },
}

impl EngineError {
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        EngineError::Validation(msg.into())
    }

    /// Create an entity validation error
    pub fn entity_validation(entity: impl Into<String>, msg: impl Into<String>) -> Self {
        EngineError::EntityValidation {
            entity: entity.into(),
            message: msg.into(),
        }
    }

    /// Create a field validation error
    pub fn field_validation(
        entity: impl Into<String>,
        field: impl Into<String>,
        msg: impl Into<String>,
    ) -> Self {
        EngineError::FieldValidation {
            entity: entity.into(),
            field: field.into(),
            message: msg.into(),
        }
    }

    /// Create a code generation error
    pub fn codegen(msg: impl Into<String>) -> Self {
        EngineError::CodeGeneration(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        EngineError::Internal(msg.into())
    }

    /// Create an error with context
    pub fn with_context(context: impl Into<String>, msg: impl Into<String>) -> Self {
        EngineError::WithContext {
            context: context.into(),
            message: msg.into(),
        }
    }

    /// Check if this error is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(
            self,
            EngineError::Validation(_)
                | EngineError::EntityValidation { .. }
                | EngineError::FieldValidation { .. }
                | EngineError::RelationshipValidation(_)
                | EngineError::EndpointValidation { .. }
        )
    }

    /// Check if this error is a not-found error
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            EngineError::EntityNotFound(_)
                | EngineError::FieldNotFound { .. }
                | EngineError::RelationshipNotFound(_)
                | EngineError::EndpointNotFound(_)
                | EngineError::ProjectNotFound(_)
        )
    }

    /// Check if this error is an IO error
    pub fn is_io(&self) -> bool {
        matches!(
            self,
            EngineError::Io(_)
                | EngineError::FileRead { .. }
                | EngineError::FileWrite { .. }
                | EngineError::DirectoryCreate { .. }
        )
    }
}

/// Result type alias using EngineError
pub type EngineResult<T> = Result<T, EngineError>;

/// Extension trait for adding context to errors
pub trait ResultExt<T> {
    /// Add context to an error
    fn with_context<C: Into<String>>(self, context: C) -> EngineResult<T>;
}

impl<T, E: Into<EngineError>> ResultExt<T> for Result<T, E> {
    fn with_context<C: Into<String>>(self, context: C) -> EngineResult<T> {
        self.map_err(|e| {
            let err: EngineError = e.into();
            EngineError::WithContext {
                context: context.into(),
                message: err.to_string(),
            }
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let err = EngineError::validation("Name is required");
        assert!(err.is_validation());
        assert!(!err.is_not_found());
        assert_eq!(err.to_string(), "Validation error: Name is required");
    }

    #[test]
    fn test_entity_validation_error() {
        let err = EngineError::entity_validation("User", "Name must be unique");
        assert!(err.is_validation());
        assert_eq!(
            err.to_string(),
            "Entity validation failed for 'User': Name must be unique"
        );
    }

    #[test]
    fn test_field_validation_error() {
        let err = EngineError::field_validation("User", "email", "Invalid email format");
        assert!(err.is_validation());
        assert_eq!(
            err.to_string(),
            "Field validation failed for 'User.email': Invalid email format"
        );
    }

    #[test]
    fn test_not_found_errors() {
        let err = EngineError::EntityNotFound("User".to_string());
        assert!(err.is_not_found());
        assert!(!err.is_validation());
        assert_eq!(err.to_string(), "Entity not found: User");
    }

    #[test]
    fn test_codegen_error() {
        let err = EngineError::codegen("Failed to generate model");
        assert!(!err.is_validation());
        assert_eq!(
            err.to_string(),
            "Code generation failed: Failed to generate model"
        );
    }

    #[test]
    fn test_error_with_context() {
        let err = EngineError::with_context("Saving project", "Permission denied");
        assert_eq!(err.to_string(), "Saving project: Permission denied");
    }

    #[test]
    fn test_duplicate_errors() {
        let err = EngineError::DuplicateEntity("User".to_string());
        assert_eq!(
            err.to_string(),
            "Duplicate entity name: 'User' already exists"
        );

        let err = EngineError::DuplicateField {
            entity: "User".to_string(),
            field: "email".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Duplicate field name: 'email' already exists in entity 'User'"
        );
    }

    #[test]
    fn test_io_error_classification() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: EngineError = io_err.into();
        assert!(err.is_io());
    }
}
