//! # Entity Card Component
//!
//! Visual representation of a database entity on the canvas.
//!
//! The entity card displays:
//! - Entity name and icon
//! - Table name (database)
//! - List of fields with their types
//! - Connection ports for relationships
//! - Selection and drag states
//!
//! ## Features
//!
//! - Collapsible to show/hide fields
//! - Draggable positioning on canvas
//! - Selection highlighting
//! - Hover states
//! - Context menu support
//! - Connection ports on left/right sides
//!

use dioxus::prelude::*;
use imortal_core::types::{EntityId, FieldId, Position};
use imortal_ir::entity::Entity;

use crate::components::field_row::FieldList;
use crate::components::port::{Port, PortClickInfo, PortType};

// ============================================================================
// Constants
// ============================================================================

/// Default card width
pub const CARD_WIDTH: f64 = 240.0;

/// Card header height
pub const CARD_HEADER_HEIGHT: f64 = 44.0;

/// Field row height
pub const FIELD_ROW_HEIGHT: f64 = 32.0;

/// Card footer height (add field button)
pub const CARD_FOOTER_HEIGHT: f64 = 36.0;

/// Port size (diameter)
pub const PORT_SIZE: f64 = 12.0;

/// Maximum visible fields before scrolling
pub const MAX_VISIBLE_FIELDS: usize = 8;

// ============================================================================
// Entity Card Component
// ============================================================================

/// Properties for the EntityCard component
#[derive(Props, Clone, PartialEq)]
pub struct EntityCardProps {
    /// The entity to display
    pub entity: Entity,

    /// Current zoom level (for scaling)
    #[props(default = 1.0)]
    pub zoom: f64,

    /// Whether this entity is currently selected
    #[props(default = false)]
    pub selected: bool,

    /// Whether this entity is being dragged
    #[props(default = false)]
    pub dragging: bool,

    /// Currently selected field ID within this entity
    #[props(default)]
    pub selected_field: Option<FieldId>,

    /// Whether to show connection ports
    #[props(default = true)]
    pub show_ports: bool,

    /// Whether the entity card should be interactive
    #[props(default = true)]
    pub interactive: bool,

    /// Callback when entity is selected (clicked)
    /// The bool indicates if shift key was held (for multi-select)
    #[props(default)]
    pub on_select: EventHandler<(EntityId, bool)>,

    /// Callback when drag starts
    #[props(default)]
    pub on_drag_start: EventHandler<(EntityId, Position)>,

    /// Callback when entity is double-clicked (for editing)
    #[props(default)]
    pub on_double_click: EventHandler<EntityId>,

    /// Callback for context menu
    #[props(default)]
    pub on_context_menu: EventHandler<(EntityId, MouseEvent)>,

    /// Callback when a field is selected
    #[props(default)]
    pub on_field_select: EventHandler<(EntityId, FieldId)>,

    /// Callback when a field is double-clicked
    #[props(default)]
    pub on_field_double_click: EventHandler<(EntityId, FieldId)>,

    /// Callback when add field button is clicked
    #[props(default)]
    pub on_add_field: EventHandler<EntityId>,

    /// Whether a connection is currently being drawn
    #[props(default = false)]
    pub is_connecting: bool,

    /// If connecting, the entity ID where the connection started
    #[props(default)]
    pub connection_start_entity: Option<EntityId>,

    /// Callback when any port is clicked (for connections)
    #[props(default)]
    pub on_port_click: EventHandler<PortClickInfo>,

    /// Callback when port is hovered (for connection preview)
    #[props(default)]
    pub on_port_hover: EventHandler<Option<PortClickInfo>>,

    /// Callback when collapse/expand is toggled
    #[props(default)]
    pub on_toggle_collapse: EventHandler<EntityId>,
}

/// Entity card component for the visual canvas
#[component]
pub fn EntityCard(props: EntityCardProps) -> Element {
    let entity = &props.entity;
    let entity_id = entity.id;
    let selected = props.selected;
    let dragging = props.dragging;
    let collapsed = entity.collapsed;
    let interactive = props.interactive;

    // Calculate position style
    let x = entity.position.x;
    let y = entity.position.y;
    let width = entity.size.width;

    // Build card classes
    let card_class = build_card_class(selected, dragging);

    // Calculate height based on content
    let content_height = if collapsed {
        CARD_HEADER_HEIGHT + 8.0 // Just header with padding
    } else {
        let field_count = entity.fields.len().min(MAX_VISIBLE_FIELDS);
        CARD_HEADER_HEIGHT + (field_count as f64 * FIELD_ROW_HEIGHT) + CARD_FOOTER_HEIGHT + 8.0
    };

    rsx! {
        div {
            class: "{card_class}",
            style: "position: absolute; left: {x}px; top: {y}px; width: {width}px; min-height: {content_height}px;",

            // Prevent text selection during drag
            draggable: false,

            // Click to select (with shift for multi-select)
            onclick: move |e| {
                if interactive {
                    e.stop_propagation();
                    let shift_held = e.modifiers().shift();
                    props.on_select.call((entity_id, shift_held));
                }
            },

            // Double-click to edit
            ondoubleclick: move |e| {
                if interactive {
                    e.stop_propagation();
                    props.on_double_click.call(entity_id);
                }
            },

            // Context menu
            oncontextmenu: move |e| {
                if interactive {
                    e.prevent_default();
                    e.stop_propagation();
                    props.on_context_menu.call((entity_id, e));
                }
            },

            // Drag start on mouse down (header area)
            onmousedown: move |e| {
                if interactive {
                    // Only start drag from header area (check if we're in header)
                    let pos = Position::new(
                        e.client_coordinates().x as f32,
                        e.client_coordinates().y as f32
                    );
                    props.on_drag_start.call((entity_id, pos));
                }
            },

            // Left port (input)
            if props.show_ports {
                Port {
                    entity_id: entity_id,
                    port_type: PortType::Input,
                    entity_selected: selected,
                    visible: true,
                    is_connecting: props.is_connecting,
                    connection_start_entity: props.connection_start_entity,
                    on_click: move |info| props.on_port_click.call(info),
                    on_mouse_enter: move |info| props.on_port_hover.call(Some(info)),
                    on_mouse_leave: move |_info| props.on_port_hover.call(None),
                }
            }

            // Right port (output)
            if props.show_ports {
                Port {
                    entity_id: entity_id,
                    port_type: PortType::Output,
                    entity_selected: selected,
                    visible: true,
                    is_connecting: props.is_connecting,
                    connection_start_entity: props.connection_start_entity,
                    on_click: move |info| props.on_port_click.call(info),
                    on_mouse_enter: move |info| props.on_port_hover.call(Some(info)),
                    on_mouse_leave: move |_info| props.on_port_hover.call(None),
                }
            }

            // Card content
            div {
                class: "relative z-10",

                // Header
                EntityCardHeader {
                    entity: entity.clone(),
                    selected: selected,
                    collapsed: collapsed,
                    on_toggle_collapse: move |_| props.on_toggle_collapse.call(entity_id),
                }

                // Body (fields)
                if !collapsed {
                    EntityCardBody {
                        entity: entity.clone(),
                        selected_field: props.selected_field,
                        on_field_click: move |field_id| {
                            props.on_field_select.call((entity_id, field_id));
                        },
                        on_field_double_click: move |field_id| {
                            props.on_field_double_click.call((entity_id, field_id));
                        },
                    }

                    // Footer (add field button)
                    EntityCardFooter {
                        entity_id: entity_id,
                        on_add_field: move |id| props.on_add_field.call(id),
                    }
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Build the CSS class string for the card
fn build_card_class(selected: bool, dragging: bool) -> String {
    let mut classes = vec![
        "entity-card",
        "relative",
        "bg-slate-800",
        "rounded-xl",
        "shadow-lg",
        "border-2",
        "overflow-hidden",
        "select-none",
        "transition-all",
        "duration-150",
    ];

    if selected {
        classes.push("border-indigo-500");
        classes.push("ring-2");
        classes.push("ring-indigo-500/30");
        classes.push("shadow-indigo-500/20");
        classes.push("shadow-xl");
    } else {
        classes.push("border-slate-700");
        classes.push("hover:border-slate-600");
        classes.push("hover:shadow-xl");
    }

    if dragging {
        classes.push("cursor-grabbing");
        classes.push("opacity-90");
        classes.push("scale-[1.02]");
    } else {
        classes.push("cursor-grab");
    }

    classes.join(" ")
}

// ============================================================================
// Entity Card Header
// ============================================================================

/// Properties for EntityCardHeader
#[derive(Props, Clone, PartialEq)]
struct EntityCardHeaderProps {
    /// The entity
    entity: Entity,

    /// Whether selected
    #[props(default = false)]
    selected: bool,

    /// Whether collapsed
    #[props(default = false)]
    collapsed: bool,

    /// Callback for collapse toggle
    on_toggle_collapse: EventHandler<()>,
}

/// Header section of entity card
#[component]
fn EntityCardHeader(props: EntityCardHeaderProps) -> Element {
    let entity = &props.entity;
    let selected = props.selected;
    let collapsed = props.collapsed;

    let header_class = if selected {
        "entity-card-header flex items-center gap-2 px-3 py-2.5 bg-indigo-600/20 border-b border-indigo-500/30"
    } else {
        "entity-card-header flex items-center gap-2 px-3 py-2.5 bg-slate-700/50 border-b border-slate-700"
    };

    rsx! {
        div {
            class: "{header_class}",

            // Entity icon
            div {
                class: "w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-sm font-bold shadow-md",
                "{entity.name.chars().next().unwrap_or('E').to_uppercase()}"
            }

            // Entity info
            div {
                class: "flex-1 min-w-0",

                // Name
                h3 {
                    class: "font-semibold text-sm text-slate-100 truncate",
                    title: "{entity.name}",
                    "{entity.name}"
                }

                // Table name
                p {
                    class: "text-xs text-slate-400 truncate",
                    title: "{entity.table_name}",
                    "📊 {entity.table_name}"
                }
            }

            // Collapse toggle
            button {
                class: "w-6 h-6 flex items-center justify-center rounded hover:bg-slate-600/50 text-slate-400 hover:text-slate-200 transition-colors",
                title: if collapsed { "Expand" } else { "Collapse" },
                onclick: move |e| {
                    e.stop_propagation();
                    props.on_toggle_collapse.call(());
                },
                if collapsed {
                    "▶"
                } else {
                    "▼"
                }
            }
        }
    }
}

// ============================================================================
// Entity Card Body
// ============================================================================

/// Properties for EntityCardBody
#[derive(Props, Clone, PartialEq)]
struct EntityCardBodyProps {
    /// The entity
    entity: Entity,

    /// Selected field ID
    #[props(default)]
    selected_field: Option<FieldId>,

    /// Field click callback
    on_field_click: EventHandler<FieldId>,

    /// Field double click callback
    on_field_double_click: EventHandler<FieldId>,
}

/// Body section with fields list
#[component]
fn EntityCardBody(props: EntityCardBodyProps) -> Element {
    let entity = &props.entity;
    let fields: Vec<_> = entity.sorted_fields().into_iter().cloned().collect();

    rsx! {
        div {
            class: "entity-card-body max-h-64 overflow-y-auto",

            FieldList {
                fields: fields,
                selected_field: props.selected_field,
                collapsed: false,
                max_visible: MAX_VISIBLE_FIELDS,
                on_field_click: move |id| props.on_field_click.call(id),
                on_field_double_click: move |id| props.on_field_double_click.call(id),
            }
        }
    }
}

// ============================================================================
// Entity Card Footer
// ============================================================================

/// Properties for EntityCardFooter
#[derive(Props, Clone, PartialEq)]
struct EntityCardFooterProps {
    /// Entity ID
    entity_id: EntityId,

    /// Add field callback
    on_add_field: EventHandler<EntityId>,
}

/// Footer section with add field button
#[component]
fn EntityCardFooter(props: EntityCardFooterProps) -> Element {
    let entity_id = props.entity_id;

    rsx! {
        div {
            class: "entity-card-footer px-2 py-2 border-t border-slate-700",

            button {
                class: "w-full py-1.5 px-3 text-xs text-slate-400 hover:text-slate-200 hover:bg-slate-700/50 rounded transition-colors flex items-center justify-center gap-1",
                onclick: move |e| {
                    e.stop_propagation();
                    props.on_add_field.call(entity_id);
                },
                span { "+" }
                span { "Add Field" }
            }
        }
    }
}

// Note: Port component is imported from crate::components::port

// ============================================================================
// Mini Entity Card (for palette/preview)
// ============================================================================

/// Properties for MiniEntityCard
#[derive(Props, Clone, PartialEq)]
pub struct MiniEntityCardProps {
    /// Entity name
    pub name: String,

    /// Number of fields
    #[props(default = 0)]
    pub field_count: usize,

    /// Whether this is selected
    #[props(default = false)]
    pub selected: bool,

    /// Click callback
    #[props(default)]
    pub on_click: EventHandler<()>,
}

/// Compact entity card for sidebar/palette
#[component]
pub fn MiniEntityCard(props: MiniEntityCardProps) -> Element {
    let selected = props.selected;

    let class = if selected {
        "mini-entity-card flex items-center gap-2 p-2 rounded-lg bg-indigo-600/20 border border-indigo-500 cursor-pointer"
    } else {
        "mini-entity-card flex items-center gap-2 p-2 rounded-lg bg-slate-800 border border-slate-700 hover:border-slate-600 cursor-pointer transition-colors"
    };

    rsx! {
        div {
            class: "{class}",
            onclick: move |_| props.on_click.call(()),

            // Icon
            div {
                class: "w-6 h-6 rounded bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-xs font-bold",
                "{props.name.chars().next().unwrap_or('E').to_uppercase()}"
            }

            // Info
            div {
                class: "flex-1 min-w-0",
                p {
                    class: "text-sm font-medium text-slate-200 truncate",
                    "{props.name}"
                }
                p {
                    class: "text-xs text-slate-500",
                    if props.field_count == 1 {
                        "1 field"
                    } else {
                        "{props.field_count} fields"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Entity Card Skeleton (Loading State)
// ============================================================================

/// Skeleton loader for entity card
#[component]
pub fn EntityCardSkeleton() -> Element {
    rsx! {
        div {
            class: "entity-card-skeleton bg-slate-800 rounded-xl border-2 border-slate-700 overflow-hidden animate-pulse",
            style: "width: 240px;",

            // Header skeleton
            div {
                class: "flex items-center gap-2 px-3 py-2.5 bg-slate-700/50 border-b border-slate-700",

                div { class: "w-8 h-8 rounded-lg bg-slate-600" }
                div {
                    class: "flex-1",
                    div { class: "h-4 bg-slate-600 rounded w-24 mb-1" }
                    div { class: "h-3 bg-slate-700 rounded w-16" }
                }
            }

            // Fields skeleton
            div {
                class: "p-2 space-y-2",
                for _ in 0..3 {
                    div {
                        class: "flex items-center gap-2 px-2 py-1.5",
                        div { class: "h-4 bg-slate-700 rounded flex-1" }
                        div { class: "h-4 bg-slate-700 rounded w-12" }
                    }
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
    fn test_card_class_default() {
        let class = build_card_class(false, false);
        assert!(class.contains("entity-card"));
        assert!(class.contains("border-slate-700"));
        assert!(!class.contains("border-indigo-500"));
    }

    #[test]
    fn test_card_class_selected() {
        let class = build_card_class(true, false);
        assert!(class.contains("border-indigo-500"));
        assert!(class.contains("ring-2"));
    }

    #[test]
    fn test_card_class_dragging() {
        let class = build_card_class(false, true);
        assert!(class.contains("cursor-grabbing"));
        assert!(class.contains("opacity-90"));
    }

    #[test]
    fn test_constants() {
        assert!(CARD_WIDTH > 0.0);
        assert!(CARD_HEADER_HEIGHT > 0.0);
        assert!(FIELD_ROW_HEIGHT > 0.0);
        assert!(PORT_SIZE > 0.0);
        assert!(MAX_VISIBLE_FIELDS > 0);
    }
}
