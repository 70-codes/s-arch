//! # Canvas Interactions Hook
//!
//! Provides hooks for managing canvas interactions including:
//! - Pan (dragging the canvas view)
//! - Zoom (mouse wheel and keyboard shortcuts)
//! - Entity dragging
//! - Mouse position tracking
//! - Coordinate transformations

use dioxus::prelude::*;
use imortal_core::types::{EntityId, Position};

use crate::state::APP_STATE;

// ============================================================================
// Constants
// ============================================================================

/// Minimum zoom level (10%)
pub const MIN_ZOOM: f32 = 0.1;

/// Maximum zoom level (300%)
pub const MAX_ZOOM: f32 = 3.0;

/// Zoom step for wheel events
pub const ZOOM_STEP: f32 = 0.1;

/// Zoom step for keyboard shortcuts
pub const ZOOM_STEP_KEYBOARD: f32 = 0.25;

/// Grid size for snapping
pub const DEFAULT_GRID_SIZE: f32 = 20.0;

// ============================================================================
// Pan State
// ============================================================================

/// State for canvas panning
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PanState {
    /// Whether panning is currently active
    pub is_panning: bool,
    /// Starting mouse position when pan began
    pub start_mouse: Position,
    /// Starting pan offset when pan began
    pub start_pan: Position,
}

impl PanState {
    /// Create a new pan state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start panning from the given mouse position
    pub fn start(&mut self, mouse_pos: Position, current_pan: Position) {
        self.is_panning = true;
        self.start_mouse = mouse_pos;
        self.start_pan = current_pan;
    }

    /// Stop panning
    pub fn stop(&mut self) {
        self.is_panning = false;
    }

    /// Calculate new pan offset based on current mouse position
    pub fn calculate_pan(&self, current_mouse: Position) -> Position {
        if !self.is_panning {
            return self.start_pan;
        }

        Position::new(
            self.start_pan.x + (current_mouse.x - self.start_mouse.x),
            self.start_pan.y + (current_mouse.y - self.start_mouse.y),
        )
    }
}

// ============================================================================
// Drag State
// ============================================================================

/// State for dragging entities
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DragState {
    /// Whether dragging is currently active
    pub is_dragging: bool,
    /// The entity being dragged (if any)
    pub entity_id: Option<EntityId>,
    /// Starting mouse position when drag began (in canvas coordinates)
    pub start_mouse: Position,
    /// Starting entity position when drag began
    pub start_entity_pos: Position,
    /// Offset from entity position to mouse position
    pub offset: Position,
}

impl DragState {
    /// Create a new drag state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start dragging an entity
    pub fn start(&mut self, entity_id: EntityId, mouse_pos: Position, entity_pos: Position) {
        self.is_dragging = true;
        self.entity_id = Some(entity_id);
        self.start_mouse = mouse_pos;
        self.start_entity_pos = entity_pos;
        self.offset = Position::new(mouse_pos.x - entity_pos.x, mouse_pos.y - entity_pos.y);
    }

    /// Stop dragging
    pub fn stop(&mut self) {
        self.is_dragging = false;
        self.entity_id = None;
    }

    /// Calculate new entity position based on current mouse position
    pub fn calculate_entity_pos(&self, current_mouse: Position) -> Position {
        Position::new(
            current_mouse.x - self.offset.x,
            current_mouse.y - self.offset.y,
        )
    }

    /// Calculate snapped entity position
    pub fn calculate_snapped_pos(&self, current_mouse: Position, grid_size: f32) -> Position {
        let pos = self.calculate_entity_pos(current_mouse);
        Position::new(
            (pos.x / grid_size).round() * grid_size,
            (pos.y / grid_size).round() * grid_size,
        )
    }
}

// ============================================================================
// Canvas Interactions
// ============================================================================

/// Canvas interaction state and handlers
#[derive(Debug, Clone)]
pub struct CanvasInteractions {
    /// Current pan state
    pub pan_state: Signal<PanState>,
    /// Current drag state
    pub drag_state: Signal<DragState>,
    /// Current mouse position (screen coordinates)
    pub mouse_screen_pos: Signal<Position>,
    /// Current mouse position (canvas coordinates)
    pub mouse_canvas_pos: Signal<Position>,
    /// Whether the mouse is over the canvas
    pub is_mouse_over: Signal<bool>,
}

impl CanvasInteractions {
    // ========================================================================
    // Coordinate Transformations
    // ========================================================================

    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen_pos: Position) -> Position {
        let state = APP_STATE.read();
        let pan = state.canvas.pan;
        let zoom = state.canvas.zoom;
        drop(state);

        Position::new((screen_pos.x - pan.x) / zoom, (screen_pos.y - pan.y) / zoom)
    }

    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas_pos: Position) -> Position {
        let state = APP_STATE.read();
        let pan = state.canvas.pan;
        let zoom = state.canvas.zoom;
        drop(state);

        Position::new(canvas_pos.x * zoom + pan.x, canvas_pos.y * zoom + pan.y)
    }

    // ========================================================================
    // Pan Handlers
    // ========================================================================

    /// Start panning (usually on middle mouse button or space+click)
    pub fn start_pan(&self, screen_pos: Position) {
        let state = APP_STATE.read();
        let current_pan = state.canvas.pan;
        drop(state);

        let mut pan_state = self.pan_state;
        pan_state.write().start(screen_pos, current_pan);
        APP_STATE.write().canvas.is_panning = true;
    }

    /// Update pan while dragging
    pub fn update_pan(&self, screen_pos: Position) {
        let pan_state = self.pan_state.read();
        if !pan_state.is_panning {
            return;
        }

        let new_pan = pan_state.calculate_pan(screen_pos);
        drop(pan_state);

        APP_STATE.write().canvas.pan = new_pan;
    }

    /// Stop panning
    pub fn stop_pan(&self) {
        let mut pan_state = self.pan_state;
        pan_state.write().stop();
        APP_STATE.write().canvas.is_panning = false;
    }

    // ========================================================================
    // Zoom Handlers
    // ========================================================================

    /// Zoom by a delta amount, centered on a point
    pub fn zoom_by(&self, delta: f32, center: Position) {
        let mut state = APP_STATE.write();
        let old_zoom = state.canvas.zoom;
        let new_zoom = (old_zoom + delta).clamp(MIN_ZOOM, MAX_ZOOM);

        if (new_zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }

        // Calculate the point under the mouse in canvas coordinates
        let canvas_point = Position::new(
            (center.x - state.canvas.pan.x) / old_zoom,
            (center.y - state.canvas.pan.y) / old_zoom,
        );

        // Update zoom
        state.canvas.zoom = new_zoom;

        // Adjust pan to keep the point under the mouse
        state.canvas.pan = Position::new(
            center.x - canvas_point.x * new_zoom,
            center.y - canvas_point.y * new_zoom,
        );
    }

    /// Zoom in by the default step
    pub fn zoom_in(&self) {
        let center = *self.mouse_screen_pos.read();
        self.zoom_by(ZOOM_STEP_KEYBOARD, center);
    }

    /// Zoom out by the default step
    pub fn zoom_out(&self) {
        let center = *self.mouse_screen_pos.read();
        self.zoom_by(-ZOOM_STEP_KEYBOARD, center);
    }

    /// Reset zoom to 100%
    pub fn reset_zoom(&self) {
        let mut state = APP_STATE.write();
        state.canvas.zoom = 1.0;
    }

    /// Fit all entities in view
    pub fn fit_to_content(&self) {
        // Future: Calculate bounding box of all entities and fit to viewport
        let mut state = APP_STATE.write();
        state.canvas.pan = Position::zero();
        state.canvas.zoom = 1.0;
    }

    // ========================================================================
    // Drag Handlers
    // ========================================================================

    /// Start dragging an entity
    pub fn start_drag(&self, entity_id: EntityId, screen_pos: Position, entity_pos: Position) {
        let canvas_pos = self.screen_to_canvas(screen_pos);
        let mut drag_state = self.drag_state;
        drag_state.write().start(entity_id, canvas_pos, entity_pos);
        APP_STATE.write().canvas.dragging_entity = Some(entity_id);
    }

    /// Update entity position while dragging
    pub fn update_drag(&self, screen_pos: Position) -> Option<(EntityId, Position)> {
        let drag_state = self.drag_state.read();
        if !drag_state.is_dragging {
            return None;
        }

        let entity_id = drag_state.entity_id?;
        let canvas_pos = self.screen_to_canvas(screen_pos);

        let state = APP_STATE.read();
        let snap_enabled = state.canvas.snap_to_grid;
        let grid_size = state.canvas.grid_size;
        drop(state);

        let new_pos = if snap_enabled {
            drag_state.calculate_snapped_pos(canvas_pos, grid_size)
        } else {
            drag_state.calculate_entity_pos(canvas_pos)
        };

        Some((entity_id, new_pos))
    }

    /// Stop dragging
    pub fn stop_drag(&self) {
        let mut drag_state = self.drag_state;
        drag_state.write().stop();
        APP_STATE.write().canvas.dragging_entity = None;
    }

    // ========================================================================
    // Mouse Tracking
    // ========================================================================

    /// Update mouse position
    pub fn update_mouse_position(&self, screen_pos: Position) {
        let mut mouse_screen = self.mouse_screen_pos;
        let mut mouse_canvas = self.mouse_canvas_pos;
        mouse_screen.set(screen_pos);
        let canvas_pos = self.screen_to_canvas(screen_pos);
        mouse_canvas.set(canvas_pos);
        APP_STATE.write().canvas.mouse_position = canvas_pos;
    }

    /// Set mouse over state
    pub fn set_mouse_over(&self, over: bool) {
        let mut is_over = self.is_mouse_over;
        is_over.set(over);
    }

    // ========================================================================
    // Grid & Snapping
    // ========================================================================

    /// Toggle grid visibility
    pub fn toggle_grid(&self) {
        let mut state = APP_STATE.write();
        state.canvas.show_grid = !state.canvas.show_grid;
    }

    /// Toggle snap to grid
    pub fn toggle_snap(&self) {
        let mut state = APP_STATE.write();
        state.canvas.snap_to_grid = !state.canvas.snap_to_grid;
    }

    /// Set grid size
    pub fn set_grid_size(&self, size: f32) {
        APP_STATE.write().canvas.grid_size = size.max(5.0);
    }

    /// Snap a position to the grid
    pub fn snap_to_grid(&self, pos: Position) -> Position {
        let grid_size = APP_STATE.read().canvas.grid_size;
        Position::new(
            (pos.x / grid_size).round() * grid_size,
            (pos.y / grid_size).round() * grid_size,
        )
    }

    // ========================================================================
    // State Queries
    // ========================================================================

    /// Check if currently panning
    pub fn is_panning(&self) -> bool {
        self.pan_state.read().is_panning
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        self.drag_state.read().is_dragging
    }

    /// Get the entity being dragged (if any)
    pub fn dragged_entity(&self) -> Option<EntityId> {
        self.drag_state.read().entity_id
    }

    /// Get current zoom level
    pub fn zoom(&self) -> f32 {
        APP_STATE.read().canvas.zoom
    }

    /// Get current pan offset
    pub fn pan(&self) -> Position {
        APP_STATE.read().canvas.pan
    }
}

// ============================================================================
// Hook
// ============================================================================

/// Hook for canvas interactions
///
/// Provides state and handlers for pan, zoom, and drag operations.
///
/// # Example
///
/// ```rust,ignore
/// fn Canvas() -> Element {
///     let interactions = use_canvas_interactions();
///
///     rsx! {
///         div {
///             onmousedown: move |e| {
///                 // Start pan on middle mouse
///                 let pos = Position::new(
///                     e.client_coordinates().x as f32,
///                     e.client_coordinates().y as f32
///                 );
///                 interactions.start_pan(pos);
///             },
///             onmousemove: move |e| {
///                 let pos = Position::new(
///                     e.client_coordinates().x as f32,
///                     e.client_coordinates().y as f32
///                 );
///                 interactions.update_mouse_position(pos);
///                 if interactions.is_panning() {
///                     interactions.update_pan(pos);
///                 }
///             },
///             onmouseup: move |_| {
///                 interactions.stop_pan();
///             },
///             onwheel: move |e| {
///                 let delta = if e.delta().y < 0.0 { ZOOM_STEP } else { -ZOOM_STEP };
///                 let pos = Position::new(
///                     e.client_coordinates().x as f32,
///                     e.client_coordinates().y as f32
///                 );
///                 interactions.zoom_by(delta, pos);
///             },
///         }
///     }
/// }
/// ```
pub fn use_canvas_interactions() -> CanvasInteractions {
    let pan_state = use_signal(PanState::new);
    let drag_state = use_signal(DragState::new);
    let mouse_screen_pos = use_signal(Position::zero);
    let mouse_canvas_pos = use_signal(Position::zero);
    let is_mouse_over = use_signal(|| false);

    CanvasInteractions {
        pan_state,
        drag_state,
        mouse_screen_pos,
        mouse_canvas_pos,
        is_mouse_over,
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract position from mouse event (converts f64 to f32)
pub fn position_from_mouse_event(e: &MouseEvent) -> Position {
    let coords = e.client_coordinates();
    Position::new(coords.x as f32, coords.y as f32)
}

/// Extract position from pointer event data (converts f64 to f32)
pub fn position_from_pointer_data(data: &PointerData) -> Position {
    let coords = data.client_coordinates();
    Position::new(coords.x as f32, coords.y as f32)
}

/// Calculate wheel delta for zoom
pub fn zoom_delta_from_wheel(e: &WheelEvent) -> f32 {
    // WheelDelta is an enum containing Vector3D types, extract the y component
    let delta_y = match e.delta() {
        dioxus::html::geometry::WheelDelta::Pixels(v) => v.y,
        dioxus::html::geometry::WheelDelta::Lines(v) => v.y * 20.0, // Approximate pixels per line
        dioxus::html::geometry::WheelDelta::Pages(v) => v.y * 400.0, // Approximate pixels per page
    };

    if delta_y < 0.0 {
        ZOOM_STEP
    } else if delta_y > 0.0 {
        -ZOOM_STEP
    } else {
        0.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pan_state() {
        let mut pan = PanState::new();
        assert!(!pan.is_panning);

        pan.start(Position::new(100.0, 100.0), Position::new(0.0, 0.0));
        assert!(pan.is_panning);

        // Move mouse 50 pixels right and down
        let new_pan = pan.calculate_pan(Position::new(150.0, 150.0));
        assert_eq!(new_pan.x, 50.0);
        assert_eq!(new_pan.y, 50.0);

        pan.stop();
        assert!(!pan.is_panning);
    }

    #[test]
    fn test_drag_state() {
        let mut drag = DragState::new();
        assert!(!drag.is_dragging);

        let entity_id = uuid::Uuid::new_v4();
        drag.start(
            entity_id,
            Position::new(110.0, 120.0),
            Position::new(100.0, 100.0),
        );

        assert!(drag.is_dragging);
        assert_eq!(drag.entity_id, Some(entity_id));
        assert_eq!(drag.offset.x, 10.0);
        assert_eq!(drag.offset.y, 20.0);

        // Move mouse to new position
        let new_pos = drag.calculate_entity_pos(Position::new(200.0, 200.0));
        assert_eq!(new_pos.x, 190.0);
        assert_eq!(new_pos.y, 180.0);

        drag.stop();
        assert!(!drag.is_dragging);
        assert!(drag.entity_id.is_none());
    }

    #[test]
    fn test_drag_state_snapping() {
        let mut drag = DragState::new();
        let entity_id = uuid::Uuid::new_v4();
        drag.start(
            entity_id,
            Position::new(100.0, 100.0),
            Position::new(100.0, 100.0),
        );

        // Test snapping with grid size of 20
        // With offset (0,0), position = mouse position
        // 115/20 = 5.75 -> rounds to 6 -> 120
        // 127/20 = 6.35 -> rounds to 6 -> 120
        let snapped = drag.calculate_snapped_pos(Position::new(115.0, 127.0), 20.0);
        assert_eq!(snapped.x, 120.0); // 115 rounds to 120
        assert_eq!(snapped.y, 120.0); // 127 rounds to 120
    }

    #[test]
    fn test_zoom_constants() {
        assert!(MIN_ZOOM > 0.0);
        assert!(MAX_ZOOM > MIN_ZOOM);
        assert!(ZOOM_STEP > 0.0);
        assert!(ZOOM_STEP_KEYBOARD > 0.0);
    }
}
