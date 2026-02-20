//! # Connection Drawing Hook
//!
//! Hook for managing connection drawing interactions on the canvas.
//!
//! This hook provides state and handlers for the drag-to-connect workflow:
//! 1. User clicks on an output port to start a connection
//! 2. A preview line is drawn from the port to the cursor
//! 3. User drags to an input port of another entity
//! 4. If valid, releasing creates a relationship
//!
//! ## Usage
//!
//! ```rust,ignore
//! let connection = use_connection_drawing();
//!
//! // Start connection from a port
//! connection.start(entity_id, PortType::Output, position);
//!
//! // Update preview while dragging
//! connection.update_preview(mouse_position);
//!
//! // Complete connection on valid target
//! if let Some((from, to)) = connection.complete(target_entity_id, PortType::Input) {
//!     // Create relationship
//! }
//! ```

use dioxus::prelude::*;
use imortal_core::types::Position;
use uuid::Uuid;

use crate::components::port::PortType;
use crate::state::{APP_STATE, ConnectionPort, Dialog};

// ============================================================================
// Constants
// ============================================================================

/// Minimum distance to consider as a valid drag (prevents accidental connections)
pub const MIN_DRAG_DISTANCE: f32 = 10.0;

/// Snap distance for port targeting (in screen pixels)
pub const PORT_SNAP_DISTANCE: f32 = 20.0;

// ============================================================================
// Connection State
// ============================================================================

/// State for an active connection being drawn
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionDrawingState {
    /// Whether a connection is currently being drawn
    pub is_drawing: bool,

    /// Entity where the connection started
    pub from_entity_id: Option<Uuid>,

    /// Port type where connection started
    pub from_port_type: Option<PortType>,

    /// Screen position where the connection started
    pub start_position: Position,

    /// Current mouse position (for preview line)
    pub current_position: Position,

    /// Entity currently being hovered over (potential target)
    pub hover_entity_id: Option<Uuid>,

    /// Port type being hovered over
    pub hover_port_type: Option<PortType>,

    /// Whether the current hover target is valid
    pub is_valid_target: bool,

    /// Total distance dragged (for minimum drag detection)
    pub drag_distance: f32,
}

impl Default for ConnectionDrawingState {
    fn default() -> Self {
        Self {
            is_drawing: false,
            from_entity_id: None,
            from_port_type: None,
            start_position: Position::zero(),
            current_position: Position::zero(),
            hover_entity_id: None,
            hover_port_type: None,
            is_valid_target: false,
            drag_distance: 0.0,
        }
    }
}

impl ConnectionDrawingState {
    /// Create a new default state
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if currently drawing a connection
    pub fn is_active(&self) -> bool {
        self.is_drawing && self.from_entity_id.is_some()
    }

    /// Reset state to default
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Check if we've dragged far enough to count as intentional
    pub fn has_min_distance(&self) -> bool {
        self.drag_distance >= MIN_DRAG_DISTANCE
    }

    /// Get the from entity ID if drawing
    pub fn source_entity(&self) -> Option<Uuid> {
        if self.is_drawing {
            self.from_entity_id
        } else {
            None
        }
    }

    /// Check if a given entity/port combination is a valid target
    pub fn is_valid_drop_target(&self, entity_id: Uuid, port_type: PortType) -> bool {
        if !self.is_drawing {
            return false;
        }

        // Can't connect to same entity
        if Some(entity_id) == self.from_entity_id {
            return false;
        }

        // Must connect output -> input
        match (self.from_port_type, port_type) {
            (Some(PortType::Output), PortType::Input) => true,
            (Some(PortType::Input), PortType::Output) => true,
            _ => false,
        }
    }
}

// ============================================================================
// Connection Drawing Result
// ============================================================================

/// Result of completing a connection
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionResult {
    /// Source entity ID
    pub from_entity_id: Uuid,
    /// Target entity ID
    pub to_entity_id: Uuid,
    /// Source port type
    pub from_port_type: PortType,
    /// Target port type
    pub to_port_type: PortType,
}

impl ConnectionResult {
    /// Create a new connection result
    pub fn new(
        from_entity_id: Uuid,
        to_entity_id: Uuid,
        from_port_type: PortType,
        to_port_type: PortType,
    ) -> Self {
        Self {
            from_entity_id,
            to_entity_id,
            from_port_type,
            to_port_type,
        }
    }

    /// Normalize the connection so output is always "from" and input is always "to"
    pub fn normalized(&self) -> Self {
        match self.from_port_type {
            PortType::Output => self.clone(),
            PortType::Input => Self {
                from_entity_id: self.to_entity_id,
                to_entity_id: self.from_entity_id,
                from_port_type: self.to_port_type,
                to_port_type: self.from_port_type,
            },
        }
    }
}

// ============================================================================
// Connection Drawing Hook
// ============================================================================

/// Hook for connection drawing interactions
#[derive(Debug, Clone)]
pub struct UseConnectionDrawing {
    /// Current connection drawing state
    pub state: Signal<ConnectionDrawingState>,
}

impl UseConnectionDrawing {
    /// Start drawing a connection from an entity port
    pub fn start(&self, entity_id: Uuid, port_type: PortType, position: Position) {
        let mut state_signal = self.state;
        {
            let mut state = state_signal.write();
            state.is_drawing = true;
            state.from_entity_id = Some(entity_id);
            state.from_port_type = Some(port_type);
            state.start_position = position;
            state.current_position = position;
            state.hover_entity_id = None;
            state.hover_port_type = None;
            state.is_valid_target = false;
            state.drag_distance = 0.0;
        }

        // Update global state
        let mut app_state = APP_STATE.write();
        app_state.canvas.is_connecting = true;
        let port = match port_type {
            PortType::Input => ConnectionPort::Input,
            PortType::Output => ConnectionPort::Output,
        };
        app_state.canvas.connection_start = Some((entity_id, port));
    }

    /// Update the preview position while drawing
    pub fn update_preview(&self, position: Position) {
        let mut state_signal = self.state;
        let is_drawing = state_signal.read().is_drawing;
        if !is_drawing {
            return;
        }

        {
            let mut state = state_signal.write();
            // Calculate drag distance
            let dx = position.x - state.start_position.x;
            let dy = position.y - state.start_position.y;
            state.drag_distance = (dx * dx + dy * dy).sqrt();
            state.current_position = position;
        }

        // Also update global mouse position
        APP_STATE.write().canvas.mouse_position = position;
    }

    /// Set the current hover target
    pub fn set_hover_target(&self, entity_id: Option<Uuid>, port_type: Option<PortType>) {
        let mut state_signal = self.state;
        let mut state = state_signal.write();
        state.hover_entity_id = entity_id;
        state.hover_port_type = port_type;

        // Update validity
        if let (Some(eid), Some(pt)) = (entity_id, port_type) {
            state.is_valid_target = state.is_valid_drop_target(eid, pt);
        } else {
            state.is_valid_target = false;
        }
    }

    /// Try to complete the connection
    ///
    /// Returns the connection result if valid, None otherwise
    pub fn complete(&self) -> Option<ConnectionResult> {
        let state = self.state.read();

        if !state.is_drawing {
            return None;
        }

        // Check if we have valid source
        let from_entity_id = state.from_entity_id?;
        let from_port_type = state.from_port_type?;

        // Check if we have valid target
        if !state.is_valid_target {
            return None;
        }

        let to_entity_id = state.hover_entity_id?;
        let to_port_type = state.hover_port_type?;

        // Ensure minimum drag distance to prevent accidents
        if !state.has_min_distance() {
            return None;
        }

        drop(state);

        Some(ConnectionResult::new(
            from_entity_id,
            to_entity_id,
            from_port_type,
            to_port_type,
        ))
    }

    /// Complete connection and show relationship dialog
    pub fn complete_and_show_dialog(&self) {
        if let Some(result) = self.complete() {
            let normalized = result.normalized();

            // Cancel first to clear state
            self.cancel();

            // Show the new relationship dialog
            APP_STATE.write().ui.show_dialog(Dialog::NewRelationship(
                Some(normalized.from_entity_id),
                Some(normalized.to_entity_id),
            ));
        } else {
            // No valid connection, just cancel
            self.cancel();
        }
    }

    /// Cancel the current connection
    pub fn cancel(&self) {
        let mut state_signal = self.state;
        state_signal.write().reset();

        // Update global state
        let mut app_state = APP_STATE.write();
        app_state.canvas.is_connecting = false;
        app_state.canvas.connection_start = None;
    }

    /// Check if currently drawing
    pub fn is_drawing(&self) -> bool {
        self.state.read().is_drawing
    }

    /// Check if the current target is valid
    pub fn is_valid_target(&self) -> bool {
        self.state.read().is_valid_target
    }

    /// Get the source entity ID
    pub fn source_entity(&self) -> Option<Uuid> {
        self.state.read().from_entity_id
    }

    /// Get the source port type
    pub fn source_port(&self) -> Option<PortType> {
        self.state.read().from_port_type
    }

    /// Get the current preview line endpoints
    pub fn preview_line(&self) -> Option<(Position, Position)> {
        let state = self.state.read();
        if state.is_drawing && state.has_min_distance() {
            Some((state.start_position, state.current_position))
        } else {
            None
        }
    }

    /// Get the preview start position
    pub fn start_position(&self) -> Position {
        self.state.read().start_position
    }

    /// Get the preview current position
    pub fn current_position(&self) -> Position {
        self.state.read().current_position
    }

    /// Check if an entity is the source of the connection
    pub fn is_source_entity(&self, entity_id: Uuid) -> bool {
        self.state.read().from_entity_id == Some(entity_id)
    }

    /// Check if an entity/port combo is a valid drop target
    pub fn is_valid_drop_target(&self, entity_id: Uuid, port_type: PortType) -> bool {
        self.state.read().is_valid_drop_target(entity_id, port_type)
    }
}

// ============================================================================
// Hook Function
// ============================================================================

/// Create a connection drawing hook
///
/// This hook manages the state for drawing connections between entities.
/// It handles:
/// - Starting a connection from a port
/// - Updating the preview line as the mouse moves
/// - Validating potential drop targets
/// - Completing or cancelling the connection
///
/// # Example
///
/// ```rust,ignore
/// let connection = use_connection_drawing();
///
/// // In port click handler:
/// connection.start(entity_id, PortType::Output, screen_position);
///
/// // In mouse move handler:
/// connection.update_preview(mouse_position);
///
/// // In port hover handler:
/// connection.set_hover_target(Some(target_id), Some(PortType::Input));
///
/// // In mouse up handler:
/// connection.complete_and_show_dialog();
/// ```
pub fn use_connection_drawing() -> UseConnectionDrawing {
    let state = use_signal(ConnectionDrawingState::default);

    UseConnectionDrawing { state }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate distance between two positions
pub fn distance(a: Position, b: Position) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

/// Check if a point is within snap distance of another point
pub fn is_within_snap_distance(point: Position, target: Position) -> bool {
    distance(point, target) <= PORT_SNAP_DISTANCE
}

/// Get the port center position for an entity
///
/// This calculates the approximate screen position of a port based on
/// the entity's position, size, and the port type.
pub fn calculate_port_center(
    entity_position: Position,
    entity_width: f32,
    entity_height: f32,
    port_type: PortType,
    zoom: f32,
    pan: Position,
) -> Position {
    // Port is at vertical center of the card
    let entity_center_y = entity_position.y + entity_height / 2.0;

    let port_x = match port_type {
        PortType::Input => entity_position.x, // Left side
        PortType::Output => entity_position.x + entity_width, // Right side
    };

    // Convert to screen coordinates
    Position::new(port_x * zoom + pan.x, entity_center_y * zoom + pan.y)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_default() {
        let state = ConnectionDrawingState::default();
        assert!(!state.is_drawing);
        assert!(state.from_entity_id.is_none());
        assert!(!state.is_active());
    }

    #[test]
    fn test_connection_state_is_active() {
        let mut state = ConnectionDrawingState::default();
        assert!(!state.is_active());

        state.is_drawing = true;
        assert!(!state.is_active()); // Still needs from_entity_id

        state.from_entity_id = Some(Uuid::new_v4());
        assert!(state.is_active());
    }

    #[test]
    fn test_connection_state_reset() {
        let mut state = ConnectionDrawingState::default();
        state.is_drawing = true;
        state.from_entity_id = Some(Uuid::new_v4());
        state.drag_distance = 100.0;

        state.reset();

        assert!(!state.is_drawing);
        assert!(state.from_entity_id.is_none());
        assert_eq!(state.drag_distance, 0.0);
    }

    #[test]
    fn test_valid_drop_target() {
        let entity1 = Uuid::new_v4();
        let entity2 = Uuid::new_v4();

        let mut state = ConnectionDrawingState::default();
        state.is_drawing = true;
        state.from_entity_id = Some(entity1);
        state.from_port_type = Some(PortType::Output);

        // Can't connect to same entity
        assert!(!state.is_valid_drop_target(entity1, PortType::Input));

        // Can connect output -> input
        assert!(state.is_valid_drop_target(entity2, PortType::Input));

        // Can't connect output -> output
        assert!(!state.is_valid_drop_target(entity2, PortType::Output));
    }

    #[test]
    fn test_connection_result_normalized() {
        let entity1 = Uuid::new_v4();
        let entity2 = Uuid::new_v4();

        // Already normalized (output -> input)
        let result1 = ConnectionResult::new(entity1, entity2, PortType::Output, PortType::Input);
        let normalized1 = result1.normalized();
        assert_eq!(normalized1.from_entity_id, entity1);
        assert_eq!(normalized1.to_entity_id, entity2);

        // Needs normalization (input -> output)
        let result2 = ConnectionResult::new(entity1, entity2, PortType::Input, PortType::Output);
        let normalized2 = result2.normalized();
        assert_eq!(normalized2.from_entity_id, entity2);
        assert_eq!(normalized2.to_entity_id, entity1);
    }

    #[test]
    fn test_has_min_distance() {
        let mut state = ConnectionDrawingState::default();

        state.drag_distance = 5.0;
        assert!(!state.has_min_distance());

        state.drag_distance = MIN_DRAG_DISTANCE;
        assert!(state.has_min_distance());

        state.drag_distance = MIN_DRAG_DISTANCE + 10.0;
        assert!(state.has_min_distance());
    }

    #[test]
    fn test_distance() {
        let a = Position::new(0.0, 0.0);
        let b = Position::new(3.0, 4.0);
        assert!((distance(a, b) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_within_snap_distance() {
        let point = Position::new(0.0, 0.0);
        let near = Position::new(5.0, 5.0);
        let far = Position::new(100.0, 100.0);

        assert!(is_within_snap_distance(point, near));
        assert!(!is_within_snap_distance(point, far));
    }
}
