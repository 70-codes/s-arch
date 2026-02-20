//! Core traits for Immortal Engine
//!
//! This module defines the fundamental traits that components throughout
//! the engine implement to provide consistent behavior for validation,
//! code generation, and persistence.

use crate::error::EngineResult;
use serde::{Serialize, de::DeserializeOwned};

// ============================================================================
// Validatable Trait
// ============================================================================

/// Trait for types that can be validated
///
/// Types implementing this trait can check their internal consistency
/// and return validation errors if the state is invalid.
///
/// # Example
///
/// ```rust,ignore
/// use imortal_core::{Validatable, EngineResult, EngineError};
///
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// impl Validatable for User {
///     fn validate(&self) -> EngineResult<()> {
///         if self.name.is_empty() {
///             return Err(EngineError::validation("Name cannot be empty"));
///         }
///         if !self.email.contains('@') {
///             return Err(EngineError::validation("Invalid email format"));
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait Validatable {
    /// Validate the current state of the object
    ///
    /// Returns `Ok(())` if valid, or an `EngineError` describing the problem.
    fn validate(&self) -> EngineResult<()>;

    /// Check if the object is valid without returning error details
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Get all validation errors (for types that can have multiple errors)
    fn validation_errors(&self) -> Vec<String> {
        match self.validate() {
            Ok(()) => vec![],
            Err(e) => vec![e.to_string()],
        }
    }
}

// ============================================================================
// CodeGenerable Trait
// ============================================================================

/// Context passed to code generation methods
///
/// Contains configuration and state needed during code generation.
#[derive(Debug, Clone, Default)]
pub struct CodeGenContext {
    /// Indentation level (number of spaces or tabs)
    pub indent_level: usize,
    /// Use spaces (true) or tabs (false) for indentation
    pub use_spaces: bool,
    /// Number of spaces per indent level (if use_spaces is true)
    pub spaces_per_indent: usize,
    /// Target database type for SQL generation
    pub database: Option<crate::types::DatabaseType>,
    /// Whether to include comments in generated code
    pub include_comments: bool,
    /// Whether to include derive macros
    pub include_derives: bool,
    /// Custom options for specific generators
    pub options: std::collections::HashMap<String, String>,
}

impl CodeGenContext {
    /// Create a new context with default settings
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            use_spaces: true,
            spaces_per_indent: 4,
            database: None,
            include_comments: true,
            include_derives: true,
            options: std::collections::HashMap::new(),
        }
    }

    /// Create context for Rust code generation
    pub fn rust() -> Self {
        Self {
            include_derives: true,
            include_comments: true,
            ..Self::new()
        }
    }

    /// Create context for SQL code generation
    pub fn sql(database: crate::types::DatabaseType) -> Self {
        Self {
            database: Some(database),
            include_comments: true,
            ..Self::new()
        }
    }

    /// Get the current indentation string
    pub fn indent(&self) -> String {
        if self.use_spaces {
            " ".repeat(self.indent_level * self.spaces_per_indent)
        } else {
            "\t".repeat(self.indent_level)
        }
    }

    /// Create a new context with increased indentation
    pub fn indented(&self) -> Self {
        Self {
            indent_level: self.indent_level + 1,
            ..self.clone()
        }
    }

    /// Set a custom option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Get a custom option
    pub fn get_option(&self, key: &str) -> Option<&String> {
        self.options.get(key)
    }
}

/// Trait for types that can generate code
///
/// Types implementing this trait can produce source code (Rust, SQL, etc.)
/// representations of themselves.
///
/// # Example
///
/// ```rust,ignore
/// use imortal_core::{CodeGenerable, CodeGenContext, EngineResult};
///
/// struct Field {
///     name: String,
///     field_type: String,
/// }
///
/// impl CodeGenerable for Field {
///     fn generate(&self, ctx: &CodeGenContext) -> EngineResult<String> {
///         Ok(format!("{}pub {}: {},", ctx.indent(), self.name, self.field_type))
///     }
/// }
/// ```
pub trait CodeGenerable {
    /// Generate code for this type
    ///
    /// # Arguments
    ///
    /// * `ctx` - The code generation context with settings and state
    ///
    /// # Returns
    ///
    /// The generated code as a String, or an error if generation fails.
    fn generate(&self, ctx: &CodeGenContext) -> EngineResult<String>;

    /// Generate code with default context
    fn generate_default(&self) -> EngineResult<String> {
        self.generate(&CodeGenContext::new())
    }
}

// ============================================================================
// Persistable Trait
// ============================================================================

/// Trait for types that can be serialized to and deserialized from files
///
/// Types implementing this trait can be saved to and loaded from
/// project files (typically JSON format).
///
/// # Example
///
/// ```rust,ignore
/// use imortal_core::Persistable;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Project {
///     name: String,
///     version: String,
/// }
///
/// impl Persistable for Project {
///     fn file_extension() -> &'static str {
///         "ieng"
///     }
///
///     fn schema_version() -> u32 {
///         1
///     }
/// }
/// ```
pub trait Persistable: Serialize + DeserializeOwned + Sized {
    /// Get the file extension for this type (without the dot)
    fn file_extension() -> &'static str;

    /// Get the schema version for migration purposes
    fn schema_version() -> u32 {
        1
    }

    /// Save to a JSON string
    fn to_json(&self) -> EngineResult<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Load from a JSON string
    fn from_json(json: &str) -> EngineResult<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Save to a file
    fn save_to_file(&self, path: &std::path::Path) -> EngineResult<()> {
        let json = self.to_json()?;
        std::fs::write(path, json).map_err(|e| crate::error::EngineError::FileWrite {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }

    /// Load from a file
    fn load_from_file(path: &std::path::Path) -> EngineResult<Self> {
        let json =
            std::fs::read_to_string(path).map_err(|e| crate::error::EngineError::FileRead {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;
        Self::from_json(&json)
    }
}

// ============================================================================
// Identifiable Trait
// ============================================================================

/// Trait for types that have a unique identifier
///
/// Types implementing this trait have a UUID-based identifier
/// that can be used for lookups and references.
pub trait Identifiable {
    /// Get the unique identifier
    fn id(&self) -> uuid::Uuid;

    /// Check if this matches another identifier
    fn matches_id(&self, id: uuid::Uuid) -> bool {
        self.id() == id
    }
}

// ============================================================================
// Named Trait
// ============================================================================

/// Trait for types that have a name
///
/// Types implementing this trait have a human-readable name
/// that can be displayed in the UI.
pub trait Named {
    /// Get the name
    fn name(&self) -> &str;

    /// Set the name
    fn set_name(&mut self, name: String);

    /// Check if the name matches (case-insensitive)
    fn name_matches(&self, other: &str) -> bool {
        self.name().eq_ignore_ascii_case(other)
    }
}

// ============================================================================
// Positioned Trait
// ============================================================================

/// Trait for types that have a position on the canvas
///
/// Types implementing this trait can be positioned and moved
/// on the visual editor canvas.
pub trait Positioned {
    /// Get the current position
    fn position(&self) -> crate::types::Position;

    /// Set the position
    fn set_position(&mut self, position: crate::types::Position);

    /// Move by a relative offset
    fn translate(&mut self, dx: f32, dy: f32) {
        let pos = self.position();
        self.set_position(crate::types::Position::new(pos.x + dx, pos.y + dy));
    }

    /// Get the bounding rectangle (requires size information)
    fn bounds(&self) -> Option<crate::types::Rect> {
        None
    }
}

// ============================================================================
// Selectable Trait
// ============================================================================

/// Trait for types that can be selected in the UI
///
/// Types implementing this trait track selection state
/// for the visual editor.
pub trait Selectable {
    /// Check if currently selected
    fn is_selected(&self) -> bool;

    /// Set the selection state
    fn set_selected(&mut self, selected: bool);

    /// Toggle the selection state
    fn toggle_selected(&mut self) {
        self.set_selected(!self.is_selected());
    }
}

// ============================================================================
// Timestamped Trait
// ============================================================================

/// Trait for types that track creation and modification times
pub trait Timestamped {
    /// Get the creation timestamp
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get the last modification timestamp
    fn modified_at(&self) -> chrono::DateTime<chrono::Utc>;

    /// Update the modification timestamp to now
    fn touch(&mut self);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_context_indent() {
        let ctx = CodeGenContext::new();
        assert_eq!(ctx.indent(), "");

        let ctx = ctx.indented();
        assert_eq!(ctx.indent(), "    ");

        let ctx = ctx.indented();
        assert_eq!(ctx.indent(), "        ");
    }

    #[test]
    fn test_codegen_context_tabs() {
        let mut ctx = CodeGenContext::new();
        ctx.use_spaces = false;
        ctx.indent_level = 2;
        assert_eq!(ctx.indent(), "\t\t");
    }

    #[test]
    fn test_codegen_context_options() {
        let ctx = CodeGenContext::new()
            .with_option("framework", "axum")
            .with_option("orm", "sea-orm");

        assert_eq!(ctx.get_option("framework"), Some(&"axum".to_string()));
        assert_eq!(ctx.get_option("orm"), Some(&"sea-orm".to_string()));
        assert_eq!(ctx.get_option("unknown"), None);
    }

    // Test implementation for Validatable
    struct TestValidatable {
        valid: bool,
    }

    impl Validatable for TestValidatable {
        fn validate(&self) -> EngineResult<()> {
            if self.valid {
                Ok(())
            } else {
                Err(crate::error::EngineError::validation("Invalid state"))
            }
        }
    }

    #[test]
    fn test_validatable_trait() {
        let valid = TestValidatable { valid: true };
        assert!(valid.is_valid());
        assert!(valid.validation_errors().is_empty());

        let invalid = TestValidatable { valid: false };
        assert!(!invalid.is_valid());
        assert!(!invalid.validation_errors().is_empty());
    }
}
