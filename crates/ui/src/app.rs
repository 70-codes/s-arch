//! Main Application Component for Immortal Engine
//!
//! This module contains the root Dioxus component that renders the entire application.
//! It provides the main layout structure including sidebar, toolbar, content area,
//! and properties panel.

use dioxus::prelude::*;

use crate::components::dialogs::{
    ConfirmDeleteDialog, EndpointDialog, EndpointDialogMode, EntityDialog, EntityDialogMode,
    FieldDialog, FieldDialogMode, RelationshipDialog, RelationshipDialogMode,
};
use crate::file_ops;
use crate::pages::welcome::add_to_recent_projects;
use crate::pages::{
    CodeGenerationPage, EndpointsPage, EntityDesignPage, ProjectSetupPage, RelationshipsPage,
    WelcomePage,
};
use crate::state::{APP_STATE, Dialog, Page, StatusLevel};

// ============================================================================
// Main App Component
// ============================================================================

/// Root application component
#[component]
pub fn App() -> Element {
    // Initialize app state on first render
    use_effect(|| {
        tracing::info!("Immortal Engine UI initialized");
    });

    rsx! {
        div {
            class: "app-container h-screen w-screen flex flex-col bg-slate-900 text-slate-100 overflow-hidden",

            // Top Toolbar
            Toolbar {}

            // Main content area with sidebar
            div {
                class: "flex flex-1 overflow-hidden",

                // Left Sidebar (navigation)
                Sidebar {}

                // Main Content Area
                MainContent {}

                // Right Properties Panel (conditional)
                PropertiesPanel {}
            }

            // Status Bar
            StatusBar {}

            // Dialog overlay (if active)
            DialogOverlay {}
        }
    }
}

// ============================================================================
// Toolbar Component
// ============================================================================

/// Top toolbar with actions and project info
#[component]
fn Toolbar() -> Element {
    let state = APP_STATE.read();
    let has_project = state.has_project();
    let is_dirty = state.is_dirty;
    let project_name = state.project_name().to_string();
    let can_undo = state.history.can_undo();
    let can_redo = state.history.can_redo();
    drop(state);

    rsx! {
        header {
            class: "toolbar h-12 bg-slate-800 border-b border-slate-700 flex items-center px-4 gap-2 shrink-0",

            // App Logo/Title
            div {
                class: "flex items-center gap-2 mr-4",
                span { class: "text-xl", "🔮" }
                span { class: "font-semibold text-sm hidden sm:inline", "Immortal Engine" }
            }

            // File Actions
            div {
                class: "flex items-center gap-1",

                ToolbarButton {
                    icon: "📄",
                    label: "New",
                    shortcut: "Ctrl+N",
                    onclick: move |_| {
                        APP_STATE.write().ui.show_dialog(Dialog::NewProject);
                    }
                }

                ToolbarButton {
                    icon: "📂",
                    label: "Open",
                    shortcut: "Ctrl+O",
                    onclick: move |_| {
                        APP_STATE.write().ui.show_dialog(Dialog::OpenProject);
                    }
                }

                ToolbarButton {
                    icon: "💾",
                    label: "Save",
                    shortcut: "Ctrl+S",
                    disabled: !has_project || !is_dirty,
                    onclick: move |_| {
                        spawn(async move {
                            let state = APP_STATE.read();
                            let project = match &state.project {
                                Some(p) => p.clone(),
                                None => return,
                            };
                            let existing_path = state.project_path.clone();
                            drop(state);

                            let result = file_ops::save_project_to_file(
                                &project,
                                existing_path,
                                None,
                            ).await;

                            match result {
                                Ok(saved_path) => {
                                    let mut state = APP_STATE.write();
                                    let project_name = state.project_name().to_string();
                                    state.mark_saved(Some(saved_path.clone()));
                                    state.ui.set_status(
                                        format!("Project saved to {}", saved_path.display()),
                                        StatusLevel::Success,
                                    );
                                    drop(state);

                                    // Track in recent projects
                                    add_to_recent_projects(&project_name, &saved_path);

                                    tracing::info!("Project saved to {}", saved_path.display());
                                }
                                Err(imortal_core::EngineError::Cancelled) => {
                                    // User cancelled the save dialog — do nothing
                                    tracing::debug!("Save cancelled by user");
                                }
                                Err(e) => {
                                    APP_STATE.write().ui.set_status(
                                        format!("Failed to save: {}", e),
                                        StatusLevel::Error,
                                    );
                                    tracing::error!("Failed to save project: {}", e);
                                }
                            }
                        });
                    }
                }
            }

            // Separator
            div { class: "w-px h-6 bg-slate-700 mx-2" }

            // Edit Actions
            div {
                class: "flex items-center gap-1",

                ToolbarButton {
                    icon: "↩️",
                    label: "Undo",
                    shortcut: "Ctrl+Z",
                    disabled: !can_undo,
                    onclick: move |_| {
                        APP_STATE.write().undo();
                    }
                }

                ToolbarButton {
                    icon: "↪️",
                    label: "Redo",
                    shortcut: "Ctrl+Y",
                    disabled: !can_redo,
                    onclick: move |_| {
                        APP_STATE.write().redo();
                    }
                }
            }

            // Spacer
            div { class: "flex-1" }

            // Project Name
            if has_project {
                div {
                    class: "flex items-center gap-2 text-sm",
                    span { class: "text-slate-400", "Project:" }
                    span { class: "font-medium", "{project_name}" }
                    if is_dirty {
                        span { class: "text-amber-400", "•" }
                    }
                }
            }

            // Spacer
            div { class: "flex-1" }

            // View Actions
            div {
                class: "flex items-center gap-1",

                ToolbarButton {
                    icon: "🌙",
                    label: "Theme",
                    onclick: move |_| {
                        APP_STATE.write().ui.toggle_dark_mode();
                    }
                }

                ToolbarButton {
                    icon: "⚙️",
                    label: "Settings",
                    onclick: move |_| {
                        APP_STATE.write().ui.navigate(Page::Settings);
                    }
                }
            }
        }
    }
}

/// Toolbar button component
#[component]
fn ToolbarButton(
    icon: &'static str,
    label: &'static str,
    #[props(default)] shortcut: &'static str,
    #[props(default = false)] disabled: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let base_class = "px-2 py-1 rounded text-sm flex items-center gap-1 transition-colors";
    let state_class = if disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        "hover:bg-slate-700 cursor-pointer"
    };

    rsx! {
        button {
            class: "{base_class} {state_class}",
            disabled: disabled,
            title: if shortcut.is_empty() { label.to_string() } else { format!("{} ({})", label, shortcut) },
            onclick: move |e| {
                if !disabled {
                    onclick.call(e);
                }
            },
            span { "{icon}" }
            span { class: "hidden lg:inline", "{label}" }
        }
    }
}

// ============================================================================
// Sidebar Component
// ============================================================================

/// Left sidebar with navigation
#[component]
fn Sidebar() -> Element {
    let state = APP_STATE.read();
    let collapsed = state.ui.sidebar_collapsed;
    let current_page = state.ui.active_page;
    let has_project = state.has_project();
    drop(state);

    rsx! {
        aside {
            class: "sidebar flex flex-col shrink-0 transition-all duration-200",
            style: if collapsed { "width: 60px;" } else { "width: 220px;" },

            // Header with toggle button
            div {
                class: "h-12 flex items-center justify-between px-3 border-b border-slate-700",

                if !collapsed {
                    span {
                        class: "text-sm font-semibold text-slate-300",
                        "Navigation"
                    }
                }

                button {
                    class: "w-8 h-8 flex items-center justify-center rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors",
                    title: if collapsed { "Expand sidebar" } else { "Collapse sidebar" },
                    onclick: move |_| {
                        APP_STATE.write().ui.toggle_sidebar();
                    },
                    if collapsed { "☰" } else { "✕" }
                }
            }

            // Navigation items
            nav {
                class: "flex-1 py-4 overflow-y-auto",

                SidebarItem {
                    page: Page::Welcome,
                    current: current_page,
                    collapsed: collapsed,
                }

                if has_project {
                    // Divider
                    div {
                        class: "my-3 mx-3 border-t border-slate-700"
                    }

                    SidebarItem {
                        page: Page::ProjectSetup,
                        current: current_page,
                        collapsed: collapsed,
                    }

                    SidebarItem {
                        page: Page::EntityDesign,
                        current: current_page,
                        collapsed: collapsed,
                    }

                    SidebarItem {
                        page: Page::Relationships,
                        current: current_page,
                        collapsed: collapsed,
                    }

                    SidebarItem {
                        page: Page::Endpoints,
                        current: current_page,
                        collapsed: collapsed,
                    }

                    SidebarItem {
                        page: Page::CodeGeneration,
                        current: current_page,
                        collapsed: collapsed,
                    }
                }
            }

            // Settings at bottom
            div {
                class: "border-t border-slate-700 py-3",
                SidebarItem {
                    page: Page::Settings,
                    current: current_page,
                    collapsed: collapsed,
                }
            }
        }
    }
}

/// Sidebar navigation item
#[component]
fn SidebarItem(page: Page, current: Page, collapsed: bool) -> Element {
    let is_active = page == current;
    let icon = page.icon();
    let name = page.display_name();

    let bg_class = if is_active {
        "background-color: rgb(79 70 229);"
    } else {
        "background-color: transparent;"
    };

    let text_color = if is_active {
        "color: white;"
    } else {
        "color: rgb(203 213 225);"
    };

    let hover_style = if is_active {
        ""
    } else {
        "onmouseenter: this.style.backgroundColor='rgb(51 65 85)'; onmouseleave: this.style.backgroundColor='transparent';"
    };

    if collapsed {
        rsx! {
            button {
                class: if is_active { "" } else { "hover:bg-slate-700" },
                style: "display: flex; align-items: center; justify-content: center; width: 44px; height: 44px; margin: 4px auto; border-radius: 8px; cursor: pointer; border: none; transition: background-color 0.15s; {bg_class} {text_color}",
                title: "{name}",
                onclick: move |_| {
                    APP_STATE.write().ui.navigate(page);
                },
                span {
                    style: "font-size: 22px; line-height: 1;",
                    "{icon}"
                }
            }
        }
    } else {
        rsx! {
            button {
                class: if is_active { "" } else { "hover:bg-slate-700" },
                style: "display: flex; align-items: center; gap: 12px; padding: 10px 16px; margin: 2px 8px; border-radius: 8px; cursor: pointer; border: none; width: calc(100% - 16px); text-align: left; transition: background-color 0.15s; {bg_class} {text_color}",
                title: "{name}",
                onclick: move |_| {
                    APP_STATE.write().ui.navigate(page);
                },
                span {
                    style: "font-size: 20px; line-height: 1; flex-shrink: 0;",
                    "{icon}"
                }
                span {
                    style: "font-size: 14px; font-weight: 500;",
                    "{name}"
                }
            }
        }
    }
}

// ============================================================================
// Main Content Component
// ============================================================================

/// Main content area that renders the active page
#[component]
fn MainContent() -> Element {
    let state = APP_STATE.read();
    let current_page = state.ui.active_page;
    let has_project = state.has_project();
    drop(state);

    // Check if page requires project but none is loaded
    if current_page.requires_project() && !has_project {
        return rsx! {
            main {
                class: "flex-1 flex items-center justify-center bg-slate-900",
                div {
                    class: "text-center",
                    p { class: "text-2xl mb-4", "📋" }
                    p { class: "text-slate-400 mb-4", "No project loaded" }
                    button {
                        class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded transition-colors",
                        onclick: move |_| {
                            APP_STATE.write().ui.show_dialog(Dialog::NewProject);
                        },
                        "Create New Project"
                    }
                }
            }
        };
    }

    rsx! {
        main {
            class: "flex-1 overflow-auto bg-slate-900",

            match current_page {
                Page::Welcome => rsx! { WelcomePage {} },
                Page::ProjectSetup => rsx! { ProjectSetupPage {} },
                Page::EntityDesign => rsx! { EntityDesignPage {} },
                Page::Relationships => rsx! { RelationshipsPage {} },
                Page::Endpoints => rsx! { EndpointsPage {} },
                Page::CodeGeneration => rsx! { CodeGenerationPage {} },
                Page::Settings => rsx! { SettingsPage {} },
            }
        }
    }
}

// ============================================================================
// Page Components (Placeholders for pages not yet in pages module)
// ============================================================================

/// Settings page (placeholder)
#[component]
fn SettingsPage() -> Element {
    let state = APP_STATE.read();
    let dark_mode = state.ui.dark_mode;
    drop(state);

    rsx! {
        div {
            class: "p-8 max-w-2xl",
            h2 { class: "text-2xl font-bold mb-6", "Settings" }

            div {
                class: "space-y-4",

                // Dark mode toggle
                div {
                    class: "flex items-center justify-between p-4 bg-slate-800 rounded-lg",
                    div {
                        h3 { class: "font-medium", "Dark Mode" }
                        p { class: "text-sm text-slate-400", "Use dark theme for the application" }
                    }
                    button {
                        class: "px-4 py-2 rounded transition-colors",
                        class: if dark_mode { "bg-indigo-600" } else { "bg-slate-600" },
                        onclick: move |_| {
                            APP_STATE.write().ui.toggle_dark_mode();
                        },
                        if dark_mode { "On" } else { "Off" }
                    }
                }

                // About section
                div {
                    class: "p-4 bg-slate-800 rounded-lg",
                    h3 { class: "font-medium mb-2", "About" }
                    p { class: "text-sm text-slate-400", "Immortal Engine v0.1.0" }
                    p { class: "text-sm text-slate-400", "Visual Code Generator for Rust Applications" }
                }
            }
        }
    }
}

// ============================================================================
// Properties Panel Component
// ============================================================================

/// Right properties panel for editing selected items
#[component]
fn PropertiesPanel() -> Element {
    let state = APP_STATE.read();
    let collapsed = state.ui.properties_collapsed;
    let has_selection = !state.selection.is_empty();
    let current_page = state.ui.active_page;
    drop(state);

    // Only show on relevant pages
    let show_panel = matches!(
        current_page,
        Page::EntityDesign | Page::Relationships | Page::Endpoints
    );

    if !show_panel {
        return rsx! {};
    }

    let width_class = if collapsed { "w-0" } else { "w-72" };

    rsx! {
        aside {
            class: "properties-panel {width_class} bg-slate-800 border-l border-slate-700 flex flex-col shrink-0 transition-all duration-200 overflow-hidden",

            // Header
            div {
                class: "h-10 border-b border-slate-700 flex items-center justify-between px-3",
                span { class: "text-sm font-medium", "Properties" }
                button {
                    class: "p-1 hover:bg-slate-700 rounded",
                    onclick: move |_| {
                        APP_STATE.write().ui.toggle_properties();
                    },
                    "✕"
                }
            }

            // Content
            div {
                class: "flex-1 overflow-auto p-3",

                if has_selection {
                    p { class: "text-sm text-slate-400", "Edit properties here" }

                } else {
                    div {
                        class: "text-center text-slate-500 mt-8",
                        p { "No selection" }
                        p { class: "text-xs mt-1", "Select an item to edit" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Status Bar Component
// ============================================================================

/// Bottom status bar
#[component]
fn StatusBar() -> Element {
    let state = APP_STATE.read();
    let status = state.ui.status_message.clone();
    let entity_count = state
        .project
        .as_ref()
        .map(|p| p.entities.len())
        .unwrap_or(0);
    let relationship_count = state
        .project
        .as_ref()
        .map(|p| p.relationships.len())
        .unwrap_or(0);
    let has_project = state.has_project();
    drop(state);

    rsx! {
        footer {
            class: "status-bar h-6 bg-slate-800 border-t border-slate-700 flex items-center px-4 text-xs text-slate-400 shrink-0",

            // Status message
            if let Some(msg) = status {
                span {
                    class: match msg.level {
                        StatusLevel::Info => "text-slate-400",
                        StatusLevel::Success => "text-green-400",
                        StatusLevel::Warning => "text-amber-400",
                        StatusLevel::Error => "text-red-400",
                    },
                    "{msg.text}"
                }
            } else {
                span { "Ready" }
            }

            // Spacer
            div { class: "flex-1" }

            // Project stats
            if has_project {
                div {
                    class: "flex items-center gap-4",
                    span { "Entities: {entity_count}" }
                    span { "Relationships: {relationship_count}" }
                }
            }
        }
    }
}

// ============================================================================
// Dialog Overlay Component
// ============================================================================

/// Modal dialog overlay
#[component]
fn DialogOverlay() -> Element {
    let state = APP_STATE.read();
    let dialog = state.ui.active_dialog.clone();
    drop(state);

    let Some(dialog) = dialog else {
        return rsx! {};
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center",

            // Backdrop
            div {
                class: "absolute inset-0 bg-black/50",
                onclick: move |_| {
                    APP_STATE.write().ui.close_dialog();
                }
            }

            // Dialog content
            div {
                class: format!(
                    "relative bg-slate-800 rounded-lg shadow-xl border border-slate-700 mx-4 {}",
                    match &dialog {
                        Dialog::NewEntity | Dialog::EditEntity(_) => "max-w-lg w-full",
                        Dialog::NewField(_) | Dialog::EditField(_, _) => "max-w-2xl w-full",
                        Dialog::NewRelationship(_, _) | Dialog::EditRelationship(_) => "max-w-2xl w-full",
                        Dialog::NewEndpoint(_) | Dialog::EditEndpoint(_) => "max-w-2xl w-full",
                        _ => "max-w-lg w-full",
                    }
                ),
                onclick: move |e| e.stop_propagation(),

                match dialog {
                    Dialog::NewProject => rsx! { NewProjectDialog {} },
                    Dialog::OpenProject => rsx! { OpenProjectDialog {} },
                    Dialog::About => rsx! { AboutDialog {} },
                    Dialog::Error(ref msg) => rsx! { ErrorDialog { message: msg.clone() } },
                    Dialog::NewEntity => rsx! {
                        EntityDialog {
                            mode: EntityDialogMode::Create,
                        }
                    },
                    Dialog::EditEntity(entity_id) => rsx! {
                        EntityDialog {
                            mode: EntityDialogMode::Edit(entity_id),
                        }
                    },
                    Dialog::NewField(entity_id) => rsx! {
                        FieldDialog {
                            entity_id: entity_id,
                            mode: FieldDialogMode::Create,
                        }
                    },
                    Dialog::EditField(entity_id, field_id) => rsx! {
                        FieldDialog {
                            entity_id: entity_id,
                            mode: FieldDialogMode::Edit(field_id),
                        }
                    },
                    Dialog::ConfirmDelete(ref target) => rsx! {
                        ConfirmDeleteDialog {
                            target: target.clone(),
                        }
                    },
                    Dialog::NewRelationship(from_entity, to_entity) => rsx! {
                        RelationshipDialog {
                            mode: RelationshipDialogMode::Create {
                                from_entity_id: from_entity,
                                to_entity_id: to_entity,
                            },
                        }
                    },
                    Dialog::EditRelationship(relationship_id) => rsx! {
                        RelationshipDialog {
                            mode: RelationshipDialogMode::Edit(relationship_id),
                        }
                    },
                    Dialog::NewEndpoint(entity_id) => rsx! {
                        EndpointDialog {
                            mode: EndpointDialogMode::Create {
                                entity_id: entity_id,
                            },
                        }
                    },
                    Dialog::EditEndpoint(endpoint_id) => rsx! {
                        EndpointDialog {
                            mode: EndpointDialogMode::Edit(endpoint_id),
                        }
                    },
                    _ => rsx! {
                        div {
                            class: "p-6",
                            p { "Dialog not implemented" }
                            button {
                                class: "mt-4 px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded",
                                onclick: move |_| {
                                    APP_STATE.write().ui.close_dialog();
                                },
                                "Close"
                            }
                        }
                    },
                }
            }
        }
    }
}

/// New project dialog
#[component]
fn NewProjectDialog() -> Element {
    let mut project_name = use_signal(|| String::from("My Project"));

    // Shared submit logic — used by both the button click and Enter key
    let do_create = move |_| {
        let name = project_name.read().clone();
        if name.trim().is_empty() {
            return;
        }
        let mut state = APP_STATE.write();
        state.new_project(name);
        state.ui.close_dialog();
    };

    rsx! {
        form {
            // Pressing Enter inside the input submits the form
            onsubmit: move |e| {
                e.prevent_default();
                do_create(());
            },

            div {
                class: "p-6",

                h2 { class: "text-xl font-bold mb-4", "New Project" }

                div {
                    class: "mb-4",
                    label {
                        class: "block text-sm font-medium mb-2",
                        "Project Name"
                    }
                    input {
                        class: "w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded focus:outline-none focus:border-indigo-500",
                        r#type: "text",
                        value: "{project_name}",
                        autofocus: true,
                        oninput: move |e| project_name.set(e.value()),
                    }
                }

                div {
                    class: "flex justify-end gap-2",
                    button {
                        class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded transition-colors",
                        r#type: "button",
                        onclick: move |_| {
                            APP_STATE.write().ui.close_dialog();
                        },
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded transition-colors",
                        r#type: "submit",
                        "Create"
                    }
                }
            }
        }
    }
}

/// Open project dialog
#[component]
fn OpenProjectDialog() -> Element {
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    // Handler for opening file dialog
    let open_file = move |_| {
        is_loading.set(true);
        error_message.set(None);

        spawn(async move {
            match file_ops::open_project(None).await {
                Ok((project, path)) => {
                    let project_name = project.meta.name.clone();
                    let saved_path = path.clone();

                    let mut state = APP_STATE.write();
                    state.load_project(project, path);
                    state.ui.close_dialog();
                    state
                        .ui
                        .set_status("Project opened successfully", StatusLevel::Success);
                    drop(state);

                    // Track in recent projects
                    add_to_recent_projects(&project_name, &saved_path);
                }
                Err(e) => {
                    if !matches!(e, imortal_core::EngineError::Cancelled) {
                        error_message.set(Some(e.to_string()));
                    }
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        div {
            class: "p-6",

            h2 { class: "text-xl font-bold mb-4", "Open Project" }

            p { class: "text-slate-400 mb-4",
                "Select an Immortal Engine project file (.ieng) to open."
            }

            // Error message
            if let Some(err) = error_message.read().as_ref() {
                div {
                    class: "mb-4 p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-300 text-sm",
                    "Error: {err}"
                }
            }

            div {
                class: "flex justify-end gap-2",

                button {
                    class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded transition-colors",
                    disabled: *is_loading.read(),
                    onclick: move |_| {
                        APP_STATE.write().ui.close_dialog();
                    },
                    "Cancel"
                }

                button {
                    class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded transition-colors flex items-center gap-2",
                    disabled: *is_loading.read(),
                    onclick: open_file,

                    if *is_loading.read() {
                        span { class: "animate-spin", "⏳" }
                        "Opening..."
                    } else {
                        span { "📂" }
                        "Browse Files"
                    }
                }
            }
        }
    }
}

/// About dialog
#[component]
fn AboutDialog() -> Element {
    rsx! {
        div {
            class: "p-6 text-center",

            p { class: "text-4xl mb-4", "🔮" }
            h2 { class: "text-xl font-bold mb-2", "Immortal Engine" }
            p { class: "text-slate-400 mb-4", "Version 0.1.0" }
            p { class: "text-sm text-slate-500 mb-4", "Visual Code Generator for Rust Applications" }

            button {
                class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded transition-colors",
                onclick: move |_| {
                    APP_STATE.write().ui.close_dialog();
                },
                "Close"
            }
        }
    }
}

/// Error dialog
#[component]
fn ErrorDialog(message: String) -> Element {
    rsx! {
        div {
            class: "p-6",

            div {
                class: "flex items-start gap-3 mb-4",
                span { class: "text-2xl", "❌" }
                div {
                    h2 { class: "text-xl font-bold text-red-400", "Error" }
                    p { class: "text-slate-300 mt-1", "{message}" }
                }
            }

            div {
                class: "flex justify-end",
                button {
                    class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded transition-colors",
                    onclick: move |_| {
                        APP_STATE.write().ui.close_dialog();
                    },
                    "Close"
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
    fn test_page_icon() {
        assert_eq!(Page::Welcome.icon(), "🏠");
        assert_eq!(Page::EntityDesign.icon(), "🗃️");
    }

    #[test]
    fn test_page_display_name() {
        assert_eq!(Page::Welcome.display_name(), "Welcome");
        assert_eq!(Page::CodeGeneration.display_name(), "Code Generation");
    }
}
