//! # UI Components
//!
//! Reusable Dioxus components for the Immortal Engine visual editor.
//!
//! This module provides the core visual components for:
//! - **Canvas**: The main visual editor canvas with pan/zoom support
//! - **Entity Card**: Visual representation of database entities
//! - **Field Row**: Individual field display within entity cards
//! - **Properties Panel**: Detailed property editing panel
//! - **Inputs**: Form input components (text, select, checkbox, etc.)
//! - **Dialogs**: Modal dialogs for entity/field/relationship creation, deletion, etc.
//! - **Port**: Connection ports on entity cards for relationships
//! - **Connection**: SVG connection lines for visualizing relationships
//!
//! ## Component Hierarchy
//!
//! ```text
//! Canvas
//! ├── ConnectionsLayer (SVG)
//! │   └── ConnectionLine (multiple)
//! ├── EntityCard
//! │   ├── FieldRow (multiple)
//! │   └── Port (input/output)
//! └── ConnectionPreview (while drawing)
//!
//! PropertiesPanel
//! ├── EntityProperties
//! ├── FieldProperties
//! ├── RelationshipProperties
//! └── Input components
//!
//! Dialogs
//! ├── EntityDialog (create/edit entities)
//! ├── FieldDialog (create/edit fields)
//! ├── RelationshipDialog (create/edit relationships)
//! ├── ConfirmDeleteDialog (delete confirmation)
//! ├── DataTypeSelector (type selection)
//! └── ValidationEditor (validation rules)
//! ```

// ============================================================================
// Module Declarations
// ============================================================================

pub mod canvas;
pub mod connection;
pub mod dialogs;
pub mod endpoint_card;
pub mod entity_card;
pub mod field_row;
pub mod inputs;
pub mod port;
pub mod properties;

// ============================================================================
// Re-exports
// ============================================================================

// Canvas components
pub use canvas::{Canvas, CanvasToolbar};

// Entity components
pub use entity_card::EntityCard;
pub use field_row::FieldRow;

// Endpoint components
pub use endpoint_card::{
    EndpointCard, GenerateEndpointsCard, http_method_class, operation_type_color,
    operation_type_description,
};

// Properties panel
pub use properties::PropertiesPanel;

// Port components
pub use port::{Port, PortClickInfo, PortPair, PortState, PortType};

// Connection components
pub use connection::{
    ConnectionLine, ConnectionPoint, ConnectionPreview, ConnectionsLayer, calculate_port_position,
    relationship_color,
};

// Re-export input components
pub use inputs::{Checkbox, NumberInput, Select, SelectOption, TextArea, TextInput, Toggle};

// Re-export dialog components
pub use dialogs::{
    ConfirmDeleteDialog, DataTypeSelector, EndpointDialog, EndpointDialogMode, EntityDialog,
    EntityDialogMode, FieldDialog, FieldDialogMode, RelationshipDialog, RelationshipDialogMode,
    ValidationEditor,
};
