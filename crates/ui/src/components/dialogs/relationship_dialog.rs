//! # Relationship Dialog Component
//!
//! Dialog for creating and editing relationships between entities.
//!
//! This dialog allows users to:
//! - Select source and target entities
//! - Choose relationship type (1:1, 1:N, N:1, N:M)
//! - Configure field mappings (from_field, to_field)
//! - Set referential actions (CASCADE, SET NULL, RESTRICT, etc.)
//! - Name the relationship and its inverse
//!
//! ## Usage
//!
//! ```rust,ignore
//! RelationshipDialog {
//!     mode: RelationshipDialogMode::Create {
//!         from_entity_id: Some(entity_id),
//!         to_entity_id: None,
//!     },
//! }
//! ```

use dioxus::prelude::*;
use imortal_core::{ReferentialAction, RelationType};
use imortal_ir::{Entity, PortPosition, Relationship};
use uuid::Uuid;

use crate::components::inputs::{Checkbox, Select, SelectOption, TextArea, TextInput};
use crate::state::APP_STATE;

// ============================================================================
// Dialog Mode
// ============================================================================

/// Mode for the relationship dialog
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipDialogMode {
    /// Creating a new relationship
    Create {
        /// Pre-selected source entity (optional)
        from_entity_id: Option<Uuid>,
        /// Pre-selected target entity (optional)
        to_entity_id: Option<Uuid>,
    },
    /// Editing an existing relationship
    Edit(Uuid),
}

impl RelationshipDialogMode {
    /// Check if this is create mode
    pub fn is_create(&self) -> bool {
        matches!(self, RelationshipDialogMode::Create { .. })
    }

    /// Get the title for the dialog
    pub fn title(&self) -> &'static str {
        match self {
            RelationshipDialogMode::Create { .. } => "Create Relationship",
            RelationshipDialogMode::Edit(_) => "Edit Relationship",
        }
    }

    /// Get the submit button text
    pub fn submit_text(&self) -> &'static str {
        match self {
            RelationshipDialogMode::Create { .. } => "Create",
            RelationshipDialogMode::Edit(_) => "Save Changes",
        }
    }
}

// ============================================================================
// Relationship Dialog Props
// ============================================================================

/// Properties for the RelationshipDialog component
#[derive(Props, Clone, PartialEq)]
pub struct RelationshipDialogProps {
    /// Dialog mode (create or edit)
    pub mode: RelationshipDialogMode,
}

// ============================================================================
// Relationship Form State
// ============================================================================

/// Internal form state for the relationship dialog
#[derive(Debug, Clone, PartialEq)]
struct RelationshipFormState {
    name: String,
    from_entity_id: Option<Uuid>,
    to_entity_id: Option<Uuid>,
    relation_type: RelationType,
    from_field: String,
    to_field: String,
    inverse_name: String,
    description: String,
    on_delete: ReferentialAction,
    on_update: ReferentialAction,
    required: bool,
    from_port: PortPosition,
    to_port: PortPosition,
}

impl Default for RelationshipFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            from_entity_id: None,
            to_entity_id: None,
            relation_type: RelationType::OneToMany,
            from_field: String::new(),
            to_field: "id".to_string(),
            inverse_name: String::new(),
            description: String::new(),
            on_delete: ReferentialAction::Restrict,
            on_update: ReferentialAction::Cascade,
            required: true,
            from_port: PortPosition::Right,
            to_port: PortPosition::Left,
        }
    }
}

impl RelationshipFormState {
    /// Create form state from an existing relationship
    fn from_relationship(rel: &Relationship) -> Self {
        Self {
            name: rel.name.clone(),
            from_entity_id: Some(rel.from_entity_id),
            to_entity_id: Some(rel.to_entity_id),
            relation_type: rel.relation_type.clone(),
            from_field: rel.from_field.clone(),
            to_field: rel.to_field.clone(),
            inverse_name: rel.inverse_name.clone().unwrap_or_default(),
            description: rel.description.clone().unwrap_or_default(),
            on_delete: rel.on_delete.clone(),
            on_update: rel.on_update.clone(),
            required: rel.required,
            from_port: rel.from_port.clone(),
            to_port: rel.to_port.clone(),
        }
    }

    /// Create a new relationship from form state
    fn to_relationship(&self) -> Option<Relationship> {
        let from_id = self.from_entity_id?;
        let to_id = self.to_entity_id?;

        let mut rel = Relationship::new(from_id, to_id, self.relation_type.clone());
        rel.name = self.name.clone();
        rel.from_field = self.from_field.clone();
        rel.to_field = self.to_field.clone();
        rel.inverse_name = if self.inverse_name.is_empty() {
            None
        } else {
            Some(self.inverse_name.clone())
        };
        rel.description = if self.description.is_empty() {
            None
        } else {
            Some(self.description.clone())
        };
        rel.on_delete = self.on_delete.clone();
        rel.on_update = self.on_update.clone();
        rel.required = self.required;
        rel.from_port = self.from_port.clone();
        rel.to_port = self.to_port.clone();

        Some(rel)
    }

    /// Validate the form state
    fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.from_entity_id.is_none() {
            errors.push("Source entity is required".to_string());
        }

        if self.to_entity_id.is_none() {
            errors.push("Target entity is required".to_string());
        }

        if self.from_entity_id == self.to_entity_id && self.from_entity_id.is_some() {
            errors.push("Source and target entities must be different".to_string());
        }

        if self.name.is_empty() {
            errors.push("Relationship name is required".to_string());
        }

        errors
    }

    /// Check if form is valid
    fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

// ============================================================================
// Relationship Dialog Component
// ============================================================================

/// Dialog for creating or editing relationships
#[component]
pub fn RelationshipDialog(props: RelationshipDialogProps) -> Element {
    // Get entities for selection
    let entities: Vec<Entity> = {
        let state = APP_STATE.read();
        state
            .project
            .as_ref()
            .map(|p| p.entities.values().cloned().collect())
            .unwrap_or_default()
    };

    // Initialize form state based on mode
    let mut form_state = use_signal(|| {
        match &props.mode {
            RelationshipDialogMode::Create {
                from_entity_id,
                to_entity_id,
            } => {
                let mut state = RelationshipFormState::default();
                state.from_entity_id = *from_entity_id;
                state.to_entity_id = *to_entity_id;

                // Auto-generate name if both entities are selected
                if let (Some(from_id), Some(to_id)) = (from_entity_id, to_entity_id) {
                    let app_state = APP_STATE.read();
                    if let Some(project) = &app_state.project {
                        let from_name = project
                            .entities
                            .get(from_id)
                            .map(|e| e.name.clone())
                            .unwrap_or_default();
                        let to_name = project
                            .entities
                            .get(to_id)
                            .map(|e| e.name.clone())
                            .unwrap_or_default();
                        state.name = format!("{}{}", from_name, to_name);
                    }
                }

                state
            }
            RelationshipDialogMode::Edit(rel_id) => {
                let state = APP_STATE.read();
                if let Some(project) = &state.project {
                    if let Some(rel) = project.relationships.get(rel_id) {
                        return RelationshipFormState::from_relationship(rel);
                    }
                }
                RelationshipFormState::default()
            }
        }
    });

    let mut errors = use_signal(Vec::<String>::new);
    let mut active_tab = use_signal(|| 0usize);

    // Validation
    let validation_errors = form_state.read().validate();
    let is_valid = validation_errors.is_empty();

    // Close dialog handler
    let close_dialog = move |_| {
        APP_STATE.write().ui.close_dialog();
    };

    // Handle form submission
    let handle_submit = {
        let mode = props.mode.clone();
        move |_| {
            let state = form_state.read();
            let errs = state.validate();

            if !errs.is_empty() {
                errors.set(errs);
                return;
            }

            if let Some(relationship) = state.to_relationship() {
                let mut app_state = APP_STATE.write();

                match &mode {
                    RelationshipDialogMode::Create { .. } => {
                        if let Some(project) = &mut app_state.project {
                            // Use create_relationship_with_fk for auto FK field generation
                            match project.create_relationship_with_fk(relationship) {
                                Ok((rel_id, fk_field_id)) => {
                                    app_state.is_dirty = true;
                                    if let Some(fk_id) = fk_field_id {
                                        tracing::info!(
                                            "Created relationship {} with FK field {}",
                                            rel_id,
                                            fk_id
                                        );
                                    } else {
                                        tracing::info!(
                                            "Created relationship {} (no FK - M:N or existing)",
                                            rel_id
                                        );
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to create relationship: {}", e);
                                    errors
                                        .set(vec![format!("Failed to create relationship: {}", e)]);
                                    return;
                                }
                            }
                        }
                    }
                    RelationshipDialogMode::Edit(rel_id) => {
                        if let Some(project) = &mut app_state.project {
                            if let Some(existing) = project.relationships.get_mut(rel_id) {
                                // Preserve the original ID
                                let mut updated = relationship;
                                updated.id = *rel_id;
                                updated.created_at = existing.created_at;
                                *existing = updated;
                                app_state.is_dirty = true;
                            }
                        }
                    }
                }

                drop(app_state);
                APP_STATE.write().save_to_history(if mode.is_create() {
                    "Create relationship with FK"
                } else {
                    "Edit relationship"
                });
                APP_STATE.write().ui.close_dialog();
            }
        }
    };

    // Entity options for select dropdowns
    let entity_options: Vec<SelectOption> = entities
        .iter()
        .map(|e| SelectOption {
            value: e.id.to_string(),
            label: e.name.clone(),
            disabled: false,
        })
        .collect();

    // Relation type options
    let relation_type_options = vec![
        SelectOption {
            value: "one_to_one".to_string(),
            label: "One-to-One (1:1)".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "one_to_many".to_string(),
            label: "One-to-Many (1:N)".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "many_to_one".to_string(),
            label: "Many-to-One (N:1)".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "many_to_many".to_string(),
            label: "Many-to-Many (N:M)".to_string(),
            disabled: false,
        },
    ];

    // Referential action options
    let ref_action_options = vec![
        SelectOption {
            value: "cascade".to_string(),
            label: "CASCADE".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "set_null".to_string(),
            label: "SET NULL".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "restrict".to_string(),
            label: "RESTRICT".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "no_action".to_string(),
            label: "NO ACTION".to_string(),
            disabled: false,
        },
    ];

    // Port position options
    let port_options = vec![
        SelectOption {
            value: "left".to_string(),
            label: "Left".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "right".to_string(),
            label: "Right".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "top".to_string(),
            label: "Top".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "bottom".to_string(),
            label: "Bottom".to_string(),
            disabled: false,
        },
    ];

    // Get field options for selected entities
    let from_field_options: Vec<SelectOption> = {
        let state = APP_STATE.read();
        if let (Some(project), Some(from_id)) = (&state.project, form_state.read().from_entity_id) {
            project
                .entities
                .get(&from_id)
                .map(|e| {
                    e.fields
                        .iter()
                        .map(|f| SelectOption {
                            value: f.name.clone(),
                            label: format!("{} ({})", f.name, f.data_type),
                            disabled: false,
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        }
    };

    let to_field_options: Vec<SelectOption> = {
        let state = APP_STATE.read();
        if let (Some(project), Some(to_id)) = (&state.project, form_state.read().to_entity_id) {
            project
                .entities
                .get(&to_id)
                .map(|e| {
                    e.fields
                        .iter()
                        .map(|f| SelectOption {
                            value: f.name.clone(),
                            label: format!("{} ({})", f.name, f.data_type),
                            disabled: false,
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        }
    };

    rsx! {
        div {
            class: "relationship-dialog",

            // Header
            div {
                class: "flex items-center justify-between p-4 border-b border-slate-700",

                h2 {
                    class: "text-xl font-semibold text-white flex items-center gap-2",
                    // Relationship icon
                    svg {
                        class: "w-6 h-6 text-indigo-400",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1",
                        }
                    }
                    "{props.mode.title()}"
                }

                button {
                    class: "text-slate-400 hover:text-white transition-colors",
                    onclick: close_dialog,
                    // Close icon
                    svg {
                        class: "w-6 h-6",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M6 18L18 6M6 6l12 12",
                        }
                    }
                }
            }

            // Tabs
            div {
                class: "flex border-b border-slate-700",

                TabButton {
                    label: "Basic",
                    active: *active_tab.read() == 0,
                    on_click: move |_| active_tab.set(0),
                }
                TabButton {
                    label: "Fields",
                    active: *active_tab.read() == 1,
                    on_click: move |_| active_tab.set(1),
                }
                TabButton {
                    label: "Actions",
                    active: *active_tab.read() == 2,
                    on_click: move |_| active_tab.set(2),
                }
                TabButton {
                    label: "Visual",
                    active: *active_tab.read() == 3,
                    on_click: move |_| active_tab.set(3),
                }
            }

            // Error display
            if !errors.read().is_empty() {
                div {
                    class: "mx-4 mt-4 p-3 bg-red-900/30 border border-red-700 rounded-lg",
                    ul {
                        class: "text-red-400 text-sm space-y-1",
                        for error in errors.read().iter() {
                            li { "{error}" }
                        }
                    }
                }
            }

            // Tab content
            div {
                class: "p-4 space-y-4 max-h-[60vh] overflow-y-auto",

                // Basic tab
                if *active_tab.read() == 0 {
                    // Relationship name
                    TextInput {
                        label: "Name",
                        value: form_state.read().name.clone(),
                        placeholder: "e.g., UserPosts, OrderItems",
                        required: true,
                        on_change: move |v: String| {
                            form_state.write().name = v;
                        },
                    }

                    // Entity selection row
                    div {
                        class: "grid grid-cols-2 gap-4",

                        // Source entity
                        Select {
                            label: "From Entity",
                            value: form_state.read().from_entity_id.map(|id| id.to_string()).unwrap_or_default(),
                            options: entity_options.clone(),
                            placeholder: "Select source entity",
                            required: true,
                            on_change: move |v: String| {
                                let id = Uuid::parse_str(&v).ok();
                                form_state.write().from_entity_id = id;

                                // Auto-update relationship name
                                if let Some(from_id) = id {
                                    let state = APP_STATE.read();
                                    if let Some(project) = &state.project {
                                        let from_name = project.entities.get(&from_id)
                                            .map(|e| e.name.clone())
                                            .unwrap_or_default();
                                        let to_name = form_state.read().to_entity_id
                                            .and_then(|to_id| project.entities.get(&to_id))
                                            .map(|e| e.name.clone())
                                            .unwrap_or_default();

                                        if form_state.read().name.is_empty() {
                                            form_state.write().name = format!("{}{}", from_name, to_name);
                                        }
                                    }
                                }
                            },
                        }

                        // Target entity
                        Select {
                            label: "To Entity",
                            value: form_state.read().to_entity_id.map(|id| id.to_string()).unwrap_or_default(),
                            options: entity_options.clone(),
                            placeholder: "Select target entity",
                            required: true,
                            on_change: move |v: String| {
                                let id = Uuid::parse_str(&v).ok();
                                form_state.write().to_entity_id = id;

                                // Auto-update relationship name and inverse
                                if let Some(to_id) = id {
                                    let state = APP_STATE.read();
                                    if let Some(project) = &state.project {
                                        let to_name = project.entities.get(&to_id)
                                            .map(|e| e.name.clone())
                                            .unwrap_or_default();
                                        let from_name = form_state.read().from_entity_id
                                            .and_then(|from_id| project.entities.get(&from_id))
                                            .map(|e| e.name.clone())
                                            .unwrap_or_default();

                                        if form_state.read().name.is_empty() {
                                            form_state.write().name = format!("{}{}", from_name, to_name);
                                        }

                                        // Auto-set inverse name
                                        if form_state.read().inverse_name.is_empty() {
                                            let mut fs = form_state.write();
                                            fs.inverse_name = to_lowercase_first(&from_name);
                                        }
                                    }
                                }
                            },
                        }
                    }

                    // Relationship type
                    Select {
                        label: "Relationship Type",
                        value: relation_type_to_string(&form_state.read().relation_type),
                        options: relation_type_options,
                        required: true,
                        on_change: move |v: String| {
                            form_state.write().relation_type = string_to_relation_type(&v);
                        },
                    }

                    // Relationship type description
                    div {
                        class: "p-3 bg-slate-700/50 rounded-lg text-sm",
                        {relationship_type_description(&form_state.read().relation_type)}
                    }

                    // Required checkbox
                    Checkbox {
                        label: "Required relationship (NOT NULL foreign key)",
                        checked: form_state.read().required,
                        on_change: move |v: bool| {
                            form_state.write().required = v;
                        },
                    }

                    // Description
                    TextArea {
                        label: "Description",
                        value: form_state.read().description.clone(),
                        placeholder: "Describe this relationship...",
                        rows: 2,
                        on_change: move |v: String| {
                            form_state.write().description = v;
                        },
                    }
                }

                // Fields tab
                if *active_tab.read() == 1 {
                    div {
                        class: "space-y-4",

                        // Info box
                        div {
                            class: "p-3 bg-indigo-900/30 border border-indigo-700 rounded-lg text-sm text-indigo-200",
                            p {
                                "Configure which fields connect the entities. The 'From Field' is typically "
                                "the foreign key field on the source entity, and 'To Field' is usually the "
                                "primary key ('id') on the target entity."
                            }
                        }

                        // From field
                        Select {
                            label: "From Field (Foreign Key)",
                            value: form_state.read().from_field.clone(),
                            options: from_field_options,
                            placeholder: "Select or enter field name",
                            help_text: "The field on the source entity that references the target",
                            on_change: move |v: String| {
                                form_state.write().from_field = v;
                            },
                        }

                        // Custom from field input
                        TextInput {
                            label: "Or enter custom field name",
                            value: form_state.read().from_field.clone(),
                            placeholder: "e.g., user_id, author_id",
                            on_change: move |v: String| {
                                form_state.write().from_field = v;
                            },
                        }

                        // To field
                        Select {
                            label: "To Field (Reference)",
                            value: form_state.read().to_field.clone(),
                            options: to_field_options,
                            placeholder: "Usually 'id'",
                            help_text: "The field on the target entity being referenced (usually primary key)",
                            on_change: move |v: String| {
                                form_state.write().to_field = v;
                            },
                        }

                        // Inverse name
                        TextInput {
                            label: "Inverse Relationship Name",
                            value: form_state.read().inverse_name.clone(),
                            placeholder: "e.g., posts, orders, items",
                            help_text: "Name for the reverse relationship (for navigation from target to source)",
                            on_change: move |v: String| {
                                form_state.write().inverse_name = v;
                            },
                        }
                    }
                }

                // Actions tab (referential actions)
                if *active_tab.read() == 2 {
                    div {
                        class: "space-y-4",

                        // Info box
                        div {
                            class: "p-3 bg-amber-900/30 border border-amber-700 rounded-lg text-sm text-amber-200",
                            p {
                                "Referential actions determine what happens when the referenced row "
                                "is deleted or updated. Choose carefully based on your data integrity needs."
                            }
                        }

                        // On Delete action
                        Select {
                            label: "On Delete",
                            value: ref_action_to_string(&form_state.read().on_delete),
                            options: ref_action_options.clone(),
                            help_text: "What happens when the referenced row is deleted",
                            on_change: move |v: String| {
                                form_state.write().on_delete = string_to_ref_action(&v);
                            },
                        }

                        // On Delete description
                        div {
                            class: "p-2 bg-slate-700/50 rounded text-xs text-slate-300",
                            {ref_action_description(&form_state.read().on_delete, "deleted")}
                        }

                        // On Update action
                        Select {
                            label: "On Update",
                            value: ref_action_to_string(&form_state.read().on_update),
                            options: ref_action_options,
                            help_text: "What happens when the referenced row's key is updated",
                            on_change: move |v: String| {
                                form_state.write().on_update = string_to_ref_action(&v);
                            },
                        }

                        // On Update description
                        div {
                            class: "p-2 bg-slate-700/50 rounded text-xs text-slate-300",
                            {ref_action_description(&form_state.read().on_update, "updated")}
                        }
                    }
                }

                // Visual tab (port positions)
                if *active_tab.read() == 3 {
                    div {
                        class: "space-y-4",

                        // Info box
                        div {
                            class: "p-3 bg-slate-700/50 rounded-lg text-sm text-slate-300",
                            p {
                                "Configure where the connection line attaches to each entity card on the canvas."
                            }
                        }

                        div {
                            class: "grid grid-cols-2 gap-4",

                            // From port position
                            Select {
                                label: "From Port Position",
                                value: port_position_to_string(&form_state.read().from_port),
                                options: port_options.clone(),
                                on_change: move |v: String| {
                                    form_state.write().from_port = string_to_port_position(&v);
                                },
                            }

                            // To port position
                            Select {
                                label: "To Port Position",
                                value: port_position_to_string(&form_state.read().to_port),
                                options: port_options,
                                on_change: move |v: String| {
                                    form_state.write().to_port = string_to_port_position(&v);
                                },
                            }
                        }

                        // Preview of connection
                        div {
                            class: "mt-4 p-4 bg-slate-900 rounded-lg",
                            h4 {
                                class: "text-sm font-medium text-slate-400 mb-3",
                                "Connection Preview"
                            }
                            ConnectionPreviewDiagram {
                                from_port: form_state.read().from_port.clone(),
                                to_port: form_state.read().to_port.clone(),
                            }
                        }
                    }
                }
            }

            // Footer
            div {
                class: "flex items-center justify-between p-4 border-t border-slate-700 bg-slate-800/50",

                // Validation status
                div {
                    class: "text-sm",
                    if !is_valid {
                        span {
                            class: "text-amber-400",
                            "⚠ {validation_errors.len()} issue(s) to fix"
                        }
                    } else {
                        span {
                            class: "text-green-400",
                            "✓ Ready to save"
                        }
                    }
                }

                // Buttons
                div {
                    class: "flex gap-3",

                    button {
                        class: "px-4 py-2 text-slate-300 hover:text-white transition-colors",
                        onclick: close_dialog,
                        "Cancel"
                    }

                    button {
                        class: "px-6 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-slate-600 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors",
                        disabled: !is_valid,
                        onclick: handle_submit,
                        "{props.mode.submit_text()}"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tab Button Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct TabButtonProps {
    label: &'static str,
    active: bool,
    on_click: EventHandler<()>,
}

#[component]
fn TabButton(props: TabButtonProps) -> Element {
    let class = if props.active {
        "px-4 py-2 text-indigo-400 border-b-2 border-indigo-400 font-medium"
    } else {
        "px-4 py-2 text-slate-400 hover:text-white border-b-2 border-transparent transition-colors"
    };

    rsx! {
        button {
            class: "{class}",
            onclick: move |_| props.on_click.call(()),
            "{props.label}"
        }
    }
}

// ============================================================================
// Connection Preview Diagram
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct ConnectionPreviewDiagramProps {
    from_port: PortPosition,
    to_port: PortPosition,
}

#[component]
fn ConnectionPreviewDiagram(props: ConnectionPreviewDiagramProps) -> Element {
    // Simple SVG preview showing connection between two boxes
    let (from_x, from_y) = port_position_coords(&props.from_port, 30.0, 80.0, 40.0);
    let (to_x, to_y) = port_position_coords(&props.to_port, 170.0, 80.0, 40.0);

    // Calculate control points for bezier
    let ctrl_offset = 40.0;
    let (ctrl1_x, ctrl1_y) = match props.from_port {
        PortPosition::Left => (from_x - ctrl_offset, from_y),
        PortPosition::Right => (from_x + ctrl_offset, from_y),
        PortPosition::Top => (from_x, from_y - ctrl_offset),
        PortPosition::Bottom => (from_x, from_y + ctrl_offset),
    };
    let (ctrl2_x, ctrl2_y) = match props.to_port {
        PortPosition::Left => (to_x - ctrl_offset, to_y),
        PortPosition::Right => (to_x + ctrl_offset, to_y),
        PortPosition::Top => (to_x, to_y - ctrl_offset),
        PortPosition::Bottom => (to_x, to_y + ctrl_offset),
    };

    let path = format!(
        "M {},{} C {},{} {},{} {},{}",
        from_x, from_y, ctrl1_x, ctrl1_y, ctrl2_x, ctrl2_y, to_x, to_y
    );

    rsx! {
        svg {
            class: "w-full h-24",
            view_box: "0 0 240 100",

            // From entity box
            rect {
                x: "10",
                y: "30",
                width: "60",
                height: "40",
                rx: "4",
                fill: "#334155",
                stroke: "#6366f1",
                stroke_width: "2",
            }
            text {
                x: "40",
                y: "55",
                text_anchor: "middle",
                font_size: "10",
                fill: "#e2e8f0",
                "From"
            }

            // To entity box
            rect {
                x: "170",
                y: "30",
                width: "60",
                height: "40",
                rx: "4",
                fill: "#334155",
                stroke: "#22c55e",
                stroke_width: "2",
            }
            text {
                x: "200",
                y: "55",
                text_anchor: "middle",
                font_size: "10",
                fill: "#e2e8f0",
                "To"
            }

            // Connection line
            path {
                d: "{path}",
                stroke: "#6366f1",
                stroke_width: "2",
                fill: "none",
                marker_end: "url(#preview-arrow)",
            }

            // Arrow marker
            defs {
                marker {
                    id: "preview-arrow",
                    marker_width: "10",
                    marker_height: "10",
                    ref_x: "8",
                    ref_y: "5",
                    orient: "auto",
                    path {
                        d: "M 0 0 L 10 5 L 0 10 Z",
                        fill: "#6366f1",
                    }
                }
            }

            // Port indicators
            circle {
                cx: "{from_x}",
                cy: "{from_y}",
                r: "4",
                fill: "#6366f1",
            }
            circle {
                cx: "{to_x}",
                cy: "{to_y}",
                r: "4",
                fill: "#22c55e",
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get port coordinates for preview diagram
fn port_position_coords(
    port: &PortPosition,
    box_x: f32,
    box_width: f32,
    box_height: f32,
) -> (f32, f32) {
    let center_x = box_x + box_width / 2.0;
    let center_y = 50.0; // Fixed Y center for diagram

    match port {
        PortPosition::Left => (box_x, center_y),
        PortPosition::Right => (box_x + box_width, center_y),
        PortPosition::Top => (center_x, center_y - box_height / 2.0),
        PortPosition::Bottom => (center_x, center_y + box_height / 2.0),
    }
}

/// Convert RelationType to string for select
fn relation_type_to_string(rt: &RelationType) -> String {
    match rt {
        RelationType::OneToOne => "one_to_one".to_string(),
        RelationType::OneToMany => "one_to_many".to_string(),
        RelationType::ManyToOne => "many_to_one".to_string(),
        RelationType::ManyToMany { .. } => "many_to_many".to_string(),
    }
}

/// Convert string to RelationType
fn string_to_relation_type(s: &str) -> RelationType {
    match s {
        "one_to_one" => RelationType::OneToOne,
        "one_to_many" => RelationType::OneToMany,
        "many_to_one" => RelationType::ManyToOne,
        "many_to_many" => RelationType::ManyToMany {
            junction_table: String::new(),
        },
        _ => RelationType::OneToMany,
    }
}

/// Get description for relationship type
fn relationship_type_description(rt: &RelationType) -> &'static str {
    match rt {
        RelationType::OneToOne => {
            "Each record in the source entity is associated with exactly one record in the target entity, and vice versa. Example: User has one Profile."
        }
        RelationType::OneToMany => {
            "Each record in the source entity can be associated with multiple records in the target entity. Example: User has many Posts."
        }
        RelationType::ManyToOne => {
            "Multiple records in the source entity can be associated with one record in the target entity. Example: Many Posts belong to one User."
        }
        RelationType::ManyToMany { .. } => {
            "Records in both entities can be associated with multiple records in the other. Requires a junction table. Example: Students and Courses."
        }
    }
}

/// Convert ReferentialAction to string
fn ref_action_to_string(action: &ReferentialAction) -> String {
    match action {
        ReferentialAction::Cascade => "cascade".to_string(),
        ReferentialAction::SetNull => "set_null".to_string(),
        ReferentialAction::Restrict => "restrict".to_string(),
        ReferentialAction::NoAction => "no_action".to_string(),
        ReferentialAction::SetDefault => "set_default".to_string(),
    }
}

/// Convert string to ReferentialAction
fn string_to_ref_action(s: &str) -> ReferentialAction {
    match s {
        "cascade" => ReferentialAction::Cascade,
        "set_null" => ReferentialAction::SetNull,
        "restrict" => ReferentialAction::Restrict,
        "no_action" => ReferentialAction::NoAction,
        "set_default" => ReferentialAction::SetDefault,
        _ => ReferentialAction::Restrict,
    }
}

/// Get description for referential action
fn ref_action_description(action: &ReferentialAction, verb: &str) -> String {
    match action {
        ReferentialAction::Cascade => format!(
            "When the referenced row is {}, all related rows will also be {}.",
            verb, verb
        ),
        ReferentialAction::SetNull => format!(
            "When the referenced row is {}, the foreign key will be set to NULL.",
            verb
        ),
        ReferentialAction::Restrict => format!("Prevents the {} if there are related rows.", verb),
        ReferentialAction::NoAction => {
            format!("Similar to RESTRICT, but checked at the end of the transaction.")
        }
        ReferentialAction::SetDefault => format!(
            "When the referenced row is {}, the foreign key will be set to its default value.",
            verb
        ),
    }
}

/// Convert PortPosition to string
fn port_position_to_string(port: &PortPosition) -> String {
    match port {
        PortPosition::Left => "left".to_string(),
        PortPosition::Right => "right".to_string(),
        PortPosition::Top => "top".to_string(),
        PortPosition::Bottom => "bottom".to_string(),
    }
}

/// Convert string to PortPosition
fn string_to_port_position(s: &str) -> PortPosition {
    match s {
        "left" => PortPosition::Left,
        "right" => PortPosition::Right,
        "top" => PortPosition::Top,
        "bottom" => PortPosition::Bottom,
        _ => PortPosition::Right,
    }
}

/// Convert first character to lowercase
fn to_lowercase_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_type_conversion() {
        assert_eq!(
            relation_type_to_string(&RelationType::OneToOne),
            "one_to_one"
        );
        assert_eq!(
            relation_type_to_string(&RelationType::OneToMany),
            "one_to_many"
        );
        assert_eq!(
            relation_type_to_string(&RelationType::ManyToOne),
            "many_to_one"
        );

        assert!(matches!(
            string_to_relation_type("one_to_one"),
            RelationType::OneToOne
        ));
        assert!(matches!(
            string_to_relation_type("one_to_many"),
            RelationType::OneToMany
        ));
        assert!(matches!(
            string_to_relation_type("many_to_one"),
            RelationType::ManyToOne
        ));
        assert!(matches!(
            string_to_relation_type("many_to_many"),
            RelationType::ManyToMany { .. }
        ));
    }

    #[test]
    fn test_ref_action_conversion() {
        assert_eq!(ref_action_to_string(&ReferentialAction::Cascade), "cascade");
        assert_eq!(
            ref_action_to_string(&ReferentialAction::SetNull),
            "set_null"
        );
        assert_eq!(
            ref_action_to_string(&ReferentialAction::Restrict),
            "restrict"
        );

        assert!(matches!(
            string_to_ref_action("cascade"),
            ReferentialAction::Cascade
        ));
        assert!(matches!(
            string_to_ref_action("set_null"),
            ReferentialAction::SetNull
        ));
        assert!(matches!(
            string_to_ref_action("restrict"),
            ReferentialAction::Restrict
        ));
    }

    #[test]
    fn test_port_position_conversion() {
        assert_eq!(port_position_to_string(&PortPosition::Left), "left");
        assert_eq!(port_position_to_string(&PortPosition::Right), "right");
        assert_eq!(port_position_to_string(&PortPosition::Top), "top");
        assert_eq!(port_position_to_string(&PortPosition::Bottom), "bottom");

        assert!(matches!(
            string_to_port_position("left"),
            PortPosition::Left
        ));
        assert!(matches!(
            string_to_port_position("right"),
            PortPosition::Right
        ));
        assert!(matches!(string_to_port_position("top"), PortPosition::Top));
        assert!(matches!(
            string_to_port_position("bottom"),
            PortPosition::Bottom
        ));
    }

    #[test]
    fn test_to_lowercase_first() {
        assert_eq!(to_lowercase_first("User"), "user");
        assert_eq!(to_lowercase_first("Post"), "post");
        assert_eq!(to_lowercase_first(""), "");
        assert_eq!(to_lowercase_first("a"), "a");
        assert_eq!(to_lowercase_first("ABC"), "aBC");
    }

    #[test]
    fn test_form_state_default() {
        let state = RelationshipFormState::default();
        assert!(state.name.is_empty());
        assert!(state.from_entity_id.is_none());
        assert!(state.to_entity_id.is_none());
        assert!(matches!(state.relation_type, RelationType::OneToMany));
        assert_eq!(state.to_field, "id");
        assert!(state.required);
    }

    #[test]
    fn test_form_state_validation() {
        let mut state = RelationshipFormState::default();

        // Should have errors when empty
        let errors = state.validate();
        assert!(!errors.is_empty());

        // Add required fields
        state.from_entity_id = Some(Uuid::new_v4());
        state.to_entity_id = Some(Uuid::new_v4());
        state.name = "TestRelationship".to_string();

        let errors = state.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_form_state_validation_same_entity() {
        let mut state = RelationshipFormState::default();
        let same_id = Uuid::new_v4();

        state.from_entity_id = Some(same_id);
        state.to_entity_id = Some(same_id);
        state.name = "SelfReference".to_string();

        let errors = state.validate();
        assert!(errors.iter().any(|e| e.contains("different")));
    }

    #[test]
    fn test_dialog_mode() {
        let create_mode = RelationshipDialogMode::Create {
            from_entity_id: None,
            to_entity_id: None,
        };
        assert!(create_mode.is_create());
        assert_eq!(create_mode.title(), "Create Relationship");
        assert_eq!(create_mode.submit_text(), "Create");

        let edit_mode = RelationshipDialogMode::Edit(Uuid::new_v4());
        assert!(!edit_mode.is_create());
        assert_eq!(edit_mode.title(), "Edit Relationship");
        assert_eq!(edit_mode.submit_text(), "Save Changes");
    }
}
