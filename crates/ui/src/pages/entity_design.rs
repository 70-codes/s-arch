//! # Entity Design Page
//!
//! The main visual editor page for designing database entities.
//!
//! This page integrates:
//! - **CanvasToolbar**: Top toolbar with canvas-specific actions
//! - **Canvas**: The main visual editor with pan/zoom and entity cards
//! - **PropertiesPanel**: Side panel for editing selected entity/field properties
//!
//! ## Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ CanvasToolbar (Add Entity, Grid Toggle, Snap Toggle)        │
//! ├─────────────────────────────────────────────────┬───────────┤
//! │                                                 │           │
//! │                                                 │ Properties│
//! │                  Canvas                         │   Panel   │
//! │          (Entity Cards, Grid, Zoom)             │           │
//! │                                                 │           │
//! └─────────────────────────────────────────────────┴───────────┘
//! ```
//!
//! ## Features
//!
//! - Create entities by double-clicking on canvas
//! - Drag entities to reposition them
//! - Select entities to edit their properties
//! - Pan with middle mouse button or space+drag
//! - Zoom with mouse wheel
//! - Toggle grid and snap-to-grid
//!

use dioxus::prelude::*;
use imortal_core::types::{EntityId, FieldId, Position, Size};
use imortal_ir::entity::Entity;

use crate::components::canvas::{Canvas, CanvasToolbar};
use crate::components::properties::PropertiesPanel;
use crate::state::{APP_STATE, Dialog};

// ============================================================================
// Entity Design Page Component
// ============================================================================

/// Main entity design page component
#[component]
pub fn EntityDesignPage() -> Element {
    // Get current state
    let state = APP_STATE.read();
    let show_grid = state.canvas.show_grid;
    let snap_to_grid = state.canvas.snap_to_grid;
    let has_project = state.has_project();
    drop(state);

    // If no project, show message (this shouldn't happen as MainContent handles it)
    if !has_project {
        return rsx! {
            div {
                class: "flex items-center justify-center h-full",
                p {
                    class: "text-slate-500",
                    "Please create or open a project first."
                }
            }
        };
    }

    // Handle toggle grid
    let handle_toggle_grid = move |_| {
        let mut state = APP_STATE.write();
        state.canvas.show_grid = !state.canvas.show_grid;
    };

    // Handle toggle snap
    let handle_toggle_snap = move |_| {
        let mut state = APP_STATE.write();
        state.canvas.snap_to_grid = !state.canvas.snap_to_grid;
    };

    // Handle add entity button click
    let handle_add_entity = move |_| {
        APP_STATE.write().ui.show_dialog(Dialog::NewEntity);
    };

    // Handle entity selection on canvas
    let handle_entity_select = move |entity_id: EntityId| {
        tracing::debug!("Entity selected: {:?}", entity_id);
        // Selection is handled by Canvas component
    };

    // Handle entity move (after drag completes)
    let handle_entity_move = move |(entity_id, position): (EntityId, Position)| {
        tracing::debug!("Entity moved: {:?} to {:?}", entity_id, position);
        // Position update is handled by Canvas, but we might want to save history here
    };

    // Handle canvas click (deselect, etc.)
    let handle_canvas_click = move |position: Position| {
        tracing::debug!("Canvas clicked at: {:?}", position);
    };

    // Handle canvas double-click (create new entity)
    let handle_canvas_double_click = move |position: Position| {
        tracing::debug!("Canvas double-clicked at: {:?}", position);
        create_entity_at_position(position);
    };

    // Handle field selection
    let handle_field_select = move |(entity_id, field_id): (EntityId, FieldId)| {
        tracing::debug!("Field selected: {:?} in entity {:?}", field_id, entity_id);
    };

    // Handle add field
    let handle_add_field = move |entity_id: EntityId| {
        tracing::debug!("Add field to entity: {:?}", entity_id);
    };

    rsx! {
        div {
            class: "entity-design-page flex flex-col h-full",

            // Canvas toolbar
            CanvasToolbar {
                show_grid: show_grid,
                snap_to_grid: snap_to_grid,
                on_toggle_grid: handle_toggle_grid,
                on_toggle_snap: handle_toggle_snap,
                on_add_entity: handle_add_entity,
            }

            // Main content area (canvas + properties)
            div {
                class: "flex flex-1 overflow-hidden",

                // Canvas area
                div {
                    class: "flex-1 relative",

                    Canvas {
                        show_grid: show_grid,
                        pan_enabled: true,
                        zoom_enabled: true,
                        drag_enabled: true,
                        show_minimap: false,
                        show_zoom_controls: true,
                        on_entity_select: handle_entity_select,
                        on_entity_move: handle_entity_move,
                        on_canvas_click: handle_canvas_click,
                        on_canvas_double_click: handle_canvas_double_click,
                        on_field_select: handle_field_select,
                        on_add_field: handle_add_field,
                    }
                }

                // Properties panel
                PropertiesPanel {
                    collapsed: false,
                    resizable: true,
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a new entity at the given canvas position
fn create_entity_at_position(position: Position) {
    let mut state = APP_STATE.write();

    // Check if we have a project
    let Some(project) = &mut state.project else {
        tracing::warn!("No project to add entity to");
        return;
    };

    // Generate a unique name
    let entity_count = project.entities.len();
    let name = format!("Entity{}", entity_count + 1);

    // Create the entity
    let mut entity = Entity::new(&name);
    entity.position = position;
    entity.size = Size::default_entity();

    // Add to project
    let entity_id = entity.id;
    project.entities.insert(entity_id, entity);

    // Select the new entity
    state.selection.select_entity(entity_id);

    // Mark as dirty
    state.is_dirty = true;

    // Save to history
    drop(state);
    APP_STATE.write().save_to_history("Create entity");

    tracing::info!(
        "Created entity '{}' at ({}, {})",
        name,
        position.x,
        position.y
    );
}

/// Delete the currently selected entities
#[allow(dead_code)]
fn delete_selected_entities() {
    let mut state = APP_STATE.write();

    let selected = state.selection.entities.clone();
    if selected.is_empty() {
        return;
    }

    // Remove entities from project
    if let Some(project) = &mut state.project {
        for entity_id in &selected {
            project.entities.remove(entity_id);
            // Future: Also remove relationships that reference this entity
        }
    }

    // Clear selection
    state.selection.clear();

    // Mark as dirty
    state.is_dirty = true;

    // Save to history
    drop(state);
    APP_STATE.write().save_to_history("Delete entities");

    tracing::info!("Deleted {} entities", selected.len());
}

/// Duplicate the currently selected entities
#[allow(dead_code)]
fn duplicate_selected_entities() {
    let mut state = APP_STATE.write();

    let selected = state.selection.entities.clone();
    if selected.is_empty() {
        return;
    }

    let Some(project) = &mut state.project else {
        return;
    };

    // Collect entities to duplicate
    let entities_to_duplicate: Vec<Entity> = selected
        .iter()
        .filter_map(|id| project.entities.get(id).cloned())
        .collect();

    // Create duplicates with offset positions
    let offset = Position::new(20.0, 20.0);
    let mut new_ids = Vec::new();

    for entity in entities_to_duplicate {
        let new_entity = entity
            .duplicate()
            .at(entity.position.x + offset.x, entity.position.y + offset.y);
        let new_id = new_entity.id;
        project.entities.insert(new_id, new_entity);
        new_ids.push(new_id);
    }

    // Select the new entities
    state.selection.clear();
    for id in &new_ids {
        state.selection.add_entity(*id);
    }

    // Mark as dirty
    state.is_dirty = true;

    // Save to history
    drop(state);
    APP_STATE.write().save_to_history("Duplicate entities");

    tracing::info!("Duplicated {} entities", new_ids.len());
}

/// Align selected entities horizontally (same Y position)
#[allow(dead_code)]
fn align_entities_horizontal() {
    let mut state = APP_STATE.write();

    let selected = state.selection.entities.clone();
    if selected.len() < 2 {
        return;
    }

    let Some(project) = &mut state.project else {
        return;
    };

    // Calculate average Y position
    let avg_y: f32 = selected
        .iter()
        .filter_map(|id| project.entities.get(id))
        .map(|e| e.position.y)
        .sum::<f32>()
        / selected.len() as f32;

    // Update positions
    for id in &selected {
        if let Some(entity) = project.entities.get_mut(id) {
            entity.position.y = avg_y;
            entity.touch();
        }
    }

    // Mark as dirty
    state.is_dirty = true;

    // Save to history
    drop(state);
    APP_STATE.write().save_to_history("Align horizontal");
}

/// Align selected entities vertically (same X position)
#[allow(dead_code)]
fn align_entities_vertical() {
    let mut state = APP_STATE.write();

    let selected = state.selection.entities.clone();
    if selected.len() < 2 {
        return;
    }

    let Some(project) = &mut state.project else {
        return;
    };

    // Calculate average X position
    let avg_x: f32 = selected
        .iter()
        .filter_map(|id| project.entities.get(id))
        .map(|e| e.position.x)
        .sum::<f32>()
        / selected.len() as f32;

    // Update positions
    for id in &selected {
        if let Some(entity) = project.entities.get_mut(id) {
            entity.position.x = avg_x;
            entity.touch();
        }
    }

    // Mark as dirty
    state.is_dirty = true;

    // Save to history
    drop(state);
    APP_STATE.write().save_to_history("Align vertical");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_design_page_exists() {
        // Just verify the module compiles
        assert!(true);
    }
}
