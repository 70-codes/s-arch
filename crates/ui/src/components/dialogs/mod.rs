//! # Dialog Components
//!
//! This module provides all dialog/modal components for the Immortal Engine UI.
//!
//! ## Dialogs
//!
//! - **EntityDialog**: Create and edit entities
//! - **FieldDialog**: Create and edit fields within entities
//! - **RelationshipDialog**: Create and edit relationships between entities
//! - **ConfirmDeleteDialog**: Confirmation dialogs for destructive actions
//! - **DataTypeSelector**: Enhanced data type selection component
//! - **ValidationEditor**: Field validation configuration
//!
//! ## Usage
//!
//! ```rust,ignore
//! use imortal_ui::components::dialogs::{EntityDialog, FieldDialog, RelationshipDialog, ConfirmDeleteDialog};
//!
//! fn MyComponent() -> Element {
//!     rsx! {
//!         EntityDialog { mode: EntityDialogMode::Create }
//!         FieldDialog { entity_id: some_id, mode: FieldDialogMode::Create }
//!         RelationshipDialog { mode: RelationshipDialogMode::Create { from_entity_id: None, to_entity_id: None } }
//!         ConfirmDeleteDialog { target: DeleteTarget::Entity(entity_id) }
//!     }
//! }
//! ```

// ============================================================================
// Module Declarations
// ============================================================================

pub mod confirm_delete;
pub mod data_type_selector;
pub mod endpoint_dialog;
pub mod entity_dialog;
pub mod field_dialog;
pub mod relationship_dialog;
pub mod validation_editor;

// ============================================================================
// Re-exports
// ============================================================================

pub use confirm_delete::ConfirmDeleteDialog;
pub use data_type_selector::{DataTypeSelector, DataTypeSelectorProps};
pub use endpoint_dialog::{EndpointDialog, EndpointDialogMode};
pub use entity_dialog::{EntityDialog, EntityDialogMode};
pub use field_dialog::{FieldDialog, FieldDialogMode};
pub use relationship_dialog::{RelationshipDialog, RelationshipDialogMode};
pub use validation_editor::{ValidationEditor, ValidationEditorProps};
