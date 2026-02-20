//! Relationship definitions between entities
//!
//! This module contains the `Relationship` struct and related types for defining
//! connections between entities (foreign key relationships) in the Immortal Engine IR.

use chrono::{DateTime, Utc};
use imortal_core::{EngineError, EngineResult, ReferentialAction, RelationType, Validatable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Relationship
// ============================================================================

/// Represents a relationship between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Unique identifier for this relationship
    pub id: Uuid,

    /// Human-readable name for the relationship (e.g., "UserPosts")
    pub name: String,

    /// ID of the source entity (the "from" side)
    pub from_entity_id: Uuid,

    /// ID of the target entity (the "to" side)
    pub to_entity_id: Uuid,

    /// Type of relationship (OneToOne, OneToMany, etc.)
    pub relation_type: RelationType,

    /// Field name on the "from" entity (usually the FK field)
    pub from_field: String,

    /// Field name on the "to" entity (usually "id")
    pub to_field: String,

    /// Name for the inverse relation (e.g., "posts" for User -> Post)
    pub inverse_name: Option<String>,

    /// Human-readable description
    pub description: Option<String>,

    /// Visual connection: port position on source entity
    pub from_port: PortPosition,

    /// Visual connection: port position on target entity
    pub to_port: PortPosition,

    /// Referential action on delete
    pub on_delete: ReferentialAction,

    /// Referential action on update
    pub on_update: ReferentialAction,

    /// Whether this relationship is required (NOT NULL FK)
    pub required: bool,

    /// Whether the relationship is selected in the UI
    pub selected: bool,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub modified_at: DateTime<Utc>,
}

impl Relationship {
    /// Create a new relationship between two entities
    pub fn new(from_entity_id: Uuid, to_entity_id: Uuid, relation_type: RelationType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            from_entity_id,
            to_entity_id,
            relation_type,
            from_field: String::new(),
            to_field: "id".to_string(),
            inverse_name: None,
            description: None,
            from_port: PortPosition::Right,
            to_port: PortPosition::Left,
            on_delete: ReferentialAction::Restrict,
            on_update: ReferentialAction::Cascade,
            required: true,
            selected: false,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        }
    }

    /// Create a one-to-one relationship
    pub fn one_to_one(from_entity_id: Uuid, to_entity_id: Uuid) -> Self {
        Self::new(from_entity_id, to_entity_id, RelationType::OneToOne)
    }

    /// Create a one-to-many relationship
    pub fn one_to_many(from_entity_id: Uuid, to_entity_id: Uuid) -> Self {
        Self::new(from_entity_id, to_entity_id, RelationType::OneToMany)
    }

    /// Create a many-to-one relationship
    pub fn many_to_one(from_entity_id: Uuid, to_entity_id: Uuid) -> Self {
        Self::new(from_entity_id, to_entity_id, RelationType::ManyToOne)
    }

    /// Create a many-to-many relationship with junction table
    pub fn many_to_many(
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        junction_table: impl Into<String>,
    ) -> Self {
        Self::new(
            from_entity_id,
            to_entity_id,
            RelationType::ManyToMany {
                junction_table: junction_table.into(),
            },
        )
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Set the relationship name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the from field name
    pub fn with_from_field(mut self, field: impl Into<String>) -> Self {
        self.from_field = field.into();
        self
    }

    /// Set the to field name
    pub fn with_to_field(mut self, field: impl Into<String>) -> Self {
        self.to_field = field.into();
        self
    }

    /// Set the inverse relation name
    pub fn with_inverse(mut self, name: impl Into<String>) -> Self {
        self.inverse_name = Some(name.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the on delete action
    pub fn on_delete(mut self, action: ReferentialAction) -> Self {
        self.on_delete = action;
        self
    }

    /// Set the on update action
    pub fn on_update(mut self, action: ReferentialAction) -> Self {
        self.on_update = action;
        self
    }

    /// Mark as optional (NULL FK allowed)
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Set port positions for visual display
    pub fn with_ports(mut self, from: PortPosition, to: PortPosition) -> Self {
        self.from_port = from;
        self.to_port = to;
        self
    }

    // ========================================================================
    // Query methods
    // ========================================================================

    /// Check if this is a one-to-one relationship
    pub fn is_one_to_one(&self) -> bool {
        matches!(self.relation_type, RelationType::OneToOne)
    }

    /// Check if this is a one-to-many relationship
    pub fn is_one_to_many(&self) -> bool {
        matches!(self.relation_type, RelationType::OneToMany)
    }

    /// Check if this is a many-to-one relationship
    pub fn is_many_to_one(&self) -> bool {
        matches!(self.relation_type, RelationType::ManyToOne)
    }

    /// Check if this is a many-to-many relationship
    pub fn is_many_to_many(&self) -> bool {
        matches!(self.relation_type, RelationType::ManyToMany { .. })
    }

    /// Check if this relationship requires a junction table
    pub fn requires_junction_table(&self) -> bool {
        self.relation_type.requires_junction_table()
    }

    /// Get the junction table name (if many-to-many)
    pub fn junction_table(&self) -> Option<&str> {
        match &self.relation_type {
            RelationType::ManyToMany { junction_table } => Some(junction_table),
            _ => None,
        }
    }

    /// Check if a given entity is part of this relationship
    pub fn involves_entity(&self, entity_id: Uuid) -> bool {
        self.from_entity_id == entity_id || self.to_entity_id == entity_id
    }

    /// Get the other entity in the relationship
    pub fn other_entity(&self, entity_id: Uuid) -> Option<Uuid> {
        if self.from_entity_id == entity_id {
            Some(self.to_entity_id)
        } else if self.to_entity_id == entity_id {
            Some(self.from_entity_id)
        } else {
            None
        }
    }

    /// Create the inverse relationship
    pub fn inverse(&self) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: self.inverse_name.clone().unwrap_or_default(),
            from_entity_id: self.to_entity_id,
            to_entity_id: self.from_entity_id,
            relation_type: self.relation_type.inverse(),
            from_field: self.to_field.clone(),
            to_field: self.from_field.clone(),
            inverse_name: Some(self.name.clone()),
            description: self.description.clone(),
            from_port: self.to_port,
            to_port: self.from_port,
            on_delete: self.on_delete,
            on_update: self.on_update,
            required: self.required,
            selected: false,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        }
    }

    // ========================================================================
    // UI methods
    // ========================================================================

    /// Select this relationship
    pub fn select(&mut self) {
        self.selected = true;
    }

    /// Deselect this relationship
    pub fn deselect(&mut self) {
        self.selected = false;
    }

    /// Toggle selection
    pub fn toggle_selection(&mut self) {
        self.selected = !self.selected;
    }

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Get display label for the relationship
    pub fn display_label(&self) -> String {
        if self.name.is_empty() {
            format!("{}", self.relation_type.display_name())
        } else {
            self.name.clone()
        }
    }

    /// Get the arrow symbol for visual representation
    pub fn arrow_symbol(&self) -> &'static str {
        self.relation_type.arrow_symbol()
    }
}

impl Validatable for Relationship {
    fn validate(&self) -> EngineResult<()> {
        // From and to entities must be different (except for self-referential)
        // Self-referential relationships are allowed

        // From field must be specified for non-M2M relationships
        if !self.is_many_to_many() && self.from_field.is_empty() {
            return Err(EngineError::RelationshipValidation(
                "From field must be specified".to_string(),
            ));
        }

        // To field must be specified
        if self.to_field.is_empty() {
            return Err(EngineError::RelationshipValidation(
                "To field must be specified".to_string(),
            ));
        }

        // Junction table must be specified for M2M
        if self.is_many_to_many() && self.junction_table().map_or(true, |t| t.is_empty()) {
            return Err(EngineError::RelationshipValidation(
                "Junction table must be specified for many-to-many relationships".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for Relationship {
    fn default() -> Self {
        Self::new(Uuid::nil(), Uuid::nil(), RelationType::OneToMany)
    }
}

impl PartialEq for Relationship {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Relationship {}

impl std::hash::Hash for Relationship {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// ============================================================================
// PortPosition
// ============================================================================

/// Position of a connection port on an entity card
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PortPosition {
    /// Top edge
    Top,
    /// Right edge (default for output)
    #[default]
    Right,
    /// Bottom edge
    Bottom,
    /// Left edge (default for input)
    Left,
}

impl PortPosition {
    /// Get the opposite port position
    pub fn opposite(&self) -> Self {
        match self {
            PortPosition::Top => PortPosition::Bottom,
            PortPosition::Right => PortPosition::Left,
            PortPosition::Bottom => PortPosition::Top,
            PortPosition::Left => PortPosition::Right,
        }
    }

    /// Get the position offset relative to entity bounds
    pub fn offset(&self, width: f32, height: f32) -> (f32, f32) {
        match self {
            PortPosition::Top => (width / 2.0, 0.0),
            PortPosition::Right => (width, height / 2.0),
            PortPosition::Bottom => (width / 2.0, height),
            PortPosition::Left => (0.0, height / 2.0),
        }
    }

    /// Check if this is a horizontal port (left or right)
    pub fn is_horizontal(&self) -> bool {
        matches!(self, PortPosition::Left | PortPosition::Right)
    }

    /// Check if this is a vertical port (top or bottom)
    pub fn is_vertical(&self) -> bool {
        matches!(self, PortPosition::Top | PortPosition::Bottom)
    }

    /// Get all port positions
    pub fn all() -> &'static [PortPosition] {
        &[
            PortPosition::Top,
            PortPosition::Right,
            PortPosition::Bottom,
            PortPosition::Left,
        ]
    }
}

impl std::fmt::Display for PortPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortPosition::Top => write!(f, "top"),
            PortPosition::Right => write!(f, "right"),
            PortPosition::Bottom => write!(f, "bottom"),
            PortPosition::Left => write!(f, "left"),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_new() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let rel = Relationship::new(from_id, to_id, RelationType::OneToMany);

        assert_eq!(rel.from_entity_id, from_id);
        assert_eq!(rel.to_entity_id, to_id);
        assert!(rel.is_one_to_many());
    }

    #[test]
    fn test_relationship_builders() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let rel = Relationship::one_to_many(from_id, to_id)
            .with_name("UserPosts")
            .with_from_field("user_id")
            .with_inverse("posts")
            .on_delete(ReferentialAction::Cascade);

        assert_eq!(rel.name, "UserPosts");
        assert_eq!(rel.from_field, "user_id");
        assert_eq!(rel.inverse_name, Some("posts".to_string()));
        assert_eq!(rel.on_delete, ReferentialAction::Cascade);
    }

    #[test]
    fn test_relationship_types() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        assert!(Relationship::one_to_one(from_id, to_id).is_one_to_one());
        assert!(Relationship::one_to_many(from_id, to_id).is_one_to_many());
        assert!(Relationship::many_to_one(from_id, to_id).is_many_to_one());
        assert!(Relationship::many_to_many(from_id, to_id, "post_tags").is_many_to_many());
    }

    #[test]
    fn test_many_to_many_junction() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let rel = Relationship::many_to_many(from_id, to_id, "post_tags");

        assert!(rel.requires_junction_table());
        assert_eq!(rel.junction_table(), Some("post_tags"));
    }

    #[test]
    fn test_relationship_involves_entity() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        let rel = Relationship::one_to_many(from_id, to_id);

        assert!(rel.involves_entity(from_id));
        assert!(rel.involves_entity(to_id));
        assert!(!rel.involves_entity(other_id));
    }

    #[test]
    fn test_relationship_other_entity() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        let rel = Relationship::one_to_many(from_id, to_id);

        assert_eq!(rel.other_entity(from_id), Some(to_id));
        assert_eq!(rel.other_entity(to_id), Some(from_id));
        assert_eq!(rel.other_entity(other_id), None);
    }

    #[test]
    fn test_relationship_inverse() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        let rel = Relationship::one_to_many(from_id, to_id)
            .with_name("UserPosts")
            .with_from_field("user_id")
            .with_inverse("posts");

        let inverse = rel.inverse();

        assert_eq!(inverse.from_entity_id, to_id);
        assert_eq!(inverse.to_entity_id, from_id);
        assert!(inverse.is_many_to_one());
        assert_eq!(inverse.name, "posts");
    }

    #[test]
    fn test_relationship_validation() {
        let from_id = Uuid::new_v4();
        let to_id = Uuid::new_v4();

        // Valid relationship
        let valid = Relationship::one_to_many(from_id, to_id).with_from_field("user_id");
        assert!(valid.validate().is_ok());

        // Missing from_field
        let mut invalid = Relationship::one_to_many(from_id, to_id);
        invalid.from_field = String::new();
        assert!(invalid.validate().is_err());

        // M2M without junction table
        let invalid_m2m = Relationship::new(
            from_id,
            to_id,
            RelationType::ManyToMany {
                junction_table: String::new(),
            },
        );
        assert!(invalid_m2m.validate().is_err());
    }

    #[test]
    fn test_port_position() {
        assert_eq!(PortPosition::Top.opposite(), PortPosition::Bottom);
        assert_eq!(PortPosition::Left.opposite(), PortPosition::Right);

        assert!(PortPosition::Left.is_horizontal());
        assert!(PortPosition::Right.is_horizontal());
        assert!(PortPosition::Top.is_vertical());
        assert!(PortPosition::Bottom.is_vertical());
    }

    #[test]
    fn test_port_position_offset() {
        let (x, y) = PortPosition::Right.offset(200.0, 100.0);
        assert_eq!(x, 200.0);
        assert_eq!(y, 50.0);

        let (x, y) = PortPosition::Top.offset(200.0, 100.0);
        assert_eq!(x, 100.0);
        assert_eq!(y, 0.0);
    }
}
