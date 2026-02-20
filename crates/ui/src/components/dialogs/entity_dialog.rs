//! # Entity Dialog Component
//!
//! Dialog for creating and editing entities in the Immortal Engine.
//!
//! ## Features
//!
//! - Create new entities with name, table name, description
//! - Edit existing entities
//! - Configure entity options (timestamps, soft delete, auditable)
//! - Select ID type (UUID, Serial, CUID, ULID)
//! - Validation with error messages
//!

use dioxus::prelude::*;
use imortal_core::types::{EntityId, IdType, Position, Size};
use imortal_ir::entity::{Entity, EntityConfig};

use crate::components::inputs::{Select, SelectOption, TextArea, TextInput, Toggle};
use crate::state::{APP_STATE, StatusLevel};

// ============================================================================
// Types
// ============================================================================

/// Mode for the entity dialog
#[derive(Debug, Clone, PartialEq)]
pub enum EntityDialogMode {
    /// Create a new entity
    Create,
    /// Edit an existing entity
    Edit(EntityId),
}

/// Form state for entity editing
#[derive(Debug, Clone)]
struct EntityFormState {
    name: String,
    table_name: String,
    description: String,
    timestamps: bool,
    soft_delete: bool,
    auditable: bool,
    generate_api: bool,
    id_type: IdType,
}

impl Default for EntityFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            table_name: String::new(),
            description: String::new(),
            timestamps: true,
            soft_delete: false,
            auditable: false,
            generate_api: true,
            id_type: IdType::Uuid,
        }
    }
}

impl EntityFormState {
    /// Create form state from an existing entity
    fn from_entity(entity: &Entity) -> Self {
        Self {
            name: entity.name.clone(),
            table_name: entity.table_name.clone(),
            description: entity.description.clone().unwrap_or_default(),
            timestamps: entity.config.timestamps,
            soft_delete: entity.config.soft_delete,
            auditable: entity.config.auditable,
            generate_api: entity.config.generate_api,
            id_type: entity.config.id_type.clone(),
        }
    }

    /// Validate the form and return errors if any
    fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate name
        if self.name.trim().is_empty() {
            errors.push("Entity name is required".to_string());
        } else if !is_valid_identifier(&self.name) {
            errors.push(
                "Entity name must be a valid identifier (PascalCase recommended)".to_string(),
            );
        }

        // Validate table name if provided
        if !self.table_name.is_empty() && !is_valid_table_name(&self.table_name) {
            errors.push(
                "Table name must be a valid SQL identifier (snake_case recommended)".to_string(),
            );
        }

        errors
    }

    /// Check if the form is valid
    fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

// ============================================================================
// Component Props
// ============================================================================

#[derive(Props, Clone, PartialEq)]
pub struct EntityDialogProps {
    /// Dialog mode (create or edit)
    pub mode: EntityDialogMode,

    /// Optional callback when entity is created/updated
    #[props(default)]
    pub on_save: EventHandler<EntityId>,

    /// Optional callback when dialog is cancelled
    #[props(default)]
    pub on_cancel: EventHandler<()>,
}

// ============================================================================
// Main Component
// ============================================================================

/// Entity creation and editing dialog
#[component]
pub fn EntityDialog(props: EntityDialogProps) -> Element {
    // Initialize form state based on mode
    let initial_state = match &props.mode {
        EntityDialogMode::Create => EntityFormState::default(),
        EntityDialogMode::Edit(entity_id) => {
            let state = APP_STATE.read();
            state
                .project
                .as_ref()
                .and_then(|p| p.entities.get(entity_id))
                .map(EntityFormState::from_entity)
                .unwrap_or_default()
        }
    };

    let mut form_state = use_signal(|| initial_state);
    let mut errors = use_signal(Vec::<String>::new);
    let mut is_saving = use_signal(|| false);

    // Auto-generate table name from entity name
    let auto_table_name = use_memo(move || {
        let state = form_state.read();
        if state.table_name.is_empty() {
            to_snake_case_plural(&state.name)
        } else {
            state.table_name.clone()
        }
    });
    let mode_for_unique = props.mode.clone();
    let mode_for_save = props.mode.clone();

    // Check if name is unique (for new entities)
    let name_is_unique = use_memo(move || {
        let state = form_state.read();
        let app_state = APP_STATE.read();

        if let Some(project) = &app_state.project {
            let is_editing = matches!(mode_for_unique, EntityDialogMode::Edit(_));
            let edit_id = if let EntityDialogMode::Edit(id) = &mode_for_unique {
                Some(*id)
            } else {
                None
            };

            // Check if any other entity has the same name
            !project.entities.values().any(|e| {
                e.name.to_lowercase() == state.name.to_lowercase() && Some(e.id) != edit_id
            })
        } else {
            true
        }
    });

    // Handle form submission
    let mut handle_save = move |_| {
        // Validate form
        let validation_errors = form_state.read().validate();
        if !validation_errors.is_empty() {
            errors.set(validation_errors);
            return;
        }

        // Check for unique name
        if !*name_is_unique.read() {
            errors.set(vec!["An entity with this name already exists".to_string()]);
            return;
        }

        is_saving.set(true);
        errors.set(Vec::new());

        let state = form_state.read();
        let entity_id = match &mode_for_save {
            EntityDialogMode::Create => {
                // Create new entity
                let mut entity = if state.timestamps {
                    Entity::with_timestamps(&state.name)
                } else {
                    Entity::new(&state.name)
                };

                // Set table name (use auto-generated if empty)
                entity.table_name = if state.table_name.is_empty() {
                    to_snake_case_plural(&state.name)
                } else {
                    state.table_name.clone()
                };

                // Set description
                if !state.description.is_empty() {
                    entity.description = Some(state.description.clone());
                }

                // Set config
                entity.config = EntityConfig {
                    timestamps: state.timestamps,
                    soft_delete: state.soft_delete,
                    id_type: state.id_type.clone(),
                    auditable: state.auditable,
                    generate_api: state.generate_api,
                    ..Default::default()
                };

                // Position at center of canvas (we'll adjust this later)
                entity.position = get_new_entity_position();
                entity.size = Size::default_entity();

                let id = entity.id;

                // Add to project
                let mut app_state = APP_STATE.write();
                if let Some(project) = &mut app_state.project {
                    project.entities.insert(id, entity);
                }
                app_state.is_dirty = true;
                app_state.selection.select_entity(id);
                app_state.ui.close_dialog();
                app_state.ui.set_status(
                    &format!("Created entity '{}'", state.name),
                    StatusLevel::Success,
                );
                drop(app_state);

                // Save to history
                APP_STATE.write().save_to_history("Create entity");

                id
            }
            EntityDialogMode::Edit(entity_id) => {
                // Update existing entity
                let mut app_state = APP_STATE.write();
                if let Some(project) = &mut app_state.project {
                    if let Some(entity) = project.entities.get_mut(entity_id) {
                        entity.name = state.name.clone();
                        entity.table_name = if state.table_name.is_empty() {
                            to_snake_case_plural(&state.name)
                        } else {
                            state.table_name.clone()
                        };
                        entity.description = if state.description.is_empty() {
                            None
                        } else {
                            Some(state.description.clone())
                        };
                        entity.config.timestamps = state.timestamps;
                        entity.config.soft_delete = state.soft_delete;
                        entity.config.auditable = state.auditable;
                        entity.config.generate_api = state.generate_api;
                        entity.config.id_type = state.id_type.clone();
                        entity.touch();
                    }
                }
                app_state.is_dirty = true;
                app_state.ui.close_dialog();
                app_state.ui.set_status(
                    &format!("Updated entity '{}'", state.name),
                    StatusLevel::Success,
                );
                drop(app_state);

                // Save to history
                APP_STATE.write().save_to_history("Update entity");

                *entity_id
            }
        };

        is_saving.set(false);
        props.on_save.call(entity_id);
    };

    // Handle cancel
    let handle_cancel = move |_| {
        APP_STATE.write().ui.close_dialog();
        props.on_cancel.call(());
    };

    // Form field handlers
    let on_name_change = move |value: String| {
        form_state.write().name = value;
    };

    let on_table_name_change = move |value: String| {
        form_state.write().table_name = value;
    };

    let on_description_change = move |value: String| {
        form_state.write().description = value;
    };

    let on_timestamps_change = move |checked: bool| {
        form_state.write().timestamps = checked;
    };

    let on_soft_delete_change = move |checked: bool| {
        form_state.write().soft_delete = checked;
    };

    let on_auditable_change = move |checked: bool| {
        form_state.write().auditable = checked;
    };

    let on_generate_api_change = move |checked: bool| {
        form_state.write().generate_api = checked;
    };

    let on_id_type_change = move |value: String| {
        let id_type = match value.as_str() {
            "uuid" => IdType::Uuid,
            "serial" => IdType::Serial,
            "cuid" => IdType::Cuid,
            "ulid" => IdType::Ulid,
            _ => IdType::Uuid,
        };
        form_state.write().id_type = id_type;
    };

    // Build ID type options
    let id_type_options = vec![
        SelectOption::new("uuid", "UUID"),
        SelectOption::new("serial", "Serial (Auto-increment)"),
        SelectOption::new("cuid", "CUID"),
        SelectOption::new("ulid", "ULID"),
    ];

    let current_id_type = match form_state.read().id_type {
        IdType::Uuid => "uuid",
        IdType::Serial => "serial",
        IdType::Cuid => "cuid",
        IdType::Ulid => "ulid",
    };

    // Determine dialog title
    let title = match &props.mode {
        EntityDialogMode::Create => "Create New Entity",
        EntityDialogMode::Edit(_) => "Edit Entity",
    };

    let save_button_text = match &props.mode {
        EntityDialogMode::Create => "Create Entity",
        EntityDialogMode::Edit(_) => "Save Changes",
    };

    let form = form_state.read();
    let error_list = errors.read();
    let saving = *is_saving.read();

    rsx! {
        div {
            class: "entity-dialog p-6 max-h-[80vh] overflow-y-auto",

            // Header
            div {
                class: "flex items-center gap-3 mb-6",
                span { class: "text-2xl", "🗃️" }
                h2 { class: "text-xl font-bold", "{title}" }
            }

            // Error messages
            if !error_list.is_empty() {
                div {
                    class: "mb-4 p-3 bg-red-500/20 border border-red-500/50 rounded-lg",
                    ul {
                        class: "text-red-300 text-sm list-disc list-inside",
                        for error in error_list.iter() {
                            li { "{error}" }
                        }
                    }
                }
            }

            // Form
            form {
                class: "space-y-6",
                onsubmit: move |e| {
                    e.prevent_default();
                    handle_save(());
                },

                // Basic Information Section
                div {
                    class: "space-y-4",

                    h3 {
                        class: "text-sm font-semibold text-slate-400 uppercase tracking-wider",
                        "Basic Information"
                    }

                    // Entity name
                    TextInput {
                        value: form.name.clone(),
                        label: "Entity Name",
                        placeholder: "e.g., User, BlogPost, OrderItem",
                        help_text: "Use PascalCase for entity names",
                        required: true,
                        error: if !name_is_unique.read().clone() {
                            Some("An entity with this name already exists".to_string())
                        } else {
                            None
                        },
                        on_change: on_name_change,
                    }

                    // Table name
                    TextInput {
                        value: form.table_name.clone(),
                        label: "Table Name",
                        placeholder: auto_table_name.read().clone(),
                        help_text: "Database table name (auto-generated if empty)",
                        on_change: on_table_name_change,
                    }

                    // Description
                    TextArea {
                        value: form.description.clone(),
                        label: "Description",
                        placeholder: "Describe what this entity represents...",
                        rows: 3,
                        on_change: on_description_change,
                    }
                }

                // Configuration Section
                div {
                    class: "space-y-4 pt-4 border-t border-slate-700",

                    h3 {
                        class: "text-sm font-semibold text-slate-400 uppercase tracking-wider",
                        "Configuration"
                    }

                    // ID Type
                    Select {
                        value: current_id_type.to_string(),
                        options: id_type_options,
                        label: "Primary Key Type",
                        help_text: "Type of identifier for this entity",
                        on_change: on_id_type_change,
                    }

                    // Toggle options in a grid
                    div {
                        class: "grid grid-cols-2 gap-4",

                        // Timestamps
                        Toggle {
                            checked: form.timestamps,
                            label: "Add Timestamps",
                            help_text: "created_at, updated_at fields",
                            on_change: on_timestamps_change,
                        }

                        // Soft delete
                        Toggle {
                            checked: form.soft_delete,
                            label: "Soft Delete",
                            help_text: "deleted_at field instead of hard delete",
                            on_change: on_soft_delete_change,
                        }

                        // Auditable
                        Toggle {
                            checked: form.auditable,
                            label: "Auditable",
                            help_text: "Track created_by, updated_by",
                            on_change: on_auditable_change,
                        }

                        // Generate API
                        Toggle {
                            checked: form.generate_api,
                            label: "Generate API",
                            help_text: "Generate CRUD endpoints",
                            on_change: on_generate_api_change,
                        }
                    }
                }

                // Actions
                div {
                    class: "flex justify-end gap-3 pt-6 border-t border-slate-700",

                    button {
                        r#type: "button",
                        class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg transition-colors",
                        disabled: saving,
                        onclick: handle_cancel,
                        "Cancel"
                    }

                    button {
                        r#type: "submit",
                        class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded-lg transition-colors flex items-center gap-2",
                        disabled: saving || !form.is_valid(),

                        if saving {
                            span { class: "animate-spin", "⏳" }
                            "Saving..."
                        } else {
                            span { "✓" }
                            "{save_button_text}"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;
    let mut prev_is_separator = true;

    for c in s.chars() {
        if c.is_uppercase() {
            if !prev_is_separator && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
            prev_is_upper = true;
        } else if c == '-' || c == ' ' || c == '_' {
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
            prev_is_separator = true;
            prev_is_upper = false;
        } else {
            result.push(c);
            prev_is_upper = false;
            prev_is_separator = false;
        }
    }

    result
}

/// Convert a string to snake_case and pluralize
fn to_snake_case_plural(s: &str) -> String {
    let snake = to_snake_case(s);
    if snake.is_empty() {
        return snake;
    }

    // Simple pluralization rules
    if snake.ends_with('s')
        || snake.ends_with('x')
        || snake.ends_with('z')
        || snake.ends_with("ch")
        || snake.ends_with("sh")
    {
        format!("{}es", snake)
    } else if snake.ends_with('y') {
        let mut chars: Vec<char> = snake.chars().collect();
        chars.pop();
        format!("{}ies", chars.into_iter().collect::<String>())
    } else {
        format!("{}s", snake)
    }
}

/// Check if a string is a valid identifier (for entity names)
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be alphabetic
    match chars.next() {
        Some(c) if c.is_alphabetic() => {}
        _ => return false,
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Check if a string is a valid SQL table name
fn is_valid_table_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be alphabetic or underscore
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Get a position for a new entity (center of visible canvas or offset from existing)
fn get_new_entity_position() -> Position {
    let state = APP_STATE.read();
    let pan = state.canvas.pan;
    let zoom = state.canvas.zoom;

    // Calculate center of visible area
    // Assuming a viewport of approximately 1200x800
    let viewport_center_x = 600.0;
    let viewport_center_y = 400.0;

    // Convert to canvas coordinates
    let canvas_x = (viewport_center_x - pan.x) / zoom;
    let canvas_y = (viewport_center_y - pan.y) / zoom;

    // Check if there are existing entities to avoid overlap
    if let Some(project) = &state.project {
        if !project.entities.is_empty() {
            // Find if any entity is near this position and offset if needed
            let mut offset = 0.0;
            for _ in 0..10 {
                // Try up to 10 offsets
                let test_x = canvas_x + offset;
                let test_y = canvas_y + offset;

                let overlaps = project.entities.values().any(|e| {
                    (e.position.x - test_x).abs() < 50.0 && (e.position.y - test_y).abs() < 50.0
                });

                if !overlaps {
                    return Position::new(test_x, test_y);
                }

                offset += 30.0;
            }
        }
    }

    Position::new(canvas_x, canvas_y)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("BlogPost"), "blog_post");
        assert_eq!(to_snake_case("OrderItem"), "order_item");
        assert_eq!(to_snake_case("XMLParser"), "xmlparser");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_to_snake_case_plural() {
        assert_eq!(to_snake_case_plural("User"), "users");
        assert_eq!(to_snake_case_plural("BlogPost"), "blog_posts");
        assert_eq!(to_snake_case_plural("Category"), "categories");
        assert_eq!(to_snake_case_plural("Box"), "boxes");
        assert_eq!(to_snake_case_plural("Address"), "addresses");
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("User"));
        assert!(is_valid_identifier("BlogPost"));
        assert!(is_valid_identifier("user_model"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123User"));
        assert!(!is_valid_identifier("user-model"));
    }

    #[test]
    fn test_is_valid_table_name() {
        assert!(is_valid_table_name("users"));
        assert!(is_valid_table_name("blog_posts"));
        assert!(is_valid_table_name("_internal"));
        assert!(!is_valid_table_name(""));
        assert!(!is_valid_table_name("123users"));
        assert!(!is_valid_table_name("user-posts"));
    }

    #[test]
    fn test_form_state_default() {
        let state = EntityFormState::default();
        assert!(state.name.is_empty());
        assert!(state.timestamps);
        assert!(!state.soft_delete);
        assert!(state.generate_api);
    }

    #[test]
    fn test_form_state_validation() {
        let mut state = EntityFormState::default();

        // Empty name should fail
        assert!(!state.is_valid());

        // Valid name should pass
        state.name = "User".to_string();
        assert!(state.is_valid());

        // Invalid name should fail
        state.name = "123Invalid".to_string();
        assert!(!state.is_valid());

        // Valid name with invalid table name
        state.name = "User".to_string();
        state.table_name = "123-invalid".to_string();
        assert!(!state.is_valid());
    }
}
