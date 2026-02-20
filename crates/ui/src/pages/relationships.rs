//! # Relationships Page
//!
//! Page for managing relationships between entities in the Immortal Engine.
//!
//! This page provides:
//! - A visual canvas showing entities and their relationships
//! - A list view of all relationships
//! - Tools for creating, editing, and deleting relationships
//! - Properties panel for selected relationship details
//!
//! ## Usage
//!
//! The relationships page is accessible from the sidebar when a project is open.
//! Users can:
//! - Click on a relationship line to select it
//! - Double-click to edit a relationship
//! - Use the toolbar to create new relationships
//! - Use keyboard shortcuts (Delete, Escape, etc.)

use dioxus::prelude::*;
use imortal_core::{ReferentialAction, RelationType};
use imortal_ir::{Entity, Relationship};
use uuid::Uuid;

use crate::components::Canvas;
use crate::components::connection::{
    ConnectionsLayer, relationship_color, relationship_type_label,
};
use crate::components::inputs::{Select, SelectOption, TextInput};
use crate::state::{APP_STATE, DeleteTarget, Dialog};

// ============================================================================
// Relationships Page Component
// ============================================================================

/// Main relationships page component
#[component]
pub fn RelationshipsPage() -> Element {
    // State for view mode
    let mut view_mode = use_signal(|| ViewMode::Canvas);
    let mut search_query = use_signal(String::new);
    let mut filter_type = use_signal(|| RelationshipFilter::All);

    // ── Auto-detect relationships from FK fields ─────────────────────────
    // Scan all entities for foreign key fields that don't have a
    // corresponding Relationship object and create one automatically.
    // This runs once when the page first mounts.
    let mut auto_detected = use_signal(|| false);

    if !*auto_detected.read() {
        auto_detected.set(true);

        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            let mut new_relationships: Vec<Relationship> = Vec::new();

            // Build a name→id lookup so we can resolve FK references by name
            // when the entity_id is nil or doesn't match any entity.
            let name_to_id: std::collections::HashMap<String, Uuid> = project
                .entities
                .values()
                .map(|e| (e.name.to_lowercase(), e.id))
                .collect();

            // Collect entity info first to avoid borrow issues
            let entity_list: Vec<(
                Uuid,
                String,
                Vec<(
                    String,
                    Uuid,
                    String,
                    String,
                    imortal_core::ReferentialAction,
                    imortal_core::ReferentialAction,
                )>,
            )> = project
                .entities
                .values()
                .map(|entity| {
                    let fk_fields: Vec<_> = entity
                        .fields
                        .iter()
                        .filter(|f| f.is_foreign_key && f.foreign_key_ref.is_some())
                        .map(|f| {
                            let fk = f.foreign_key_ref.as_ref().unwrap();
                            (
                                f.name.clone(),
                                fk.entity_id,
                                fk.entity_name.clone(),
                                fk.field_name.clone(),
                                fk.on_delete.clone(),
                                fk.on_update.clone(),
                            )
                        })
                        .collect();
                    (entity.id, entity.name.clone(), fk_fields)
                })
                .collect();

            // Check each FK field for a matching relationship
            for (entity_id, entity_name, fk_fields) in &entity_list {
                for (field_name, ref_entity_id, ref_entity_name, ref_field, on_delete, on_update) in
                    fk_fields
                {
                    // Resolve the referenced entity ID:
                    // 1. Try the stored entity_id first
                    // 2. If nil or not found, fall back to name lookup
                    let resolved_ref_id = if *ref_entity_id != Uuid::nil()
                        && project.entities.contains_key(ref_entity_id)
                    {
                        *ref_entity_id
                    } else if !ref_entity_name.is_empty() {
                        // Look up by name (case-insensitive)
                        if let Some(&id) = name_to_id.get(&ref_entity_name.to_lowercase()) {
                            tracing::debug!(
                                "FK '{}' on '{}': resolved '{}' by name (stored ID was {})",
                                field_name,
                                entity_name,
                                ref_entity_name,
                                ref_entity_id
                            );
                            id
                        } else {
                            tracing::warn!(
                                "FK '{}' on '{}': referenced entity '{}' not found by name or ID ({})",
                                field_name,
                                entity_name,
                                ref_entity_name,
                                ref_entity_id
                            );
                            continue; // Skip — can't resolve the target entity
                        }
                    } else {
                        tracing::warn!(
                            "FK '{}' on '{}': no entity name or valid ID to resolve",
                            field_name,
                            entity_name
                        );
                        continue; // Skip — no way to resolve
                    };

                    // Don't create self-referencing relationships for now
                    if resolved_ref_id == *entity_id {
                        continue;
                    }

                    // Check if a relationship already exists for this FK
                    let already_exists = project.relationships.values().any(|rel| {
                        (rel.from_entity_id == resolved_ref_id && rel.to_entity_id == *entity_id)
                            || (rel.from_entity_id == *entity_id
                                && rel.to_entity_id == resolved_ref_id)
                    });

                    if !already_exists {
                        // Get the resolved entity's name for the relationship label
                        let resolved_name = project
                            .entities
                            .get(&resolved_ref_id)
                            .map(|e| e.name.clone())
                            .unwrap_or_else(|| ref_entity_name.clone());

                        // Create a relationship: referenced entity (1) → this entity (N)
                        let mut rel =
                            Relationship::new(resolved_ref_id, *entity_id, RelationType::OneToMany);
                        rel.name = format!("{} → {}", resolved_name, entity_name);
                        rel.from_field = ref_field.clone();
                        rel.to_field = field_name.clone();
                        rel.on_delete = on_delete.clone();
                        rel.on_update = on_update.clone();
                        rel.description = Some(format!(
                            "Auto-detected from FK field '{}' on '{}'",
                            field_name, entity_name
                        ));

                        tracing::info!(
                            "Auto-detected relationship: {} --[1:N]--> {} (via {}.{})",
                            resolved_name,
                            entity_name,
                            entity_name,
                            field_name
                        );

                        new_relationships.push(rel);
                    }
                }
            }

            // Add the new relationships to the project
            let count = new_relationships.len();
            for rel in new_relationships {
                project.add_relationship(rel);
            }

            if count > 0 {
                state.is_dirty = true;
                tracing::info!("Auto-detected {} relationship(s) from FK fields", count);
            }
        }
        drop(state);
    }

    // Get data from state
    let state = APP_STATE.read();
    let has_project = state.project.is_some();

    let entities: Vec<Entity> = state
        .project
        .as_ref()
        .map(|p| p.entities.values().cloned().collect())
        .unwrap_or_default();

    let relationships: Vec<Relationship> = state
        .project
        .as_ref()
        .map(|p| p.relationships.values().cloned().collect())
        .unwrap_or_default();

    let selected_relationships = state.selection.relationships.clone();
    let _selected_entities = state.selection.entities.clone();
    drop(state);
    let selected_rels_for_delete = selected_relationships.clone();
    let selected_rels_for_canvas = selected_relationships.clone();
    let selected_rels_for_list = selected_relationships.clone();
    let selected_rels_for_panel = selected_relationships.clone();

    // Filter relationships based on search and filter
    let filtered_relationships: Vec<&Relationship> = relationships
        .iter()
        .filter(|r| {
            // Apply type filter
            let type_match = match *filter_type.read() {
                RelationshipFilter::All => true,
                RelationshipFilter::OneToOne => matches!(r.relation_type, RelationType::OneToOne),
                RelationshipFilter::OneToMany => matches!(r.relation_type, RelationType::OneToMany),
                RelationshipFilter::ManyToOne => matches!(r.relation_type, RelationType::ManyToOne),
                RelationshipFilter::ManyToMany => {
                    matches!(r.relation_type, RelationType::ManyToMany { .. })
                }
            };

            // Apply search filter
            let search = search_query.read().to_lowercase();
            let search_match = search.is_empty()
                || r.name.to_lowercase().contains(&search)
                || r.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&search))
                    .unwrap_or(false);

            type_match && search_match
        })
        .collect();

    // Handle relationship selection
    let on_relationship_click = move |rel_id: Uuid| {
        let mut state = APP_STATE.write();
        state.selection.clear();
        state.selection.relationships.insert(rel_id);
    };

    // Handle relationship double-click (edit)
    let on_relationship_double_click = move |rel_id: Uuid| {
        APP_STATE
            .write()
            .ui
            .show_dialog(Dialog::EditRelationship(rel_id));
    };

    // Handle create new relationship
    let on_create_relationship = move |_| {
        // Pre-select first entity if one is selected
        let state = APP_STATE.read();
        let from_entity = state.selection.entities.iter().next().copied();
        drop(state);

        APP_STATE
            .write()
            .ui
            .show_dialog(Dialog::NewRelationship(from_entity, None));
    };

    // Handle delete relationship
    let on_delete_relationship = move |rel_id: Uuid| {
        APP_STATE
            .write()
            .ui
            .show_dialog(Dialog::ConfirmDelete(DeleteTarget::Relationship(rel_id)));
    };

    // Filter options
    let filter_options = vec![
        SelectOption {
            value: "all".to_string(),
            label: "All Types".to_string(),
            disabled: false,
        },
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

    if !has_project {
        return rsx! {
            NoProjectState {}
        };
    }

    rsx! {
        div {
            class: "relationships-page flex flex-col h-full",

            // Toolbar
            RelationshipsToolbar {
                view_mode: *view_mode.read(),
                relationship_count: relationships.len(),
                selected_count: selected_relationships.len(),
                on_view_mode_change: move |mode| view_mode.set(mode),
                on_create: on_create_relationship,
                on_delete: move |_| {
                    if let Some(rel_id) = selected_rels_for_delete.iter().next() {
                        on_delete_relationship(*rel_id);
                    }
                },
            }

            // Main content area
            div {
                class: "flex-1 flex overflow-hidden",

                // Left panel (list or canvas)
                div {
                    class: "flex-1 flex flex-col overflow-hidden",

                    // Search and filter bar
                    div {
                        class: "p-3 border-b border-slate-700 flex gap-3",

                        // Search input
                        div {
                            class: "flex-1",
                            TextInput {
                                value: search_query.read().clone(),
                                placeholder: "Search relationships...",
                                on_change: move |v: String| search_query.set(v),
                            }
                        }

                        // Type filter
                        div {
                            class: "w-48",
                            Select {
                                value: filter_to_string(&filter_type.read()),
                                options: filter_options,
                                on_change: move |v: String| {
                                    filter_type.set(string_to_filter(&v));
                                },
                            }
                        }
                    }

                    // Content based on view mode
                    div {
                        class: "flex-1 overflow-hidden",

                        match *view_mode.read() {
                            ViewMode::Canvas => rsx! {
                                RelationshipsCanvas {
                                    relationships: relationships.clone(),
                                    selected_relationships: selected_rels_for_canvas.iter().copied().collect(),
                                    on_click: on_relationship_click,
                                    on_double_click: on_relationship_double_click,
                                }
                            },
                            ViewMode::List => rsx! {
                                RelationshipsList {
                                    relationships: filtered_relationships.iter().map(|r| (*r).clone()).collect(),
                                    entities: entities.clone(),
                                    selected_relationships: selected_rels_for_list.iter().copied().collect(),
                                    on_select: on_relationship_click,
                                    on_edit: on_relationship_double_click,
                                    on_delete: on_delete_relationship,
                                }
                            },
                        }
                    }
                }

                // Right panel (properties)
                div {
                    class: "w-80 border-l border-slate-700 overflow-y-auto bg-slate-800/50",

                    RelationshipPropertiesPanel {
                        selected_relationship: selected_rels_for_panel.iter().next().copied(),
                        entities: entities.clone(),
                    }
                }
            }

            // Status bar
            {
                let selected_count = selected_relationships.len();
                rsx! {
                    RelationshipsStatusBar {
                        total: relationships.len(),
                        filtered: filtered_relationships.len(),
                        selected: selected_count,
                    }
                }
            }
        }
    }
}

// ============================================================================
// View Mode
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Canvas,
    List,
}

// ============================================================================
// Filter
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RelationshipFilter {
    #[default]
    All,
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

fn filter_to_string(filter: &RelationshipFilter) -> String {
    match filter {
        RelationshipFilter::All => "all".to_string(),
        RelationshipFilter::OneToOne => "one_to_one".to_string(),
        RelationshipFilter::OneToMany => "one_to_many".to_string(),
        RelationshipFilter::ManyToOne => "many_to_one".to_string(),
        RelationshipFilter::ManyToMany => "many_to_many".to_string(),
    }
}

fn string_to_filter(s: &str) -> RelationshipFilter {
    match s {
        "one_to_one" => RelationshipFilter::OneToOne,
        "one_to_many" => RelationshipFilter::OneToMany,
        "many_to_one" => RelationshipFilter::ManyToOne,
        "many_to_many" => RelationshipFilter::ManyToMany,
        _ => RelationshipFilter::All,
    }
}

// ============================================================================
// Toolbar Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct RelationshipsToolbarProps {
    view_mode: ViewMode,
    relationship_count: usize,
    selected_count: usize,
    on_view_mode_change: EventHandler<ViewMode>,
    on_create: EventHandler<()>,
    on_delete: EventHandler<()>,
}

#[component]
fn RelationshipsToolbar(props: RelationshipsToolbarProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between px-4 py-2 bg-slate-800 border-b border-slate-700",

            // Left side - title and count
            div {
                class: "flex items-center gap-4",

                h2 {
                    class: "text-lg font-semibold text-white flex items-center gap-2",
                    // Link icon
                    svg {
                        class: "w-5 h-5 text-indigo-400",
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
                    "Relationships"
                }

                span {
                    class: "px-2 py-0.5 bg-slate-700 rounded text-sm text-slate-300",
                    "{props.relationship_count} total"
                }

                if props.selected_count > 0 {
                    span {
                        class: "px-2 py-0.5 bg-indigo-600 rounded text-sm text-white",
                        "{props.selected_count} selected"
                    }
                }
            }

            // Center - view mode toggle
            div {
                class: "flex items-center gap-1 bg-slate-700 rounded-lg p-1",

                // Canvas view
                button {
                    class: format!(
                        "px-3 py-1 rounded text-sm transition-colors {}",
                        if props.view_mode == ViewMode::Canvas {
                            "bg-indigo-600 text-white"
                        } else {
                            "text-slate-300 hover:text-white"
                        }
                    ),
                    onclick: move |_| props.on_view_mode_change.call(ViewMode::Canvas),
                    "Canvas"
                }

                // List view
                button {
                    class: format!(
                        "px-3 py-1 rounded text-sm transition-colors {}",
                        if props.view_mode == ViewMode::List {
                            "bg-indigo-600 text-white"
                        } else {
                            "text-slate-300 hover:text-white"
                        }
                    ),
                    onclick: move |_| props.on_view_mode_change.call(ViewMode::List),
                    "List"
                }
            }

            // Right side - actions
            div {
                class: "flex items-center gap-2",

                // Delete button (if selected)
                if props.selected_count > 0 {
                    button {
                        class: "px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white text-sm rounded transition-colors flex items-center gap-1",
                        onclick: move |_| props.on_delete.call(()),
                        // Trash icon
                        svg {
                            class: "w-4 h-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                            }
                        }
                        "Delete"
                    }
                }

                // Create button
                button {
                    class: "px-3 py-1.5 bg-indigo-600 hover:bg-indigo-700 text-white text-sm rounded transition-colors flex items-center gap-1",
                    onclick: move |_| props.on_create.call(()),
                    // Plus icon
                    svg {
                        class: "w-4 h-4",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 4v16m8-8H4",
                        }
                    }
                    "New Relationship"
                }
            }
        }
    }
}

// ============================================================================
// Canvas View Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct RelationshipsCanvasProps {
    relationships: Vec<Relationship>,
    selected_relationships: Vec<Uuid>,
    on_click: EventHandler<Uuid>,
    on_double_click: EventHandler<Uuid>,
}

#[component]
fn RelationshipsCanvas(props: RelationshipsCanvasProps) -> Element {
    rsx! {
        div {
            class: "relative w-full h-full bg-slate-900 overflow-hidden",

            // Canvas with entities and connections
            Canvas {
                show_grid: true,
                pan_enabled: true,
                zoom_enabled: true,
                drag_enabled: true,
                show_zoom_controls: true,
                on_entity_select: move |_| {},
                on_entity_move: move |_| {},
                on_canvas_click: move |_| {
                    // Clear relationship selection when clicking canvas
                    APP_STATE.write().selection.relationships.clear();
                },
                on_canvas_double_click: move |_| {},
            }

            // Connections layer overlay — rendered above the canvas
            // Re-reads pan/zoom from global state on every render so it
            // stays in sync with the Canvas component's transforms.
            {
                let state = APP_STATE.read();
                let pan = state.canvas.pan;
                let zoom = state.canvas.zoom;
                drop(state);
                rsx! {
                    div {
                        class: "absolute inset-0",
                        style: "z-index: 20; pointer-events: none;",

                        ConnectionsLayer {
                            zoom: zoom,
                            pan_x: pan.x,
                            pan_y: pan.y,
                            on_connection_click: move |id| props.on_click.call(id),
                            on_connection_double_click: move |id| props.on_double_click.call(id),
                        }
                    }
                }
            }

            // Help overlay
            if props.relationships.is_empty() {
                div {
                    class: "absolute inset-0 flex items-center justify-center pointer-events-none",
                    div {
                        class: "text-center p-8 bg-slate-800/90 rounded-xl border border-slate-700 max-w-md",
                        // Icon
                        div {
                            class: "w-16 h-16 mx-auto mb-4 rounded-full bg-indigo-900/50 flex items-center justify-center",
                            svg {
                                class: "w-8 h-8 text-indigo-400",
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
                        }
                        h3 {
                            class: "text-xl font-semibold text-white mb-2",
                            "No Relationships Yet"
                        }
                        p {
                            class: "text-slate-400 mb-4",
                            "Create relationships to connect your entities. "
                            "Relationships define how data is linked between tables."
                        }
                        p {
                            class: "text-sm text-slate-500",
                            "Click the \"New Relationship\" button or drag between entity ports on the Entity Design page."
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// List View Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct RelationshipsListProps {
    relationships: Vec<Relationship>,
    entities: Vec<Entity>,
    selected_relationships: Vec<Uuid>,
    on_select: EventHandler<Uuid>,
    on_edit: EventHandler<Uuid>,
    on_delete: EventHandler<Uuid>,
}

#[component]
fn RelationshipsList(props: RelationshipsListProps) -> Element {
    if props.relationships.is_empty() {
        return rsx! {
            div {
                class: "flex-1 flex items-center justify-center p-8",
                div {
                    class: "text-center",
                    p {
                        class: "text-slate-400",
                        "No relationships match your filter"
                    }
                }
            }
        };
    }

    // Helper function to get entity name by ID
    let get_entity_name = |entity_id: Uuid| -> String {
        props
            .entities
            .iter()
            .find(|e| e.id == entity_id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    };

    rsx! {
        div {
            class: "flex-1 overflow-y-auto",

            // Table header
            div {
                class: "sticky top-0 bg-slate-800 border-b border-slate-700 grid grid-cols-6 gap-4 px-4 py-2 text-sm font-medium text-slate-400",
                span { "Name" }
                span { "From" }
                span { "To" }
                span { "Type" }
                span { "Required" }
                span { "Actions" }
            }

            // Table body
            div {
                class: "divide-y divide-slate-700/50",

                for relationship in props.relationships.iter() {
                    {
                        let rel_id = relationship.id;
                        let is_selected = props.selected_relationships.contains(&rel_id);
                        let from_name = get_entity_name(relationship.from_entity_id);
                        let to_name = get_entity_name(relationship.to_entity_id);
                        let type_label = relationship_type_label(&relationship.relation_type);
                        let type_color = relationship_color(&relationship.relation_type);

                        rsx! {
                            div {
                                key: "{rel_id}",
                                class: format!(
                                    "grid grid-cols-6 gap-4 px-4 py-3 cursor-pointer transition-colors {}",
                                    if is_selected {
                                        "bg-indigo-900/30 border-l-2 border-indigo-500"
                                    } else {
                                        "hover:bg-slate-700/30 border-l-2 border-transparent"
                                    }
                                ),
                                onclick: move |_| props.on_select.call(rel_id),
                                ondoubleclick: move |_| props.on_edit.call(rel_id),

                                // Name
                                span {
                                    class: "font-medium text-white truncate",
                                    "{relationship.name}"
                                }

                                // From entity
                                span {
                                    class: "text-slate-300 truncate flex items-center gap-1",
                                    span {
                                        class: "w-2 h-2 rounded-full bg-blue-500"
                                    }
                                    "{from_name}"
                                }

                                // To entity
                                span {
                                    class: "text-slate-300 truncate flex items-center gap-1",
                                    span {
                                        class: "w-2 h-2 rounded-full bg-green-500"
                                    }
                                    "{to_name}"
                                }

                                // Type badge
                                span {
                                    span {
                                        class: "px-2 py-0.5 rounded text-xs font-medium",
                                        style: "background-color: {type_color}20; color: {type_color};",
                                        "{type_label}"
                                    }
                                }

                                // Required
                                span {
                                    class: "text-sm",
                                    if relationship.required {
                                        span {
                                            class: "text-amber-400",
                                            "Required"
                                        }
                                    } else {
                                        span {
                                            class: "text-slate-500",
                                            "Optional"
                                        }
                                    }
                                }

                                // Actions
                                span {
                                    class: "flex items-center gap-2",

                                    // Edit button
                                    button {
                                        class: "p-1 text-slate-400 hover:text-white transition-colors",
                                        title: "Edit relationship",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            props.on_edit.call(rel_id);
                                        },
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z",
                                            }
                                        }
                                    }

                                    // Delete button
                                    button {
                                        class: "p-1 text-slate-400 hover:text-red-400 transition-colors",
                                        title: "Delete relationship",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            props.on_delete.call(rel_id);
                                        },
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Properties Panel Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct RelationshipPropertiesPanelProps {
    selected_relationship: Option<Uuid>,
    entities: Vec<Entity>,
}

#[component]
fn RelationshipPropertiesPanel(props: RelationshipPropertiesPanelProps) -> Element {
    let Some(rel_id) = props.selected_relationship else {
        return rsx! {
            div {
                class: "p-4",
                // Empty state
                div {
                    class: "text-center py-12",
                    div {
                        class: "w-12 h-12 mx-auto mb-4 rounded-full bg-slate-700 flex items-center justify-center",
                        svg {
                            class: "w-6 h-6 text-slate-500",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                            }
                        }
                    }
                    h3 {
                        class: "text-sm font-medium text-slate-300 mb-1",
                        "No Selection"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Select a relationship to view its properties"
                    }
                }
            }
        };
    };

    // Get relationship from state
    let state = APP_STATE.read();
    let relationship = state
        .project
        .as_ref()
        .and_then(|p| p.relationships.get(&rel_id))
        .cloned();
    drop(state);

    let Some(rel) = relationship else {
        return rsx! {
            div {
                class: "p-4 text-slate-400",
                "Relationship not found"
            }
        };
    };

    // Helper to get entity name
    let get_entity_name = |entity_id: Uuid| -> String {
        props
            .entities
            .iter()
            .find(|e| e.id == entity_id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    };

    let from_name = get_entity_name(rel.from_entity_id);
    let to_name = get_entity_name(rel.to_entity_id);
    let type_label = relationship_type_label(&rel.relation_type);
    let type_color = relationship_color(&rel.relation_type);

    rsx! {
        div {
            class: "p-4 space-y-4",

            // Header
            div {
                class: "pb-3 border-b border-slate-700",
                h3 {
                    class: "font-semibold text-white",
                    "{rel.name}"
                }
                p {
                    class: "text-xs text-slate-500",
                    "ID: {rel.id}"
                }
            }

            // Type badge
            div {
                class: "flex items-center gap-2",
                span {
                    class: "text-sm text-slate-400",
                    "Type:"
                }
                span {
                    class: "px-2 py-1 rounded text-sm font-medium",
                    style: "background-color: {type_color}20; color: {type_color};",
                    "{type_label}"
                }
            }

            // Entities involved
            div {
                class: "space-y-2",

                // From entity
                div {
                    class: "p-3 bg-slate-700/50 rounded-lg",
                    div {
                        class: "text-xs text-slate-500 mb-1",
                        "From Entity"
                    }
                    div {
                        class: "flex items-center gap-2",
                        span {
                            class: "w-3 h-3 rounded-full bg-blue-500"
                        }
                        span {
                            class: "font-medium text-white",
                            "{from_name}"
                        }
                    }
                    if !rel.from_field.is_empty() {
                        div {
                            class: "text-xs text-slate-400 mt-1",
                            "Field: {rel.from_field}"
                        }
                    }
                }

                // To entity
                div {
                    class: "p-3 bg-slate-700/50 rounded-lg",
                    div {
                        class: "text-xs text-slate-500 mb-1",
                        "To Entity"
                    }
                    div {
                        class: "flex items-center gap-2",
                        span {
                            class: "w-3 h-3 rounded-full bg-green-500"
                        }
                        span {
                            class: "font-medium text-white",
                            "{to_name}"
                        }
                    }
                    if !rel.to_field.is_empty() {
                        div {
                            class: "text-xs text-slate-400 mt-1",
                            "Field: {rel.to_field}"
                        }
                    }
                }
            }

            // Referential actions
            div {
                class: "space-y-2",
                h4 {
                    class: "text-sm font-medium text-slate-300",
                    "Referential Actions"
                }
                div {
                    class: "grid grid-cols-2 gap-2 text-sm",
                    div {
                        class: "p-2 bg-slate-700/50 rounded",
                        div {
                            class: "text-xs text-slate-500",
                            "On Delete"
                        }
                        div {
                            class: "text-amber-400 font-mono",
                            "{rel.on_delete}"
                        }
                    }
                    div {
                        class: "p-2 bg-slate-700/50 rounded",
                        div {
                            class: "text-xs text-slate-500",
                            "On Update"
                        }
                        div {
                            class: "text-blue-400 font-mono",
                            "{rel.on_update}"
                        }
                    }
                }
            }

            // Required/Optional
            div {
                class: "flex items-center gap-2",
                if rel.required {
                    span {
                        class: "px-2 py-1 bg-amber-900/30 text-amber-400 rounded text-sm",
                        "Required (NOT NULL)"
                    }
                } else {
                    span {
                        class: "px-2 py-1 bg-slate-700 text-slate-400 rounded text-sm",
                        "Optional (NULLABLE)"
                    }
                }
            }

            // Inverse name
            if let Some(inverse) = &rel.inverse_name {
                div {
                    class: "p-3 bg-slate-700/50 rounded-lg",
                    div {
                        class: "text-xs text-slate-500 mb-1",
                        "Inverse Relationship Name"
                    }
                    div {
                        class: "font-mono text-indigo-400",
                        "{inverse}"
                    }
                }
            }

            // Description
            if let Some(desc) = &rel.description {
                div {
                    class: "p-3 bg-slate-700/50 rounded-lg",
                    div {
                        class: "text-xs text-slate-500 mb-1",
                        "Description"
                    }
                    div {
                        class: "text-sm text-slate-300",
                        "{desc}"
                    }
                }
            }

            // Timestamps
            {
                let created = rel.created_at.format("%Y-%m-%d %H:%M").to_string();
                let modified = rel.modified_at.format("%Y-%m-%d %H:%M").to_string();
                rsx! {
                    div {
                        class: "pt-3 border-t border-slate-700 space-y-1 text-xs text-slate-500",
                        div {
                            "Created: {created}"
                        }
                        div {
                            "Modified: {modified}"
                        }
                    }
                }
            }

            // Edit button
            div {
                class: "pt-3",
                button {
                    class: "w-full px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors",
                    onclick: move |_| {
                        APP_STATE.write().ui.show_dialog(Dialog::EditRelationship(rel_id));
                    },
                    "Edit Relationship"
                }
            }
        }
    }
}

// ============================================================================
// No Project State Component
// ============================================================================

#[component]
fn NoProjectState() -> Element {
    rsx! {
        div {
            class: "flex-1 flex items-center justify-center p-8",
            div {
                class: "text-center max-w-md",
                // Icon
                div {
                    class: "w-20 h-20 mx-auto mb-6 rounded-full bg-slate-800 flex items-center justify-center",
                    svg {
                        class: "w-10 h-10 text-slate-600",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z",
                        }
                    }
                }
                h2 {
                    class: "text-2xl font-bold text-white mb-2",
                    "No Project Open"
                }
                p {
                    class: "text-slate-400 mb-6",
                    "Open or create a project to start managing relationships between entities."
                }
                div {
                    class: "flex justify-center gap-3",
                    button {
                        class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors",
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::NewProject);
                        },
                        "New Project"
                    }
                    button {
                        class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors",
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::OpenProject);
                        },
                        "Open Project"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Status Bar Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct RelationshipsStatusBarProps {
    total: usize,
    filtered: usize,
    selected: usize,
}

#[component]
fn RelationshipsStatusBar(props: RelationshipsStatusBarProps) -> Element {
    rsx! {
        div {
            class: "px-4 py-2 bg-slate-800 border-t border-slate-700 flex items-center justify-between text-sm",

            // Left side - counts
            div {
                class: "flex items-center gap-4 text-slate-400",

                span {
                    "{props.total} relationship(s)"
                }

                if props.filtered != props.total {
                    span {
                        class: "text-indigo-400",
                        "({props.filtered} shown)"
                    }
                }

                if props.selected > 0 {
                    span {
                        class: "text-green-400",
                        "{props.selected} selected"
                    }
                }
            }

            // Right side - hints
            div {
                class: "flex items-center gap-4 text-slate-500",

                span {
                    "Double-click to edit"
                }
                span {
                    "•"
                }
                span {
                    "Del to delete"
                }
            }
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
    fn test_filter_conversion() {
        assert_eq!(filter_to_string(&RelationshipFilter::All), "all");
        assert_eq!(
            filter_to_string(&RelationshipFilter::OneToOne),
            "one_to_one"
        );
        assert_eq!(
            filter_to_string(&RelationshipFilter::OneToMany),
            "one_to_many"
        );
        assert_eq!(
            filter_to_string(&RelationshipFilter::ManyToOne),
            "many_to_one"
        );
        assert_eq!(
            filter_to_string(&RelationshipFilter::ManyToMany),
            "many_to_many"
        );

        assert_eq!(string_to_filter("all"), RelationshipFilter::All);
        assert_eq!(string_to_filter("one_to_one"), RelationshipFilter::OneToOne);
        assert_eq!(
            string_to_filter("one_to_many"),
            RelationshipFilter::OneToMany
        );
        assert_eq!(
            string_to_filter("many_to_one"),
            RelationshipFilter::ManyToOne
        );
        assert_eq!(
            string_to_filter("many_to_many"),
            RelationshipFilter::ManyToMany
        );
        assert_eq!(string_to_filter("invalid"), RelationshipFilter::All);
    }

    #[test]
    fn test_view_mode_equality() {
        assert_eq!(ViewMode::Canvas, ViewMode::Canvas);
        assert_eq!(ViewMode::List, ViewMode::List);
        assert_ne!(ViewMode::Canvas, ViewMode::List);
    }
}
