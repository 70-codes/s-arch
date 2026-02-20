//! Validation rules and utilities for Immortal Engine IR
//!
//! This module provides validation capabilities for project graphs,
//! entities, relationships, and endpoints.

use crate::ProjectGraph;
use imortal_core::{EngineError, EngineResult};
use std::collections::HashSet;

// ============================================================================
// ValidationResult
// ============================================================================

/// Result of a validation operation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub valid: bool,

    /// List of errors (empty if valid)
    pub errors: Vec<ValidationError>,

    /// List of warnings (non-fatal issues)
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result with an error
    pub fn error(error: ValidationError) -> Self {
        Self {
            valid: false,
            errors: vec![error],
            warnings: Vec::new(),
        }
    }

    /// Create a result with a warning
    pub fn warning(warning: ValidationWarning) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: vec![warning],
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Merge another validation result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.valid {
            self.valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Convert to EngineResult (fails if any errors)
    pub fn to_result(self) -> EngineResult<()> {
        if self.valid {
            Ok(())
        } else {
            let msg = self
                .errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join("; ");
            Err(EngineError::validation(msg))
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::ok()
    }
}

// ============================================================================
// ValidationError
// ============================================================================

/// A validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error code for programmatic handling
    pub code: ValidationErrorCode,

    /// Human-readable error message
    pub message: String,

    /// Path to the problematic element (e.g., "entities.User.fields.email")
    pub path: Option<String>,

    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(code: ValidationErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
            suggestion: None,
        }
    }

    /// Add a path to the error
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Add a suggestion to the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "[{}] {}", path, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

// ============================================================================
// ValidationErrorCode
// ============================================================================

/// Error codes for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationErrorCode {
    // Entity errors
    EmptyEntityName,
    InvalidEntityName,
    DuplicateEntityName,
    NoFields,
    NoPrimaryKey,

    // Field errors
    EmptyFieldName,
    InvalidFieldName,
    DuplicateFieldName,
    InvalidFieldType,
    ForeignKeyMissingReference,

    // Relationship errors
    InvalidRelationship,
    OrphanRelationship,
    DuplicateRelationship,
    MissingJunctionTable,

    // Endpoint errors
    InvalidEndpointPath,
    DuplicateEndpointPath,
    OrphanEndpoint,

    // Project errors
    EmptyProjectName,
    InvalidProjectName,

    // Generic
    Custom,
}

// ============================================================================
// ValidationWarning
// ============================================================================

/// A validation warning (non-fatal issue)
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning code
    pub code: ValidationWarningCode,

    /// Human-readable warning message
    pub message: String,

    /// Path to the element
    pub path: Option<String>,
}

impl ValidationWarning {
    /// Create a new warning
    pub fn new(code: ValidationWarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            path: None,
        }
    }

    /// Add a path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "[{}] Warning: {}", path, self.message)
        } else {
            write!(f, "Warning: {}", self.message)
        }
    }
}

// ============================================================================
// ValidationWarningCode
// ============================================================================

/// Warning codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationWarningCode {
    NoDescription,
    NoEndpoints,
    NoRelationships,
    UnusedEntity,
    MissingIndex,
    WeakPassword,
    NoValidation,
    Custom,
}

// ============================================================================
// ValidationRule Trait
// ============================================================================

/// Trait for validation rules
pub trait ValidationRule {
    /// Get the rule name
    fn name(&self) -> &'static str;

    /// Get the rule description
    fn description(&self) -> &'static str;

    /// Validate a project and return the result
    fn validate(&self, project: &ProjectGraph) -> ValidationResult;
}

// ============================================================================
// Validator
// ============================================================================

/// Project validator that runs multiple validation rules
#[derive(Default)]
pub struct Validator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create a validator with default rules
    pub fn with_default_rules() -> Self {
        let mut validator = Self::new();
        validator.add_rule(Box::new(EntityNamesRule));
        validator.add_rule(Box::new(EntityFieldsRule));
        validator.add_rule(Box::new(RelationshipsRule));
        validator.add_rule(Box::new(EndpointsRule));
        validator.add_rule(Box::new(ProjectMetaRule));
        validator
    }

    /// Add a validation rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate a project with all rules
    pub fn validate(&self, project: &ProjectGraph) -> ValidationResult {
        let mut result = ValidationResult::ok();

        for rule in &self.rules {
            let rule_result = rule.validate(project);
            result.merge(rule_result);
        }

        result
    }

    /// Validate and return Result
    pub fn validate_result(&self, project: &ProjectGraph) -> EngineResult<()> {
        self.validate(project).to_result()
    }
}

// ============================================================================
// Built-in Validation Rules
// ============================================================================

/// Rule: Validate entity names
pub struct EntityNamesRule;

impl ValidationRule for EntityNamesRule {
    fn name(&self) -> &'static str {
        "entity_names"
    }

    fn description(&self) -> &'static str {
        "Validates that entity names are valid and unique"
    }

    fn validate(&self, project: &ProjectGraph) -> ValidationResult {
        let mut result = ValidationResult::ok();
        let mut seen_names: HashSet<String> = HashSet::new();

        for entity in project.entities.values() {
            if entity.name.is_empty() {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::EmptyEntityName,
                        "Entity name cannot be empty",
                    )
                    .with_path(format!("entities.{}", entity.id)),
                );
                continue;
            }

            if !is_valid_identifier(&entity.name) {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::InvalidEntityName,
                        format!("Entity name '{}' is not a valid identifier", entity.name),
                    )
                    .with_path(format!("entities.{}", entity.name))
                    .with_suggestion("Use PascalCase with only letters and numbers"),
                );
            }

            let lower_name = entity.name.to_lowercase();
            if seen_names.contains(&lower_name) {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::DuplicateEntityName,
                        format!("Duplicate entity name: '{}'", entity.name),
                    )
                    .with_path(format!("entities.{}", entity.name)),
                );
            }
            seen_names.insert(lower_name);
        }

        result
    }
}

/// Rule: Validate entity fields
pub struct EntityFieldsRule;

impl ValidationRule for EntityFieldsRule {
    fn name(&self) -> &'static str {
        "entity_fields"
    }

    fn description(&self) -> &'static str {
        "Validates that entities have valid fields with a primary key"
    }

    fn validate(&self, project: &ProjectGraph) -> ValidationResult {
        let mut result = ValidationResult::ok();

        for entity in project.entities.values() {
            let entity_path = format!("entities.{}", entity.name);

            if entity.fields.is_empty() {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::NoFields,
                        format!("Entity '{}' has no fields", entity.name),
                    )
                    .with_path(&entity_path),
                );
                continue;
            }

            if !entity.fields.iter().any(|f| f.is_primary_key) {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::NoPrimaryKey,
                        format!("Entity '{}' has no primary key", entity.name),
                    )
                    .with_path(&entity_path)
                    .with_suggestion("Add a field with is_primary_key = true"),
                );
            }

            let mut seen_fields: HashSet<String> = HashSet::new();
            for field in &entity.fields {
                let field_path = format!("{}.fields.{}", entity_path, field.name);

                // Empty field name
                if field.name.is_empty() {
                    result.add_error(
                        ValidationError::new(
                            ValidationErrorCode::EmptyFieldName,
                            "Field name cannot be empty",
                        )
                        .with_path(&field_path),
                    );
                    continue;
                }

                // Invalid field name
                if !is_valid_identifier(&field.name) {
                    result.add_error(
                        ValidationError::new(
                            ValidationErrorCode::InvalidFieldName,
                            format!("Field name '{}' is not a valid identifier", field.name),
                        )
                        .with_path(&field_path),
                    );
                }

                // Duplicate field name
                let lower_name = field.name.to_lowercase();
                if seen_fields.contains(&lower_name) {
                    result.add_error(
                        ValidationError::new(
                            ValidationErrorCode::DuplicateFieldName,
                            format!("Duplicate field name: '{}'", field.name),
                        )
                        .with_path(&field_path),
                    );
                }
                seen_fields.insert(lower_name);

                // Foreign key without reference
                if field.is_foreign_key && field.foreign_key_ref.is_none() {
                    result.add_error(
                        ValidationError::new(
                            ValidationErrorCode::ForeignKeyMissingReference,
                            format!("Foreign key '{}' has no reference", field.name),
                        )
                        .with_path(&field_path),
                    );
                }
            }

            // Warning: no description
            if entity.description.is_none() {
                result.add_warning(
                    ValidationWarning::new(
                        ValidationWarningCode::NoDescription,
                        format!("Entity '{}' has no description", entity.name),
                    )
                    .with_path(&entity_path),
                );
            }
        }

        result
    }
}

/// Rule: Validate relationships
pub struct RelationshipsRule;

impl ValidationRule for RelationshipsRule {
    fn name(&self) -> &'static str {
        "relationships"
    }

    fn description(&self) -> &'static str {
        "Validates that relationships reference existing entities"
    }

    fn validate(&self, project: &ProjectGraph) -> ValidationResult {
        let mut result = ValidationResult::ok();

        for relationship in project.relationships.values() {
            let rel_path = format!("relationships.{}", relationship.id);

            if !project.entities.contains_key(&relationship.from_entity_id) {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::OrphanRelationship,
                        format!(
                            "Relationship '{}' references non-existent source entity",
                            relationship.name
                        ),
                    )
                    .with_path(&rel_path),
                );
            }

            if !project.entities.contains_key(&relationship.to_entity_id) {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::OrphanRelationship,
                        format!(
                            "Relationship '{}' references non-existent target entity",
                            relationship.name
                        ),
                    )
                    .with_path(&rel_path),
                );
            }

            if relationship.requires_junction_table() {
                if let Some(junction) = relationship.junction_table() {
                    if junction.is_empty() {
                        result.add_error(
                            ValidationError::new(
                                ValidationErrorCode::MissingJunctionTable,
                                "Many-to-many relationship must specify a junction table name",
                            )
                            .with_path(&rel_path),
                        );
                    }
                }
            }
        }

        // Warning: no relationships
        if project.relationships.is_empty() && project.entities.len() > 1 {
            result.add_warning(ValidationWarning::new(
                ValidationWarningCode::NoRelationships,
                "Project has multiple entities but no relationships defined",
            ));
        }

        result
    }
}

/// Rule: Validate endpoints
pub struct EndpointsRule;

impl ValidationRule for EndpointsRule {
    fn name(&self) -> &'static str {
        "endpoints"
    }

    fn description(&self) -> &'static str {
        "Validates that endpoints are properly configured"
    }

    fn validate(&self, project: &ProjectGraph) -> ValidationResult {
        let mut result = ValidationResult::ok();
        let mut seen_paths: HashSet<String> = HashSet::new();

        for endpoint in project.endpoints.values() {
            let endpoint_path = format!("endpoints.{}", endpoint.id);

            if !project.entities.contains_key(&endpoint.entity_id) {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::OrphanEndpoint,
                        format!(
                            "Endpoint '{}' references non-existent entity",
                            endpoint.base_path
                        ),
                    )
                    .with_path(&endpoint_path),
                );
            }

            if endpoint.base_path.is_empty() {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::InvalidEndpointPath,
                        "Endpoint base path cannot be empty",
                    )
                    .with_path(&endpoint_path),
                );
            } else if !endpoint.base_path.starts_with('/') {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::InvalidEndpointPath,
                        format!("Endpoint path '{}' must start with '/'", endpoint.base_path),
                    )
                    .with_path(&endpoint_path),
                );
            }

            if seen_paths.contains(&endpoint.base_path) {
                result.add_error(
                    ValidationError::new(
                        ValidationErrorCode::DuplicateEndpointPath,
                        format!("Duplicate endpoint path: '{}'", endpoint.base_path),
                    )
                    .with_path(&endpoint_path),
                );
            }
            seen_paths.insert(endpoint.base_path.clone());
        }

        // Warning: entities without endpoints
        for entity in project.entities.values() {
            if entity.config.generate_api {
                let has_endpoint = project.endpoints.values().any(|e| e.entity_id == entity.id);

                if !has_endpoint {
                    result.add_warning(
                        ValidationWarning::new(
                            ValidationWarningCode::NoEndpoints,
                            format!(
                                "Entity '{}' has generate_api enabled but no endpoint configured",
                                entity.name
                            ),
                        )
                        .with_path(format!("entities.{}", entity.name)),
                    );
                }
            }
        }

        result
    }
}

/// Rule: Validate project metadata
pub struct ProjectMetaRule;

impl ValidationRule for ProjectMetaRule {
    fn name(&self) -> &'static str {
        "project_meta"
    }

    fn description(&self) -> &'static str {
        "Validates project metadata"
    }

    fn validate(&self, project: &ProjectGraph) -> ValidationResult {
        let mut result = ValidationResult::ok();

        if project.meta.name.is_empty() {
            result.add_error(ValidationError::new(
                ValidationErrorCode::EmptyProjectName,
                "Project name cannot be empty",
            ));
        }

        if project.meta.name.len() > 100 {
            result.add_error(
                ValidationError::new(
                    ValidationErrorCode::InvalidProjectName,
                    "Project name is too long (max 100 characters)",
                )
                .with_suggestion("Use a shorter project name"),
            );
        }

        // Warning: no description
        if project.meta.description.is_none() {
            result.add_warning(ValidationWarning::new(
                ValidationWarningCode::NoDescription,
                "Project has no description",
            ));
        }

        result
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a string is a valid identifier
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
    use crate::{Entity, Field};
    use imortal_core::DataType;

    #[test]
    fn test_validation_result_ok() {
        let result = ValidationResult::ok();
        assert!(result.valid);
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_validation_result_error() {
        let error = ValidationError::new(ValidationErrorCode::EmptyEntityName, "Test error");
        let result = ValidationResult::error(error);
        assert!(!result.valid);
        assert!(result.has_errors());
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::ok();
        let result2 = ValidationResult::error(ValidationError::new(
            ValidationErrorCode::EmptyEntityName,
            "Error",
        ));

        result1.merge(result2);
        assert!(!result1.valid);
        assert!(result1.has_errors());
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::new(ValidationErrorCode::EmptyEntityName, "Name is empty")
            .with_path("entities.User");

        let display = format!("{}", error);
        assert!(display.contains("entities.User"));
        assert!(display.contains("Name is empty"));
    }

    #[test]
    fn test_validator_with_valid_project() {
        let mut project = ProjectGraph::new("Test Project");
        let entity = Entity::with_timestamps("User")
            .with_field(Field::new("email", DataType::String).required().unique());
        project.add_entity(entity);

        let validator = Validator::with_default_rules();
        let result = validator.validate(&project);

        assert!(result.valid);
    }

    #[test]
    fn test_validator_with_invalid_project() {
        let mut project = ProjectGraph::new("");
        project.meta.name = String::new();

        let validator = Validator::with_default_rules();
        let result = validator.validate(&project);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::EmptyProjectName)
        );
    }

    #[test]
    fn test_entity_names_rule() {
        let mut project = ProjectGraph::new("Test");

        // Add entity with empty name
        let mut entity = Entity::new("User");
        entity.name = String::new();
        project.entities.insert(entity.id, entity);

        let rule = EntityNamesRule;
        let result = rule.validate(&project);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::EmptyEntityName)
        );
    }

    #[test]
    fn test_entity_fields_rule() {
        let mut project = ProjectGraph::new("Test");

        // Add entity with no fields
        let mut entity = Entity::new("User");
        entity.fields.clear();
        project.entities.insert(entity.id, entity);

        let rule = EntityFieldsRule;
        let result = rule.validate(&project);

        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.code == ValidationErrorCode::NoFields)
        );
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("user_id"));
        assert!(is_valid_identifier("User"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("user123"));

        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123user"));
        assert!(!is_valid_identifier("user-id"));
        assert!(!is_valid_identifier("user id"));
    }
}
