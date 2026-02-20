//! # Endpoint Dialog Component
//!
//! Dialog for creating and editing endpoint groups with full CRUD configuration.
//!
//! This dialog allows users to:
//! - Select which entity the endpoints are for
//! - Configure the base API path and version
//! - Toggle individual CRUD operations (Create, Read, ReadAll, Update, Delete)
//! - Set global security (open, authenticated, role-based)
//! - Configure per-operation security overrides
//! - Set rate limiting per operation
//! - Add tags and descriptions for API documentation
//!
//! ## Usage
//!
//! ```rust,ignore
//! EndpointDialog {
//!     mode: EndpointDialogMode::Create { entity_id: Some(id) },
//! }
//! ```

use dioxus::prelude::*;
use imortal_ir::{CrudOperation, EndpointGroup, EndpointSecurity, OperationType, RateLimit};
use uuid::Uuid;

use crate::components::inputs::{NumberInput, Select, SelectOption, TextArea, TextInput, Toggle};
use crate::state::{APP_STATE, StatusLevel};

// ============================================================================
// Dialog Mode
// ============================================================================

/// Mode for the endpoint dialog
#[derive(Debug, Clone, PartialEq)]
pub enum EndpointDialogMode {
    /// Creating a new endpoint group
    Create {
        /// Pre-selected entity (optional)
        entity_id: Option<Uuid>,
    },
    /// Editing an existing endpoint group
    Edit(Uuid),
}

impl EndpointDialogMode {
    /// Check if this is create mode
    pub fn is_create(&self) -> bool {
        matches!(self, EndpointDialogMode::Create { .. })
    }

    /// Get the title for the dialog
    pub fn title(&self) -> &'static str {
        match self {
            EndpointDialogMode::Create { .. } => "Configure Endpoints",
            EndpointDialogMode::Edit(_) => "Edit Endpoint Configuration",
        }
    }

    /// Get the submit button text
    pub fn submit_text(&self) -> &'static str {
        match self {
            EndpointDialogMode::Create { .. } => "Create Endpoints",
            EndpointDialogMode::Edit(_) => "Save Changes",
        }
    }
}

// ============================================================================
// Endpoint Dialog Props
// ============================================================================

/// Properties for the EndpointDialog component
#[derive(Props, Clone, PartialEq)]
pub struct EndpointDialogProps {
    /// Dialog mode (create or edit)
    pub mode: EndpointDialogMode,
}

// ============================================================================
// Active Tab
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    General,
    Operations,
    Security,
    RateLimits,
}

impl ActiveTab {
    fn label(&self) -> &'static str {
        match self {
            ActiveTab::General => "General",
            ActiveTab::Operations => "Operations",
            ActiveTab::Security => "Security",
            ActiveTab::RateLimits => "Rate Limits",
        }
    }

    fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::General,
            ActiveTab::Operations,
            ActiveTab::Security,
            ActiveTab::RateLimits,
        ]
    }
}

// ============================================================================
// Endpoint Dialog Component
// ============================================================================

/// Endpoint configuration dialog component
#[component]
pub fn EndpointDialog(props: EndpointDialogProps) -> Element {
    let mut active_tab = use_signal(|| ActiveTab::General);

    // Get entities for the entity selector
    let state = APP_STATE.read();
    let entities: Vec<(Uuid, String)> = state
        .project
        .as_ref()
        .map(|p| {
            let mut list: Vec<(Uuid, String)> = p
                .entities
                .values()
                .map(|e| (e.id, e.name.clone()))
                .collect();
            list.sort_by(|a, b| a.1.cmp(&b.1));
            list
        })
        .unwrap_or_default();

    // Existing endpoint IDs (to know which entities already have endpoints)
    let existing_endpoint_entity_ids: Vec<Uuid> = state
        .project
        .as_ref()
        .map(|p| p.endpoints.values().map(|ep| ep.entity_id).collect())
        .unwrap_or_default();
    drop(state);

    // Initialize form state based on mode
    let existing_endpoint: Option<EndpointGroup> = match &props.mode {
        EndpointDialogMode::Edit(id) => {
            let state = APP_STATE.read();
            state
                .project
                .as_ref()
                .and_then(|p| p.endpoints.get(id))
                .cloned()
        }
        EndpointDialogMode::Create { entity_id } => entity_id.map(|eid| {
            let state = APP_STATE.read();
            let entity_name = state
                .project
                .as_ref()
                .and_then(|p| p.entities.get(&eid))
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "Entity".to_string());
            EndpointGroup::new(eid, entity_name)
        }),
    };

    // Form signals
    let default_ep = existing_endpoint.clone().unwrap_or_default();

    let mut selected_entity_id = use_signal(|| match &props.mode {
        EndpointDialogMode::Create { entity_id } => {
            entity_id.map(|id| id.to_string()).unwrap_or_default()
        }
        EndpointDialogMode::Edit(_) => default_ep.entity_id.to_string(),
    });

    let mut base_path = use_signal(|| default_ep.base_path.clone());
    let mut api_version = use_signal(|| default_ep.api_version.clone().unwrap_or_default());
    let mut description = use_signal(|| default_ep.description.clone().unwrap_or_default());
    let mut tags_str = use_signal(|| default_ep.tags.join(", "));
    let mut is_enabled = use_signal(|| default_ep.enabled);

    // Operation enables
    let mut op_create_enabled = use_signal(|| {
        default_ep
            .get_operation(OperationType::Create)
            .map(|o| o.enabled)
            .unwrap_or(true)
    });
    let mut op_read_enabled = use_signal(|| {
        default_ep
            .get_operation(OperationType::Read)
            .map(|o| o.enabled)
            .unwrap_or(true)
    });
    let mut op_read_all_enabled = use_signal(|| {
        default_ep
            .get_operation(OperationType::ReadAll)
            .map(|o| o.enabled)
            .unwrap_or(true)
    });
    let mut op_update_enabled = use_signal(|| {
        default_ep
            .get_operation(OperationType::Update)
            .map(|o| o.enabled)
            .unwrap_or(true)
    });
    let mut op_delete_enabled = use_signal(|| {
        default_ep
            .get_operation(OperationType::Delete)
            .map(|o| o.enabled)
            .unwrap_or(true)
    });

    // Global security
    let mut global_auth_required = use_signal(|| default_ep.global_security.auth_required);
    let mut global_roles = use_signal(|| default_ep.global_security.roles.join(", "));
    let mut global_cors_enabled = use_signal(|| default_ep.global_security.cors_enabled);

    // Per-operation security overrides (store as strings for the selected op)
    let _selected_op_for_security = use_signal(|| OperationType::Create);
    let mut per_op_auth_overrides: Signal<Vec<(OperationType, Option<bool>)>> = use_signal(|| {
        OperationType::all()
            .iter()
            .map(|op_type| {
                let override_auth = default_ep
                    .get_operation(*op_type)
                    .and_then(|op| op.security.as_ref())
                    .map(|s| s.auth_required);
                (*op_type, override_auth)
            })
            .collect()
    });

    // Rate limit state
    let _rate_limit_op = use_signal(|| OperationType::Create);
    let mut rate_limits: Signal<Vec<(OperationType, Option<(u32, u32)>)>> = use_signal(|| {
        OperationType::all()
            .iter()
            .map(|op_type| {
                let rl = default_ep
                    .get_operation(*op_type)
                    .and_then(|op| op.rate_limit.as_ref())
                    .map(|rl| (rl.requests, rl.window_seconds));
                (*op_type, rl)
            })
            .collect()
    });

    // Error message
    let mut error_message: Signal<Option<String>> = use_signal(|| None);

    // Build entity selector options
    let entity_options: Vec<SelectOption> = {
        let mut opts = vec![SelectOption {
            value: String::new(),
            label: "Select an entity...".to_string(),
            disabled: true,
        }];
        for (eid, ename) in &entities {
            let already_has = existing_endpoint_entity_ids.contains(eid)
                && !matches!(&props.mode, EndpointDialogMode::Edit(_));
            opts.push(SelectOption {
                value: eid.to_string(),
                label: if already_has {
                    format!("{} (already configured)", ename)
                } else {
                    ename.clone()
                },
                disabled: already_has,
            });
        }
        opts
    };

    // Auto-update base_path when entity changes (create mode only)
    let entities_for_path = entities.clone();
    let mode_is_create = props.mode.is_create();

    // Handle form submission
    let mode_for_submit = props.mode.clone();
    let on_submit = move |_| {
        // Validate
        let entity_id_str = selected_entity_id.read().clone();
        let entity_id = match Uuid::parse_str(&entity_id_str) {
            Ok(id) => id,
            Err(_) => {
                error_message.set(Some("Please select an entity".to_string()));
                return;
            }
        };

        let bp = base_path.read().clone();
        if bp.is_empty() || !bp.starts_with('/') {
            error_message.set(Some("Base path must start with '/'".to_string()));
            return;
        }

        // Build operations
        let mut operations = CrudOperation::default_all();
        for op in &mut operations {
            match op.operation_type {
                OperationType::Create => op.enabled = *op_create_enabled.read(),
                OperationType::Read => op.enabled = *op_read_enabled.read(),
                OperationType::ReadAll => op.enabled = *op_read_all_enabled.read(),
                OperationType::Update => op.enabled = *op_update_enabled.read(),
                OperationType::Delete => op.enabled = *op_delete_enabled.read(),
            }

            // Apply per-op security overrides
            let overrides = per_op_auth_overrides.read();
            if let Some((_, Some(auth_req))) =
                overrides.iter().find(|(ot, _)| *ot == op.operation_type)
            {
                let mut sec = EndpointSecurity::new();
                sec.auth_required = *auth_req;
                op.security = Some(sec);
            }

            // Apply rate limits
            let rls = rate_limits.read();
            if let Some((_, Some((requests, window)))) =
                rls.iter().find(|(ot, _)| *ot == op.operation_type)
            {
                if *requests > 0 && *window > 0 {
                    op.rate_limit = Some(RateLimit::new(*requests, *window));
                }
            }
        }

        // Build global security
        let roles_str = global_roles.read().clone();
        let roles: Vec<String> = roles_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let global_security = EndpointSecurity {
            auth_required: *global_auth_required.read(),
            roles,
            scopes: Vec::new(),
            cors_enabled: *global_cors_enabled.read(),
            cors_origins: Vec::new(),
            allow_public_preview: false,
        };

        // Build tags
        let tags_string = tags_str.read().clone();
        let tags: Vec<String> = tags_string
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // API version
        let version = api_version.read().clone();
        let api_ver = if version.is_empty() {
            None
        } else {
            Some(version)
        };

        // Description
        let desc_val = description.read().clone();
        let desc = if desc_val.is_empty() {
            None
        } else {
            Some(desc_val)
        };

        // Get entity name
        let entity_name = {
            let state = APP_STATE.read();
            state
                .project
                .as_ref()
                .and_then(|p| p.entities.get(&entity_id))
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "Entity".to_string())
        };

        let mut state = APP_STATE.write();
        match &mode_for_submit {
            EndpointDialogMode::Create { .. } => {
                let mut endpoint = EndpointGroup::new(entity_id, &entity_name);
                endpoint.base_path = bp;
                endpoint.api_version = api_ver;
                endpoint.description = desc;
                endpoint.tags = tags;
                endpoint.enabled = *is_enabled.read();
                endpoint.operations = operations;
                endpoint.global_security = global_security;

                if let Some(project) = &mut state.project {
                    project.add_endpoint(endpoint);
                }

                tracing::info!("Created endpoint group for entity '{}'", entity_name);
                state.ui.set_status(
                    format!("Created endpoints for {}", entity_name),
                    StatusLevel::Success,
                );
            }
            EndpointDialogMode::Edit(endpoint_id) => {
                if let Some(project) = &mut state.project {
                    if let Some(ep) = project.get_endpoint_mut(*endpoint_id) {
                        ep.entity_id = entity_id;
                        ep.entity_name = entity_name.clone();
                        ep.base_path = bp;
                        ep.api_version = api_ver;
                        ep.description = desc;
                        ep.tags = tags;
                        ep.enabled = *is_enabled.read();
                        ep.operations = operations;
                        ep.global_security = global_security;
                    }
                }

                tracing::info!("Updated endpoint group for entity '{}'", entity_name);
                state.ui.set_status(
                    format!("Updated endpoints for {}", entity_name),
                    StatusLevel::Success,
                );
            }
        }

        state.is_dirty = true;
        state.ui.close_dialog();
    };

    // Title and submit text from mode
    let title = props.mode.title();
    let submit_text = props.mode.submit_text();

    rsx! {
        div {
            class: "flex flex-col max-h-[85vh]",

            // Dialog header
            div {
                class: "px-6 py-4 border-b border-slate-700 flex items-center justify-between",

                h2 {
                    class: "text-xl font-semibold text-white flex items-center gap-2",
                    // API icon
                    svg {
                        class: "w-6 h-6 text-indigo-400",
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
                    "{title}"
                }

                button {
                    class: "text-slate-400 hover:text-white transition-colors p-1 rounded hover:bg-slate-700",
                    onclick: move |_| {
                        APP_STATE.write().ui.close_dialog();
                    },
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M6 18L18 6M6 6l12 12",
                        }
                    }
                }
            }

            // Tabs
            div {
                class: "flex border-b border-slate-700",

                for tab in ActiveTab::all().iter() {
                    {
                        let t = *tab;
                        rsx! {
                            button {
                                key: "{tab.label()}",
                                class: format!(
                                    "px-4 py-2.5 text-sm font-medium border-b-2 transition-colors {}",
                                    if *active_tab.read() == t {
                                        "border-indigo-500 text-indigo-400"
                                    } else {
                                        "border-transparent text-slate-400 hover:text-slate-300 hover:border-slate-600"
                                    }
                                ),
                                onclick: move |_| active_tab.set(t),
                                "{tab.label()}"
                            }
                        }
                    }
                }
            }

            // Tab content (scrollable)
            div {
                class: "flex-1 overflow-y-auto px-6 py-4",

                // Error message
                if let Some(err) = error_message.read().as_ref() {
                    div {
                        class: "mb-4 p-3 bg-red-900/30 border border-red-700 rounded-lg text-red-400 text-sm",
                        "{err}"
                    }
                }

                match *active_tab.read() {
                    ActiveTab::General => rsx! {
                        GeneralTab {
                            mode_is_create: mode_is_create,
                            entity_options: entity_options.clone(),
                            selected_entity_id: selected_entity_id.read().clone(),
                            on_entity_change: move |v: String| {
                                selected_entity_id.set(v.clone());
                                // Auto-update base path
                                if mode_is_create {
                                    if let Ok(eid) = Uuid::parse_str(&v) {
                                        if let Some((_, name)) = entities_for_path.iter().find(|(id, _)| *id == eid) {
                                            let snake = to_snake_case_plural(name);
                                            base_path.set(format!("/api/{}", snake));
                                        }
                                    }
                                }
                            },
                            base_path: base_path.read().clone(),
                            on_base_path_change: move |v: String| base_path.set(v),
                            api_version: api_version.read().clone(),
                            on_api_version_change: move |v: String| api_version.set(v),
                            description: description.read().clone(),
                            on_description_change: move |v: String| description.set(v),
                            tags: tags_str.read().clone(),
                            on_tags_change: move |v: String| tags_str.set(v),
                            is_enabled: *is_enabled.read(),
                            on_enabled_change: move |v: bool| is_enabled.set(v),
                        }
                    },
                    ActiveTab::Operations => rsx! {
                        OperationsTab {
                            create_enabled: *op_create_enabled.read(),
                            on_create_toggle: move |v: bool| op_create_enabled.set(v),
                            read_enabled: *op_read_enabled.read(),
                            on_read_toggle: move |v: bool| op_read_enabled.set(v),
                            read_all_enabled: *op_read_all_enabled.read(),
                            on_read_all_toggle: move |v: bool| op_read_all_enabled.set(v),
                            update_enabled: *op_update_enabled.read(),
                            on_update_toggle: move |v: bool| op_update_enabled.set(v),
                            delete_enabled: *op_delete_enabled.read(),
                            on_delete_toggle: move |v: bool| op_delete_enabled.set(v),
                            base_path: base_path.read().clone(),
                        }
                    },
                    ActiveTab::Security => rsx! {
                        SecurityTab {
                            auth_required: *global_auth_required.read(),
                            on_auth_change: move |v: bool| global_auth_required.set(v),
                            roles: global_roles.read().clone(),
                            on_roles_change: move |v: String| global_roles.set(v),
                            cors_enabled: *global_cors_enabled.read(),
                            on_cors_change: move |v: bool| global_cors_enabled.set(v),
                            per_op_overrides: per_op_auth_overrides.read().clone(),
                            on_per_op_override_change: move |(op_type, val): (OperationType, Option<bool>)| {
                                let mut overrides = per_op_auth_overrides.write();
                                if let Some(entry) = overrides.iter_mut().find(|(ot, _)| *ot == op_type) {
                                    entry.1 = val;
                                }
                            },
                        }
                    },
                    ActiveTab::RateLimits => rsx! {
                        RateLimitsTab {
                            rate_limits: rate_limits.read().clone(),
                            on_rate_limit_change: move |(op_type, val): (OperationType, Option<(u32, u32)>)| {
                                let mut rls = rate_limits.write();
                                if let Some(entry) = rls.iter_mut().find(|(ot, _)| *ot == op_type) {
                                    entry.1 = val;
                                }
                            },
                        }
                    },
                }
            }

            // Dialog footer
            div {
                class: "px-6 py-4 border-t border-slate-700 flex items-center justify-between",

                // Left side: helpful text
                div {
                    class: "text-xs text-slate-500",
                    "Endpoints will generate Axum handlers with SeaORM queries."
                }

                // Right side: buttons
                div {
                    class: "flex items-center gap-3",

                    button {
                        class: "px-4 py-2 text-slate-300 hover:text-white hover:bg-slate-700 rounded-lg transition-colors",
                        onclick: move |_| {
                            APP_STATE.write().ui.close_dialog();
                        },
                        "Cancel"
                    }

                    button {
                        class: "px-6 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium transition-colors",
                        onclick: on_submit,
                        "{submit_text}"
                    }
                }
            }
        }
    }
}

// ============================================================================
// General Tab
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct GeneralTabProps {
    mode_is_create: bool,
    entity_options: Vec<SelectOption>,
    selected_entity_id: String,
    on_entity_change: EventHandler<String>,
    base_path: String,
    on_base_path_change: EventHandler<String>,
    api_version: String,
    on_api_version_change: EventHandler<String>,
    description: String,
    on_description_change: EventHandler<String>,
    tags: String,
    on_tags_change: EventHandler<String>,
    is_enabled: bool,
    on_enabled_change: EventHandler<bool>,
}

#[component]
fn GeneralTab(props: GeneralTabProps) -> Element {
    rsx! {
        div {
            class: "space-y-5",

            // Entity selector
            div {
                Select {
                    label: "Entity",
                    value: props.selected_entity_id.clone(),
                    options: props.entity_options.clone(),
                    on_change: move |v: String| props.on_entity_change.call(v),
                }
                p {
                    class: "mt-1 text-xs text-slate-500",
                    "Select the entity to generate endpoints for."
                }
            }

            // Base path
            div {
                TextInput {
                    label: "Base Path",
                    value: props.base_path.clone(),
                    placeholder: "/api/users",
                    on_change: move |v: String| props.on_base_path_change.call(v),
                }
                p {
                    class: "mt-1 text-xs text-slate-500",
                    "The URL prefix for all operations. Must start with '/'."
                }
            }

            // Two-column: API version + Enabled
            div {
                class: "grid grid-cols-2 gap-4",

                // API version
                div {
                    TextInput {
                        label: "API Version (optional)",
                        value: props.api_version.clone(),
                        placeholder: "v1",
                        on_change: move |v: String| props.on_api_version_change.call(v),
                    }
                }

                // Enabled toggle
                div {
                    class: "flex items-end pb-1",
                    Toggle {
                        label: "Enabled",
                        checked: props.is_enabled,
                        on_change: move |v: bool| props.on_enabled_change.call(v),
                    }
                }
            }

            // Description
            div {
                TextArea {
                    label: "Description (optional)",
                    value: props.description.clone(),
                    placeholder: "CRUD endpoints for managing...",
                    on_change: move |v: String| props.on_description_change.call(v),
                }
            }

            // Tags
            div {
                TextInput {
                    label: "Tags (comma-separated)",
                    value: props.tags.clone(),
                    placeholder: "users, admin",
                    on_change: move |v: String| props.on_tags_change.call(v),
                }
                p {
                    class: "mt-1 text-xs text-slate-500",
                    "Tags for OpenAPI documentation grouping."
                }
            }

            // Preview box
            div {
                class: "p-4 bg-slate-900 rounded-lg border border-slate-700",
                h4 {
                    class: "text-sm font-medium text-slate-300 mb-2",
                    "Endpoint Preview"
                }
                div {
                    class: "space-y-1 font-mono text-xs",

                    // Show what the final paths look like
                    {
                        let bp = if props.api_version.is_empty() {
                            props.base_path.clone()
                        } else {
                            format!("/api/{}{}", props.api_version, props.base_path.trim_start_matches("/api"))
                        };
                        rsx! {
                            div {
                                class: "flex items-center gap-2",
                                span { class: "px-1.5 py-0.5 rounded bg-blue-900/40 text-blue-400 text-xs font-bold", "POST" }
                                span { class: "text-slate-400", "{bp}" }
                            }
                            div {
                                class: "flex items-center gap-2",
                                span { class: "px-1.5 py-0.5 rounded bg-green-900/40 text-green-400 text-xs font-bold", "GET" }
                                span { class: "text-slate-400", "{bp}" }
                            }
                            div {
                                class: "flex items-center gap-2",
                                span { class: "px-1.5 py-0.5 rounded bg-green-900/40 text-green-400 text-xs font-bold", "GET" }
                                span { class: "text-slate-400", "{bp}/:id" }
                            }
                            div {
                                class: "flex items-center gap-2",
                                span { class: "px-1.5 py-0.5 rounded bg-amber-900/40 text-amber-400 text-xs font-bold", "PUT" }
                                span { class: "text-slate-400", "{bp}/:id" }
                            }
                            div {
                                class: "flex items-center gap-2",
                                span { class: "px-1.5 py-0.5 rounded bg-red-900/40 text-red-400 text-xs font-bold", "DELETE" }
                                span { class: "text-slate-400", "{bp}/:id" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Operations Tab
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct OperationsTabProps {
    create_enabled: bool,
    on_create_toggle: EventHandler<bool>,
    read_enabled: bool,
    on_read_toggle: EventHandler<bool>,
    read_all_enabled: bool,
    on_read_all_toggle: EventHandler<bool>,
    update_enabled: bool,
    on_update_toggle: EventHandler<bool>,
    delete_enabled: bool,
    on_delete_toggle: EventHandler<bool>,
    base_path: String,
}

#[component]
fn OperationsTab(props: OperationsTabProps) -> Element {
    // Quick toggle helpers
    let all_enabled = props.create_enabled
        && props.read_enabled
        && props.read_all_enabled
        && props.update_enabled
        && props.delete_enabled;

    let none_enabled = !props.create_enabled
        && !props.read_enabled
        && !props.read_all_enabled
        && !props.update_enabled
        && !props.delete_enabled;

    rsx! {
        div {
            class: "space-y-4",

            // Quick actions
            div {
                class: "flex items-center gap-3 mb-2",

                button {
                    class: format!(
                        "px-3 py-1 rounded text-xs font-medium transition-colors {}",
                        if all_enabled {
                            "bg-indigo-600 text-white"
                        } else {
                            "bg-slate-700 text-slate-300 hover:bg-slate-600"
                        }
                    ),
                    onclick: move |_| {
                        props.on_create_toggle.call(true);
                        props.on_read_toggle.call(true);
                        props.on_read_all_toggle.call(true);
                        props.on_update_toggle.call(true);
                        props.on_delete_toggle.call(true);
                    },
                    "Enable All"
                }

                button {
                    class: "px-3 py-1 rounded text-xs font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors",
                    onclick: move |_| {
                        props.on_create_toggle.call(true);
                        props.on_read_toggle.call(true);
                        props.on_read_all_toggle.call(true);
                        props.on_update_toggle.call(false);
                        props.on_delete_toggle.call(false);
                    },
                    "Read Only"
                }

                button {
                    class: "px-3 py-1 rounded text-xs font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors",
                    onclick: move |_| {
                        props.on_create_toggle.call(false);
                        props.on_read_toggle.call(false);
                        props.on_read_all_toggle.call(false);
                        props.on_update_toggle.call(false);
                        props.on_delete_toggle.call(false);
                    },
                    "Disable All"
                }
            }

            // Operation rows
            OperationToggleRow {
                method: "POST",
                label: "Create",
                description: "Create a new resource",
                path: props.base_path.clone(),
                path_suffix: "",
                enabled: props.create_enabled,
                on_toggle: move |v| props.on_create_toggle.call(v),
            }

            OperationToggleRow {
                method: "GET",
                label: "Read",
                description: "Get a single resource by ID",
                path: props.base_path.clone(),
                path_suffix: "/:id",
                enabled: props.read_enabled,
                on_toggle: move |v| props.on_read_toggle.call(v),
            }

            OperationToggleRow {
                method: "GET",
                label: "List",
                description: "List all resources with pagination and filtering",
                path: props.base_path.clone(),
                path_suffix: "",
                enabled: props.read_all_enabled,
                on_toggle: move |v| props.on_read_all_toggle.call(v),
            }

            OperationToggleRow {
                method: "PUT",
                label: "Update",
                description: "Update an existing resource by ID",
                path: props.base_path.clone(),
                path_suffix: "/:id",
                enabled: props.update_enabled,
                on_toggle: move |v| props.on_update_toggle.call(v),
            }

            OperationToggleRow {
                method: "DELETE",
                label: "Delete",
                description: "Delete a resource by ID",
                path: props.base_path.clone(),
                path_suffix: "/:id",
                enabled: props.delete_enabled,
                on_toggle: move |v| props.on_delete_toggle.call(v),
            }

            // Summary
            div {
                class: "pt-3 border-t border-slate-700 text-sm text-slate-400",
                {
                    let count = [
                        props.create_enabled,
                        props.read_enabled,
                        props.read_all_enabled,
                        props.update_enabled,
                        props.delete_enabled,
                    ]
                    .iter()
                    .filter(|&&v| v)
                    .count();
                    rsx! {
                        span {
                            class: if count == 5 { "text-green-400" } else if count == 0 { "text-red-400" } else { "text-slate-400" },
                            "{count} of 5 operations enabled"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Operation Toggle Row
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct OperationToggleRowProps {
    method: String,
    label: String,
    description: String,
    path: String,
    path_suffix: String,
    enabled: bool,
    on_toggle: EventHandler<bool>,
}

#[component]
fn OperationToggleRow(props: OperationToggleRowProps) -> Element {
    let method_class = match props.method.as_str() {
        "GET" => "bg-green-900/40 text-green-400",
        "POST" => "bg-blue-900/40 text-blue-400",
        "PUT" => "bg-amber-900/40 text-amber-400",
        "DELETE" => "bg-red-900/40 text-red-400",
        _ => "bg-slate-700 text-slate-400",
    };

    rsx! {
        div {
            class: format!(
                "flex items-center gap-4 p-3 rounded-lg border transition-colors {}",
                if props.enabled {
                    "bg-slate-800 border-slate-700"
                } else {
                    "bg-slate-800/30 border-slate-700/50 opacity-60"
                }
            ),

            // Toggle
            div {
                class: "flex-shrink-0",
                Toggle {
                    checked: props.enabled,
                    on_change: move |v: bool| props.on_toggle.call(v),
                }
            }

            // Method badge
            span {
                class: format!("px-2.5 py-1 rounded text-xs font-bold uppercase min-w-[56px] text-center {}", method_class),
                "{props.method}"
            }

            // Info
            div {
                class: "flex-1 min-w-0",

                div {
                    class: "flex items-center gap-2",
                    span {
                        class: "font-medium text-white text-sm",
                        "{props.label}"
                    }
                    span {
                        class: "font-mono text-xs text-slate-500 truncate",
                        "{props.path}{props.path_suffix}"
                    }
                }

                p {
                    class: "text-xs text-slate-500 mt-0.5",
                    "{props.description}"
                }
            }
        }
    }
}

// ============================================================================
// Security Tab
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct SecurityTabProps {
    auth_required: bool,
    on_auth_change: EventHandler<bool>,
    roles: String,
    on_roles_change: EventHandler<String>,
    cors_enabled: bool,
    on_cors_change: EventHandler<bool>,
    per_op_overrides: Vec<(OperationType, Option<bool>)>,
    on_per_op_override_change: EventHandler<(OperationType, Option<bool>)>,
}

#[component]
fn SecurityTab(props: SecurityTabProps) -> Element {
    rsx! {
        div {
            class: "space-y-6",

            // Global security section
            div {
                h3 {
                    class: "text-sm font-semibold text-white mb-3 flex items-center gap-2",
                    svg {
                        class: "w-4 h-4 text-indigo-400",
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
                    "Global Security"
                }

                div {
                    class: "space-y-4 p-4 bg-slate-800 rounded-lg border border-slate-700",

                    // Auth required
                    div {
                        Toggle {
                            label: "Require Authentication",
                            checked: props.auth_required,
                            on_change: move |v: bool| props.on_auth_change.call(v),
                        }
                        p {
                            class: "mt-1 ml-12 text-xs text-slate-500",
                            "When enabled, all operations require a valid JWT token by default."
                        }
                    }

                    // Roles
                    if props.auth_required {
                        div {
                            TextInput {
                                label: "Required Roles (comma-separated)",
                                value: props.roles.clone(),
                                placeholder: "admin, editor",
                                on_change: move |v: String| props.on_roles_change.call(v),
                            }
                            p {
                                class: "mt-1 text-xs text-slate-500",
                                "Users must have at least one of these roles. Leave empty for any authenticated user."
                            }
                        }
                    }

                    // CORS
                    div {
                        Toggle {
                            label: "Enable CORS",
                            checked: props.cors_enabled,
                            on_change: move |v: bool| props.on_cors_change.call(v),
                        }
                        p {
                            class: "mt-1 ml-12 text-xs text-slate-500",
                            "Allow cross-origin requests to these endpoints."
                        }
                    }
                }
            }

            // Per-operation overrides
            div {
                h3 {
                    class: "text-sm font-semibold text-white mb-3 flex items-center gap-2",
                    svg {
                        class: "w-4 h-4 text-amber-400",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4",
                        }
                    }
                    "Per-Operation Overrides"
                }

                p {
                    class: "text-xs text-slate-500 mb-3",
                    "Override the global security setting for individual operations. \"Inherit\" uses the global setting."
                }

                div {
                    class: "space-y-2",

                    for (op_type, override_val) in props.per_op_overrides.iter() {
                        {
                            let ot = *op_type;
                            let current = *override_val;
                            let method = ot.http_method();
                            let method_class = match method {
                                "GET" => "bg-green-900/40 text-green-400",
                                "POST" => "bg-blue-900/40 text-blue-400",
                                "PUT" => "bg-amber-900/40 text-amber-400",
                                "DELETE" => "bg-red-900/40 text-red-400",
                                _ => "bg-slate-700 text-slate-400",
                            };

                            let options = vec![
                                SelectOption {
                                    value: "inherit".to_string(),
                                    label: "Inherit Global".to_string(),
                                    disabled: false,
                                },
                                SelectOption {
                                    value: "open".to_string(),
                                    label: "Open (No Auth)".to_string(),
                                    disabled: false,
                                },
                                SelectOption {
                                    value: "secured".to_string(),
                                    label: "Secured (Auth Required)".to_string(),
                                    disabled: false,
                                },
                            ];

                            let current_value = match current {
                                None => "inherit".to_string(),
                                Some(false) => "open".to_string(),
                                Some(true) => "secured".to_string(),
                            };

                            rsx! {
                                div {
                                    key: "{ot.display_name()}",
                                    class: "flex items-center gap-3 p-2 rounded-lg bg-slate-800/50",

                                    // Method badge
                                    span {
                                        class: format!("px-2 py-0.5 rounded text-xs font-bold min-w-[48px] text-center {}", method_class),
                                        "{method}"
                                    }

                                    // Operation name
                                    span {
                                        class: "text-sm text-slate-300 min-w-[60px]",
                                        "{ot.display_name()}"
                                    }

                                    // Override selector
                                    div {
                                        class: "flex-1",
                                        Select {
                                            value: current_value,
                                            options: options,
                                            on_change: move |v: String| {
                                                let new_val = match v.as_str() {
                                                    "open" => Some(false),
                                                    "secured" => Some(true),
                                                    _ => None,
                                                };
                                                props.on_per_op_override_change.call((ot, new_val));
                                            },
                                        }
                                    }

                                    // Effective indicator
                                    {
                                        let effective = current.unwrap_or(props.auth_required);
                                        rsx! {
                                            span {
                                                class: format!(
                                                    "px-2 py-0.5 rounded text-xs {}",
                                                    if effective {
                                                        "bg-amber-900/30 text-amber-400"
                                                    } else {
                                                        "bg-green-900/30 text-green-400"
                                                    }
                                                ),
                                                if effective { "\u{1F512}" } else { "\u{1F310}" }
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
// Rate Limits Tab
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct RateLimitsTabProps {
    rate_limits: Vec<(OperationType, Option<(u32, u32)>)>,
    on_rate_limit_change: EventHandler<(OperationType, Option<(u32, u32)>)>,
}

#[component]
fn RateLimitsTab(props: RateLimitsTabProps) -> Element {
    rsx! {
        div {
            class: "space-y-5",

            // Introduction
            div {
                class: "p-4 bg-slate-800/50 rounded-lg border border-slate-700",
                div {
                    class: "flex items-start gap-3",
                    svg {
                        class: "w-5 h-5 text-cyan-400 mt-0.5 flex-shrink-0",
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
                    div {
                        p {
                            class: "text-sm text-slate-300",
                            "Rate limiting protects your API from abuse. Configure per-operation limits or leave disabled for no rate limiting."
                        }
                        p {
                            class: "text-xs text-slate-500 mt-1",
                            "Limits are applied per-user by default. Generated code uses a token-bucket algorithm."
                        }
                    }
                }
            }

            // Preset buttons
            div {
                class: "flex items-center gap-2",
                span {
                    class: "text-xs text-slate-500 mr-2",
                    "Presets:"
                }

                button {
                    class: "px-3 py-1 rounded text-xs font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors",
                    onclick: move |_| {
                        for op_type in OperationType::all() {
                            props.on_rate_limit_change.call((*op_type, None));
                        }
                    },
                    "No Limits"
                }

                button {
                    class: "px-3 py-1 rounded text-xs font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors",
                    onclick: move |_| {
                        for op_type in OperationType::all() {
                            props.on_rate_limit_change.call((*op_type, Some((100, 60))));
                        }
                    },
                    "Permissive (100/min)"
                }

                button {
                    class: "px-3 py-1 rounded text-xs font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors",
                    onclick: move |_| {
                        for op_type in OperationType::all() {
                            if op_type.is_write() {
                                props.on_rate_limit_change.call((*op_type, Some((20, 60))));
                            } else {
                                props.on_rate_limit_change.call((*op_type, Some((60, 60))));
                            }
                        }
                    },
                    "Moderate"
                }

                button {
                    class: "px-3 py-1 rounded text-xs font-medium bg-slate-700 text-slate-300 hover:bg-slate-600 transition-colors",
                    onclick: move |_| {
                        for op_type in OperationType::all() {
                            props.on_rate_limit_change.call((*op_type, Some((10, 60))));
                        }
                    },
                    "Strict (10/min)"
                }
            }

            // Per-operation rate limits
            div {
                class: "space-y-3",

                for (op_type, rate_limit) in props.rate_limits.iter() {
                    {
                        let ot = *op_type;
                        let rl = *rate_limit;
                        let is_enabled = rl.is_some();
                        let requests = rl.map(|(r, _)| r).unwrap_or(100);
                        let window = rl.map(|(_, w)| w).unwrap_or(60);

                        let method = ot.http_method();
                        let method_class = match method {
                            "GET" => "bg-green-900/40 text-green-400",
                            "POST" => "bg-blue-900/40 text-blue-400",
                            "PUT" => "bg-amber-900/40 text-amber-400",
                            "DELETE" => "bg-red-900/40 text-red-400",
                            _ => "bg-slate-700 text-slate-400",
                        };

                        rsx! {
                            div {
                                key: "{ot.display_name()}-rl",
                                class: format!(
                                    "p-3 rounded-lg border transition-colors {}",
                                    if is_enabled {
                                        "bg-slate-800 border-slate-700"
                                    } else {
                                        "bg-slate-800/30 border-slate-700/50"
                                    }
                                ),

                                // Header row
                                div {
                                    class: "flex items-center gap-3 mb-2",

                                    // Toggle
                                    Toggle {
                                        checked: is_enabled,
                                        on_change: move |enabled: bool| {
                                            if enabled {
                                                props.on_rate_limit_change.call((ot, Some((100, 60))));
                                            } else {
                                                props.on_rate_limit_change.call((ot, None));
                                            }
                                        },
                                    }

                                    // Method badge
                                    span {
                                        class: format!("px-2 py-0.5 rounded text-xs font-bold min-w-[48px] text-center {}", method_class),
                                        "{method}"
                                    }

                                    // Operation name
                                    span {
                                        class: format!(
                                            "text-sm font-medium {}",
                                            if is_enabled { "text-white" } else { "text-slate-500" }
                                        ),
                                        "{ot.display_name()}"
                                    }

                                    // Rate display
                                    if is_enabled {
                                        span {
                                            class: "ml-auto text-xs text-cyan-400 font-mono",
                                            "{requests} req / {window}s"
                                        }
                                    }
                                }

                                // Config inputs (only when enabled)
                                if is_enabled {
                                    div {
                                        class: "flex items-center gap-3 ml-12",

                                        div {
                                            class: "flex-1",
                                            NumberInput {
                                                label: "Requests",
                                                value: requests as f64,
                                                min: 1.0,
                                                max: 100000.0,
                                                step: 1.0,
                                                on_change: move |v: f64| {
                                                    props.on_rate_limit_change.call((ot, Some((v as u32, window))));
                                                },
                                            }
                                        }

                                        div {
                                            class: "flex-1",
                                            NumberInput {
                                                label: "Window (seconds)",
                                                value: window as f64,
                                                min: 1.0,
                                                max: 86400.0,
                                                step: 1.0,
                                                on_change: move |v: f64| {
                                                    props.on_rate_limit_change.call((ot, Some((requests, v as u32))));
                                                },
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
// Helper Functions
// ============================================================================

/// Simple snake_case pluralization (matches the IR crate's logic)
fn to_snake_case_plural(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_was_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_was_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_was_upper = true;
        } else {
            result.push(c);
            prev_was_upper = false;
        }
    }

    let snake = result;
    if snake.ends_with('s')
        || snake.ends_with('x')
        || snake.ends_with("ch")
        || snake.ends_with("sh")
    {
        format!("{}es", snake)
    } else if snake.ends_with('y')
        && !snake.ends_with("ey")
        && !snake.ends_with("ay")
        && !snake.ends_with("oy")
    {
        format!("{}ies", &snake[..snake.len() - 1])
    } else {
        format!("{}s", snake)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_mode_create() {
        let mode = EndpointDialogMode::Create { entity_id: None };
        assert!(mode.is_create());
        assert_eq!(mode.title(), "Configure Endpoints");
        assert_eq!(mode.submit_text(), "Create Endpoints");
    }

    #[test]
    fn test_dialog_mode_edit() {
        let mode = EndpointDialogMode::Edit(Uuid::new_v4());
        assert!(!mode.is_create());
        assert_eq!(mode.title(), "Edit Endpoint Configuration");
        assert_eq!(mode.submit_text(), "Save Changes");
    }

    #[test]
    fn test_active_tab() {
        assert_eq!(ActiveTab::General.label(), "General");
        assert_eq!(ActiveTab::Operations.label(), "Operations");
        assert_eq!(ActiveTab::Security.label(), "Security");
        assert_eq!(ActiveTab::RateLimits.label(), "Rate Limits");
        assert_eq!(ActiveTab::all().len(), 4);
    }

    #[test]
    fn test_to_snake_case_plural() {
        assert_eq!(to_snake_case_plural("User"), "users");
        assert_eq!(to_snake_case_plural("Post"), "posts");
        assert_eq!(to_snake_case_plural("Category"), "categories");
        assert_eq!(to_snake_case_plural("Address"), "addresses");
    }
}
