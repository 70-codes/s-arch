//! # Properties Panel Component
//!
//! Enhanced side panel for editing properties of selected entities, fields, and relationships.
//!
//! ## Features
//!
//! - **Dynamic content** based on selection state
//! - **Entity properties**: Name, table name, description, configuration
//! - **Field list** with inline editing, reordering, and quick actions
//! - **Field properties**: Type, constraints, validations, foreign key info
//! - **Position & Size** display for selected entities
//! - **Relationships view**: Shows connections to other entities
//! - **Bulk actions** for multi-selection (align, duplicate, delete)
//! - **Keyboard shortcuts** reference
//! - **Validation status** indicators
//! - **Collapsible sections** for organization
//!
//! ## Panel States
//!
//! 1. **No project**: Prompt to create/open project
//! 2. **No selection**: Project overview with stats and quick actions
//! 3. **Single entity**: Full entity editing with field list
//! 4. **Single field**: Detailed field editing
//! 5. **Multiple entities**: Bulk actions and alignment tools

use dioxus::prelude::*;
use imortal_core::types::DataType;
use imortal_ir::entity::Entity;
use imortal_ir::field::Field;
use uuid::Uuid;

use crate::components::inputs::{Select, SelectOption, TextArea, TextInput, Toggle};
use crate::state::{APP_STATE, DeleteTarget, Dialog, Page};

// ============================================================================
// Constants
// ============================================================================

/// Default panel width
pub const PANEL_WIDTH: &str = "320px";

/// Minimum panel width for resizing
pub const MIN_PANEL_WIDTH: &str = "280px";

/// Maximum panel width for resizing
pub const MAX_PANEL_WIDTH: &str = "480px";

// ============================================================================
// Properties Panel Component
// ============================================================================

/// Properties for the PropertiesPanel component
#[derive(Props, Clone, PartialEq)]
pub struct PropertiesPanelProps {
    /// Whether the panel is collapsed
    #[props(default = false)]
    pub collapsed: bool,

    /// Whether the panel can be resized
    #[props(default = true)]
    pub resizable: bool,

    /// Callback when panel is toggled
    #[props(default)]
    pub on_toggle: EventHandler<()>,
}

/// Main properties panel component
#[component]
pub fn PropertiesPanel(props: PropertiesPanelProps) -> Element {
    let state = APP_STATE.read();
    let collapsed = state.ui.properties_collapsed || props.collapsed;
    let has_project = state.has_project();

    // Get selection info
    let selected_entities = state.selection.entities.clone();
    let selected_field = state.selection.field;
    let entity_count = selected_entities.len();

    // Get selected entity (if single selection)
    let selected_entity: Option<Entity> = if entity_count == 1 {
        let entity_id = selected_entities.iter().next().copied();
        entity_id.and_then(|id| {
            state
                .project
                .as_ref()
                .and_then(|p| p.entities.get(&id).cloned())
        })
    } else {
        None
    };

    // Get selected field data
    let selected_field_data: Option<(Entity, Field)> = selected_field.and_then(|(eid, fid)| {
        state.project.as_ref().and_then(|p| {
            p.entities
                .get(&eid)
                .and_then(|e| e.get_field(fid).map(|f| (e.clone(), f.clone())))
        })
    });

    // Get relationships for selected entity
    let relationships: Vec<(String, String, String)> = if let Some(ref entity) = selected_entity {
        state
            .project
            .as_ref()
            .map(|p| {
                p.relationships
                    .values()
                    .filter(|r| r.from_entity_id == entity.id || r.to_entity_id == entity.id)
                    .map(|r| {
                        let other_id = if r.from_entity_id == entity.id {
                            r.to_entity_id
                        } else {
                            r.from_entity_id
                        };
                        let other_name = p
                            .entities
                            .get(&other_id)
                            .map(|e| e.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());
                        let direction = if r.from_entity_id == entity.id {
                            "→"
                        } else {
                            "←"
                        };
                        let rel_type = format!("{:?}", r.relation_type);
                        (direction.to_string(), other_name, rel_type)
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    drop(state);

    // Collapsed state
    if collapsed {
        return rsx! {
            CollapsedPanel {
                on_expand: move |_| {
                    APP_STATE.write().ui.toggle_properties();
                    props.on_toggle.call(());
                }
            }
        };
    }

    // Get panel title
    let title = if selected_field_data.is_some() {
        "Field Properties"
    } else if entity_count > 1 {
        "Multiple Selection"
    } else if entity_count == 1 {
        "Entity Properties"
    } else {
        "Properties"
    };

    rsx! {
        aside {
            class: "properties-panel flex flex-col bg-slate-800 border-l border-slate-700 shrink-0 overflow-hidden",
            style: "width: {PANEL_WIDTH}; min-width: {MIN_PANEL_WIDTH}; max-width: {MAX_PANEL_WIDTH};",

            // Panel header
            PanelHeader {
                title: title,
                on_collapse: move |_| {
                    APP_STATE.write().ui.toggle_properties();
                    props.on_toggle.call(());
                }
            }

            // Panel content
            div {
                class: "flex-1 overflow-y-auto",

                if !has_project {
                    NoProjectState {}
                } else if let Some((entity, field)) = selected_field_data {
                    FieldPropertiesPanel {
                        entity: entity,
                        field: field,
                    }
                } else if entity_count > 1 {
                    MultiSelectionPanel {
                        count: entity_count,
                        entity_ids: selected_entities.into_iter().collect(),
                    }
                } else if let Some(entity) = selected_entity {
                    EntityPropertiesPanel {
                        entity: entity,
                        relationships: relationships,
                    }
                } else {
                    ProjectOverview {}
                }
            }
        }
    }
}

// ============================================================================
// Panel Header Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct PanelHeaderProps {
    title: &'static str,
    on_collapse: EventHandler<()>,
}

#[component]
fn PanelHeader(props: PanelHeaderProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between px-4 py-3 border-b border-slate-700 bg-slate-850",

            h2 {
                class: "text-sm font-semibold text-slate-200",
                "{props.title}"
            }

            button {
                class: "w-6 h-6 flex items-center justify-center rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors",
                title: "Collapse Panel",
                onclick: move |_| props.on_collapse.call(()),
                "✕"
            }
        }
    }
}

// ============================================================================
// Collapsed Panel Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct CollapsedPanelProps {
    on_expand: EventHandler<()>,
}

#[component]
fn CollapsedPanel(props: CollapsedPanelProps) -> Element {
    rsx! {
        aside {
            class: "flex flex-col items-center py-4 bg-slate-800 border-l border-slate-700 w-12",

            button {
                class: "w-8 h-8 flex items-center justify-center rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors",
                title: "Expand Properties Panel",
                onclick: move |_| props.on_expand.call(()),
                "◀"
            }

            div {
                class: "mt-4 text-slate-500 text-xs",
                style: "writing-mode: vertical-rl; text-orientation: mixed;",
                "Properties"
            }
        }
    }
}

// ============================================================================
// No Project State
// ============================================================================

#[component]
fn NoProjectState() -> Element {
    rsx! {
        div {
            class: "p-6 text-center",

            div { class: "text-4xl mb-4 opacity-30", "📋" }

            p { class: "text-sm text-slate-500", "No project loaded" }

            p {
                class: "text-xs text-slate-600 mt-2",
                "Create or open a project to see properties here."
            }

            button {
                class: "mt-4 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm rounded-lg transition-colors",
                onclick: move |_| {
                    APP_STATE.write().ui.show_dialog(Dialog::NewProject);
                },
                "Create Project"
            }
        }
    }
}

// ============================================================================
// Project Overview Panel
// ============================================================================

#[component]
fn ProjectOverview() -> Element {
    let state = APP_STATE.read();
    let project = state.project.as_ref();

    let entity_count = project.map(|p| p.entities.len()).unwrap_or(0);
    let relationship_count = project.map(|p| p.relationships.len()).unwrap_or(0);
    let endpoint_count = project.map(|p| p.endpoints.len()).unwrap_or(0);
    let project_name = project.map(|p| p.meta.name.clone()).unwrap_or_default();
    let project_type = project
        .map(|p| format!("{:?}", p.config.project_type))
        .unwrap_or_default();
    let database = project
        .map(|p| format!("{:?}", p.config.database))
        .unwrap_or_default();

    // Count total fields
    let field_count: usize = project
        .map(|p| p.entities.values().map(|e| e.fields.len()).sum())
        .unwrap_or(0);

    drop(state);

    rsx! {
        div {
            class: "p-4 space-y-5",

            // Project info header
            div {
                class: "pb-4 border-b border-slate-700",

                h3 {
                    class: "text-lg font-semibold text-slate-200 mb-1",
                    "{project_name}"
                }

                div {
                    class: "flex items-center gap-2 text-xs text-slate-500",
                    span {
                        class: "px-2 py-0.5 bg-slate-700 rounded",
                        "{project_type}"
                    }
                    span {
                        class: "px-2 py-0.5 bg-slate-700 rounded",
                        "{database}"
                    }
                }
            }

            // Statistics
            Section {
                title: "Statistics",
                icon: "📊",
                default_open: true,

                div {
                    class: "grid grid-cols-2 gap-3",

                    StatCard { label: "Entities", value: entity_count, icon: "📦" }
                    StatCard { label: "Fields", value: field_count, icon: "📝" }
                    StatCard { label: "Relations", value: relationship_count, icon: "🔗" }
                    StatCard { label: "Endpoints", value: endpoint_count, icon: "🌐" }
                }
            }

            // Quick Actions
            Section {
                title: "Quick Actions",
                icon: "⚡",
                default_open: true,

                div {
                    class: "space-y-2",

                    QuickActionButton {
                        icon: "➕",
                        label: "Add Entity",
                        shortcut: "Double-click canvas",
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::NewEntity);
                        }
                    }

                    QuickActionButton {
                        icon: "⚙️",
                        label: "Project Settings",
                        shortcut: "",
                        onclick: move |_| {
                            APP_STATE.write().ui.navigate(Page::ProjectSetup);
                        }
                    }

                    QuickActionButton {
                        icon: "📤",
                        label: "Generate Code",
                        shortcut: "",
                        onclick: move |_| {
                            APP_STATE.write().ui.navigate(Page::CodeGeneration);
                        }
                    }
                }
            }

            // Keyboard Shortcuts
            Section {
                title: "Keyboard Shortcuts",
                icon: "⌨️",
                default_open: false,

                div {
                    class: "space-y-1 text-xs",

                    ShortcutRow { keys: "Delete", action: "Delete selected" }
                    ShortcutRow { keys: "Escape", action: "Clear selection" }
                    ShortcutRow { keys: "Ctrl+A", action: "Select all" }
                    ShortcutRow { keys: "Ctrl+D", action: "Duplicate" }
                    ShortcutRow { keys: "Ctrl+Z", action: "Undo" }
                    ShortcutRow { keys: "Ctrl+Y", action: "Redo" }
                    ShortcutRow { keys: "Arrow keys", action: "Move selected" }
                    ShortcutRow { keys: "Shift+Arrows", action: "Move faster" }
                    ShortcutRow { keys: "Space+Drag", action: "Pan canvas" }
                    ShortcutRow { keys: "Scroll", action: "Zoom" }
                }
            }
        }
    }
}

// ============================================================================
// Entity Properties Panel
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct EntityPropertiesPanelProps {
    entity: Entity,
    relationships: Vec<(String, String, String)>,
}

#[component]
fn EntityPropertiesPanel(props: EntityPropertiesPanelProps) -> Element {
    let entity = props.entity.clone();
    let entity_id = entity.id;
    let relationships = props.relationships.clone();

    // Local state
    let mut name = use_signal(|| entity.name.clone());
    let mut table_name = use_signal(|| entity.table_name.clone());
    let mut description = use_signal(|| entity.description.clone().unwrap_or_default());

    // Update handlers
    let update_name = move |v: String| {
        name.set(v.clone());
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(e) = project.entities.get_mut(&entity_id) {
                e.name = v;
                e.touch();
            }
        }
        state.mark_dirty();
    };

    let update_table_name = move |v: String| {
        table_name.set(v.clone());
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(e) = project.entities.get_mut(&entity_id) {
                e.table_name = v;
                e.touch();
            }
        }
        state.mark_dirty();
    };

    let update_description = move |v: String| {
        description.set(v.clone());
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(e) = project.entities.get_mut(&entity_id) {
                e.description = if v.is_empty() { None } else { Some(v) };
                e.touch();
            }
        }
        state.mark_dirty();
    };

    let toggle_timestamps = move |v: bool| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(e) = project.entities.get_mut(&entity_id) {
                e.config.timestamps = v;
                e.touch();
            }
        }
        state.mark_dirty();
    };

    let toggle_soft_delete = move |v: bool| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(e) = project.entities.get_mut(&entity_id) {
                e.config.soft_delete = v;
                e.touch();
            }
        }
        state.mark_dirty();
    };

    let toggle_auditable = move |v: bool| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(e) = project.entities.get_mut(&entity_id) {
                e.config.auditable = v;
                e.touch();
            }
        }
        state.mark_dirty();
    };

    rsx! {
        div {
            class: "p-4 space-y-5",

            // Basic Information
            Section {
                title: "Basic Information",
                icon: "📝",
                default_open: true,

                div {
                    class: "space-y-3",

                    TextInput {
                        label: Some("Entity Name".to_string()),
                        value: name(),
                        placeholder: Some("e.g., User, Product".to_string()),
                        help_text: Some("Name for the Rust struct".to_string()),
                        required: true,
                        on_change: update_name,
                    }

                    TextInput {
                        label: Some("Table Name".to_string()),
                        value: table_name(),
                        placeholder: Some("e.g., users, products".to_string()),
                        help_text: Some("Database table name".to_string()),
                        required: true,
                        on_change: update_table_name,
                    }

                    TextArea {
                        label: Some("Description".to_string()),
                        value: description(),
                        placeholder: Some("Describe this entity...".to_string()),
                        rows: 2,
                        on_change: update_description,
                    }
                }
            }

            // Position & Size
            Section {
                title: "Position & Size",
                icon: "📐",
                default_open: false,

                div {
                    class: "grid grid-cols-2 gap-3 text-sm",

                    InfoRow { label: "X", value: format!("{:.0}", entity.position.x) }
                    InfoRow { label: "Y", value: format!("{:.0}", entity.position.y) }
                    InfoRow { label: "Width", value: format!("{:.0}", entity.size.width) }
                    InfoRow { label: "Height", value: format!("{:.0}", entity.size.height) }
                }
            }

            // Configuration
            Section {
                title: "Configuration",
                icon: "⚙️",
                default_open: true,

                div {
                    class: "space-y-3",

                    Toggle {
                        label: Some("Timestamps".to_string()),
                        help_text: Some("Add created_at, updated_at".to_string()),
                        checked: entity.config.timestamps,
                        on_change: toggle_timestamps,
                    }

                    Toggle {
                        label: Some("Soft Delete".to_string()),
                        help_text: Some("Add deleted_at field".to_string()),
                        checked: entity.config.soft_delete,
                        on_change: toggle_soft_delete,
                    }

                    Toggle {
                        label: Some("Auditable".to_string()),
                        help_text: Some("Track created_by, updated_by".to_string()),
                        checked: entity.config.auditable,
                        on_change: toggle_auditable,
                    }
                }
            }

            // Fields List
            Section {
                title: "Fields",
                icon: "📋",
                default_open: true,
                badge: Some(entity.fields.len().to_string()),

                div {
                    class: "space-y-1",

                    // Field list
                    for field in entity.fields.iter() {
                        FieldListItem {
                            key: "{field.id}",
                            entity_id: entity_id,
                            field: field.clone(),
                        }
                    }

                    // Add field button
                    button {
                        class: "w-full py-2 mt-2 text-sm text-indigo-400 hover:text-indigo-300 hover:bg-indigo-500/10 rounded-lg transition-colors flex items-center justify-center gap-2 border border-dashed border-slate-600 hover:border-indigo-500/50",
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::NewField(entity_id));
                        },
                        span { "+" }
                        span { "Add Field" }
                    }
                }
            }

            // Relationships
            if !relationships.is_empty() {
                Section {
                    title: "Relationships",
                    icon: "🔗",
                    default_open: true,
                    badge: Some(relationships.len().to_string()),

                    div {
                        class: "space-y-2",

                        for (direction, other_name, rel_type) in relationships.iter() {
                            div {
                                class: "flex items-center gap-2 px-3 py-2 bg-slate-700/50 rounded-lg text-sm",
                                span { class: "text-slate-400", "{direction}" }
                                span { class: "font-medium text-slate-200", "{other_name}" }
                                span {
                                    class: "ml-auto text-xs px-2 py-0.5 bg-slate-600 rounded text-slate-400",
                                    "{rel_type}"
                                }
                            }
                        }
                    }
                }
            }

            // Actions
            Section {
                title: "Actions",
                icon: "🎬",
                default_open: true,

                div {
                    class: "space-y-2",

                    ActionButton {
                        icon: "✏️",
                        label: "Edit Entity",
                        variant: ActionVariant::Secondary,
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::EditEntity(entity_id));
                        }
                    }

                    ActionButton {
                        icon: "📋",
                        label: "Duplicate Entity",
                        variant: ActionVariant::Secondary,
                        onclick: move |_| {
                            duplicate_entity(entity_id);
                        }
                    }

                    ActionButton {
                        icon: "🗑️",
                        label: "Delete Entity",
                        variant: ActionVariant::Danger,
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::ConfirmDelete(DeleteTarget::Entity(entity_id)));
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Field List Item Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct FieldListItemProps {
    entity_id: Uuid,
    field: Field,
}

#[component]
fn FieldListItem(props: FieldListItemProps) -> Element {
    let field = &props.field;
    let entity_id = props.entity_id;
    let field_id = field.id;

    // Format data type display
    let type_display = format!("{:?}", field.data_type)
        .split('(')
        .next()
        .unwrap_or("?")
        .to_string();

    // Build attribute badges
    let mut badges = Vec::new();
    if field.is_primary_key {
        badges.push(("🔑", "PK", "text-amber-400"));
    }
    if field.is_foreign_key {
        badges.push(("🔗", "FK", "text-blue-400"));
    }
    if field.required && !field.is_primary_key {
        badges.push(("*", "Req", "text-rose-400"));
    }
    if field.unique && !field.is_primary_key {
        badges.push(("!", "Uniq", "text-purple-400"));
    }

    rsx! {
        div {
            class: "group flex items-center gap-2 px-2 py-1.5 rounded hover:bg-slate-700/50 cursor-pointer transition-colors",
            onclick: move |_| {
                APP_STATE.write().selection.field = Some((entity_id, field_id));
            },

            // Field name
            div {
                class: "flex-1 min-w-0",

                div {
                    class: "flex items-center gap-1",

                    // Badges
                    for (icon, _title, color) in badges.iter() {
                        span {
                            class: "text-xs {color}",
                            title: "{_title}",
                            "{icon}"
                        }
                    }

                    span {
                        class: "text-sm text-slate-200 truncate",
                        "{field.name}"
                    }
                }
            }

            // Type badge
            span {
                class: "text-xs px-1.5 py-0.5 bg-slate-600 rounded text-slate-400 shrink-0",
                "{type_display}"
            }

            // Actions (visible on hover)
            div {
                class: "hidden group-hover:flex items-center gap-1",

                button {
                    class: "p-1 hover:bg-slate-600 rounded text-slate-400 hover:text-slate-200",
                    title: "Edit Field",
                    onclick: move |e| {
                        e.stop_propagation();
                        APP_STATE.write().ui.show_dialog(Dialog::EditField(entity_id, field_id));
                    },
                    "✏️"
                }

                if !field.is_primary_key {
                    button {
                        class: "p-1 hover:bg-rose-500/20 rounded text-slate-400 hover:text-rose-400",
                        title: "Delete Field",
                        onclick: move |e| {
                            e.stop_propagation();
                            APP_STATE.write().ui.show_dialog(Dialog::ConfirmDelete(DeleteTarget::Field(entity_id, field_id)));
                        },
                        "🗑️"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Field Properties Panel
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct FieldPropertiesPanelProps {
    entity: Entity,
    field: Field,
}

#[component]
fn FieldPropertiesPanel(props: FieldPropertiesPanelProps) -> Element {
    let entity = &props.entity;
    let entity_id = entity.id;
    let field = props.field.clone();
    let field_id = field.id;

    // Local state
    let mut name = use_signal(|| field.name.clone());
    let mut column_name = use_signal(|| field.column_name.clone());
    let mut description = use_signal(|| field.description.clone().unwrap_or_default());

    // Data type options
    let data_type_options = vec![
        SelectOption::new("String", "String"),
        SelectOption::new("Text", "Text (Long)"),
        SelectOption::new("Int32", "Int32"),
        SelectOption::new("Int64", "Int64"),
        SelectOption::new("Float32", "Float32"),
        SelectOption::new("Float64", "Float64"),
        SelectOption::new("Bool", "Boolean"),
        SelectOption::new("Uuid", "UUID"),
        SelectOption::new("DateTime", "DateTime"),
        SelectOption::new("Date", "Date"),
        SelectOption::new("Time", "Time"),
        SelectOption::new("Json", "JSON"),
        SelectOption::new("Bytes", "Bytes"),
    ];

    let current_type = format!("{:?}", field.data_type)
        .split('(')
        .next()
        .unwrap_or("String")
        .to_string();

    // Update helper
    let update_field_prop = move |updater: Box<dyn FnOnce(&mut Field)>| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(e) = project.entities.get_mut(&entity_id) {
                if let Some(f) = e.get_field_mut(field_id) {
                    updater(f);
                }
                e.touch();
            }
        }
        state.mark_dirty();
    };

    rsx! {
        div {
            class: "p-4 space-y-5",

            // Back button
            button {
                class: "flex items-center gap-2 text-sm text-slate-400 hover:text-slate-200 transition-colors mb-2",
                onclick: move |_| {
                    APP_STATE.write().selection.field = None;
                },
                "← {entity.name}"
            }

            // Field header with type indicator
            div {
                class: "flex items-center gap-3 pb-4 border-b border-slate-700",

                div {
                    class: "w-10 h-10 rounded-lg bg-indigo-500/20 flex items-center justify-center text-lg",
                    if field.is_primary_key { "🔑" } else if field.is_foreign_key { "🔗" } else { "📝" }
                }

                div {
                    class: "flex-1",
                    h3 {
                        class: "text-lg font-semibold text-slate-200",
                        "{field.name}"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "{current_type} • {entity.name}"
                    }
                }
            }

            // Basic Information
            Section {
                title: "Field Information",
                icon: "📝",
                default_open: true,

                div {
                    class: "space-y-3",

                    TextInput {
                        label: Some("Field Name".to_string()),
                        value: name(),
                        placeholder: Some("e.g., email, created_at".to_string()),
                        help_text: Some("Rust struct field name".to_string()),
                        required: true,
                        disabled: field.is_primary_key,
                        on_change: move |v: String| {
                            name.set(v.clone());
                            let v2 = v.clone();
                            update_field_prop(Box::new(move |f| f.name = v2));
                        },
                    }

                    TextInput {
                        label: Some("Column Name".to_string()),
                        value: column_name(),
                        placeholder: Some("e.g., email, created_at".to_string()),
                        help_text: Some("Database column name".to_string()),
                        on_change: move |v: String| {
                            column_name.set(v.clone());
                            let v2 = v.clone();
                            update_field_prop(Box::new(move |f| f.column_name = v2));
                        },
                    }

                    Select {
                        label: Some("Data Type".to_string()),
                        value: current_type.clone(),
                        options: data_type_options,
                        on_change: move |v: String| {
                            let new_type = match v.as_str() {
                                "String" => DataType::String,
                                "Text" => DataType::Text,
                                "Int32" => DataType::Int32,
                                "Int64" => DataType::Int64,
                                "Float32" => DataType::Float32,
                                "Float64" => DataType::Float64,
                                "Bool" => DataType::Bool,
                                "Uuid" => DataType::Uuid,
                                "DateTime" => DataType::DateTime,
                                "Date" => DataType::Date,
                                "Time" => DataType::Time,
                                "Json" => DataType::Json,
                                "Bytes" => DataType::Bytes,
                                _ => DataType::String,
                            };
                            update_field_prop(Box::new(move |f| f.data_type = new_type.clone()));
                        },
                    }

                    TextArea {
                        label: Some("Description".to_string()),
                        value: description(),
                        placeholder: Some("Describe this field...".to_string()),
                        rows: 2,
                        on_change: move |v: String| {
                            description.set(v.clone());
                            let v2 = if v.is_empty() { None } else { Some(v) };
                            update_field_prop(Box::new(move |f| f.description = v2));
                        },
                    }
                }
            }

            // Constraints
            Section {
                title: "Constraints",
                icon: "🔒",
                default_open: true,

                div {
                    class: "space-y-3",

                    Toggle {
                        label: Some("Required".to_string()),
                        help_text: Some("Field cannot be null".to_string()),
                        checked: field.required,
                        disabled: field.is_primary_key,
                        on_change: move |v| {
                            update_field_prop(Box::new(move |f| f.required = v));
                        },
                    }

                    Toggle {
                        label: Some("Unique".to_string()),
                        help_text: Some("Values must be unique".to_string()),
                        checked: field.unique,
                        disabled: field.is_primary_key,
                        on_change: move |v| {
                            update_field_prop(Box::new(move |f| f.unique = v));
                        },
                    }

                    Toggle {
                        label: Some("Indexed".to_string()),
                        help_text: Some("Create database index".to_string()),
                        checked: field.indexed,
                        disabled: field.is_primary_key || field.unique,
                        on_change: move |v| {
                            update_field_prop(Box::new(move |f| f.indexed = v));
                        },
                    }
                }
            }

            // Special indicators
            if field.is_primary_key || field.is_foreign_key {
                Section {
                    title: "Special",
                    icon: "⭐",
                    default_open: true,

                    div {
                        class: "space-y-2",

                        if field.is_primary_key {
                            div {
                                class: "flex items-center gap-2 px-3 py-2 bg-amber-500/10 border border-amber-500/20 rounded-lg text-amber-300 text-sm",
                                span { "🔑" }
                                span { "Primary Key" }
                            }
                        }

                        if field.is_foreign_key {
                            div {
                                class: "px-3 py-2 bg-blue-500/10 border border-blue-500/20 rounded-lg text-sm",
                                div {
                                    class: "flex items-center gap-2 text-blue-300",
                                    span { "🔗" }
                                    span { "Foreign Key" }
                                }
                                if let Some(ref fk) = field.foreign_key_ref {
                                    div {
                                        class: "mt-1 text-xs text-blue-400",
                                        "References: {fk.entity_name}.{fk.field_name}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Actions
            Section {
                title: "Actions",
                icon: "🎬",
                default_open: true,

                div {
                    class: "space-y-2",

                    ActionButton {
                        icon: "✏️",
                        label: "Edit Field",
                        variant: ActionVariant::Secondary,
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::EditField(entity_id, field_id));
                        }
                    }

                    if !field.is_primary_key {
                        ActionButton {
                            icon: "🗑️",
                            label: "Delete Field",
                            variant: ActionVariant::Danger,
                            onclick: move |_| {
                                APP_STATE.write().ui.show_dialog(Dialog::ConfirmDelete(DeleteTarget::Field(entity_id, field_id)));
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Multi Selection Panel
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct MultiSelectionPanelProps {
    count: usize,
    entity_ids: Vec<Uuid>,
}

#[component]
fn MultiSelectionPanel(props: MultiSelectionPanelProps) -> Element {
    let entity_ids = props.entity_ids.clone();

    rsx! {
        div {
            class: "p-4 space-y-5",

            // Selection info header
            div {
                class: "text-center py-4 border-b border-slate-700",

                div { class: "text-3xl mb-2", "📦" }

                p {
                    class: "text-lg font-semibold text-slate-200",
                    "{props.count} entities selected"
                }

                p {
                    class: "text-xs text-slate-500 mt-1",
                    "Use Shift+Click to add/remove from selection"
                }
            }

            // Alignment actions
            Section {
                title: "Alignment",
                icon: "📐",
                default_open: true,

                div {
                    class: "grid grid-cols-2 gap-2",

                    ActionButton {
                        icon: "↔️",
                        label: "Align H",
                        variant: ActionVariant::Secondary,
                        onclick: move |_| {
                            align_entities_horizontal();
                        }
                    }

                    ActionButton {
                        icon: "↕️",
                        label: "Align V",
                        variant: ActionVariant::Secondary,
                        onclick: move |_| {
                            align_entities_vertical();
                        }
                    }

                    ActionButton {
                        icon: "⬅️",
                        label: "Align Left",
                        variant: ActionVariant::Secondary,
                        onclick: move |_| {
                            align_entities_left();
                        }
                    }

                    ActionButton {
                        icon: "➡️",
                        label: "Align Right",
                        variant: ActionVariant::Secondary,
                        onclick: move |_| {
                            align_entities_right();
                        }
                    }
                }
            }

            // Bulk actions
            Section {
                title: "Bulk Actions",
                icon: "⚡",
                default_open: true,

                div {
                    class: "space-y-2",

                    ActionButton {
                        icon: "📋",
                        label: "Duplicate All",
                        variant: ActionVariant::Secondary,
                        onclick: move |_| {
                            duplicate_selected_entities();
                        }
                    }

                    ActionButton {
                        icon: "🗑️",
                        label: "Delete All",
                        variant: ActionVariant::Danger,
                        onclick: {
                            let ids = entity_ids.clone();
                            move |_| {
                                APP_STATE.write().ui.show_dialog(Dialog::ConfirmDelete(DeleteTarget::Entities(ids.clone())));
                            }
                        }
                    }
                }
            }

            // Clear selection
            button {
                class: "w-full py-2 text-sm text-slate-400 hover:text-slate-200 hover:bg-slate-700/50 rounded-lg transition-colors",
                onclick: move |_| {
                    APP_STATE.write().selection.clear();
                },
                "Clear Selection (Esc)"
            }
        }
    }
}

// ============================================================================
// Section Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct SectionProps {
    title: &'static str,
    #[props(default)]
    icon: Option<&'static str>,
    #[props(default = true)]
    default_open: bool,
    #[props(default)]
    badge: Option<String>,
    children: Element,
}

#[component]
fn Section(props: SectionProps) -> Element {
    let mut collapsed = use_signal(|| !props.default_open);
    let is_collapsed = *collapsed.read();

    rsx! {
        div {
            class: "section",

            // Header
            button {
                class: "w-full flex items-center gap-2 py-2 text-xs font-semibold text-slate-400 uppercase tracking-wider hover:text-slate-300 transition-colors",
                onclick: move |_| collapsed.set(!is_collapsed),

                if let Some(icon) = props.icon {
                    span { class: "text-sm", "{icon}" }
                }

                span { class: "flex-1 text-left", "{props.title}" }

                if let Some(ref badge) = props.badge {
                    span {
                        class: "px-1.5 py-0.5 text-xs bg-slate-700 rounded text-slate-400",
                        "{badge}"
                    }
                }

                span {
                    class: "text-slate-500 text-xs",
                    if is_collapsed { "▶" } else { "▼" }
                }
            }

            // Content
            if !is_collapsed {
                div {
                    class: "pb-4",
                    {props.children}
                }
            }
        }
    }
}

// ============================================================================
// Helper Components
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct StatCardProps {
    label: &'static str,
    value: usize,
    icon: &'static str,
}

#[component]
fn StatCard(props: StatCardProps) -> Element {
    rsx! {
        div {
            class: "bg-slate-700/50 rounded-lg p-3 text-center",

            div { class: "text-lg mb-1", "{props.icon}" }
            div { class: "text-xl font-bold text-slate-200", "{props.value}" }
            div { class: "text-xs text-slate-500", "{props.label}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct QuickActionButtonProps {
    icon: &'static str,
    label: &'static str,
    #[props(default)]
    shortcut: &'static str,
    onclick: EventHandler<()>,
}

#[component]
fn QuickActionButton(props: QuickActionButtonProps) -> Element {
    rsx! {
        button {
            class: "w-full flex items-center gap-3 px-3 py-2 text-sm text-slate-300 hover:text-slate-100 hover:bg-slate-700/50 rounded-lg transition-colors",
            onclick: move |_| props.onclick.call(()),

            span { "{props.icon}" }
            span { class: "flex-1 text-left", "{props.label}" }
            if !props.shortcut.is_empty() {
                span { class: "text-xs text-slate-500", "{props.shortcut}" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ShortcutRowProps {
    keys: &'static str,
    action: &'static str,
}

#[component]
fn ShortcutRow(props: ShortcutRowProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between py-1",
            span {
                class: "px-1.5 py-0.5 bg-slate-700 rounded text-slate-300 font-mono",
                "{props.keys}"
            }
            span { class: "text-slate-500", "{props.action}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct InfoRowProps {
    label: &'static str,
    value: String,
}

#[component]
fn InfoRow(props: InfoRowProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between px-2 py-1 bg-slate-700/30 rounded",
            span { class: "text-slate-500", "{props.label}" }
            span { class: "text-slate-300 font-mono", "{props.value}" }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ActionVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
}

impl ActionVariant {
    fn classes(&self) -> &'static str {
        match self {
            ActionVariant::Primary => "bg-indigo-600 hover:bg-indigo-700 text-white",
            ActionVariant::Secondary => "bg-slate-700 hover:bg-slate-600 text-slate-200",
            ActionVariant::Danger => {
                "bg-rose-600/20 hover:bg-rose-600/30 text-rose-400 border border-rose-500/30"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ActionButtonProps {
    icon: &'static str,
    label: &'static str,
    #[props(default)]
    variant: ActionVariant,
    #[props(default = false)]
    disabled: bool,
    onclick: EventHandler<()>,
}

#[component]
fn ActionButton(props: ActionButtonProps) -> Element {
    let variant_classes = props.variant.classes();
    let disabled_class = if props.disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    rsx! {
        button {
            class: "w-full flex items-center justify-center gap-2 px-3 py-2 text-sm rounded-lg transition-colors {variant_classes} {disabled_class}",
            disabled: props.disabled,
            onclick: move |_| {
                if !props.disabled {
                    props.onclick.call(());
                }
            },

            span { "{props.icon}" }
            span { "{props.label}" }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn duplicate_entity(entity_id: Uuid) {
    let mut state = APP_STATE.write();
    if let Some(project) = &mut state.project {
        if let Some(entity) = project.entities.get(&entity_id).cloned() {
            let new_entity = entity
                .duplicate()
                .at(entity.position.x + 30.0, entity.position.y + 30.0);
            let new_id = new_entity.id;
            project.entities.insert(new_id, new_entity);
            state.selection.select_entity(new_id);
            state.is_dirty = true;
        }
    }
    drop(state);
    APP_STATE.write().save_to_history("Duplicate entity");
}

fn duplicate_selected_entities() {
    let mut state = APP_STATE.write();
    let selected: Vec<_> = state.selection.entities.iter().copied().collect();

    if let Some(project) = &mut state.project {
        let mut new_ids = Vec::new();
        for id in selected {
            if let Some(entity) = project.entities.get(&id).cloned() {
                let new_entity = entity
                    .duplicate()
                    .at(entity.position.x + 30.0, entity.position.y + 30.0);
                let new_id = new_entity.id;
                project.entities.insert(new_id, new_entity);
                new_ids.push(new_id);
            }
        }
        state.selection.clear();
        for id in new_ids {
            state.selection.add_entity(id);
        }
        state.is_dirty = true;
    }
    drop(state);
    APP_STATE.write().save_to_history("Duplicate entities");
}

fn align_entities_horizontal() {
    let mut state = APP_STATE.write();
    let selected: Vec<_> = state.selection.entities.iter().copied().collect();

    if selected.len() < 2 {
        return;
    }

    if let Some(project) = &mut state.project {
        let avg_y: f32 = selected
            .iter()
            .filter_map(|id| project.entities.get(id))
            .map(|e| e.position.y)
            .sum::<f32>()
            / selected.len() as f32;

        for id in &selected {
            if let Some(entity) = project.entities.get_mut(id) {
                entity.position.y = avg_y;
                entity.touch();
            }
        }
        state.is_dirty = true;
    }
    drop(state);
    APP_STATE.write().save_to_history("Align horizontal");
}

fn align_entities_vertical() {
    let mut state = APP_STATE.write();
    let selected: Vec<_> = state.selection.entities.iter().copied().collect();

    if selected.len() < 2 {
        return;
    }

    if let Some(project) = &mut state.project {
        let avg_x: f32 = selected
            .iter()
            .filter_map(|id| project.entities.get(id))
            .map(|e| e.position.x)
            .sum::<f32>()
            / selected.len() as f32;

        for id in &selected {
            if let Some(entity) = project.entities.get_mut(id) {
                entity.position.x = avg_x;
                entity.touch();
            }
        }
        state.is_dirty = true;
    }
    drop(state);
    APP_STATE.write().save_to_history("Align vertical");
}

fn align_entities_left() {
    let mut state = APP_STATE.write();
    let selected: Vec<_> = state.selection.entities.iter().copied().collect();

    if selected.len() < 2 {
        return;
    }

    if let Some(project) = &mut state.project {
        let min_x: f32 = selected
            .iter()
            .filter_map(|id| project.entities.get(id))
            .map(|e| e.position.x)
            .fold(f32::MAX, |a, b| a.min(b));

        for id in &selected {
            if let Some(entity) = project.entities.get_mut(id) {
                entity.position.x = min_x;
                entity.touch();
            }
        }
        state.is_dirty = true;
    }
    drop(state);
    APP_STATE.write().save_to_history("Align left");
}

fn align_entities_right() {
    let mut state = APP_STATE.write();
    let selected: Vec<_> = state.selection.entities.iter().copied().collect();

    if selected.len() < 2 {
        return;
    }

    if let Some(project) = &mut state.project {
        let max_x: f32 = selected
            .iter()
            .filter_map(|id| project.entities.get(id))
            .map(|e| e.position.x + e.size.width)
            .fold(f32::MIN, |a, b| a.max(b));

        for id in &selected {
            if let Some(entity) = project.entities.get_mut(id) {
                entity.position.x = max_x - entity.size.width;
                entity.touch();
            }
        }
        state.is_dirty = true;
    }
    drop(state);
    APP_STATE.write().save_to_history("Align right");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_variant_classes() {
        assert!(ActionVariant::Primary.classes().contains("indigo"));
        assert!(ActionVariant::Secondary.classes().contains("slate"));
        assert!(ActionVariant::Danger.classes().contains("rose"));
    }

    #[test]
    fn test_constants() {
        assert!(!PANEL_WIDTH.is_empty());
        assert!(!MIN_PANEL_WIDTH.is_empty());
        assert!(!MAX_PANEL_WIDTH.is_empty());
    }
}
