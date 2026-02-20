//! # Endpoints Configuration Page
//!
//! Page for configuring REST API endpoints for each entity.
//!
//! This page provides:
//! - A grid/list of endpoint groups per entity
//! - CRUD operation toggles (Create, Read, ReadAll, Update, Delete)
//! - Security configuration (open, authenticated, role-based)
//! - Rate limiting per operation
//! - Auto-generation of endpoints for uncovered entities
//! - Integrated (nested) endpoint previews based on relationships
//!
//! ## Usage
//!
//! The endpoints page is accessible from the sidebar when a project is open.
//! Users can:
//! - Click on an endpoint card to select it
//! - Double-click to open the full configuration dialog
//! - Toggle individual CRUD operations directly on the cards
//! - Use the toolbar to create or auto-generate endpoint groups
//! - View and edit security and rate-limiting in the properties panel

use dioxus::prelude::*;
use imortal_ir::{EndpointGroup, EndpointSecurity, Entity, OperationType, Relationship};
use uuid::Uuid;

use crate::components::endpoint_card::{EndpointCard, GenerateEndpointsCard, http_method_class};
use crate::components::inputs::{Select, SelectOption, TextInput, Toggle};
use crate::state::{APP_STATE, DeleteTarget, Dialog, StatusLevel};

// ============================================================================
// Endpoints Page Component
// ============================================================================

/// Main endpoints configuration page component
#[component]
pub fn EndpointsPage() -> Element {
    // Local UI state
    let mut view_mode = use_signal(|| ViewMode::Grid);
    let mut search_query = use_signal(String::new);
    let mut filter = use_signal(|| EndpointFilter::All);
    let mut show_integrated = use_signal(|| true);

    // Read global state
    let state = APP_STATE.read();
    let has_project = state.project.is_some();

    let entities: Vec<Entity> = state
        .project
        .as_ref()
        .map(|p| {
            let mut list: Vec<Entity> = p.entities.values().cloned().collect();
            list.sort_by(|a, b| a.name.cmp(&b.name));
            list
        })
        .unwrap_or_default();

    let endpoints: Vec<EndpointGroup> = state
        .project
        .as_ref()
        .map(|p| {
            let mut list: Vec<EndpointGroup> = p.endpoints.values().cloned().collect();
            list.sort_by(|a, b| a.entity_name.cmp(&b.entity_name));
            list
        })
        .unwrap_or_default();

    let relationships: Vec<Relationship> = state
        .project
        .as_ref()
        .map(|p| p.relationships.values().cloned().collect())
        .unwrap_or_default();

    let selected_endpoints = state.selection.endpoints.clone();

    // Auth config
    let auth_enabled = state
        .project
        .as_ref()
        .map(|p| p.config.auth.enabled)
        .unwrap_or(false);

    drop(state);

    // Entities without endpoints
    let endpoint_entity_ids: Vec<Uuid> = endpoints.iter().map(|ep| ep.entity_id).collect();
    let uncovered_entities: Vec<&Entity> = entities
        .iter()
        .filter(|e| !endpoint_entity_ids.contains(&e.id))
        .collect();
    let uncovered_count = uncovered_entities.len();

    // Filter endpoints
    let filtered_endpoints: Vec<&EndpointGroup> = endpoints
        .iter()
        .filter(|ep| {
            // Search filter
            let search = search_query.read().to_lowercase();
            let search_match = search.is_empty()
                || ep.entity_name.to_lowercase().contains(&search)
                || ep.base_path.to_lowercase().contains(&search)
                || ep
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&search))
                    .unwrap_or(false)
                || ep.tags.iter().any(|t| t.to_lowercase().contains(&search));

            // Type filter
            let filter_match = match *filter.read() {
                EndpointFilter::All => true,
                EndpointFilter::Enabled => ep.enabled,
                EndpointFilter::Disabled => !ep.enabled,
                EndpointFilter::Secured => ep.requires_auth(),
                EndpointFilter::Open => !ep.requires_auth(),
            };

            search_match && filter_match
        })
        .collect();

    // Build integrated (nested) endpoints from relationships — always computed
    let initial_integrated = build_integrated_endpoints(&endpoints, &relationships, &entities);
    let mut integrated_endpoints: Signal<Vec<IntegratedEndpoint>> =
        use_signal(|| initial_integrated);

    // Build auth endpoints when auth is enabled
    let auth_endpoints: Vec<AuthEndpoint> = if auth_enabled {
        build_auth_endpoints()
    } else {
        Vec::new()
    };
    let selected_for_toolbar = selected_endpoints.clone();
    let selected_for_grid = selected_endpoints.clone();
    let selected_for_list = selected_endpoints.clone();
    let selected_for_panel = selected_endpoints.clone();

    // Event handlers
    let on_select = move |ep_id: Uuid| {
        let mut state = APP_STATE.write();
        state.selection.clear();
        state.selection.endpoints.insert(ep_id);
    };

    let on_edit = move |ep_id: Uuid| {
        APP_STATE
            .write()
            .ui
            .show_dialog(Dialog::EditEndpoint(ep_id));
    };

    let on_delete = move |ep_id: Uuid| {
        APP_STATE
            .write()
            .ui
            .show_dialog(Dialog::ConfirmDelete(DeleteTarget::Endpoint(ep_id)));
    };

    let on_create = move |_| {
        APP_STATE.write().ui.show_dialog(Dialog::NewEndpoint(None));
    };

    let on_toggle_operation = move |(ep_id, op_type, enabled): (Uuid, OperationType, bool)| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(ep) = project.get_endpoint_mut(ep_id) {
                if enabled {
                    ep.enable_operation(op_type);
                } else {
                    ep.disable_operation(op_type);
                }
            }
        }
        state.is_dirty = true;
    };

    let on_toggle_enabled = move |(ep_id, enabled): (Uuid, bool)| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            if let Some(ep) = project.get_endpoint_mut(ep_id) {
                ep.enabled = enabled;
            }
        }
        state.is_dirty = true;
    };

    // Auto-generate endpoints for all uncovered entities
    let entities_for_generate = entities.clone();
    let endpoint_entity_ids_for_gen = endpoint_entity_ids.clone();
    let on_generate_all = move |_| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            let mut count = 0;
            for entity in &entities_for_generate {
                if !endpoint_entity_ids_for_gen.contains(&entity.id) {
                    let endpoint = EndpointGroup::new(entity.id, &entity.name);
                    project.add_endpoint(endpoint);
                    count += 1;
                }
            }
            if count > 0 {
                tracing::info!("Auto-generated endpoints for {} entities", count);
                state.is_dirty = true;
                state.ui.set_status(
                    format!("Generated endpoints for {} entities", count),
                    StatusLevel::Success,
                );
            }
        }
    };

    // Secure all / unsecure all
    let on_secure_all = move |_| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            for ep in project.endpoints.values_mut() {
                ep.global_security.auth_required = true;
            }
            state.is_dirty = true;
            state.ui.set_status(
                "All endpoints now require authentication".to_string(),
                StatusLevel::Success,
            );
        }
    };

    let on_open_all = move |_| {
        let mut state = APP_STATE.write();
        if let Some(project) = &mut state.project {
            for ep in project.endpoints.values_mut() {
                ep.global_security = EndpointSecurity::open();
            }
            state.is_dirty = true;
            state.ui.set_status(
                "All endpoints set to public".to_string(),
                StatusLevel::Success,
            );
        }
    };

    // Filter options
    let filter_options = vec![
        SelectOption {
            value: "all".to_string(),
            label: "All Endpoints".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "enabled".to_string(),
            label: "Enabled Only".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "disabled".to_string(),
            label: "Disabled Only".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "secured".to_string(),
            label: "Secured (Auth Required)".to_string(),
            disabled: false,
        },
        SelectOption {
            value: "open".to_string(),
            label: "Public (No Auth)".to_string(),
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
            class: "endpoints-page flex flex-col h-full",

            // Toolbar
            EndpointsToolbar {
                endpoint_count: endpoints.len(),
                entity_count: entities.len(),
                selected_count: selected_for_toolbar.len(),
                uncovered_count: uncovered_count,
                auth_enabled: auth_enabled,
                view_mode: *view_mode.read(),
                on_view_mode_change: move |mode| view_mode.set(mode),
                on_create: on_create,
                on_delete: move |_| {
                    if let Some(ep_id) = selected_for_toolbar.iter().next() {
                        APP_STATE.write().ui.show_dialog(
                            Dialog::ConfirmDelete(DeleteTarget::Endpoint(*ep_id)),
                        );
                    }
                },
                on_generate_all: on_generate_all,
                on_secure_all: on_secure_all,
                on_open_all: on_open_all,
            }

            // Main content area
            div {
                class: "flex-1 flex overflow-hidden",

                // Left side: endpoint list / grid
                div {
                    class: "flex-1 flex flex-col overflow-hidden",

                    // Search and filter bar
                    div {
                        class: "p-3 border-b border-slate-700 flex gap-3 items-center",

                        // Search
                        div {
                            class: "flex-1",
                            TextInput {
                                value: search_query.read().clone(),
                                placeholder: "Search endpoints, entities, tags...",
                                on_change: move |v: String| search_query.set(v),
                            }
                        }

                        // Filter
                        div {
                            class: "w-48",
                            Select {
                                value: filter_to_string(&filter.read()),
                                options: filter_options,
                                on_change: move |v: String| {
                                    filter.set(string_to_filter(&v));
                                },
                            }
                        }

                        // Show nested/relationship endpoints toggle
                        div {
                            class: "flex items-center",
                            Toggle {
                                label: "Nested Routes",
                                checked: *show_integrated.read(),
                                on_change: move |v: bool| show_integrated.set(v),
                            }
                        }
                    }

                    // Content
                    div {
                        class: "flex-1 overflow-y-auto p-4",

                        // Auto-generate prompt (if entities lack endpoints)
                        if uncovered_count > 0 {
                            div {
                                class: "mb-4",
                                GenerateEndpointsCard {
                                    uncovered_count: uncovered_count,
                                    on_generate: move |_| {
                                        // Trigger the same logic as toolbar generate
                                        let mut state = APP_STATE.write();
                                        if let Some(project) = &mut state.project {
                                            let mut count = 0;
                                            let existing: Vec<Uuid> = project.endpoints.values().map(|ep| ep.entity_id).collect();
                                            let entity_list: Vec<(Uuid, String)> = project.entities.values().map(|e| (e.id, e.name.clone())).collect();
                                            for (eid, ename) in &entity_list {
                                                if !existing.contains(eid) {
                                                    let endpoint = EndpointGroup::new(*eid, ename);
                                                    project.add_endpoint(endpoint);
                                                    count += 1;
                                                }
                                            }
                                            if count > 0 {
                                                state.is_dirty = true;
                                                state.ui.set_status(format!("Generated endpoints for {} entities", count), StatusLevel::Success);
                                            }
                                        }
                                    },
                                }
                            }
                        }

                        // Empty state
                        if filtered_endpoints.is_empty() && endpoints.is_empty() && uncovered_count == 0 {
                            EmptyState {}
                        } else if filtered_endpoints.is_empty() && !endpoints.is_empty() {
                            div {
                                class: "flex items-center justify-center py-12",
                                div {
                                    class: "text-center",
                                    p {
                                        class: "text-slate-400 mb-2",
                                        "No endpoints match your filter"
                                    }
                                    button {
                                        class: "text-sm text-indigo-400 hover:text-indigo-300",
                                        onclick: move |_| {
                                            search_query.set(String::new());
                                            filter.set(EndpointFilter::All);
                                        },
                                        "Clear filters"
                                    }
                                }
                            }
                        } else {
                            // Endpoint cards
                            match *view_mode.read() {
                                ViewMode::Grid => rsx! {
                                    div {
                                        class: "grid grid-cols-1 xl:grid-cols-2 gap-4",

                                        for ep in filtered_endpoints.iter() {
                                            {
                                                let endpoint = (*ep).clone();
                                                let ep_id = endpoint.id;
                                                let is_selected = selected_for_grid.contains(&ep_id);
                                                rsx! {
                                                    EndpointCard {
                                                        key: "{ep_id}",
                                                        endpoint: endpoint,
                                                        is_selected: is_selected,
                                                        on_select: on_select,
                                                        on_edit: on_edit,
                                                        on_toggle_operation: on_toggle_operation,
                                                        on_toggle_enabled: on_toggle_enabled,
                                                        on_delete: on_delete,
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                ViewMode::List => rsx! {
                                    EndpointsListView {
                                        endpoints: filtered_endpoints.iter().map(|e| (*e).clone()).collect(),
                                        entities: entities.clone(),
                                        selected_endpoints: selected_for_list.iter().copied().collect(),
                                        on_select: on_select,
                                        on_edit: on_edit,
                                        on_delete: on_delete,
                                        on_toggle_enabled: on_toggle_enabled,
                                    }
                                },
                                ViewMode::Compact => rsx! {
                                    div {
                                        class: "grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3",

                                        for ep in filtered_endpoints.iter() {
                                            {
                                                let endpoint = (*ep).clone();
                                                let ep_id = endpoint.id;
                                                let is_selected = selected_for_grid.contains(&ep_id);
                                                rsx! {
                                                    EndpointCard {
                                                        key: "{ep_id}",
                                                        endpoint: endpoint,
                                                        is_selected: is_selected,
                                                        compact: true,
                                                        on_select: on_select,
                                                        on_edit: on_edit,
                                                        on_toggle_operation: on_toggle_operation,
                                                        on_toggle_enabled: on_toggle_enabled,
                                                        on_delete: on_delete,
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                            }
                        }

                        // Auth endpoints section (when auth enabled)
                        if !auth_endpoints.is_empty() {
                            div {
                                class: "mt-6",
                                AuthEndpointsSection {
                                    endpoints: auth_endpoints.clone(),
                                }
                            }
                        }

                        // Relationship-based (nested) endpoints
                        if *show_integrated.read() && !integrated_endpoints.read().is_empty() {
                            div {
                                class: "mt-6",
                                IntegratedEndpointsSection {
                                    endpoints: integrated_endpoints.read().clone(),
                                    on_toggle: move |id: String| {
                                        let mut eps = integrated_endpoints.write();
                                        if let Some(ep) = eps.iter_mut().find(|e| e.id == id) {
                                            ep.enabled = !ep.enabled;
                                        }
                                    },
                                    on_toggle_all: move |enabled: bool| {
                                        let mut eps = integrated_endpoints.write();
                                        for ep in eps.iter_mut() {
                                            ep.enabled = enabled;
                                        }
                                    },
                                }
                            }
                        }
                    }
                }

                // Right panel: properties
                div {
                    class: "w-80 border-l border-slate-700 overflow-y-auto bg-slate-800/50",

                    EndpointPropertiesPanel {
                        selected_endpoint: selected_for_panel.iter().next().copied(),
                        entities: entities.clone(),
                        auth_enabled: auth_enabled,
                    }
                }
            }

            // Status bar
            EndpointsStatusBar {
                total: endpoints.len(),
                filtered: filtered_endpoints.len(),
                selected: selected_endpoints.len(),
                entities_total: entities.len(),
                uncovered: uncovered_count,
                auth_enabled: auth_enabled,
            }
        }
    }
}

// ============================================================================
// View Mode
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Grid,
    List,
    Compact,
}

// ============================================================================
// Filter
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum EndpointFilter {
    #[default]
    All,
    Enabled,
    Disabled,
    Secured,
    Open,
}

fn filter_to_string(filter: &EndpointFilter) -> String {
    match filter {
        EndpointFilter::All => "all".to_string(),
        EndpointFilter::Enabled => "enabled".to_string(),
        EndpointFilter::Disabled => "disabled".to_string(),
        EndpointFilter::Secured => "secured".to_string(),
        EndpointFilter::Open => "open".to_string(),
    }
}

fn string_to_filter(s: &str) -> EndpointFilter {
    match s {
        "enabled" => EndpointFilter::Enabled,
        "disabled" => EndpointFilter::Disabled,
        "secured" => EndpointFilter::Secured,
        "open" => EndpointFilter::Open,
        _ => EndpointFilter::All,
    }
}

// ============================================================================
// Integrated (Nested) Endpoint
// ============================================================================

/// Represents a nested/integrated endpoint derived from a relationship
/// e.g., GET /users/:id/posts
#[derive(Debug, Clone, PartialEq)]
struct IntegratedEndpoint {
    /// Unique identifier for toggle tracking
    id: String,
    /// Display description — dynamically generated from entity names
    description: String,
    /// Detailed explanation of what this endpoint does
    explanation: String,
    /// HTTP method
    method: String,
    /// Full path (e.g., /api/users/:user_id/posts)
    path: String,
    /// Parent entity name
    parent_entity: String,
    /// Child entity name
    child_entity: String,
    /// Relationship name
    relationship_name: String,
    /// Whether this endpoint is enabled (user can toggle)
    enabled: bool,
}

/// Represents an authentication endpoint auto-generated when auth is enabled
#[derive(Debug, Clone, PartialEq)]
struct AuthEndpoint {
    /// HTTP method
    method: String,
    /// Full path (e.g., /api/auth/login)
    path: String,
    /// Display name
    name: String,
    /// Description
    description: String,
    /// Whether this endpoint requires authentication
    requires_auth: bool,
}

/// Build the standard authentication endpoints.
fn build_auth_endpoints() -> Vec<AuthEndpoint> {
    vec![
        AuthEndpoint {
            method: "POST".to_string(),
            path: "/api/auth/register".to_string(),
            name: "Register".to_string(),
            description: "Create a new user account with email and password".to_string(),
            requires_auth: false,
        },
        AuthEndpoint {
            method: "POST".to_string(),
            path: "/api/auth/login".to_string(),
            name: "Login".to_string(),
            description: "Authenticate with email/password and receive a JWT token".to_string(),
            requires_auth: false,
        },
        AuthEndpoint {
            method: "POST".to_string(),
            path: "/api/auth/logout".to_string(),
            name: "Logout".to_string(),
            description: "Invalidate the current session/token".to_string(),
            requires_auth: true,
        },
        AuthEndpoint {
            method: "GET".to_string(),
            path: "/api/auth/me".to_string(),
            name: "Current User".to_string(),
            description: "Get the profile of the currently authenticated user".to_string(),
            requires_auth: true,
        },
        AuthEndpoint {
            method: "POST".to_string(),
            path: "/api/auth/refresh".to_string(),
            name: "Refresh Token".to_string(),
            description: "Exchange a valid token for a new one with extended expiry".to_string(),
            requires_auth: true,
        },
        AuthEndpoint {
            method: "PUT".to_string(),
            path: "/api/auth/me/password".to_string(),
            name: "Change Password".to_string(),
            description: "Change the current user's password (requires old password)".to_string(),
            requires_auth: true,
        },
        AuthEndpoint {
            method: "POST".to_string(),
            path: "/api/auth/forgot-password".to_string(),
            name: "Forgot Password".to_string(),
            description: "Request a password reset email".to_string(),
            requires_auth: false,
        },
        AuthEndpoint {
            method: "POST".to_string(),
            path: "/api/auth/reset-password".to_string(),
            name: "Reset Password".to_string(),
            description: "Reset password using a token from the reset email".to_string(),
            requires_auth: false,
        },
    ]
}

fn build_integrated_endpoints(
    endpoints: &[EndpointGroup],
    relationships: &[Relationship],
    entities: &[Entity],
) -> Vec<IntegratedEndpoint> {
    let mut integrated = Vec::new();

    let get_entity_name = |id: Uuid| -> Option<String> {
        entities.iter().find(|e| e.id == id).map(|e| e.name.clone())
    };

    let get_endpoint_for_entity = |entity_id: Uuid| -> Option<&EndpointGroup> {
        endpoints.iter().find(|ep| ep.entity_id == entity_id)
    };

    for rel in relationships {
        // For OneToMany: parent has many children
        let (parent_id, child_id) = match &rel.relation_type {
            imortal_core::RelationType::OneToMany => (rel.from_entity_id, rel.to_entity_id),
            imortal_core::RelationType::ManyToOne => (rel.to_entity_id, rel.from_entity_id),
            _ => continue, // Skip 1:1 and M:N for now
        };

        let Some(parent_name) = get_entity_name(parent_id) else {
            continue;
        };
        let Some(child_name) = get_entity_name(child_id) else {
            continue;
        };
        let Some(parent_ep) = get_endpoint_for_entity(parent_id) else {
            continue;
        };

        let parent_singular = to_snake_case(&parent_name);
        let child_singular = to_snake_case(&child_name);
        let child_plural = to_snake_case_plural(&child_name);
        let base = format!(
            "{}/:{}_{}/{}",
            parent_ep.base_path, parent_singular, "id", child_plural
        );
        let rel_name = rel.name.clone();
        let pair_key = format!("{}_{}", parent_singular, child_singular);

        // GET /api/users/:user_id/posts — list children of parent
        integrated.push(IntegratedEndpoint {
            id: format!("{}_list", pair_key),
            description: format!("List {}s of a {}", child_name, parent_name),
            explanation: format!(
                "Returns a paginated list of all {} records that belong to the specified {}. \
                 Useful for displaying a {}'s {} on their profile or detail page. \
                 Filters by the foreign key relationship automatically.",
                child_name, parent_name, parent_name, child_plural
            ),
            method: "GET".to_string(),
            path: base.clone(),
            parent_entity: parent_name.clone(),
            child_entity: child_name.clone(),
            relationship_name: rel_name.clone(),
            enabled: true,
        });

        // POST /api/users/:user_id/posts — create child under parent
        integrated.push(IntegratedEndpoint {
            id: format!("{}_create", pair_key),
            description: format!("Create {} for a {}", child_name, parent_name),
            explanation: format!(
                "Creates a new {} and automatically associates it with the specified {} \
                 by setting the foreign key. The {}'s ID is taken from the URL path, \
                 so the request body doesn't need to include it.",
                child_name, parent_name, parent_name
            ),
            method: "POST".to_string(),
            path: base.clone(),
            parent_entity: parent_name.clone(),
            child_entity: child_name.clone(),
            relationship_name: rel_name.clone(),
            enabled: true,
        });

        // GET /api/users/:user_id/posts/:post_id — get specific child under parent
        integrated.push(IntegratedEndpoint {
            id: format!("{}_get", pair_key),
            description: format!("Get specific {} of a {}", child_name, parent_name),
            explanation: format!(
                "Retrieves a single {} by its ID, but only if it belongs to the specified {}. \
                 Returns 404 if the {} doesn't exist or doesn't belong to that {}. \
                 This ensures proper data scoping and prevents unauthorized access.",
                child_name, parent_name, child_name, parent_name
            ),
            method: "GET".to_string(),
            path: format!("{}/:{}_id", base, child_singular),
            parent_entity: parent_name.clone(),
            child_entity: child_name.clone(),
            relationship_name: rel_name.clone(),
            enabled: false, // Disabled by default — the flat GET /:id usually suffices
        });

        // DELETE /api/users/:user_id/posts/:post_id — remove child from parent
        integrated.push(IntegratedEndpoint {
            id: format!("{}_delete", pair_key),
            description: format!("Delete {} from a {}", child_name, parent_name),
            explanation: format!(
                "Deletes a {} that belongs to the specified {}. Verifies ownership before \
                 deleting — the {} must actually belong to that {} or the request is rejected. \
                 Uses the entity's soft-delete setting if configured.",
                child_name, parent_name, child_name, parent_name
            ),
            method: "DELETE".to_string(),
            path: format!("{}/:{}_id", base, child_singular),
            parent_entity: parent_name.clone(),
            child_entity: child_name.clone(),
            relationship_name: rel_name.clone(),
            enabled: false, // Disabled by default
        });

        // GET /api/posts/:post_id/comments/count — count children
        integrated.push(IntegratedEndpoint {
            id: format!("{}_count", pair_key),
            description: format!("Count {}s of a {}", child_name, parent_name),
            explanation: format!(
                "Returns just the count of {} records belonging to the specified {}, \
                 without fetching the actual data. Efficient for displaying badges \
                 like \"12 comments\" or \"5 posts\" without loading all records.",
                child_name, parent_name
            ),
            method: "GET".to_string(),
            path: format!("{}/count", base),
            parent_entity: parent_name.clone(),
            child_entity: child_name.clone(),
            relationship_name: rel_name.clone(),
            enabled: true,
        });
    }

    integrated
}

// ============================================================================
// Toolbar Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct EndpointsToolbarProps {
    endpoint_count: usize,
    entity_count: usize,
    selected_count: usize,
    uncovered_count: usize,
    auth_enabled: bool,
    view_mode: ViewMode,
    on_view_mode_change: EventHandler<ViewMode>,
    on_create: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_generate_all: EventHandler<()>,
    on_secure_all: EventHandler<()>,
    on_open_all: EventHandler<()>,
}

#[component]
fn EndpointsToolbar(props: EndpointsToolbarProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between px-4 py-2 bg-slate-800 border-b border-slate-700",

            // Left side: title and counts
            div {
                class: "flex items-center gap-4",

                h2 {
                    class: "text-lg font-semibold text-white flex items-center gap-2",
                    // API endpoint icon
                    svg {
                        class: "w-5 h-5 text-indigo-400",
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
                    "Endpoints"
                }

                span {
                    class: "px-2 py-0.5 bg-slate-700 rounded text-sm text-slate-300",
                    "{props.endpoint_count} configured"
                }

                if props.uncovered_count > 0 {
                    span {
                        class: "px-2 py-0.5 bg-amber-900/30 text-amber-400 rounded text-sm",
                        "{props.uncovered_count} uncovered"
                    }
                }

                if props.selected_count > 0 {
                    span {
                        class: "px-2 py-0.5 bg-indigo-600 rounded text-sm text-white",
                        "{props.selected_count} selected"
                    }
                }

                // Auth status indicator
                span {
                    class: format!(
                        "flex items-center gap-1 px-2 py-0.5 rounded text-xs {}",
                        if props.auth_enabled {
                            "bg-green-900/30 text-green-400"
                        } else {
                            "bg-slate-700 text-slate-500"
                        }
                    ),
                    if props.auth_enabled {
                        "🔐 Auth Enabled"
                    } else {
                        "🔓 No Auth"
                    }
                }
            }

            // Center: view mode toggle
            div {
                class: "flex items-center gap-1 bg-slate-700 rounded-lg p-1",

                button {
                    class: format!(
                        "px-3 py-1 rounded text-sm transition-colors {}",
                        if props.view_mode == ViewMode::Grid {
                            "bg-indigo-600 text-white"
                        } else {
                            "text-slate-300 hover:text-white"
                        }
                    ),
                    onclick: move |_| props.on_view_mode_change.call(ViewMode::Grid),
                    "Grid"
                }

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

                button {
                    class: format!(
                        "px-3 py-1 rounded text-sm transition-colors {}",
                        if props.view_mode == ViewMode::Compact {
                            "bg-indigo-600 text-white"
                        } else {
                            "text-slate-300 hover:text-white"
                        }
                    ),
                    onclick: move |_| props.on_view_mode_change.call(ViewMode::Compact),
                    "Compact"
                }
            }

            // Right side: actions
            div {
                class: "flex items-center gap-2",

                // Bulk security actions
                if props.endpoint_count > 0 {
                    div {
                        class: "flex items-center gap-1 mr-2",

                        button {
                            class: "px-2 py-1 text-xs text-slate-400 hover:text-amber-400 hover:bg-slate-700 rounded transition-colors",
                            title: "Require auth on all endpoints",
                            onclick: move |_| props.on_secure_all.call(()),
                            "🔒 Secure All"
                        }

                        button {
                            class: "px-2 py-1 text-xs text-slate-400 hover:text-green-400 hover:bg-slate-700 rounded transition-colors",
                            title: "Make all endpoints public",
                            onclick: move |_| props.on_open_all.call(()),
                            "🌐 Open All"
                        }
                    }
                }

                // Auto-generate button
                if props.uncovered_count > 0 {
                    button {
                        class: "px-3 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white text-sm rounded transition-colors flex items-center gap-1",
                        onclick: move |_| props.on_generate_all.call(()),
                        svg {
                            class: "w-4 h-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M13 10V3L4 14h7v7l9-11h-7z",
                            }
                        }
                        "Auto-Generate"
                    }
                }

                // Delete button (if selected)
                if props.selected_count > 0 {
                    button {
                        class: "px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white text-sm rounded transition-colors flex items-center gap-1",
                        onclick: move |_| props.on_delete.call(()),
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
                    "New Endpoint"
                }
            }
        }
    }
}

// ============================================================================
// List View Component
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct EndpointsListViewProps {
    endpoints: Vec<EndpointGroup>,
    entities: Vec<Entity>,
    selected_endpoints: Vec<Uuid>,
    on_select: EventHandler<Uuid>,
    on_edit: EventHandler<Uuid>,
    on_delete: EventHandler<Uuid>,
    on_toggle_enabled: EventHandler<(Uuid, bool)>,
}

#[component]
fn EndpointsListView(props: EndpointsListViewProps) -> Element {
    if props.endpoints.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "flex-1 overflow-y-auto",

            // Table header
            div {
                class: "sticky top-0 bg-slate-800 border-b border-slate-700 grid grid-cols-8 gap-3 px-4 py-2 text-sm font-medium text-slate-400",
                span { "Enabled" }
                span { "Entity" }
                span { "Base Path" }
                span { "Operations" }
                span { "Security" }
                span { "Rate Limit" }
                span { "Version" }
                span { "Actions" }
            }

            // Table body
            div {
                class: "divide-y divide-slate-700/50",

                for endpoint in props.endpoints.iter() {
                    {
                        let ep_id = endpoint.id;
                        let is_selected = props.selected_endpoints.contains(&ep_id);
                        let enabled_ops = endpoint.enabled_operations().len();
                        let total_ops = endpoint.operations.len();
                        let has_auth = endpoint.requires_auth();
                        let has_rate_limit = endpoint.operations.iter().any(|op| op.rate_limit.is_some());
                        let is_enabled = endpoint.enabled;

                        rsx! {
                            div {
                                key: "{ep_id}",
                                class: format!(
                                    "grid grid-cols-8 gap-3 px-4 py-3 cursor-pointer transition-colors {}",
                                    if is_selected {
                                        "bg-indigo-900/30 border-l-2 border-indigo-500"
                                    } else {
                                        "hover:bg-slate-700/30 border-l-2 border-transparent"
                                    }
                                ),
                                onclick: move |_| props.on_select.call(ep_id),
                                ondoubleclick: move |_| props.on_edit.call(ep_id),

                                // Enabled toggle
                                span {
                                    class: "flex items-center",
                                    button {
                                        class: format!(
                                            "relative inline-flex h-5 w-9 items-center rounded-full transition-colors {}",
                                            if is_enabled { "bg-indigo-600" } else { "bg-slate-600" }
                                        ),
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            props.on_toggle_enabled.call((ep_id, !is_enabled));
                                        },
                                        span {
                                            class: format!(
                                                "inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform {}",
                                                if is_enabled { "translate-x-4" } else { "translate-x-1" }
                                            ),
                                        }
                                    }
                                }

                                // Entity name
                                span {
                                    class: format!(
                                        "font-medium truncate {}",
                                        if is_enabled { "text-white" } else { "text-slate-500" }
                                    ),
                                    "{endpoint.entity_name}"
                                }

                                // Base path
                                span {
                                    class: "font-mono text-xs text-slate-400 truncate flex items-center",
                                    "{endpoint.full_base_path()}"
                                }

                                // Operations count
                                span {
                                    class: "flex items-center gap-1",
                                    // Mini method badges
                                    for op in endpoint.operations.iter() {
                                        {
                                            let method = op.http_method();
                                            rsx! {
                                                span {
                                                    class: format!(
                                                        "w-1.5 h-1.5 rounded-full {}",
                                                        if op.enabled {
                                                            match method {
                                                                "GET" => "bg-green-400",
                                                                "POST" => "bg-blue-400",
                                                                "PUT" => "bg-amber-400",
                                                                "DELETE" => "bg-red-400",
                                                                _ => "bg-slate-400",
                                                            }
                                                        } else {
                                                            "bg-slate-600"
                                                        }
                                                    ),
                                                    title: format!("{} {}", method, if op.enabled { "enabled" } else { "disabled" }),
                                                }
                                            }
                                        }
                                    }
                                    span {
                                        class: "text-xs text-slate-500 ml-1",
                                        "{enabled_ops}/{total_ops}"
                                    }
                                }

                                // Security
                                span {
                                    if has_auth {
                                        span {
                                            class: "px-2 py-0.5 bg-amber-900/30 text-amber-400 rounded text-xs",
                                            "🔒 Auth"
                                        }
                                    } else {
                                        span {
                                            class: "px-2 py-0.5 bg-green-900/30 text-green-400 rounded text-xs",
                                            "🌐 Open"
                                        }
                                    }
                                }

                                // Rate limit
                                span {
                                    if has_rate_limit {
                                        span {
                                            class: "px-2 py-0.5 bg-cyan-900/30 text-cyan-400 rounded text-xs",
                                            "⏱ Limited"
                                        }
                                    } else {
                                        span {
                                            class: "text-xs text-slate-600",
                                            "—"
                                        }
                                    }
                                }

                                // Version
                                span {
                                    class: "text-xs text-slate-500 font-mono",
                                    if let Some(v) = &endpoint.api_version {
                                        "{v}"
                                    } else {
                                        "—"
                                    }
                                }

                                // Actions
                                span {
                                    class: "flex items-center gap-2",

                                    button {
                                        class: "p-1 text-slate-400 hover:text-white transition-colors",
                                        title: "Edit endpoint",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            props.on_edit.call(ep_id);
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

                                    button {
                                        class: "p-1 text-slate-400 hover:text-red-400 transition-colors",
                                        title: "Delete endpoint",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            props.on_delete.call(ep_id);
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
// Integrated Endpoints Section
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct IntegratedEndpointsSectionProps {
    endpoints: Vec<IntegratedEndpoint>,
    on_toggle: EventHandler<String>,
    on_toggle_all: EventHandler<bool>,
}

#[component]
fn IntegratedEndpointsSection(props: IntegratedEndpointsSectionProps) -> Element {
    let enabled_count = props.endpoints.iter().filter(|e| e.enabled).count();
    let total_count = props.endpoints.len();
    let all_enabled = enabled_count == total_count;
    let none_enabled = enabled_count == 0;

    // Group endpoints by relationship (parent → child)
    let mut groups: Vec<(String, Vec<&IntegratedEndpoint>)> = Vec::new();
    for ep in &props.endpoints {
        let group_key = format!("{} → {}", ep.parent_entity, ep.child_entity);
        if let Some(group) = groups.iter_mut().find(|(k, _)| k == &group_key) {
            group.1.push(ep);
        } else {
            groups.push((group_key, vec![ep]));
        }
    }

    rsx! {
        div {
            class: "rounded-xl border border-slate-700 bg-slate-800/50 overflow-hidden",

            // Header
            div {
                class: "px-4 py-3 border-b border-slate-700 flex items-center justify-between",

                div {
                    class: "flex items-center gap-2",
                    svg {
                        class: "w-5 h-5 text-emerald-400",
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
                    h3 {
                        class: "font-semibold text-white",
                        "Relationship Endpoints"
                    }
                    span {
                        class: "px-2 py-0.5 bg-emerald-900/30 text-emerald-400 rounded text-xs",
                        "{enabled_count}/{total_count} enabled"
                    }
                }

                // Bulk toggle buttons
                div {
                    class: "flex items-center gap-2",

                    button {
                        class: format!(
                            "px-2.5 py-1 rounded text-xs font-medium transition-colors {}",
                            if all_enabled {
                                "bg-emerald-600 text-white"
                            } else {
                                "bg-slate-700 text-slate-300 hover:bg-slate-600"
                            }
                        ),
                        onclick: move |_| props.on_toggle_all.call(true),
                        "Enable All"
                    }

                    button {
                        class: format!(
                            "px-2.5 py-1 rounded text-xs font-medium transition-colors {}",
                            if none_enabled {
                                "bg-slate-600 text-white"
                            } else {
                                "bg-slate-700 text-slate-300 hover:bg-slate-600"
                            }
                        ),
                        onclick: move |_| props.on_toggle_all.call(false),
                        "Disable All"
                    }
                }
            }

            // Info notice
            div {
                class: "px-4 py-2 bg-emerald-900/10 border-b border-slate-700 text-xs text-slate-400",
                "Toggle the endpoints you need. These are generated from your entity relationships. "
                "Hover or click on any endpoint to see a detailed explanation."
            }

            // Grouped endpoint list
            div {
                class: "divide-y divide-slate-700",

                for (group_name, group_eps) in groups.iter() {
                    div {
                        // Group header
                        div {
                            class: "px-4 py-2 bg-slate-800 flex items-center gap-2",

                            span {
                                class: "text-xs font-semibold text-slate-400 uppercase tracking-wider",
                                "{group_name}"
                            }

                            span {
                                class: "text-xs text-slate-600",
                                {
                                    let on = group_eps.iter().filter(|e| e.enabled).count();
                                    format!("({}/{} on)", on, group_eps.len())
                                }
                            }
                        }

                        // Endpoints in group
                        div {
                            class: "divide-y divide-slate-700/30",

                            for endpoint in group_eps.iter() {
                                {
                                    let ep_id = endpoint.id.clone();
                                    let is_enabled = endpoint.enabled;
                                    rsx! {
                                        div {
                                            key: "{ep_id}",
                                            class: format!(
                                                "px-4 py-3 transition-colors {}",
                                                if is_enabled {
                                                    "hover:bg-slate-700/30"
                                                } else {
                                                    "opacity-50 hover:opacity-70"
                                                }
                                            ),

                                            // Top row: toggle + method + path + description
                                            div {
                                                class: "flex items-center gap-3",

                                                // Toggle checkbox
                                                button {
                                                    class: format!(
                                                        "w-5 h-5 rounded border flex items-center justify-center flex-shrink-0 transition-colors cursor-pointer {}",
                                                        if is_enabled {
                                                            "bg-emerald-600 border-emerald-600"
                                                        } else {
                                                            "border-slate-600 hover:border-slate-500"
                                                        }
                                                    ),
                                                    onclick: move |_| props.on_toggle.call(ep_id.clone()),

                                                    if is_enabled {
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

                                                // Method badge
                                                span {
                                                    class: format!(
                                                        "px-2 py-0.5 rounded text-xs font-bold min-w-[56px] text-center {}",
                                                        http_method_class(&endpoint.method)
                                                    ),
                                                    "{endpoint.method}"
                                                }

                                                // Path
                                                span {
                                                    class: format!(
                                                        "font-mono text-sm flex-1 {}",
                                                        if is_enabled { "text-slate-200" } else { "text-slate-500" }
                                                    ),
                                                    "{endpoint.path}"
                                                }

                                                // Short description
                                                span {
                                                    class: "text-xs text-slate-400 max-w-[180px] truncate hidden lg:inline",
                                                    "{endpoint.description}"
                                                }

                                                // Relationship badge
                                                span {
                                                    class: "px-2 py-0.5 bg-slate-700 text-slate-500 rounded text-xs hidden md:inline",
                                                    "{endpoint.parent_entity} → {endpoint.child_entity}"
                                                }
                                            }

                                            // Bottom row: detailed explanation (always visible)
                                            div {
                                                class: "mt-1.5 ml-8 text-xs text-slate-500 leading-relaxed",
                                                "{endpoint.explanation}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Summary footer
            div {
                class: "px-4 py-2 border-t border-slate-700 text-xs text-slate-500 flex justify-between",

                span {
                    "{enabled_count} of {total_count} relationship endpoints will be generated"
                }

                span {
                    class: "text-slate-600",
                    "Disabled endpoints are excluded from code generation"
                }
            }
        }
    }
}

// ============================================================================
// Auth Endpoints Section
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct AuthEndpointsSectionProps {
    endpoints: Vec<AuthEndpoint>,
}

#[component]
fn AuthEndpointsSection(props: AuthEndpointsSectionProps) -> Element {
    rsx! {
        div {
            class: "rounded-xl border border-amber-700/30 bg-amber-900/10 overflow-hidden",

            // Header
            div {
                class: "px-4 py-3 border-b border-amber-700/30 flex items-center gap-2",

                span { class: "text-lg", "🔐" }

                h3 {
                    class: "font-semibold text-amber-300",
                    "Authentication Endpoints"
                }

                span {
                    class: "px-2 py-0.5 bg-amber-900/30 text-amber-400 rounded text-xs",
                    "{props.endpoints.len()} auth routes"
                }
            }

            // Info
            div {
                class: "px-4 py-2 bg-amber-900/5 border-b border-amber-700/20 text-xs text-slate-400",
                "These authentication endpoints are auto-generated because auth is enabled in your project settings. "
                "They handle user registration, login, token management, and password operations."
            }

            // Endpoint list
            div {
                class: "divide-y divide-amber-700/10",

                for endpoint in props.endpoints.iter() {
                    div {
                        class: "px-4 py-3 flex items-center gap-3",

                        // Method badge
                        span {
                            class: format!(
                                "px-2 py-0.5 rounded text-xs font-bold min-w-[56px] text-center {}",
                                http_method_class(&endpoint.method)
                            ),
                            "{endpoint.method}"
                        }

                        // Path
                        span {
                            class: "font-mono text-sm text-amber-200 min-w-[220px]",
                            "{endpoint.path}"
                        }

                        // Name
                        span {
                            class: "text-sm text-white font-medium min-w-[120px]",
                            "{endpoint.name}"
                        }

                        // Description
                        span {
                            class: "text-xs text-slate-400 flex-1 truncate",
                            "{endpoint.description}"
                        }

                        // Auth indicator
                        span {
                            class: format!(
                                "px-2 py-0.5 rounded text-xs {}",
                                if endpoint.requires_auth {
                                    "bg-amber-900/30 text-amber-400"
                                } else {
                                    "bg-green-900/30 text-green-400"
                                }
                            ),
                            if endpoint.requires_auth { "🔒 Auth" } else { "🌐 Public" }
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
struct EndpointPropertiesPanelProps {
    selected_endpoint: Option<Uuid>,
    entities: Vec<Entity>,
    auth_enabled: bool,
}

#[component]
fn EndpointPropertiesPanel(props: EndpointPropertiesPanelProps) -> Element {
    let Some(ep_id) = props.selected_endpoint else {
        return rsx! {
            div {
                class: "p-4",
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
                                d: "M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z",
                            }
                        }
                    }
                    h3 {
                        class: "text-sm font-medium text-slate-300 mb-1",
                        "No Selection"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Select an endpoint group to view and edit its properties"
                    }
                }

                // Summary stats when nothing selected
                div {
                    class: "mt-6 space-y-3",

                    h4 {
                        class: "text-xs font-semibold text-slate-500 uppercase tracking-wider",
                        "Quick Tips"
                    }

                    div {
                        class: "space-y-2 text-xs text-slate-500",

                        div {
                            class: "flex items-start gap-2",
                            span { class: "text-indigo-400 mt-0.5", "•" }
                            span { "Click a card to select it" }
                        }
                        div {
                            class: "flex items-start gap-2",
                            span { class: "text-indigo-400 mt-0.5", "•" }
                            span { "Double-click to open the full editor" }
                        }
                        div {
                            class: "flex items-start gap-2",
                            span { class: "text-indigo-400 mt-0.5", "•" }
                            span { "Toggle operations directly on the cards" }
                        }
                        div {
                            class: "flex items-start gap-2",
                            span { class: "text-indigo-400 mt-0.5", "•" }
                            span { "Use Auto-Generate to add endpoints for all entities" }
                        }
                    }
                }
            }
        };
    };

    // Get endpoint from state
    let state = APP_STATE.read();
    let endpoint = state
        .project
        .as_ref()
        .and_then(|p| p.endpoints.get(&ep_id))
        .cloned();
    drop(state);

    let Some(ep) = endpoint else {
        return rsx! {
            div {
                class: "p-4 text-slate-400",
                "Endpoint not found"
            }
        };
    };

    let enabled_ops = ep.enabled_operations();
    let enabled_count = enabled_ops.len();
    let total_count = ep.operations.len();
    let has_auth = ep.requires_auth();
    let has_rate_limit = ep.operations.iter().any(|op| op.rate_limit.is_some());

    rsx! {
        div {
            class: "p-4 space-y-4",

            // Header
            div {
                class: "pb-3 border-b border-slate-700",
                div {
                    class: "flex items-center gap-2",
                    div {
                        class: format!(
                            "w-8 h-8 rounded-lg flex items-center justify-center {}",
                            if ep.enabled {
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
                    div {
                        h3 {
                            class: "font-semibold text-white",
                            "{ep.entity_name}"
                        }
                        p {
                            class: "text-xs text-slate-500 font-mono",
                            "{ep.full_base_path()}"
                        }
                    }
                }
            }

            // Status badges
            div {
                class: "flex flex-wrap gap-2",

                // Enabled/disabled
                span {
                    class: format!(
                        "px-2 py-1 rounded text-xs font-medium {}",
                        if ep.enabled {
                            "bg-green-900/30 text-green-400"
                        } else {
                            "bg-red-900/30 text-red-400"
                        }
                    ),
                    if ep.enabled { "✓ Enabled" } else { "✗ Disabled" }
                }

                // Auth
                span {
                    class: format!(
                        "px-2 py-1 rounded text-xs font-medium {}",
                        if has_auth {
                            "bg-amber-900/30 text-amber-400"
                        } else {
                            "bg-slate-700 text-slate-400"
                        }
                    ),
                    if has_auth { "🔒 Secured" } else { "🌐 Public" }
                }

                // Rate limit
                if has_rate_limit {
                    span {
                        class: "px-2 py-1 bg-cyan-900/30 text-cyan-400 rounded text-xs font-medium",
                        "⏱ Rate Limited"
                    }
                }

                // API version
                if let Some(version) = &ep.api_version {
                    span {
                        class: "px-2 py-1 bg-slate-700 text-slate-400 rounded text-xs font-mono",
                        "{version}"
                    }
                }
            }

            // Operations list
            div {
                class: "space-y-2",
                h4 {
                    class: "text-sm font-medium text-slate-300",
                    "Operations ({enabled_count}/{total_count})"
                }

                for op in ep.operations.iter() {
                    {
                        let method = op.http_method();
                        let path = op.full_path(&ep.base_path);
                        let op_security = op.security.as_ref().unwrap_or(&ep.global_security);
                        let op_has_rl = op.rate_limit.is_some();

                        rsx! {
                            div {
                                class: format!(
                                    "flex items-center gap-2 p-2 rounded {}",
                                    if op.enabled {
                                        "bg-slate-700/30"
                                    } else {
                                        "bg-slate-800/30 opacity-50"
                                    }
                                ),

                                // Method badge
                                span {
                                    class: format!(
                                        "px-1.5 py-0.5 rounded text-xs font-bold min-w-[44px] text-center {}",
                                        http_method_class(method)
                                    ),
                                    "{method}"
                                }

                                // Path
                                span {
                                    class: "flex-1 font-mono text-xs text-slate-400 truncate",
                                    "{path}"
                                }

                                // Indicators
                                if op_security.auth_required {
                                    span {
                                        class: "text-amber-400 text-xs",
                                        title: "Auth required",
                                        "🔒"
                                    }
                                }
                                if op_has_rl {
                                    span {
                                        class: "text-cyan-400 text-xs",
                                        title: "Rate limited",
                                        "⏱"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Security details
            div {
                class: "space-y-2",
                h4 {
                    class: "text-sm font-medium text-slate-300",
                    "Global Security"
                }

                div {
                    class: "p-3 bg-slate-700/30 rounded-lg text-sm space-y-1",

                    div {
                        class: "flex justify-between",
                        span { class: "text-slate-500", "Auth Required" }
                        span {
                            class: if ep.global_security.auth_required { "text-amber-400" } else { "text-green-400" },
                            if ep.global_security.auth_required { "Yes" } else { "No" }
                        }
                    }

                    if !ep.global_security.roles.is_empty() {
                        div {
                            class: "flex justify-between",
                            span { class: "text-slate-500", "Roles" }
                            span {
                                class: "text-purple-400 text-xs",
                                "{ep.global_security.roles.join(\", \")}"
                            }
                        }
                    }

                    div {
                        class: "flex justify-between",
                        span { class: "text-slate-500", "CORS" }
                        span {
                            class: "text-slate-400",
                            if ep.global_security.cors_enabled { "Enabled" } else { "Disabled" }
                        }
                    }
                }
            }

            // Tags
            if !ep.tags.is_empty() {
                div {
                    class: "space-y-2",
                    h4 {
                        class: "text-sm font-medium text-slate-300",
                        "Tags"
                    }
                    div {
                        class: "flex flex-wrap gap-1",

                        for tag in ep.tags.iter() {
                            span {
                                class: "px-2 py-0.5 bg-slate-700 text-slate-400 rounded text-xs",
                                "#{tag}"
                            }
                        }
                    }
                }
            }

            // Description
            if let Some(desc) = &ep.description {
                div {
                    class: "p-3 bg-slate-700/30 rounded-lg",
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

            // Middleware
            if !ep.middleware.is_empty() {
                div {
                    class: "space-y-2",
                    h4 {
                        class: "text-sm font-medium text-slate-300",
                        "Middleware"
                    }
                    div {
                        class: "space-y-1",
                        for mw in ep.middleware.iter() {
                            div {
                                class: "px-2 py-1 bg-slate-700/30 rounded text-xs font-mono text-slate-400",
                                "{mw}"
                            }
                        }
                    }
                }
            }

            // ID
            div {
                class: "pt-3 border-t border-slate-700 text-xs text-slate-600",
                "ID: {ep.id}"
            }

            // Edit button
            div {
                class: "pt-2",
                button {
                    class: "w-full px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors",
                    onclick: move |_| {
                        APP_STATE
                            .write()
                            .ui
                            .show_dialog(Dialog::EditEndpoint(ep_id));
                    },
                    "Edit Configuration"
                }
            }
        }
    }
}

// ============================================================================
// Empty State
// ============================================================================

#[component]
fn EmptyState() -> Element {
    rsx! {
        div {
            class: "flex-1 flex items-center justify-center py-16",
            div {
                class: "text-center max-w-md",

                div {
                    class: "w-16 h-16 mx-auto mb-4 rounded-full bg-indigo-900/20 flex items-center justify-center",
                    svg {
                        class: "w-8 h-8 text-indigo-400",
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

                h3 {
                    class: "text-xl font-semibold text-white mb-2",
                    "No Endpoints Configured"
                }

                p {
                    class: "text-slate-400 mb-2",
                    "Define entities first, then configure API endpoints for them."
                }

                p {
                    class: "text-sm text-slate-500 mb-6",
                    "Each entity can have CRUD endpoints (Create, Read, List, Update, Delete) with security and rate limiting."
                }

                div {
                    class: "flex justify-center gap-3",

                    button {
                        class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg text-sm transition-colors",
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::NewEndpoint(None));
                        },
                        "Create Endpoint"
                    }

                    button {
                        class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg text-sm transition-colors",
                        onclick: move |_| {
                            APP_STATE.write().ui.navigate(crate::state::Page::EntityDesign);
                        },
                        "Design Entities"
                    }
                }
            }
        }
    }
}

// ============================================================================
// No Project State
// ============================================================================

#[component]
fn NoProjectState() -> Element {
    rsx! {
        div {
            class: "flex-1 flex items-center justify-center p-8",
            div {
                class: "text-center max-w-md",

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
                    "Open or create a project to start configuring API endpoints."
                }

                div {
                    class: "flex justify-center gap-3",

                    button {
                        class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors",
                        onclick: move |_| {
                            APP_STATE
                                .write()
                                .ui
                                .show_dialog(Dialog::NewProject);
                        },
                        "New Project"
                    }

                    button {
                        class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors",
                        onclick: move |_| {
                            APP_STATE
                                .write()
                                .ui
                                .show_dialog(Dialog::OpenProject);
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
struct EndpointsStatusBarProps {
    total: usize,
    filtered: usize,
    selected: usize,
    entities_total: usize,
    uncovered: usize,
    auth_enabled: bool,
}

#[component]
fn EndpointsStatusBar(props: EndpointsStatusBarProps) -> Element {
    rsx! {
        div {
            class: "px-4 py-2 bg-slate-800 border-t border-slate-700 flex items-center justify-between text-sm",

            // Left side: counts
            div {
                class: "flex items-center gap-4 text-slate-400",

                span { "{props.total} endpoint group(s)" }

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

                span {
                    class: "text-slate-600",
                    "•"
                }

                span {
                    "{props.entities_total} entities"
                }

                if props.uncovered > 0 {
                    span {
                        class: "text-amber-400",
                        "({props.uncovered} without endpoints)"
                    }
                }
            }

            // Right side: hints
            div {
                class: "flex items-center gap-4 text-slate-500",

                span { "Double-click to edit" }
                span { "•" }
                span { "Toggle operations on cards" }

                if props.auth_enabled {
                    span {
                        class: "text-green-500",
                        "• Auth configured"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Simple snake_case conversion
fn to_snake_case(s: &str) -> String {
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

    result
}

/// Simple snake_case plural
fn to_snake_case_plural(s: &str) -> String {
    let snake = to_snake_case(s);

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
    fn test_filter_conversion() {
        assert_eq!(filter_to_string(&EndpointFilter::All), "all");
        assert_eq!(filter_to_string(&EndpointFilter::Enabled), "enabled");
        assert_eq!(filter_to_string(&EndpointFilter::Disabled), "disabled");
        assert_eq!(filter_to_string(&EndpointFilter::Secured), "secured");
        assert_eq!(filter_to_string(&EndpointFilter::Open), "open");

        assert_eq!(string_to_filter("all"), EndpointFilter::All);
        assert_eq!(string_to_filter("enabled"), EndpointFilter::Enabled);
        assert_eq!(string_to_filter("disabled"), EndpointFilter::Disabled);
        assert_eq!(string_to_filter("secured"), EndpointFilter::Secured);
        assert_eq!(string_to_filter("open"), EndpointFilter::Open);
        assert_eq!(string_to_filter("invalid"), EndpointFilter::All);
    }

    #[test]
    fn test_view_mode_equality() {
        assert_eq!(ViewMode::Grid, ViewMode::Grid);
        assert_eq!(ViewMode::List, ViewMode::List);
        assert_eq!(ViewMode::Compact, ViewMode::Compact);
        assert_ne!(ViewMode::Grid, ViewMode::List);
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("BlogPost"), "blog_post");
        assert_eq!(to_snake_case("HTTPRequest"), "h_t_t_p_request");
    }

    #[test]
    fn test_to_snake_case_plural() {
        assert_eq!(to_snake_case_plural("User"), "users");
        assert_eq!(to_snake_case_plural("Post"), "posts");
        assert_eq!(to_snake_case_plural("Category"), "categories");
        assert_eq!(to_snake_case_plural("Address"), "addresses");
    }

    #[test]
    fn test_build_integrated_endpoints_empty() {
        let endpoints = vec![];
        let relationships = vec![];
        let entities = vec![];
        let result = build_integrated_endpoints(&endpoints, &relationships, &entities);
        assert!(result.is_empty());
    }

    #[test]
    fn test_integrated_endpoints_have_ids() {
        let endpoints = build_auth_endpoints();
        // Auth endpoints should have paths
        for ep in &endpoints {
            assert!(!ep.path.is_empty());
        }
    }

    #[test]
    fn test_build_auth_endpoints() {
        let endpoints = build_auth_endpoints();
        assert!(endpoints.len() >= 6);

        // Check key endpoints exist
        assert!(endpoints.iter().any(|e| e.path == "/api/auth/register"));
        assert!(endpoints.iter().any(|e| e.path == "/api/auth/login"));
        assert!(endpoints.iter().any(|e| e.path == "/api/auth/me"));
        assert!(endpoints.iter().any(|e| e.path == "/api/auth/refresh"));
        assert!(endpoints.iter().any(|e| e.path.contains("password")));

        // Register and login should be public
        let register = endpoints
            .iter()
            .find(|e| e.path == "/api/auth/register")
            .unwrap();
        assert!(!register.requires_auth);

        let login = endpoints
            .iter()
            .find(|e| e.path == "/api/auth/login")
            .unwrap();
        assert!(!login.requires_auth);

        // Me endpoint should require auth
        let me = endpoints.iter().find(|e| e.path == "/api/auth/me").unwrap();
        assert!(me.requires_auth);
    }
}
