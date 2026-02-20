//! # Connection Component
//!
//! SVG-based connection lines for visualizing relationships between entities.
//!
//! This module provides components for rendering:
//! - **Relationship lines**: Permanent connections between entities
//! - **Connection preview**: Temporary line while drawing a new connection
//! - **Arrow markers**: Visual indicators for relationship direction and type
//!
//! ## Connection Types
//!
//! Different relationship types are rendered with distinct visual styles:
//! - **One-to-One**: Single line with single arrow
//! - **One-to-Many**: Single line with crow's foot marker
//! - **Many-to-One**: Single line with crow's foot on source
//! - **Many-to-Many**: Dashed line with crow's foot on both ends

use dioxus::prelude::*;
use imortal_core::{Position, RelationType};
use imortal_ir::{PortPosition, Relationship};
use uuid::Uuid;

use crate::components::entity_card::{CARD_HEADER_HEIGHT, CARD_WIDTH, FIELD_ROW_HEIGHT};
use crate::state::APP_STATE;

// ============================================================================
// Constants
// ============================================================================

/// Stroke width for normal connection lines
pub const STROKE_WIDTH: f32 = 2.0;

/// Stroke width for selected connection lines
pub const STROKE_WIDTH_SELECTED: f32 = 3.0;

/// Stroke width for hover state
pub const STROKE_WIDTH_HOVER: f32 = 2.5;

/// Control point offset for bezier curves (as fraction of distance)
pub const BEZIER_CONTROL_OFFSET: f32 = 0.5;

/// Minimum control point distance
pub const MIN_CONTROL_DISTANCE: f32 = 50.0;

/// Arrow marker size
pub const ARROW_SIZE: f32 = 10.0;

/// Crow's foot marker size
pub const CROWS_FOOT_SIZE: f32 = 12.0;

/// Hit area padding for selection (pixels)
pub const HIT_AREA_PADDING: f32 = 8.0;

/// Animation duration for hover effects (ms)
pub const HOVER_TRANSITION_MS: u32 = 150;

// ============================================================================
// Connection Colors
// ============================================================================

/// Colors for different connection states and types
pub mod colors {
    /// Default connection color
    pub const DEFAULT: &str = "#64748b"; // slate-500

    /// Selected connection color
    pub const SELECTED: &str = "#6366f1"; // indigo-500

    /// Hovered connection color
    pub const HOVERED: &str = "#818cf8"; // indigo-400

    /// Preview connection color (while drawing)
    pub const PREVIEW: &str = "#a5b4fc"; // indigo-300

    /// Invalid connection color
    pub const INVALID: &str = "#ef4444"; // red-500

    /// One-to-one relationship color
    pub const ONE_TO_ONE: &str = "#22c55e"; // green-500

    /// One-to-many relationship color
    pub const ONE_TO_MANY: &str = "#3b82f6"; // blue-500

    /// Many-to-many relationship color
    pub const MANY_TO_MANY: &str = "#f59e0b"; // amber-500
}

// ============================================================================
// Connection Point
// ============================================================================

/// A point on a connection line
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionPoint {
    pub x: f32,
    pub y: f32,
}

impl ConnectionPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_position(pos: Position) -> Self {
        Self { x: pos.x, y: pos.y }
    }

    pub fn distance_to(&self, other: &ConnectionPoint) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn midpoint(&self, other: &ConnectionPoint) -> ConnectionPoint {
        ConnectionPoint {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }
}

impl From<(f32, f32)> for ConnectionPoint {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

// ============================================================================
// Bezier Path Calculation
// ============================================================================

/// Calculate bezier curve control points for a smooth connection
pub fn calculate_bezier_path(
    start: ConnectionPoint,
    end: ConnectionPoint,
    start_port: PortPosition,
    end_port: PortPosition,
) -> (ConnectionPoint, ConnectionPoint) {
    let distance = start.distance_to(&end);
    let control_offset = (distance * BEZIER_CONTROL_OFFSET).max(MIN_CONTROL_DISTANCE);

    let control1 = match start_port {
        PortPosition::Right => ConnectionPoint::new(start.x + control_offset, start.y),
        PortPosition::Left => ConnectionPoint::new(start.x - control_offset, start.y),
        PortPosition::Top => ConnectionPoint::new(start.x, start.y - control_offset),
        PortPosition::Bottom => ConnectionPoint::new(start.x, start.y + control_offset),
    };

    let control2 = match end_port {
        PortPosition::Right => ConnectionPoint::new(end.x + control_offset, end.y),
        PortPosition::Left => ConnectionPoint::new(end.x - control_offset, end.y),
        PortPosition::Top => ConnectionPoint::new(end.x, end.y - control_offset),
        PortPosition::Bottom => ConnectionPoint::new(end.x, end.y + control_offset),
    };

    (control1, control2)
}

/// Generate SVG path data for a bezier curve
pub fn bezier_path_data(
    start: ConnectionPoint,
    end: ConnectionPoint,
    control1: ConnectionPoint,
    control2: ConnectionPoint,
) -> String {
    format!(
        "M {},{} C {},{} {},{} {},{}",
        start.x, start.y, control1.x, control1.y, control2.x, control2.y, end.x, end.y
    )
}

/// Generate a simple curved path (quadratic bezier)
pub fn simple_curve_path(start: ConnectionPoint, end: ConnectionPoint) -> String {
    let mid = start.midpoint(&end);
    let control = ConnectionPoint::new(
        mid.x,
        if (end.x - start.x).abs() > (end.y - start.y).abs() {
            mid.y
        } else {
            start.y + (end.y - start.y) * 0.5
        },
    );

    format!(
        "M {},{} Q {},{} {},{}",
        start.x, start.y, control.x, control.y, end.x, end.y
    )
}

// ============================================================================
// Port Position Calculation
// ============================================================================

/// Calculate the actual position of a port on an entity card
pub fn calculate_port_position(
    entity_pos: Position,
    port: PortPosition,
    card_width: f64,
    card_height: f64,
) -> ConnectionPoint {
    match port {
        PortPosition::Left => {
            ConnectionPoint::new(entity_pos.x, entity_pos.y + card_height as f32 / 2.0)
        }
        PortPosition::Right => ConnectionPoint::new(
            entity_pos.x + card_width as f32,
            entity_pos.y + card_height as f32 / 2.0,
        ),
        PortPosition::Top => {
            ConnectionPoint::new(entity_pos.x + card_width as f32 / 2.0, entity_pos.y)
        }
        PortPosition::Bottom => ConnectionPoint::new(
            entity_pos.x + card_width as f32 / 2.0,
            entity_pos.y + card_height as f32,
        ),
    }
}

/// Estimate entity card height based on field count
pub fn estimate_card_height(field_count: usize, collapsed: bool) -> f64 {
    if collapsed {
        CARD_HEADER_HEIGHT + 8.0 // Just header + padding
    } else {
        CARD_HEADER_HEIGHT + (field_count as f64 * FIELD_ROW_HEIGHT) + 40.0 // header + fields + footer
    }
}

// ============================================================================
// Connection Line Props
// ============================================================================

/// Properties for the ConnectionLine component
#[derive(Props, Clone, PartialEq)]
pub struct ConnectionLineProps {
    /// The relationship to render
    pub relationship: Relationship,

    /// Zoom level of the canvas
    #[props(default = 1.0)]
    pub zoom: f32,

    /// Pan offset X
    #[props(default = 0.0)]
    pub pan_x: f32,

    /// Pan offset Y
    #[props(default = 0.0)]
    pub pan_y: f32,

    /// Whether this connection is selected
    #[props(default = false)]
    pub selected: bool,

    /// Click callback
    #[props(default)]
    pub on_click: EventHandler<Uuid>,

    /// Double-click callback (for editing)
    #[props(default)]
    pub on_double_click: EventHandler<Uuid>,

    /// Context menu callback
    #[props(default)]
    pub on_context_menu: EventHandler<Uuid>,
}

// ============================================================================
// Connection Line Component
// ============================================================================

/// Renders a relationship connection line between two entities
#[component]
pub fn ConnectionLine(props: ConnectionLineProps) -> Element {
    let mut is_hovered = use_signal(|| false);
    let relationship = &props.relationship;
    let relationship_id = relationship.id;

    // Get entity positions from state
    let state = APP_STATE.read();
    let project = state.project.as_ref();

    let (start_pos, end_pos, from_height, to_height) = if let Some(project) = project {
        let from_entity = project.entities.get(&relationship.from_entity_id);
        let to_entity = project.entities.get(&relationship.to_entity_id);

        match (from_entity, to_entity) {
            (Some(from), Some(to)) => {
                let from_height = estimate_card_height(from.fields.len(), from.collapsed);
                let to_height = estimate_card_height(to.fields.len(), to.collapsed);
                (from.position, to.position, from_height, to_height)
            }
            _ => return rsx! {}, // Entities not found
        }
    } else {
        return rsx! {}; // No project
    };
    drop(state);

    // Calculate port positions
    let start = calculate_port_position(
        start_pos,
        relationship.from_port.clone(),
        CARD_WIDTH,
        from_height,
    );
    let end = calculate_port_position(end_pos, relationship.to_port.clone(), CARD_WIDTH, to_height);

    // Calculate bezier control points
    let (control1, control2) =
        calculate_bezier_path(start, end, relationship.from_port, relationship.to_port);

    // Generate path data
    let path_data = bezier_path_data(start, end, control1, control2);

    // Determine visual properties based on state
    let (stroke_color, stroke_width) = if props.selected {
        (colors::SELECTED, STROKE_WIDTH_SELECTED)
    } else if *is_hovered.read() {
        (colors::HOVERED, STROKE_WIDTH_HOVER)
    } else {
        (
            relationship_color(&relationship.relation_type),
            STROKE_WIDTH,
        )
    };

    // Dash pattern for many-to-many
    let stroke_dasharray = if matches!(relationship.relation_type, RelationType::ManyToMany { .. })
    {
        "8,4"
    } else {
        ""
    };

    // Marker IDs
    let marker_id = format!("arrow-{}", relationship_id);
    let _marker_url = format!("url(#{})", marker_id);

    rsx! {
        g {
            class: "connection-group",
            "data-relationship-id": "{relationship_id}",

            // Arrow marker definition
            defs {
                marker {
                    id: "{marker_id}",
                    marker_width: "{ARROW_SIZE}",
                    marker_height: "{ARROW_SIZE}",
                    ref_x: "{ARROW_SIZE - 2.0}",
                    ref_y: "{ARROW_SIZE / 2.0}",
                    orient: "auto",
                    marker_units: "userSpaceOnUse",

                    path {
                        d: "M 0 0 L {ARROW_SIZE} {ARROW_SIZE / 2.0} L 0 {ARROW_SIZE} Z",
                        fill: "{stroke_color}",
                    }
                }

                // Crow's foot marker for one-to-many
                if matches!(relationship.relation_type, RelationType::OneToMany | RelationType::ManyToMany { .. }) {
                    marker {
                        id: "crows-foot-{relationship_id}",
                        marker_width: "{CROWS_FOOT_SIZE}",
                        marker_height: "{CROWS_FOOT_SIZE}",
                        ref_x: "0",
                        ref_y: "{CROWS_FOOT_SIZE / 2.0}",
                        orient: "auto",
                        marker_units: "userSpaceOnUse",

                        // Crow's foot shape (three lines spreading out)
                        path {
                            d: "M 0,{CROWS_FOOT_SIZE / 2.0} L {CROWS_FOOT_SIZE},{CROWS_FOOT_SIZE * 0.1} M 0,{CROWS_FOOT_SIZE / 2.0} L {CROWS_FOOT_SIZE},{CROWS_FOOT_SIZE / 2.0} M 0,{CROWS_FOOT_SIZE / 2.0} L {CROWS_FOOT_SIZE},{CROWS_FOOT_SIZE * 0.9}",
                            stroke: "{stroke_color}",
                            stroke_width: "2",
                            fill: "none",
                        }
                    }
                }
            }

            // Invisible wider path for easier clicking
            path {
                d: "{path_data}",
                stroke: "transparent",
                stroke_width: "{HIT_AREA_PADDING * 2.0}",
                fill: "none",
                cursor: "pointer",

                onclick: move |e| {
                    e.stop_propagation();
                    props.on_click.call(relationship_id);
                },
                ondoubleclick: move |e| {
                    e.stop_propagation();
                    props.on_double_click.call(relationship_id);
                },
                oncontextmenu: move |e| {
                    e.prevent_default();
                    e.stop_propagation();
                    props.on_context_menu.call(relationship_id);
                },
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| is_hovered.set(false),
            }

            // Visible connection line
            path {
                d: "{path_data}",
                stroke: "{stroke_color}",
                stroke_width: "{stroke_width}",
                stroke_dasharray: "{stroke_dasharray}",
                fill: "none",
                pointer_events: "none",
                marker_end: "url(#{marker_id})",

                // Animation class for transitions
                class: "transition-all duration-150",
            }

            // Relationship label at midpoint
            if props.selected || *is_hovered.read() {
                {
                    let mid = start.midpoint(&end);
                    rsx! {
                        g {
                            transform: "translate({mid.x}, {mid.y})",

                            // Background for label
                            rect {
                                x: "-40",
                                y: "-10",
                                width: "80",
                                height: "20",
                                rx: "4",
                                fill: "#1e293b",
                                stroke: "{stroke_color}",
                                stroke_width: "1",
                            }

                            // Label text
                            text {
                                x: "0",
                                y: "5",
                                text_anchor: "middle",
                                font_size: "11",
                                fill: "#e2e8f0",
                                font_family: "ui-monospace, monospace",
                                "{relationship.display_label()}"
                            }
                        }
                    }
                }
            }

            // Selection indicator (circles at endpoints)
            if props.selected {
                circle {
                    cx: "{start.x}",
                    cy: "{start.y}",
                    r: "6",
                    fill: "{colors::SELECTED}",
                    stroke: "white",
                    stroke_width: "2",
                }
                circle {
                    cx: "{end.x}",
                    cy: "{end.y}",
                    r: "6",
                    fill: "{colors::SELECTED}",
                    stroke: "white",
                    stroke_width: "2",
                }
            }
        }
    }
}

// ============================================================================
// Connection Preview Props
// ============================================================================

/// Properties for the ConnectionPreview component
#[derive(Props, Clone, PartialEq)]
pub struct ConnectionPreviewProps {
    /// Start position (from port)
    pub start: ConnectionPoint,

    /// Current mouse/end position
    pub end: ConnectionPoint,

    /// Port position on start entity
    #[props(default = PortPosition::Right)]
    pub start_port: PortPosition,

    /// Whether the current target is valid
    #[props(default = true)]
    pub is_valid: bool,
}

// ============================================================================
// Connection Preview Component
// ============================================================================

/// Renders a preview connection line while drawing a new relationship
#[component]
pub fn ConnectionPreview(props: ConnectionPreviewProps) -> Element {
    let color = if props.is_valid {
        colors::PREVIEW
    } else {
        colors::INVALID
    };

    // Calculate control points (simpler for preview)
    let (control1, control2) = calculate_bezier_path(
        props.start,
        props.end,
        props.start_port,
        props.start_port.opposite(),
    );

    let path_data = bezier_path_data(props.start, props.end, control1, control2);

    rsx! {
        g {
            class: "connection-preview",

            // Animated dashed line
            path {
                d: "{path_data}",
                stroke: "{color}",
                stroke_width: "{STROKE_WIDTH}",
                stroke_dasharray: "6,3",
                fill: "none",
                pointer_events: "none",
                opacity: "0.8",

                // Animation for the dashes
                style: "animation: dash-flow 0.5s linear infinite;",
            }

            // End point indicator
            circle {
                cx: "{props.end.x}",
                cy: "{props.end.y}",
                r: "5",
                fill: "{color}",
                opacity: "0.6",
            }
        }
    }
}

// ============================================================================
// Connections Layer Props
// ============================================================================

/// Properties for the ConnectionsLayer component
#[derive(Props, Clone, PartialEq)]
pub struct ConnectionsLayerProps {
    /// Zoom level
    #[props(default = 1.0)]
    pub zoom: f32,

    /// Pan X offset
    #[props(default = 0.0)]
    pub pan_x: f32,

    /// Pan Y offset
    #[props(default = 0.0)]
    pub pan_y: f32,

    /// Preview connection (while drawing)
    #[props(default)]
    pub preview: Option<(ConnectionPoint, ConnectionPoint, bool)>,

    /// Callback when a connection is clicked
    #[props(default)]
    pub on_connection_click: EventHandler<Uuid>,

    /// Callback when a connection is double-clicked
    #[props(default)]
    pub on_connection_double_click: EventHandler<Uuid>,
}

// ============================================================================
// Connections Layer Component
// ============================================================================

/// SVG layer containing all relationship connection lines
#[component]
pub fn ConnectionsLayer(props: ConnectionsLayerProps) -> Element {
    let state = APP_STATE.read();

    let relationships: Vec<Relationship> = state
        .project
        .as_ref()
        .map(|p| p.relationships.values().cloned().collect())
        .unwrap_or_default();

    let selected_relationships = state.selection.relationships.clone();
    drop(state);

    rsx! {
        svg {
            class: "connections-layer absolute inset-0 pointer-events-none overflow-visible",
            style: "z-index: 5;",

            // CSS for dash animation
            style {
                r#"
                @keyframes dash-flow {{
                    to {{
                        stroke-dashoffset: -18;
                    }}
                }}
                "#
            }

            // Transform group for zoom/pan
            g {
                transform: "translate({props.pan_x}, {props.pan_y}) scale({props.zoom})",

                // Render all relationship lines
                for relationship in relationships.iter() {
                    ConnectionLine {
                        key: "{relationship.id}",
                        relationship: relationship.clone(),
                        zoom: props.zoom,
                        pan_x: props.pan_x,
                        pan_y: props.pan_y,
                        selected: selected_relationships.contains(&relationship.id),
                        on_click: move |id| props.on_connection_click.call(id),
                        on_double_click: move |id| props.on_connection_double_click.call(id),
                    }
                }
            }

            // Preview connection (drawn in screen space, not transformed)
            if let Some((start, end, is_valid)) = props.preview {
                ConnectionPreview {
                    start: start,
                    end: end,
                    is_valid: is_valid,
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get color for a relationship type
pub fn relationship_color(relation_type: &RelationType) -> &'static str {
    match relation_type {
        RelationType::OneToOne => colors::ONE_TO_ONE,
        RelationType::OneToMany | RelationType::ManyToOne => colors::ONE_TO_MANY,
        RelationType::ManyToMany { .. } => colors::MANY_TO_MANY,
    }
}

/// Get a human-readable label for a relationship type
pub fn relationship_type_label(relation_type: &RelationType) -> &'static str {
    match relation_type {
        RelationType::OneToOne => "1:1",
        RelationType::OneToMany => "1:N",
        RelationType::ManyToOne => "N:1",
        RelationType::ManyToMany { .. } => "N:M",
    }
}

/// Check if a point is near a bezier curve (for hit testing)
pub fn point_near_bezier(
    point: ConnectionPoint,
    start: ConnectionPoint,
    end: ConnectionPoint,
    control1: ConnectionPoint,
    control2: ConnectionPoint,
    tolerance: f32,
) -> bool {
    // Sample points along the curve and check distance
    const SAMPLES: usize = 20;

    for i in 0..=SAMPLES {
        let t = i as f32 / SAMPLES as f32;
        let curve_point = bezier_point(t, start, control1, control2, end);

        if point.distance_to(&curve_point) <= tolerance {
            return true;
        }
    }

    false
}

/// Calculate a point on a cubic bezier curve
fn bezier_point(
    t: f32,
    p0: ConnectionPoint,
    p1: ConnectionPoint,
    p2: ConnectionPoint,
    p3: ConnectionPoint,
) -> ConnectionPoint {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    ConnectionPoint {
        x: mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        y: mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    }
}

/// Find the closest point on a bezier curve to a given point
pub fn closest_point_on_bezier(
    point: ConnectionPoint,
    start: ConnectionPoint,
    end: ConnectionPoint,
    control1: ConnectionPoint,
    control2: ConnectionPoint,
) -> (ConnectionPoint, f32) {
    const SAMPLES: usize = 50;
    let mut closest = start;
    let mut min_dist = f32::MAX;
    let mut closest_t = 0.0;

    for i in 0..=SAMPLES {
        let t = i as f32 / SAMPLES as f32;
        let curve_point = bezier_point(t, start, control1, control2, end);
        let dist = point.distance_to(&curve_point);

        if dist < min_dist {
            min_dist = dist;
            closest = curve_point;
            closest_t = t;
        }
    }

    (closest, closest_t)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_point_distance() {
        let p1 = ConnectionPoint::new(0.0, 0.0);
        let p2 = ConnectionPoint::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_connection_point_midpoint() {
        let p1 = ConnectionPoint::new(0.0, 0.0);
        let p2 = ConnectionPoint::new(10.0, 10.0);
        let mid = p1.midpoint(&p2);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 5.0);
    }

    #[test]
    fn test_calculate_port_position() {
        let entity_pos = Position::new(100.0, 100.0);
        let card_width = 200.0_f64;
        let card_height = 100.0_f64;

        let left = calculate_port_position(entity_pos, PortPosition::Left, card_width, card_height);
        assert_eq!(left.x, 100.0);
        assert_eq!(left.y, 150.0);

        let right =
            calculate_port_position(entity_pos, PortPosition::Right, card_width, card_height);
        assert_eq!(right.x, 300.0);
        assert_eq!(right.y, 150.0);

        let top = calculate_port_position(entity_pos, PortPosition::Top, card_width, card_height);
        assert_eq!(top.x, 200.0);
        assert_eq!(top.y, 100.0);

        let bottom =
            calculate_port_position(entity_pos, PortPosition::Bottom, card_width, card_height);
        assert_eq!(bottom.x, 200.0);
        assert_eq!(bottom.y, 200.0);
    }

    #[test]
    fn test_bezier_path_data() {
        let start = ConnectionPoint::new(0.0, 0.0);
        let end = ConnectionPoint::new(100.0, 100.0);
        let control1 = ConnectionPoint::new(50.0, 0.0);
        let control2 = ConnectionPoint::new(50.0, 100.0);

        let path = bezier_path_data(start, end, control1, control2);
        assert!(path.starts_with("M 0,0 C"));
        assert!(path.contains("100,100"));
    }

    #[test]
    fn test_relationship_color() {
        assert_eq!(
            relationship_color(&RelationType::OneToOne),
            colors::ONE_TO_ONE
        );
        assert_eq!(
            relationship_color(&RelationType::OneToMany),
            colors::ONE_TO_MANY
        );
        assert_eq!(
            relationship_color(&RelationType::ManyToOne),
            colors::ONE_TO_MANY
        );
        assert_eq!(
            relationship_color(&RelationType::ManyToMany {
                junction_table: String::new()
            }),
            colors::MANY_TO_MANY
        );
    }

    #[test]
    fn test_relationship_type_label() {
        assert_eq!(relationship_type_label(&RelationType::OneToOne), "1:1");
        assert_eq!(relationship_type_label(&RelationType::OneToMany), "1:N");
        assert_eq!(relationship_type_label(&RelationType::ManyToOne), "N:1");
        assert_eq!(
            relationship_type_label(&RelationType::ManyToMany {
                junction_table: String::new()
            }),
            "N:M"
        );
    }

    #[test]
    fn test_bezier_point() {
        let p0 = ConnectionPoint::new(0.0, 0.0);
        let p1 = ConnectionPoint::new(0.0, 50.0);
        let p2 = ConnectionPoint::new(100.0, 50.0);
        let p3 = ConnectionPoint::new(100.0, 100.0);

        // At t=0, should be at start
        let start = bezier_point(0.0, p0, p1, p2, p3);
        assert!((start.x - p0.x).abs() < 0.001);
        assert!((start.y - p0.y).abs() < 0.001);

        // At t=1, should be at end
        let end = bezier_point(1.0, p0, p1, p2, p3);
        assert!((end.x - p3.x).abs() < 0.001);
        assert!((end.y - p3.y).abs() < 0.001);
    }

    #[test]
    fn test_estimate_card_height() {
        let collapsed_height = estimate_card_height(5, true);
        let expanded_height = estimate_card_height(5, false);

        assert!(collapsed_height < expanded_height);
        assert!(expanded_height > CARD_HEADER_HEIGHT);
    }

    #[test]
    fn test_point_from_tuple() {
        let point: ConnectionPoint = (10.0, 20.0).into();
        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);
    }
}
