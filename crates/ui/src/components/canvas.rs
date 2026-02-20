//! # Canvas Component
//!
//! The main visual editor canvas for designing entities and relationships.
//!
//! ## Features
//!
//! - **Pan**: Middle mouse button or space + drag to pan the canvas
//! - **Zoom**: Mouse wheel to zoom in/out, centered on cursor position
//! - **Grid**: Optional grid background with configurable size
//! - **Entity Rendering**: Displays entity cards at their positions
//! - **Selection**: Click to select entities, shift+click for multi-select
//! - **Drag & Drop**: Drag entities to reposition them

use dioxus::prelude::*;
use imortal_core::types::{EntityId, FieldId, Position};
use imortal_ir::entity::Entity;

use crate::components::connection::{ConnectionPoint, ConnectionsLayer};
use crate::components::entity_card::EntityCard;
use crate::components::port::{PortClickInfo, PortType};
use crate::hooks::use_canvas::{
    position_from_mouse_event, use_canvas_interactions, zoom_delta_from_wheel,
};
use crate::hooks::use_connection::use_connection_drawing;
use crate::state::{APP_STATE, Dialog};

// ============================================================================
// Constants
// ============================================================================

/// Default grid size in pixels
pub const DEFAULT_GRID_SIZE: f32 = 20.0;

/// Grid line color (light)
pub const GRID_LINE_COLOR: &str = "rgba(71, 85, 105, 0.3)";

/// Grid line color (major lines every 5 cells)
pub const GRID_MAJOR_COLOR: &str = "rgba(71, 85, 105, 0.5)";

/// Canvas background color
pub const CANVAS_BG_COLOR: &str = "#0f172a";

/// Minimum entities before showing minimap
pub const MINIMAP_THRESHOLD: usize = 5;

/// Arrow key movement step (in canvas pixels)
pub const ARROW_MOVE_STEP: f32 = 10.0;

/// Arrow key movement step with shift (larger step)
pub const ARROW_MOVE_STEP_LARGE: f32 = 50.0;

// ============================================================================
// Canvas Component
// ============================================================================

/// Properties for the Canvas component
#[derive(Props, Clone, PartialEq)]
pub struct CanvasProps {
    /// Whether to show the grid
    #[props(default = true)]
    pub show_grid: bool,

    /// Whether to enable panning
    #[props(default = true)]
    pub pan_enabled: bool,

    /// Whether to enable zooming
    #[props(default = true)]
    pub zoom_enabled: bool,

    /// Whether to enable entity dragging
    #[props(default = true)]
    pub drag_enabled: bool,

    /// Whether to show the minimap
    #[props(default = false)]
    pub show_minimap: bool,

    /// Whether to show zoom controls
    #[props(default = true)]
    pub show_zoom_controls: bool,

    /// Callback when an entity is selected
    #[props(default)]
    pub on_entity_select: EventHandler<EntityId>,

    /// Callback when an entity is moved (after drag)
    #[props(default)]
    pub on_entity_move: EventHandler<(EntityId, Position)>,

    /// Callback when canvas background is clicked
    #[props(default)]
    pub on_canvas_click: EventHandler<Position>,

    /// Callback when canvas is double-clicked (for creating new entity)
    #[props(default)]
    pub on_canvas_double_click: EventHandler<Position>,

    /// Callback when a field is selected
    #[props(default)]
    pub on_field_select: EventHandler<(EntityId, FieldId)>,

    /// Callback when add field is clicked
    #[props(default)]
    pub on_add_field: EventHandler<EntityId>,
}

/// Main canvas component for visual entity editing
#[component]
pub fn Canvas(props: CanvasProps) -> Element {
    // Get canvas interactions hook
    let interactions = use_canvas_interactions();

    // Get connection drawing hook
    let connection = use_connection_drawing();

    // Track if space is held (for pan mode)
    let mut space_held = use_signal(|| false);

    // Get current state
    let state = APP_STATE.read();
    let entities: Vec<Entity> = state
        .project
        .as_ref()
        .map(|p| p.entities.values().cloned().collect())
        .unwrap_or_default();
    let pan = state.canvas.pan;
    let zoom = state.canvas.zoom;
    let show_grid = state.canvas.show_grid && props.show_grid;
    let grid_size = state.canvas.grid_size;
    let selected_entities = state.selection.entities.clone();
    let selected_field = state.selection.field;
    let dragging_entity = state.canvas.dragging_entity;
    let is_connecting = state.canvas.is_connecting;
    let connection_start = state.canvas.connection_start.as_ref().map(|(id, _)| *id);
    let relationships = state
        .project
        .as_ref()
        .map(|p| p.relationships.values().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    drop(state);

    // Calculate transform style
    let transform_style = format!(
        "transform: translate({}px, {}px) scale({});",
        pan.x, pan.y, zoom
    );

    // Handle mouse down on canvas
    let handle_mouse_down = {
        let interactions = interactions.clone();
        let connection = connection.clone();
        let space_held = space_held;
        move |e: MouseEvent| {
            let pos = position_from_mouse_event(&e);

            // Middle mouse button = button 1 (auxiliary), or space + left click
            let buttons = e.held_buttons();
            let is_middle = buttons.contains(dioxus::html::input_data::MouseButton::Auxiliary);
            let is_left = buttons.contains(dioxus::html::input_data::MouseButton::Primary);

            if is_middle || (*space_held.read() && is_left) {
                if props.pan_enabled {
                    interactions.start_pan(pos);
                }
            } else if is_left {
                // If we're connecting, cancel on canvas click
                if connection.is_drawing() {
                    connection.cancel();
                    return;
                }

                let canvas_pos = interactions.screen_to_canvas(pos);
                props.on_canvas_click.call(canvas_pos);

                // Clear selection if clicking on empty canvas
                if !e.modifiers().shift() {
                    APP_STATE.write().selection.clear();
                }
            }
        }
    };

    // Handle mouse move
    let handle_mouse_move = {
        let interactions = interactions.clone();
        let connection = connection.clone();
        move |e: MouseEvent| {
            let pos = position_from_mouse_event(&e);
            interactions.update_mouse_position(pos);

            // Update connection preview if drawing
            if connection.is_drawing() {
                connection.update_preview(pos);
            }

            // Update pan if panning
            if interactions.is_panning() {
                interactions.update_pan(pos);
            }

            // Update drag if dragging
            if props.drag_enabled && interactions.is_dragging() {
                if let Some((entity_id, new_pos)) = interactions.update_drag(pos) {
                    // Update entity position in state
                    let mut state = APP_STATE.write();
                    if let Some(project) = &mut state.project {
                        if let Some(entity) = project.entities.get_mut(&entity_id) {
                            entity.position = new_pos;
                        }
                    }
                }
            }
        }
    };

    // Handle mouse up
    let handle_mouse_up = {
        let interactions = interactions.clone();
        let connection = connection.clone();
        move |_e: MouseEvent| {
            // Complete connection if drawing
            if connection.is_drawing() {
                connection.complete_and_show_dialog();
            }

            // Stop panning
            if interactions.is_panning() {
                interactions.stop_pan();
            }

            // Stop dragging and emit move event
            if interactions.is_dragging() {
                if let Some(entity_id) = interactions.dragged_entity() {
                    let state = APP_STATE.read();
                    if let Some(project) = &state.project {
                        if let Some(entity) = project.entities.get(&entity_id) {
                            let final_pos = entity.position;
                            drop(state);
                            props.on_entity_move.call((entity_id, final_pos));
                        }
                    }
                }
                interactions.stop_drag();

                // Save to history after drag
                APP_STATE.write().save_to_history("Move entity");
            }
        }
    };

    // Handle mouse wheel (zoom)
    let handle_wheel = {
        let interactions = interactions.clone();
        move |e: WheelEvent| {
            if !props.zoom_enabled {
                return;
            }

            e.prevent_default();

            let delta = zoom_delta_from_wheel(&e);
            if delta.abs() > f32::EPSILON {
                let pos = Position::new(
                    e.client_coordinates().x as f32,
                    e.client_coordinates().y as f32,
                );
                interactions.zoom_by(delta, pos);
            }
        }
    };

    // Handle double click (create new entity)
    let handle_double_click = {
        let interactions = interactions.clone();
        move |e: MouseEvent| {
            let pos = position_from_mouse_event(&e);
            let canvas_pos = interactions.screen_to_canvas(pos);
            props.on_canvas_double_click.call(canvas_pos);
        }
    };

    // Handle key down (space for pan mode, Delete, Escape, arrow keys, etc.)
    let handle_key_down = {
        let interactions = interactions.clone();
        let connection = connection.clone();
        move |e: KeyboardEvent| {
            let key = e.key();
            let modifiers = e.modifiers();
            let is_ctrl = modifiers.ctrl() || modifiers.meta();
            let is_shift = modifiers.shift();

            match key {
                // Space for pan mode
                Key::Character(ref c) if c == " " => {
                    space_held.set(true);
                }

                // Delete or Backspace - delete selected entities
                Key::Delete | Key::Backspace => {
                    e.prevent_default();
                    delete_selected_entities_from_canvas();
                }

                // Escape - clear selection or cancel operation
                Key::Escape => {
                    e.prevent_default();
                    // Cancel connection drawing first
                    if connection.is_drawing() {
                        connection.cancel();
                        return;
                    }
                    // Stop any ongoing operations
                    if interactions.is_panning() {
                        interactions.stop_pan();
                    }
                    if interactions.is_dragging() {
                        interactions.stop_drag();
                    }
                    // Clear selection
                    APP_STATE.write().selection.clear();
                }

                // Arrow keys - move selected entities
                Key::ArrowUp => {
                    e.prevent_default();
                    let step = if is_shift {
                        ARROW_MOVE_STEP_LARGE
                    } else {
                        ARROW_MOVE_STEP
                    };
                    move_selected_entities(0.0, -step);
                }
                Key::ArrowDown => {
                    e.prevent_default();
                    let step = if is_shift {
                        ARROW_MOVE_STEP_LARGE
                    } else {
                        ARROW_MOVE_STEP
                    };
                    move_selected_entities(0.0, step);
                }
                Key::ArrowLeft => {
                    e.prevent_default();
                    let step = if is_shift {
                        ARROW_MOVE_STEP_LARGE
                    } else {
                        ARROW_MOVE_STEP
                    };
                    move_selected_entities(-step, 0.0);
                }
                Key::ArrowRight => {
                    e.prevent_default();
                    let step = if is_shift {
                        ARROW_MOVE_STEP_LARGE
                    } else {
                        ARROW_MOVE_STEP
                    };
                    move_selected_entities(step, 0.0);
                }

                // Ctrl+A - select all entities
                Key::Character(ref c) if (c == "a" || c == "A") && is_ctrl => {
                    e.prevent_default();
                    select_all_entities();
                }

                // Ctrl+D - duplicate selected entities
                Key::Character(ref c) if (c == "d" || c == "D") && is_ctrl => {
                    e.prevent_default();
                    duplicate_selected_entities_on_canvas();
                }

                _ => {}
            }
        }
    };

    // Handle key up
    let handle_key_up = move |e: KeyboardEvent| {
        if let Key::Character(ref c) = e.key() {
            if c == " " {
                space_held.set(false);
            }
        }
    };

    // Entity selection handler with multi-select support
    // Receives (entity_id, shift_held) from EntityCard
    let handle_entity_select = {
        move |(entity_id, is_shift): (EntityId, bool)| {
            let mut state = APP_STATE.write();

            if is_shift {
                // Shift+click: toggle entity in selection (multi-select)
                state.selection.toggle_entity(entity_id);
            } else {
                // Normal click: select only this entity
                state.selection.select_entity(entity_id);
            }
            drop(state);

            props.on_entity_select.call(entity_id);
        }
    };

    // Entity drag start handler - needs to be cloned for each entity card
    let drag_enabled = props.drag_enabled;
    let interactions_for_drag = interactions.clone();

    // Entity double click handler (edit)
    let handle_entity_double_click = move |entity_id: EntityId| {
        APP_STATE
            .write()
            .ui
            .show_dialog(Dialog::EditEntity(entity_id));
    };

    // Field selection handler
    let handle_field_select = move |(entity_id, field_id): (EntityId, FieldId)| {
        APP_STATE.write().selection.field = Some((entity_id, field_id));
        props.on_field_select.call((entity_id, field_id));
    };

    // Add field handler
    let handle_add_field = move |entity_id: EntityId| {
        // Select the entity first
        APP_STATE.write().selection.select_entity(entity_id);
        // Show new field dialog
        APP_STATE
            .write()
            .ui
            .show_dialog(Dialog::NewField(entity_id));
        props.on_add_field.call(entity_id);
    };

    // Toggle collapse handler
    let handle_toggle_collapse = move |entity_id: EntityId| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(entity) = project.entities.get_mut(&entity_id) {
                entity.collapsed = !entity.collapsed;
            }
        }
    };

    // Clone connection for port handlers
    let connection_for_ports = connection.clone();

    // Cursor style based on state
    let cursor_class = if interactions.is_panning() || *space_held.read() {
        "cursor-grabbing"
    } else if interactions.is_dragging() {
        "cursor-grabbing"
    } else if is_connecting {
        "cursor-crosshair"
    } else {
        "cursor-default"
    };

    // Get connection preview data - convert to ConnectionPoint format
    let connection_preview: Option<(ConnectionPoint, ConnectionPoint, bool)> =
        connection.preview_line().map(|(start, end)| {
            (
                ConnectionPoint::from_position(start),
                ConnectionPoint::from_position(end),
                connection.is_valid_target(),
            )
        });

    // Get the selected field id for the current entity (if applicable)
    let get_selected_field_for_entity = |entity_id: EntityId| -> Option<FieldId> {
        selected_field.and_then(|(eid, fid)| if eid == entity_id { Some(fid) } else { None })
    };

    // Entity count for display
    let entity_count = entities.len();

    rsx! {
        div {
            class: "canvas-container relative w-full h-full overflow-hidden bg-slate-950 {cursor_class}",
            tabindex: 0,

            // Mouse events
            onmousedown: handle_mouse_down,
            onmousemove: handle_mouse_move,
            onmouseup: handle_mouse_up,
            onmouseleave: {
                let interactions = interactions.clone();
                move |_| {
                    interactions.stop_pan();
                    interactions.stop_drag();
                }
            },

            // Wheel event for zoom
            onwheel: handle_wheel,

            // Double click for new entity
            ondoubleclick: handle_double_click,

            // Keyboard events
            onkeydown: handle_key_down,
            onkeyup: handle_key_up,

            // Prevent context menu on canvas
            oncontextmenu: move |e| {
                e.prevent_default();
            },

            // Grid background (SVG pattern)
            if show_grid {
                CanvasGrid {
                    grid_size: grid_size,
                    pan: pan,
                    zoom: zoom,
                }
            }

            // Render relationship connections
            ConnectionsLayer {
                zoom: zoom,
                pan_x: pan.x,
                pan_y: pan.y,
                preview: connection_preview,
            }

            // Transform container for entities
            div {
                class: "canvas-transform-layer absolute inset-0 origin-top-left",
                style: "{transform_style}",

                // Render entities
                for entity in entities.iter() {
                    {
                        let interactions_drag = interactions_for_drag.clone();
                        let connection_click = connection_for_ports.clone();
                        let connection_hover = connection_for_ports.clone();
                        rsx! {
                            EntityCard {
                                key: "{entity.id}",
                                entity: entity.clone(),
                                zoom: zoom as f64,
                                selected: selected_entities.contains(&entity.id),
                                dragging: dragging_entity == Some(entity.id),
                                selected_field: get_selected_field_for_entity(entity.id),
                                show_ports: true,
                                interactive: true,
                                is_connecting: is_connecting,
                                connection_start_entity: connection_start,
                                on_select: handle_entity_select,
                                on_drag_start: move |(entity_id, pos): (EntityId, Position)| {
                                    if !drag_enabled {
                                        return;
                                    }
                                    let state = APP_STATE.read();
                                    if let Some(project) = &state.project {
                                        if let Some(e) = project.entities.get(&entity_id) {
                                            let entity_pos = e.position;
                                            drop(state);
                                            interactions_drag.start_drag(entity_id, pos, entity_pos);
                                        }
                                    }
                                },
                                on_double_click: handle_entity_double_click,
                                on_field_select: handle_field_select,
                                on_add_field: handle_add_field,
                                on_toggle_collapse: handle_toggle_collapse,
                                on_port_click: move |info: PortClickInfo| {
                                    if connection_click.is_drawing() {
                                        connection_click.set_hover_target(Some(info.entity_id), Some(info.port_type));
                                        connection_click.complete_and_show_dialog();
                                    } else if info.port_type == PortType::Output {
                                        let pos = Position::new(info.screen_position.0, info.screen_position.1);
                                        connection_click.start(info.entity_id, info.port_type, pos);
                                    }
                                },
                                on_port_hover: move |info: Option<PortClickInfo>| {
                                    if let Some(info) = info {
                                        connection_hover.set_hover_target(Some(info.entity_id), Some(info.port_type));
                                    } else {
                                        connection_hover.set_hover_target(None, None);
                                    }
                                },
                            }
                        }
                    }
                }
            }

            // Zoom controls overlay
            if props.show_zoom_controls {
                {
                    let i1 = interactions.clone();
                    let i2 = interactions.clone();
                    let i3 = interactions.clone();
                    let i4 = interactions.clone();
                    rsx! {
                        ZoomControls {
                            zoom: zoom as f64,
                            on_zoom_in: move |_| i1.zoom_in(),
                            on_zoom_out: move |_| i2.zoom_out(),
                            on_zoom_reset: move |_| i3.reset_zoom(),
                            on_fit_content: move |_| i4.fit_to_content(),
                        }
                    }
                }
            }

            // Canvas info overlay (coordinates, zoom level)
            CanvasInfo {
                zoom: zoom as f64,
                pan: pan,
                entity_count: entity_count,
            }

            // Empty state
            if entities.is_empty() {
                CanvasEmptyState {}
            }
        }
    }
}

// ============================================================================
// Canvas Grid Component
// ============================================================================

/// Properties for CanvasGrid component
#[derive(Props, Clone, PartialEq)]
struct CanvasGridProps {
    /// Grid cell size
    grid_size: f32,
    /// Current pan offset
    pan: Position,
    /// Current zoom level
    zoom: f32,
}

/// SVG grid pattern background
#[component]
fn CanvasGrid(props: CanvasGridProps) -> Element {
    let scaled_size = props.grid_size * props.zoom;
    let major_size = scaled_size * 5.0; // Major grid every 5 cells

    // Calculate offset for grid alignment
    let offset_x = props.pan.x % major_size;
    let offset_y = props.pan.y % major_size;

    let pattern_id = "canvas-grid-pattern";
    let major_pattern_id = "canvas-major-grid-pattern";

    rsx! {
        svg {
            class: "absolute inset-0 w-full h-full pointer-events-none",
            xmlns: "http://www.w3.org/2000/svg",

            defs {
                // Minor grid pattern
                pattern {
                    id: "{pattern_id}",
                    width: "{scaled_size}",
                    height: "{scaled_size}",
                    pattern_units: "userSpaceOnUse",
                    x: "{offset_x}",
                    y: "{offset_y}",

                    path {
                        d: "M {scaled_size} 0 L 0 0 0 {scaled_size}",
                        fill: "none",
                        stroke: "{GRID_LINE_COLOR}",
                        stroke_width: "0.5",
                    }
                }

                // Major grid pattern
                pattern {
                    id: "{major_pattern_id}",
                    width: "{major_size}",
                    height: "{major_size}",
                    pattern_units: "userSpaceOnUse",
                    x: "{offset_x}",
                    y: "{offset_y}",

                    path {
                        d: "M {major_size} 0 L 0 0 0 {major_size}",
                        fill: "none",
                        stroke: "{GRID_MAJOR_COLOR}",
                        stroke_width: "1",
                    }
                }
            }

            // Minor grid
            rect {
                width: "100%",
                height: "100%",
                fill: "url(#{pattern_id})",
            }

            // Major grid
            rect {
                width: "100%",
                height: "100%",
                fill: "url(#{major_pattern_id})",
            }
        }
    }
}

// ============================================================================
// Zoom Controls Component
// ============================================================================

/// Properties for ZoomControls component
#[derive(Props, Clone, PartialEq)]
struct ZoomControlsProps {
    /// Current zoom level
    zoom: f64,
    /// Zoom in callback
    on_zoom_in: EventHandler<()>,
    /// Zoom out callback
    on_zoom_out: EventHandler<()>,
    /// Reset zoom callback
    on_zoom_reset: EventHandler<()>,
    /// Fit to content callback
    on_fit_content: EventHandler<()>,
}

/// Zoom control buttons overlay
#[component]
fn ZoomControls(props: ZoomControlsProps) -> Element {
    let zoom_percent = (props.zoom * 100.0).round() as i32;

    rsx! {
        div {
            class: "zoom-controls absolute bottom-4 right-4 flex items-center gap-1 bg-slate-800/90 backdrop-blur-sm rounded-lg p-1 shadow-lg border border-slate-700",

            // Zoom out button
            button {
                class: "w-8 h-8 flex items-center justify-center rounded hover:bg-slate-700 text-slate-300 hover:text-white transition-colors",
                title: "Zoom Out (Scroll Down)",
                onclick: move |_| props.on_zoom_out.call(()),
                "−"
            }

            // Zoom level display / reset button
            button {
                class: "px-2 h-8 min-w-[60px] flex items-center justify-center text-sm text-slate-300 hover:text-white hover:bg-slate-700 rounded transition-colors",
                title: "Reset Zoom (Click)",
                onclick: move |_| props.on_zoom_reset.call(()),
                "{zoom_percent}%"
            }

            // Zoom in button
            button {
                class: "w-8 h-8 flex items-center justify-center rounded hover:bg-slate-700 text-slate-300 hover:text-white transition-colors",
                title: "Zoom In (Scroll Up)",
                onclick: move |_| props.on_zoom_in.call(()),
                "+"
            }

            // Separator
            div {
                class: "w-px h-6 bg-slate-600 mx-1",
            }

            // Fit to content button
            button {
                class: "w-8 h-8 flex items-center justify-center rounded hover:bg-slate-700 text-slate-300 hover:text-white transition-colors",
                title: "Fit to Content",
                onclick: move |_| props.on_fit_content.call(()),
                "⊡"
            }
        }
    }
}

// ============================================================================
// Canvas Info Component
// ============================================================================

/// Properties for CanvasInfo component
#[derive(Props, Clone, PartialEq)]
struct CanvasInfoProps {
    /// Current zoom level
    zoom: f64,
    /// Current pan position
    pan: Position,
    /// Number of entities
    entity_count: usize,
}

/// Canvas information overlay (position, stats)
#[component]
fn CanvasInfo(props: CanvasInfoProps) -> Element {
    let entity_label = if props.entity_count == 1 {
        "entity"
    } else {
        "entities"
    };
    let pan_x = props.pan.x as i32;
    let pan_y = props.pan.y as i32;

    rsx! {
        div {
            class: "canvas-info absolute bottom-4 left-4 flex items-center gap-4 text-xs text-slate-500",

            // Entity count
            span {
                class: "flex items-center gap-1",
                "📦 {props.entity_count} {entity_label}"
            }

            // Separator
            span { "•" }

            // Pan position (for debugging/reference)
            span {
                "Pan: ({pan_x}, {pan_y})"
            }
        }
    }
}

// ============================================================================
// Canvas Empty State Component
// ============================================================================

/// Empty state shown when no entities exist
#[component]
fn CanvasEmptyState() -> Element {
    rsx! {
        div {
            class: "absolute inset-0 flex items-center justify-center pointer-events-none",

            div {
                class: "text-center max-w-md p-8",

                // Icon
                div {
                    class: "text-6xl mb-4 opacity-30",
                    "📐"
                }

                // Title
                h3 {
                    class: "text-xl font-semibold text-slate-400 mb-2",
                    "No Entities Yet"
                }

                // Description
                p {
                    class: "text-slate-500 mb-6",
                    "Double-click anywhere on the canvas to create your first entity, or use the \"Add Entity\" button in the toolbar."
                }

                // Hints
                div {
                    class: "text-xs text-slate-600 space-y-1",
                    p { "💡 Tip: Use the scroll wheel to zoom in/out" }
                    p { "💡 Tip: Hold space and drag to pan the canvas" }
                    p { "💡 Tip: Press Ctrl+N to create a new entity" }
                }
            }
        }
    }
}

// ============================================================================
// Canvas Toolbar Component (for embedding above canvas)
// ============================================================================

/// Properties for CanvasToolbar component
#[derive(Props, Clone, PartialEq)]
pub struct CanvasToolbarProps {
    /// Whether grid is visible
    pub show_grid: bool,

    /// Whether snap to grid is enabled
    pub snap_to_grid: bool,

    /// Callback to toggle grid
    #[props(default)]
    pub on_toggle_grid: EventHandler<()>,

    /// Callback to toggle snap
    #[props(default)]
    pub on_toggle_snap: EventHandler<()>,

    /// Callback to add new entity
    #[props(default)]
    pub on_add_entity: EventHandler<()>,
}

/// Toolbar for canvas-specific actions
#[component]
pub fn CanvasToolbar(props: CanvasToolbarProps) -> Element {
    rsx! {
        div {
            class: "canvas-toolbar flex items-center gap-2 px-4 py-2 bg-slate-800 border-b border-slate-700",

            // Add Entity button
            button {
                class: "px-3 py-1.5 bg-indigo-600 hover:bg-indigo-700 text-white text-sm rounded-lg transition-colors flex items-center gap-1.5",
                onclick: move |_| props.on_add_entity.call(()),
                span { "+" }
                span { "Add Entity" }
            }

            // Separator
            div { class: "w-px h-6 bg-slate-700" }

            // Grid toggle
            button {
                class: "px-3 py-1.5 text-sm rounded-lg transition-colors flex items-center gap-1.5",
                class: if props.show_grid { "bg-slate-700 text-white" } else { "text-slate-400 hover:text-white hover:bg-slate-700/50" },
                onclick: move |_| props.on_toggle_grid.call(()),
                span { "⊞" }
                span { "Grid" }
            }

            // Snap toggle
            button {
                class: "px-3 py-1.5 text-sm rounded-lg transition-colors flex items-center gap-1.5",
                class: if props.snap_to_grid { "bg-slate-700 text-white" } else { "text-slate-400 hover:text-white hover:bg-slate-700/50" },
                onclick: move |_| props.on_toggle_snap.call(()),
                span { "🧲" }
                span { "Snap" }
            }
        }
    }
}

// ============================================================================
// Canvas Helper Functions
// ============================================================================

/// Delete selected entities from the canvas
fn delete_selected_entities_from_canvas() {
    let mut state = APP_STATE.write();

    let selected = state.selection.entities.clone();
    if selected.is_empty() {
        return;
    }

    // Show confirmation dialog for multiple entities
    if selected.len() > 1 {
        drop(state);
        APP_STATE
            .write()
            .ui
            .show_dialog(crate::state::Dialog::ConfirmDelete(
                crate::state::DeleteTarget::Entities(selected.into_iter().collect()),
            ));
        return;
    }

    // Single entity - delete directly or show confirmation
    if let Some(entity_id) = selected.iter().next().copied() {
        drop(state);
        APP_STATE
            .write()
            .ui
            .show_dialog(crate::state::Dialog::ConfirmDelete(
                crate::state::DeleteTarget::Entity(entity_id),
            ));
    }
}

/// Select all entities on the canvas
fn select_all_entities() {
    let mut state = APP_STATE.write();

    if let Some(project) = &state.project {
        let all_ids: Vec<_> = project.entities.keys().copied().collect();
        state.selection.clear();
        for id in all_ids {
            state.selection.add_entity(id);
        }
    }
}

/// Duplicate selected entities on the canvas
fn duplicate_selected_entities_on_canvas() {
    let mut state = APP_STATE.write();

    let selected = state.selection.entities.clone();
    if selected.is_empty() {
        return;
    }

    let Some(project) = &mut state.project else {
        return;
    };

    // Collect entities to duplicate
    let entities_to_duplicate: Vec<imortal_ir::entity::Entity> = selected
        .iter()
        .filter_map(|id| project.entities.get(id).cloned())
        .collect();

    // Create duplicates with offset positions
    let offset = Position::new(30.0, 30.0);
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

/// Move selected entities by the given offset
fn move_selected_entities(dx: f32, dy: f32) {
    let mut state = APP_STATE.write();

    let selected = state.selection.entities.clone();
    if selected.is_empty() {
        return;
    }

    let snap_to_grid = state.canvas.snap_to_grid;
    let grid_size = state.canvas.grid_size;

    let Some(project) = &mut state.project else {
        return;
    };

    // Move each selected entity
    for id in &selected {
        if let Some(entity) = project.entities.get_mut(id) {
            let mut new_x = entity.position.x + dx;
            let mut new_y = entity.position.y + dy;

            // Snap to grid if enabled
            if snap_to_grid {
                new_x = (new_x / grid_size).round() * grid_size;
                new_y = (new_y / grid_size).round() * grid_size;
            }

            entity.position = Position::new(new_x, new_y);
            entity.touch();
        }
    }

    // Mark as dirty
    state.is_dirty = true;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_constants() {
        assert!(DEFAULT_GRID_SIZE > 0.0);
        assert!(!GRID_LINE_COLOR.is_empty());
        assert!(!GRID_MAJOR_COLOR.is_empty());
        assert!(!CANVAS_BG_COLOR.is_empty());
    }

    #[test]
    fn test_minimap_threshold() {
        assert!(MINIMAP_THRESHOLD > 0);
    }

    #[test]
    fn test_arrow_move_constants() {
        assert!(ARROW_MOVE_STEP > 0.0);
        assert!(ARROW_MOVE_STEP_LARGE > ARROW_MOVE_STEP);
    }
}
