//! # Port Component
//!
//! Connection ports on entity cards for creating relationships.
//!
//! Ports are the visual connection points on entity cards that allow users
//! to draw relationships between entities. Each entity has:
//! - **Input ports** (left side) for incoming relationships
//! - **Output ports** (right side) for outgoing relationships
//!
//! ## Connection Flow
//!
//! 1. User clicks on an output port to start a connection
//! 2. A line is drawn from the port to the cursor
//! 3. User clicks on an input port of another entity to complete
//! 4. A relationship dialog appears to configure the relationship

use dioxus::prelude::*;
use imortal_ir::PortPosition;
use uuid::Uuid;

use crate::state::APP_STATE;

// ============================================================================
// Constants
// ============================================================================

/// Default port size in pixels
pub const PORT_SIZE: f32 = 12.0;

/// Port size when hovered
pub const PORT_SIZE_HOVER: f32 = 16.0;

/// Port border width
pub const PORT_BORDER_WIDTH: f32 = 2.0;

/// Port hit area padding (makes clicking easier)
pub const PORT_HIT_AREA_PADDING: f32 = 8.0;

// ============================================================================
// Port Type
// ============================================================================

/// Type of connection port
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortType {
    /// Input port (left side) - receives incoming connections
    Input,
    /// Output port (right side) - sends outgoing connections
    Output,
}

impl PortType {
    /// Get the CSS position class for this port type
    pub fn position_class(&self) -> &'static str {
        match self {
            PortType::Input => "left-0 -translate-x-1/2",
            PortType::Output => "right-0 translate-x-1/2",
        }
    }

    /// Get the tooltip text for this port type
    pub fn tooltip(&self) -> &'static str {
        match self {
            PortType::Input => "Input: Drop a connection here",
            PortType::Output => "Output: Drag to create a relationship",
        }
    }

    /// Convert to PortPosition for IR
    pub fn to_port_position(&self) -> PortPosition {
        match self {
            PortType::Input => PortPosition::Left,
            PortType::Output => PortPosition::Right,
        }
    }

    /// Create from PortPosition
    pub fn from_port_position(pos: PortPosition) -> Self {
        match pos {
            PortPosition::Left => PortType::Input,
            PortPosition::Right | PortPosition::Top | PortPosition::Bottom => PortType::Output,
        }
    }

    /// Get opposite port type
    pub fn opposite(&self) -> Self {
        match self {
            PortType::Input => PortType::Output,
            PortType::Output => PortType::Input,
        }
    }
}

// ============================================================================
// Port State
// ============================================================================

/// Visual state of a port
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortState {
    /// Normal idle state
    #[default]
    Idle,
    /// Mouse is hovering over the port
    Hovered,
    /// Port is the start of an active connection
    Connecting,
    /// Port is a valid drop target for current connection
    ValidTarget,
    /// Port is an invalid drop target (same entity, incompatible, etc.)
    InvalidTarget,
    /// Port is disabled (entity not selected, etc.)
    Disabled,
}

impl PortState {
    /// Get CSS classes for this state
    pub fn css_classes(&self) -> &'static str {
        match self {
            PortState::Idle => "bg-slate-600 border-slate-500",
            PortState::Hovered => "bg-indigo-500 border-indigo-400 scale-125",
            PortState::Connecting => "bg-indigo-600 border-indigo-400 scale-125 animate-pulse",
            PortState::ValidTarget => "bg-green-500 border-green-400 scale-125",
            PortState::InvalidTarget => "bg-red-500 border-red-400 opacity-50",
            PortState::Disabled => "bg-slate-700 border-slate-600 opacity-30",
        }
    }

    /// Check if port is interactive
    pub fn is_interactive(&self) -> bool {
        !matches!(self, PortState::Disabled | PortState::InvalidTarget)
    }
}

// ============================================================================
// Port Click Info
// ============================================================================

/// Information about a port click event
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortClickInfo {
    /// ID of the entity this port belongs to
    pub entity_id: Uuid,
    /// Type of port clicked
    pub port_type: PortType,
    /// Screen position of the port center
    pub screen_position: (f32, f32),
}

impl PortClickInfo {
    /// Create new port click info
    pub fn new(entity_id: Uuid, port_type: PortType, x: f32, y: f32) -> Self {
        Self {
            entity_id,
            port_type,
            screen_position: (x, y),
        }
    }
}

// ============================================================================
// Port Props
// ============================================================================

/// Properties for the Port component
#[derive(Props, Clone, PartialEq)]
pub struct PortProps {
    /// Entity this port belongs to
    pub entity_id: Uuid,

    /// Type of port (input/output)
    pub port_type: PortType,

    /// Whether the parent entity is selected
    #[props(default = false)]
    pub entity_selected: bool,

    /// Whether ports should be visible (show on hover or always when connecting)
    #[props(default = true)]
    pub visible: bool,

    /// Whether a connection is currently being drawn
    #[props(default = false)]
    pub is_connecting: bool,

    /// If connecting, the entity ID where the connection started
    #[props(default)]
    pub connection_start_entity: Option<Uuid>,

    /// Click callback - called when port is clicked
    #[props(default)]
    pub on_click: EventHandler<PortClickInfo>,

    /// Mouse enter callback
    #[props(default)]
    pub on_mouse_enter: EventHandler<PortClickInfo>,

    /// Mouse leave callback
    #[props(default)]
    pub on_mouse_leave: EventHandler<PortClickInfo>,
}

// ============================================================================
// Port Component
// ============================================================================

/// Connection port component
///
/// Renders a circular port on the side of an entity card that can be used
/// to create relationships between entities.
#[component]
pub fn Port(props: PortProps) -> Element {
    let mut is_hovered = use_signal(|| false);

    // Calculate port state
    let state = {
        if !props.visible {
            PortState::Disabled
        } else if props.is_connecting {
            // We're in connection mode
            if let Some(start_entity) = props.connection_start_entity {
                if start_entity == props.entity_id {
                    // This is the source entity
                    if props.port_type == PortType::Output {
                        PortState::Connecting
                    } else {
                        PortState::InvalidTarget
                    }
                } else {
                    // This is a potential target
                    if props.port_type == PortType::Input {
                        if *is_hovered.read() {
                            PortState::ValidTarget
                        } else {
                            PortState::Idle
                        }
                    } else {
                        PortState::InvalidTarget
                    }
                }
            } else {
                PortState::Idle
            }
        } else if *is_hovered.read() {
            PortState::Hovered
        } else {
            PortState::Idle
        }
    };

    let position_class = props.port_type.position_class();
    let state_classes = state.css_classes();
    let tooltip = props.port_type.tooltip();

    // Cursor style based on state
    let cursor = if state.is_interactive() {
        if props.port_type == PortType::Output {
            "cursor-crosshair"
        } else {
            "cursor-pointer"
        }
    } else {
        "cursor-not-allowed"
    };

    // Visibility classes
    let visibility_class = if props.visible {
        "opacity-100"
    } else {
        "opacity-0 pointer-events-none"
    };

    let entity_id = props.entity_id;
    let port_type = props.port_type;

    rsx! {
        button {
            class: "port absolute top-1/2 -translate-y-1/2 {position_class}
                    w-3 h-3 rounded-full border-2 {state_classes}
                    transition-all duration-150 z-30 {cursor} {visibility_class}
                    hover:ring-2 hover:ring-indigo-400/30",
            title: "{tooltip}",
            tabindex: "-1",
            disabled: !state.is_interactive(),

            onclick: move |e| {
                e.stop_propagation();
                if state.is_interactive() {
                    let rect = e.client_coordinates();
                    props.on_click.call(PortClickInfo::new(
                        entity_id,
                        port_type,
                        rect.x as f32,
                        rect.y as f32,
                    ));
                }
            },

            onmouseenter: move |e| {
                is_hovered.set(true);
                if state.is_interactive() {
                    let rect = e.client_coordinates();
                    props.on_mouse_enter.call(PortClickInfo::new(
                        entity_id,
                        port_type,
                        rect.x as f32,
                        rect.y as f32,
                    ));
                }
            },

            onmouseleave: move |e| {
                is_hovered.set(false);
                if state.is_interactive() {
                    let rect = e.client_coordinates();
                    props.on_mouse_leave.call(PortClickInfo::new(
                        entity_id,
                        port_type,
                        rect.x as f32,
                        rect.y as f32,
                    ));
                }
            },

            // Inner dot for visual feedback
            if matches!(state, PortState::Hovered | PortState::ValidTarget | PortState::Connecting) {
                span {
                    class: "absolute inset-1 rounded-full bg-white/50"
                }
            }
        }
    }
}

// ============================================================================
// Port Pair Component
// ============================================================================

/// Properties for the PortPair component
#[derive(Props, Clone, PartialEq)]
pub struct PortPairProps {
    /// Entity this port pair belongs to
    pub entity_id: Uuid,

    /// Whether the parent entity is selected
    #[props(default = false)]
    pub entity_selected: bool,

    /// Whether to show ports (typically on hover or when connecting)
    #[props(default = true)]
    pub show_ports: bool,

    /// Whether a connection is currently being drawn
    #[props(default = false)]
    pub is_connecting: bool,

    /// If connecting, the entity ID where the connection started
    #[props(default)]
    pub connection_start_entity: Option<Uuid>,

    /// Callback when any port is clicked
    #[props(default)]
    pub on_port_click: EventHandler<PortClickInfo>,
}

/// Port pair component - renders both input and output ports
///
/// This is a convenience component that renders both the input (left)
/// and output (right) ports for an entity card.
#[component]
pub fn PortPair(props: PortPairProps) -> Element {
    rsx! {
        // Input port (left side)
        Port {
            entity_id: props.entity_id,
            port_type: PortType::Input,
            entity_selected: props.entity_selected,
            visible: props.show_ports,
            is_connecting: props.is_connecting,
            connection_start_entity: props.connection_start_entity,
            on_click: move |info| props.on_port_click.call(info),
        }

        // Output port (right side)
        Port {
            entity_id: props.entity_id,
            port_type: PortType::Output,
            entity_selected: props.entity_selected,
            visible: props.show_ports,
            is_connecting: props.is_connecting,
            connection_start_entity: props.connection_start_entity,
            on_click: move |info| props.on_port_click.call(info),
        }
    }
}

// ============================================================================
// Field Port Component
// ============================================================================

/// Properties for field-level ports
#[derive(Props, Clone, PartialEq)]
pub struct FieldPortProps {
    /// Entity this port belongs to
    pub entity_id: Uuid,

    /// Field ID this port is for
    pub field_id: Uuid,

    /// Port type
    pub port_type: PortType,

    /// Whether visible
    #[props(default = false)]
    pub visible: bool,

    /// Click callback
    #[props(default)]
    pub on_click: EventHandler<(Uuid, Uuid, PortType)>,
}

/// Port attached to a specific field (for field-level relationships)
#[component]
pub fn FieldPort(props: FieldPortProps) -> Element {
    let mut is_hovered = use_signal(|| false);

    let position_class = props.port_type.position_class();

    let state_classes = if *is_hovered.read() {
        "bg-indigo-500 border-indigo-400 scale-125"
    } else {
        "bg-slate-600 border-slate-500"
    };

    let visibility_class = if props.visible {
        "opacity-100"
    } else {
        "opacity-0 group-hover:opacity-100"
    };

    let entity_id = props.entity_id;
    let field_id = props.field_id;
    let port_type = props.port_type;

    rsx! {
        button {
            class: "field-port absolute top-1/2 -translate-y-1/2 {position_class}
                    w-2 h-2 rounded-full border {state_classes}
                    transition-all duration-150 z-20 cursor-crosshair {visibility_class}",
            title: "Connect field",
            tabindex: "-1",

            onclick: move |e| {
                e.stop_propagation();
                props.on_click.call((entity_id, field_id, port_type));
            },

            onmouseenter: move |_| is_hovered.set(true),
            onmouseleave: move |_| is_hovered.set(false),
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Calculate the screen position of a port given entity position and zoom
pub fn calculate_port_position(
    entity_x: f32,
    entity_y: f32,
    entity_width: f32,
    entity_height: f32,
    port_type: PortType,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
) -> (f32, f32) {
    let (local_x, local_y) = match port_type {
        PortType::Input => (0.0, entity_height / 2.0),
        PortType::Output => (entity_width, entity_height / 2.0),
    };

    let screen_x = (entity_x + local_x) * zoom + pan_x;
    let screen_y = (entity_y + local_y) * zoom + pan_y;

    (screen_x, screen_y)
}

/// Check if a point is within a port's hit area
pub fn point_in_port_hit_area(
    point_x: f32,
    point_y: f32,
    port_x: f32,
    port_y: f32,
    zoom: f32,
) -> bool {
    let hit_radius = (PORT_SIZE / 2.0 + PORT_HIT_AREA_PADDING) * zoom;
    let dx = point_x - port_x;
    let dy = point_y - port_y;
    dx * dx + dy * dy <= hit_radius * hit_radius
}

/// Get connection start info from app state
pub fn get_connection_state() -> Option<(Uuid, PortType)> {
    let state = APP_STATE.read();
    if state.canvas.is_connecting {
        state
            .canvas
            .connection_start
            .as_ref()
            .map(|(entity_id, port)| {
                let port_type = match port {
                    crate::state::ConnectionPort::Input => PortType::Input,
                    crate::state::ConnectionPort::Output => PortType::Output,
                };
                (*entity_id, port_type)
            })
    } else {
        None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_type_position_class() {
        assert_eq!(PortType::Input.position_class(), "left-0 -translate-x-1/2");
        assert_eq!(PortType::Output.position_class(), "right-0 translate-x-1/2");
    }

    #[test]
    fn test_port_type_opposite() {
        assert_eq!(PortType::Input.opposite(), PortType::Output);
        assert_eq!(PortType::Output.opposite(), PortType::Input);
    }

    #[test]
    fn test_port_state_interactive() {
        assert!(PortState::Idle.is_interactive());
        assert!(PortState::Hovered.is_interactive());
        assert!(PortState::Connecting.is_interactive());
        assert!(PortState::ValidTarget.is_interactive());
        assert!(!PortState::InvalidTarget.is_interactive());
        assert!(!PortState::Disabled.is_interactive());
    }

    #[test]
    fn test_port_click_info() {
        let info = PortClickInfo::new(Uuid::nil(), PortType::Output, 100.0, 200.0);
        assert_eq!(info.entity_id, Uuid::nil());
        assert_eq!(info.port_type, PortType::Output);
        assert_eq!(info.screen_position, (100.0, 200.0));
    }

    #[test]
    fn test_calculate_port_position() {
        let (x, y) = calculate_port_position(
            100.0,
            100.0, // entity position
            200.0,
            100.0, // entity size
            PortType::Input,
            1.0, // zoom
            0.0,
            0.0, // pan
        );
        assert_eq!(x, 100.0); // left edge
        assert_eq!(y, 150.0); // center vertically

        let (x, y) =
            calculate_port_position(100.0, 100.0, 200.0, 100.0, PortType::Output, 1.0, 0.0, 0.0);
        assert_eq!(x, 300.0); // right edge
        assert_eq!(y, 150.0); // center vertically
    }

    #[test]
    fn test_point_in_port_hit_area() {
        // Point at port center
        assert!(point_in_port_hit_area(100.0, 100.0, 100.0, 100.0, 1.0));

        // Point just inside hit area
        assert!(point_in_port_hit_area(105.0, 100.0, 100.0, 100.0, 1.0));

        // Point far outside
        assert!(!point_in_port_hit_area(200.0, 200.0, 100.0, 100.0, 1.0));
    }

    #[test]
    fn test_port_type_to_port_position() {
        assert_eq!(PortType::Input.to_port_position(), PortPosition::Left);
        assert_eq!(PortType::Output.to_port_position(), PortPosition::Right);
    }

    #[test]
    fn test_port_type_from_port_position() {
        assert_eq!(
            PortType::from_port_position(PortPosition::Left),
            PortType::Input
        );
        assert_eq!(
            PortType::from_port_position(PortPosition::Right),
            PortType::Output
        );
        assert_eq!(
            PortType::from_port_position(PortPosition::Top),
            PortType::Output
        );
        assert_eq!(
            PortType::from_port_position(PortPosition::Bottom),
            PortType::Output
        );
    }
}
