//! # Endpoint Card Component
//!
//! Visual card component for displaying and configuring API endpoint groups.
//!
//! Each card represents an entity's endpoint group with:
//! - Entity name and base path
//! - CRUD operation toggles (Create, Read, ReadAll, Update, Delete)
//! - Security badges (open, authenticated, role-based)
//! - Rate limiting indicators
//! - Quick actions (edit, delete, enable/disable)
//!
//! ## Usage
//!
//! ```rust,ignore
//! EndpointCard {
//!     endpoint: endpoint_group.clone(),
//!     entity_name: "User".to_string(),
//!     is_selected: false,
//!     on_select: move |id| { /* handle selection */ },
//!     on_edit: move |id| { /* open edit dialog */ },
//!     on_toggle_operation: move |(id, op_type, enabled)| { /* toggle op */ },
//! }
//! ```

use dioxus::prelude::*;
use imortal_ir::{CrudOperation, EndpointGroup, EndpointSecurity, OperationType};
use uuid::Uuid;

// ============================================================================
// Endpoint Card Component
// ============================================================================

/// Properties for EndpointCard component
#[derive(Props, Clone, PartialEq)]
pub struct EndpointCardProps {
    /// The endpoint group to display
    pub endpoint: EndpointGroup,

    /// Whether this card is selected
    #[props(default = false)]
    pub is_selected: bool,

    /// Whether the card is in compact mode
    #[props(default = false)]
    pub compact: bool,

    /// Callback when the card is clicked (selection)
    pub on_select: EventHandler<Uuid>,

    /// Callback when the card is double-clicked (edit)
    #[props(default)]
    pub on_edit: Option<EventHandler<Uuid>>,

    /// Callback when an operation is toggled (endpoint_id, operation_type, new_enabled)
    #[props(default)]
    pub on_toggle_operation: Option<EventHandler<(Uuid, OperationType, bool)>>,

    /// Callback when the entire endpoint group is toggled
    #[props(default)]
    pub on_toggle_enabled: Option<EventHandler<(Uuid, bool)>>,

    /// Callback when delete is requested
    #[props(default)]
    pub on_delete: Option<EventHandler<Uuid>>,
}

/// Endpoint card component displaying an entity's API endpoints
#[component]
pub fn EndpointCard(props: EndpointCardProps) -> Element {
    let endpoint = &props.endpoint;
    let endpoint_id = endpoint.id;
    let is_enabled = endpoint.enabled;
    let has_auth = endpoint.requires_auth();
    let enabled_count = endpoint.enabled_operations().len();
    let total_count = endpoint.operations.len();

    // Security summary
    let security_info = SecuritySummary::from_endpoint(endpoint);

    // Rate limit summary
    let has_rate_limit = endpoint.operations.iter().any(|op| op.rate_limit.is_some());

    rsx! {
        div {
            class: format!(
                "endpoint-card rounded-xl border-2 transition-all duration-200 cursor-pointer {} {}",
                if props.is_selected {
                    "border-indigo-500 ring-2 ring-indigo-300/30 shadow-xl shadow-indigo-500/10"
                } else if !is_enabled {
                    "border-slate-700 opacity-60 hover:border-slate-600"
                } else {
                    "border-slate-700 hover:border-indigo-400/50 hover:shadow-lg"
                },
                if is_enabled { "bg-slate-800" } else { "bg-slate-800/50" }
            ),
            onclick: move |_| props.on_select.call(endpoint_id),
            ondoubleclick: move |_| {
                if let Some(handler) = &props.on_edit {
                    handler.call(endpoint_id);
                }
            },

            // Card Header
            EndpointCardHeader {
                entity_name: endpoint.entity_name.clone(),
                base_path: endpoint.full_base_path(),
                is_enabled: is_enabled,
                has_auth: has_auth,
                api_version: endpoint.api_version.clone(),
                on_toggle_enabled: move |enabled| {
                    if let Some(handler) = &props.on_toggle_enabled {
                        handler.call((endpoint_id, enabled));
                    }
                },
            }

            // CRUD Operations
            if !props.compact {
                div {
                    class: "px-4 py-3 space-y-1.5",

                    for operation in endpoint.operations.iter() {
                        {
                            let op = operation.clone();
                            let op_type = op.operation_type;
                            rsx! {
                                OperationRow {
                                    operation: op,
                                    base_path: endpoint.base_path.clone(),
                                    global_security: endpoint.global_security.clone(),
                                    parent_enabled: is_enabled,
                                    on_toggle: move |enabled| {
                                        if let Some(handler) = &props.on_toggle_operation {
                                            handler.call((endpoint_id, op_type, enabled));
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            } else {
                // Compact mode: show summary badges
                div {
                    class: "px-4 py-2 flex flex-wrap gap-1.5",

                    for operation in endpoint.operations.iter() {
                        {
                            let op = operation.clone();
                            rsx! {
                                OperationBadge {
                                    operation: op,
                                }
                            }
                        }
                    }
                }
            }

            // Security & Rate Limit Footer
            EndpointCardFooter {
                security: security_info,
                has_rate_limit: has_rate_limit,
                enabled_count: enabled_count,
                total_count: total_count,
                tags: endpoint.tags.clone(),
            }

            // Actions bar (only when selected)
            if props.is_selected {
                div {
                    class: "px-4 py-2 border-t border-slate-700 flex items-center justify-between",

                    // Left: info
                    div {
                        class: "text-xs text-slate-500",
                        if let Some(desc) = &endpoint.description {
                            span { "{desc}" }
                        } else {
                            span { "No description" }
                        }
                    }

                    // Right: action buttons
                    div {
                        class: "flex items-center gap-2",

                        // Edit button
                        button {
                            class: "p-1.5 text-slate-400 hover:text-indigo-400 hover:bg-slate-700 rounded transition-colors",
                            title: "Edit endpoint configuration",
                            onclick: move |e| {
                                e.stop_propagation();
                                if let Some(handler) = &props.on_edit {
                                    handler.call(endpoint_id);
                                }
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
                            class: "p-1.5 text-slate-400 hover:text-red-400 hover:bg-slate-700 rounded transition-colors",
                            title: "Remove endpoint group",
                            onclick: move |e| {
                                e.stop_propagation();
                                if let Some(handler) = &props.on_delete {
                                    handler.call(endpoint_id);
                                }
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

// ============================================================================
// Card Header
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct EndpointCardHeaderProps {
    entity_name: String,
    base_path: String,
    is_enabled: bool,
    has_auth: bool,
    api_version: Option<String>,
    on_toggle_enabled: EventHandler<bool>,
}

#[component]
fn EndpointCardHeader(props: EndpointCardHeaderProps) -> Element {
    rsx! {
        div {
            class: "px-4 py-3 border-b border-slate-700 flex items-center justify-between",

            // Left side: entity name + path
            div {
                class: "flex-1 min-w-0",

                // Entity name row
                div {
                    class: "flex items-center gap-2",

                    // API icon
                    div {
                        class: format!(
                            "w-8 h-8 rounded-lg flex items-center justify-center {}",
                            if props.is_enabled {
                                "bg-indigo-600/20 text-indigo-400"
                            } else {
                                "bg-slate-700 text-slate-500"
                            }
                        ),
                        svg {
                            class: "w-4 h-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z",
                            }
                        }
                    }

                    // Entity name
                    h3 {
                        class: format!(
                            "font-semibold truncate {}",
                            if props.is_enabled { "text-white" } else { "text-slate-400" }
                        ),
                        "{props.entity_name}"
                    }

                    // Version badge
                    if let Some(version) = &props.api_version {
                        span {
                            class: "px-1.5 py-0.5 bg-slate-700 text-slate-400 rounded text-xs font-mono",
                            "{version}"
                        }
                    }

                    // Auth badge
                    if props.has_auth {
                        span {
                            class: "flex items-center gap-1 px-1.5 py-0.5 bg-amber-900/30 text-amber-400 rounded text-xs",
                            title: "Authentication required",
                            // Lock icon
                            svg {
                                class: "w-3 h-3",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z",
                                }
                            }
                            "Auth"
                        }
                    }
                }

                // Base path
                div {
                    class: "mt-1 font-mono text-xs text-slate-500 truncate",
                    "{props.base_path}"
                }
            }

            // Right side: enable toggle
            div {
                class: "ml-3 flex-shrink-0",
                button {
                    class: format!(
                        "relative inline-flex h-5 w-9 items-center rounded-full transition-colors {}",
                        if props.is_enabled {
                            "bg-indigo-600"
                        } else {
                            "bg-slate-600"
                        }
                    ),
                    title: if props.is_enabled { "Disable endpoint group" } else { "Enable endpoint group" },
                    onclick: move |e| {
                        e.stop_propagation();
                        props.on_toggle_enabled.call(!props.is_enabled);
                    },
                    span {
                        class: format!(
                            "inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform {}",
                            if props.is_enabled { "translate-x-4" } else { "translate-x-1" }
                        ),
                    }
                }
            }
        }
    }
}

// ============================================================================
// Operation Row
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct OperationRowProps {
    operation: CrudOperation,
    base_path: String,
    global_security: EndpointSecurity,
    parent_enabled: bool,
    on_toggle: EventHandler<bool>,
}

#[component]
fn OperationRow(props: OperationRowProps) -> Element {
    let op = props.operation.clone();
    let _op_type = op.operation_type;
    let op_enabled = op.enabled;
    let is_enabled = op.enabled && props.parent_enabled;
    let method = op.http_method().to_string();
    let full_path = op.full_path(&props.base_path);

    // Determine effective security
    let effective_security = op.security.as_ref().unwrap_or(&props.global_security);
    let has_auth = effective_security.auth_required;
    let has_roles = effective_security.has_roles();
    let roles_display = if has_roles {
        format!("Roles: {}", effective_security.roles.join(", "))
    } else {
        "Authenticated".to_string()
    };
    let has_rate_limit = op.rate_limit.is_some();
    let rate_limit_display = op
        .rate_limit
        .as_ref()
        .map(|rl| format!("{} req / {}s", rl.requests, rl.window_seconds))
        .unwrap_or_default();
    let handler_name = op.handler_name(&op.operation_type.display_name());

    rsx! {
        div {
            class: format!(
                "flex items-center gap-2 px-2 py-1.5 rounded-lg transition-colors group {}",
                if is_enabled {
                    "hover:bg-slate-700/50"
                } else {
                    "opacity-50"
                }
            ),

            // Enable/disable checkbox
            button {
                class: format!(
                    "w-4 h-4 rounded border flex items-center justify-center flex-shrink-0 transition-colors {}",
                    if op_enabled {
                        "bg-indigo-600 border-indigo-600"
                    } else {
                        "border-slate-600 hover:border-slate-500"
                    }
                ),
                onclick: move |e| {
                    e.stop_propagation();
                    props.on_toggle.call(!op_enabled);
                },
                if op_enabled {
                    svg {
                        class: "w-3 h-3 text-white",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "3",
                            d: "M5 13l4 4L19 7",
                        }
                    }
                }
            }

            // HTTP method badge
            span {
                class: format!(
                    "px-2 py-0.5 rounded text-xs font-bold uppercase min-w-[52px] text-center {}",
                    http_method_class(&method)
                ),
                "{method}"
            }

            // Path
            span {
                class: format!(
                    "flex-1 font-mono text-xs truncate {}",
                    if is_enabled { "text-slate-300" } else { "text-slate-500" }
                ),
                "{full_path}"
            }

            // Security indicators (visible on hover or when active)
            div {
                class: "flex items-center gap-1 opacity-70 group-hover:opacity-100 transition-opacity",

                // Auth indicator
                if has_auth {
                    span {
                        class: "w-4 h-4 flex items-center justify-center text-amber-400",
                        title: "{roles_display}",
                        svg {
                            class: "w-3.5 h-3.5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z",
                            }
                        }
                    }
                }

                // Rate limit indicator
                if has_rate_limit {
                    {
                        rsx! {
                            span {
                                class: "w-4 h-4 flex items-center justify-center text-cyan-400",
                                title: "{rate_limit_display}",
                                svg {
                                    class: "w-3.5 h-3.5",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Handler name (shown on hover)
            span {
                class: "hidden group-hover:inline text-xs text-slate-600 font-mono truncate max-w-[120px]",
                "{handler_name}"
            }
        }
    }
}

// ============================================================================
// Operation Badge (Compact Mode)
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct OperationBadgeProps {
    operation: CrudOperation,
}

#[component]
fn OperationBadge(props: OperationBadgeProps) -> Element {
    let op = &props.operation;
    let method = op.http_method();

    rsx! {
        span {
            class: format!(
                "px-2 py-0.5 rounded text-xs font-bold {} {}",
                http_method_class(method),
                if op.enabled { "" } else { "opacity-30 line-through" }
            ),
            title: format!(
                "{} {} - {}",
                method,
                op.path_suffix,
                if op.enabled { "Enabled" } else { "Disabled" }
            ),
            "{op.operation_type.display_name()}"
        }
    }
}

// ============================================================================
// Card Footer
// ============================================================================

#[derive(Clone, PartialEq)]
struct SecuritySummary {
    label: String,
    level: SecurityLevel,
    roles: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SecurityLevel {
    Open,
    Authenticated,
    RoleBased,
    Mixed,
}

impl SecuritySummary {
    fn from_endpoint(endpoint: &EndpointGroup) -> Self {
        let enabled_ops = endpoint.enabled_operations();
        if enabled_ops.is_empty() {
            return Self {
                label: "No operations".to_string(),
                level: SecurityLevel::Open,
                roles: Vec::new(),
            };
        }

        let mut has_open = false;
        let mut has_auth = false;
        let mut has_roles = false;
        let mut all_roles: Vec<String> = Vec::new();

        for op in &enabled_ops {
            let security = op.security.as_ref().unwrap_or(&endpoint.global_security);

            if security.auth_required {
                has_auth = true;
                if !security.roles.is_empty() {
                    has_roles = true;
                    for role in &security.roles {
                        if !all_roles.contains(role) {
                            all_roles.push(role.clone());
                        }
                    }
                }
            } else {
                has_open = true;
            }
        }

        if has_open && (has_auth || has_roles) {
            Self {
                label: "Mixed security".to_string(),
                level: SecurityLevel::Mixed,
                roles: all_roles,
            }
        } else if has_roles {
            Self {
                label: format!("Roles: {}", all_roles.join(", ")),
                level: SecurityLevel::RoleBased,
                roles: all_roles,
            }
        } else if has_auth {
            Self {
                label: "Authenticated".to_string(),
                level: SecurityLevel::Authenticated,
                roles: Vec::new(),
            }
        } else {
            Self {
                label: "Public".to_string(),
                level: SecurityLevel::Open,
                roles: Vec::new(),
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct EndpointCardFooterProps {
    security: SecuritySummary,
    has_rate_limit: bool,
    enabled_count: usize,
    total_count: usize,
    tags: Vec<String>,
}

#[component]
fn EndpointCardFooter(props: EndpointCardFooterProps) -> Element {
    rsx! {
        div {
            class: "px-4 py-2 border-t border-slate-700/50 flex items-center justify-between",

            // Left side: security info
            div {
                class: "flex items-center gap-2",

                // Security badge
                span {
                    class: format!(
                        "flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium {}",
                        match props.security.level {
                            SecurityLevel::Open => "bg-green-900/30 text-green-400",
                            SecurityLevel::Authenticated => "bg-amber-900/30 text-amber-400",
                            SecurityLevel::RoleBased => "bg-purple-900/30 text-purple-400",
                            SecurityLevel::Mixed => "bg-blue-900/30 text-blue-400",
                        }
                    ),

                    // Icon
                    match props.security.level {
                        SecurityLevel::Open => rsx! {
                            svg {
                                class: "w-3 h-3",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                                }
                            }
                        },
                        _ => rsx! {
                            svg {
                                class: "w-3 h-3",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z",
                                }
                            }
                        },
                    }

                    "{props.security.label}"
                }

                // Rate limit badge
                if props.has_rate_limit {
                    span {
                        class: "flex items-center gap-1 px-2 py-0.5 bg-cyan-900/30 text-cyan-400 rounded text-xs",
                        svg {
                            class: "w-3 h-3",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
                            }
                        }
                        "Rate Limited"
                    }
                }
            }

            // Right side: operation count
            div {
                class: "flex items-center gap-2",

                // Tags (show first tag if any)
                if let Some(tag) = props.tags.first() {
                    span {
                        class: "px-1.5 py-0.5 bg-slate-700 text-slate-400 rounded text-xs",
                        "#{tag}"
                    }
                }

                // Count
                span {
                    class: format!(
                        "text-xs font-medium {}",
                        if props.enabled_count == props.total_count {
                            "text-green-400"
                        } else if props.enabled_count == 0 {
                            "text-slate-500"
                        } else {
                            "text-slate-400"
                        }
                    ),
                    "{props.enabled_count}/{props.total_count} ops"
                }
            }
        }
    }
}

// ============================================================================
// Bulk Endpoint Card (for auto-generating endpoints for all entities)
// ============================================================================

/// Properties for the "Generate Endpoints" prompt card
#[derive(Props, Clone, PartialEq)]
pub struct GenerateEndpointsCardProps {
    /// Number of entities without endpoints
    pub uncovered_count: usize,

    /// Callback to generate endpoints for all uncovered entities
    pub on_generate: EventHandler<()>,
}

/// Card prompting the user to generate endpoints for entities without them
#[component]
pub fn GenerateEndpointsCard(props: GenerateEndpointsCardProps) -> Element {
    if props.uncovered_count == 0 {
        return rsx! {};
    }

    rsx! {
        div {
            class: "rounded-xl border-2 border-dashed border-indigo-500/30 bg-indigo-900/10 p-6 text-center",

            div {
                class: "w-12 h-12 mx-auto mb-3 rounded-full bg-indigo-600/20 flex items-center justify-center",
                svg {
                    class: "w-6 h-6 text-indigo-400",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M12 6v6m0 0v6m0-6h6m-6 0H6",
                    }
                }
            }

            h4 {
                class: "text-white font-medium mb-1",
                {
                    let suffix = if props.uncovered_count == 1 { "y" } else { "ies" };
                    format!("{} entit{} without endpoints", props.uncovered_count, suffix)
                }
            }

            p {
                class: "text-sm text-slate-400 mb-4",
                "Generate default CRUD endpoints for entities that don't have them yet."
            }

            button {
                class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg text-sm font-medium transition-colors",
                onclick: move |_| props.on_generate.call(()),
                "Generate Endpoints"
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the Tailwind CSS class for an HTTP method badge
pub fn http_method_class(method: &str) -> &'static str {
    match method {
        "GET" => "bg-green-900/40 text-green-400",
        "POST" => "bg-blue-900/40 text-blue-400",
        "PUT" => "bg-amber-900/40 text-amber-400",
        "PATCH" => "bg-orange-900/40 text-orange-400",
        "DELETE" => "bg-red-900/40 text-red-400",
        _ => "bg-slate-700 text-slate-400",
    }
}

/// Get a color for an operation type
pub fn operation_type_color(op_type: &OperationType) -> &'static str {
    match op_type {
        OperationType::Create => "#3B82F6",  // blue
        OperationType::Read => "#22C55E",    // green
        OperationType::ReadAll => "#10B981", // emerald
        OperationType::Update => "#F59E0B",  // amber
        OperationType::Delete => "#EF4444",  // red
    }
}

/// Get a description for an operation type
pub fn operation_type_description(op_type: &OperationType) -> &'static str {
    match op_type {
        OperationType::Create => "Create a new resource",
        OperationType::Read => "Get a single resource by ID",
        OperationType::ReadAll => "List all resources with pagination",
        OperationType::Update => "Update an existing resource",
        OperationType::Delete => "Delete a resource by ID",
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_class() {
        assert!(http_method_class("GET").contains("green"));
        assert!(http_method_class("POST").contains("blue"));
        assert!(http_method_class("PUT").contains("amber"));
        assert!(http_method_class("DELETE").contains("red"));
        assert!(http_method_class("UNKNOWN").contains("slate"));
    }

    #[test]
    fn test_operation_type_color() {
        assert_eq!(operation_type_color(&OperationType::Create), "#3B82F6");
        assert_eq!(operation_type_color(&OperationType::Delete), "#EF4444");
    }

    #[test]
    fn test_operation_type_description() {
        assert!(operation_type_description(&OperationType::Create).contains("Create"));
        assert!(operation_type_description(&OperationType::ReadAll).contains("List"));
    }

    #[test]
    fn test_security_summary_open() {
        let endpoint = EndpointGroup::new(Uuid::new_v4(), "User");
        let summary = SecuritySummary::from_endpoint(&endpoint);
        assert_eq!(summary.level, SecurityLevel::Open);
        assert!(summary.label.contains("Public"));
    }

    #[test]
    fn test_security_summary_authenticated() {
        let endpoint = EndpointGroup::new(Uuid::new_v4(), "User").secured();
        let summary = SecuritySummary::from_endpoint(&endpoint);
        assert_eq!(summary.level, SecurityLevel::Authenticated);
    }

    #[test]
    fn test_security_summary_role_based() {
        let endpoint =
            EndpointGroup::new(Uuid::new_v4(), "User").with_roles(vec!["admin".to_string()]);
        let summary = SecuritySummary::from_endpoint(&endpoint);
        assert_eq!(summary.level, SecurityLevel::RoleBased);
        assert!(summary.roles.contains(&"admin".to_string()));
    }

    #[test]
    fn test_security_summary_mixed() {
        let mut endpoint = EndpointGroup::new(Uuid::new_v4(), "User");
        // Make Create secured, leave Read open
        endpoint.set_operation_security(OperationType::Create, EndpointSecurity::authenticated());
        let summary = SecuritySummary::from_endpoint(&endpoint);
        assert_eq!(summary.level, SecurityLevel::Mixed);
    }
}
