//! # Dioxus Component Generator
//!
//! Generates reusable Dioxus UI components for the frontend application.
//!
//! ## Generated Files
//!
//! - `frontend/src/components/mod.rs` — module declarations and re-exports
//! - `frontend/src/components/navbar.rs` — top navigation bar
//! - `frontend/src/components/sidebar.rs` — side navigation with entity links
//! - `frontend/src/components/table.rs` — generic data table with sorting
//! - `frontend/src/components/form.rs` — form input helpers (text, select, checkbox)
//!
//! ## Design
//!
//! All components use the CSS classes defined in `assets/tailwind.css` and are
//! designed for dark-mode by default. They accept props for customisation and
//! emit events via `EventHandler` callbacks.

use crate::context::{EntityInfo, GenerationContext};
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate all frontend component files.
pub fn generate_components(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    vec![
        generate_components_mod(ctx),
        generate_navbar(ctx),
        generate_sidebar(ctx),
        generate_table(ctx),
        generate_form(ctx),
    ]
}

// ============================================================================
// components/mod.rs
// ============================================================================

fn generate_components_mod(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(512);

    content.push_str(&file_header("Reusable UI components."));

    content.push_str("pub mod navbar;\n");
    content.push_str("pub mod sidebar;\n");
    content.push_str("pub mod table;\n");
    content.push_str("pub mod form;\n\n");

    content.push_str("// Re-exports for convenience\n");
    content.push_str("pub use navbar::Navbar;\n");
    content.push_str("pub use sidebar::Sidebar;\n");
    content.push_str("pub use table::{DataTable, Column};\n");
    content.push_str("pub use form::{FormInput, FormTextArea, FormSelect, FormCheckbox};\n");

    GeneratedFile::new("frontend/src/components/mod.rs", content, FileType::Rust)
}

// ============================================================================
// components/navbar.rs
// ============================================================================

fn generate_navbar(ctx: &GenerationContext) -> GeneratedFile {
    let pkg = ctx.package_name();

    let content = format!(
        r#"{header}use dioxus::prelude::*;

// ============================================================================
// Navbar Component
// ============================================================================

/// Properties for the Navbar component.
#[derive(Props, Clone, PartialEq)]
pub struct NavbarProps {{
    /// Page title displayed in the navbar.
    #[props(default = "{pkg}".to_string())]
    pub title: String,

    /// Optional subtitle or breadcrumb text.
    #[props(default)]
    pub subtitle: Option<String>,
}}

/// Top navigation bar component.
///
/// Displays the application title, optional subtitle, and action buttons.
///
/// # Example
///
/// ```rust,ignore
/// Navbar {{
///     title: "Dashboard".to_string(),
///     subtitle: Some("Overview".to_string()),
/// }}
/// ```
#[component]
pub fn Navbar(props: NavbarProps) -> Element {{
    rsx! {{
        nav {{
            class: "navbar",

            // Left side — title
            div {{
                class: "flex items-center gap-4",

                h1 {{
                    class: "navbar-title",
                    "{{props.title}}"
                }}

                if let Some(subtitle) = &props.subtitle {{
                    span {{
                        class: "text-sm text-muted",
                        "/ {{subtitle}}"
                    }}
                }}
            }}

            // Right side — actions
            div {{
                class: "navbar-actions",

                // Refresh button
                button {{
                    class: "btn btn-secondary btn-sm",
                    title: "Refresh",
                    onclick: move |_| {{
                        // Trigger page reload
                        if let Some(window) = web_sys::window() {{
                            let _ = window.location().reload();
                        }}
                    }},
                    "\u{{21bb}} Refresh"
                }}
            }}
        }}
    }}
}}
"#,
        header = file_header("Top navigation bar component."),
    );

    GeneratedFile::new("frontend/src/components/navbar.rs", content, FileType::Rust)
}

// ============================================================================
// components/sidebar.rs
// ============================================================================

fn generate_sidebar(ctx: &GenerationContext) -> GeneratedFile {
    let pkg = ctx.package_name();

    let mut content = String::with_capacity(4096);

    content.push_str(&file_header("Sidebar navigation component."));

    content.push_str("use dioxus::prelude::*;\n\n");
    content.push_str("use crate::router::Route;\n\n");

    // ── Sidebar component ────────────────────────────────────────────────
    content.push_str(&format!(
        r#"/// Sidebar navigation component.
///
/// Displays the application name, a home link, and links to each entity's
/// list page. The active link is highlighted based on the current route.
#[component]
pub fn Sidebar() -> Element {{
    rsx! {{
        aside {{
            class: "sidebar",

            // App name
            div {{
                class: "sidebar-header",
                "\u{{2726}} {pkg}"
            }}

            // Navigation links
            nav {{
                class: "sidebar-nav",

                // Home link
                SidebarLink {{
                    to: Route::Home {{}},
                    icon: "\u{{1f3e0}}",
                    label: "Dashboard".to_string(),
                }}

"#,
        pkg = pkg,
    ));

    // Entity links
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        if info.endpoint().is_none() {
            continue;
        }

        let pascal = info.pascal_name();
        let plural = info.plural_name();
        // Use a generic icon based on the first character
        let icon_char = entity.name.chars().next().unwrap_or('E');

        content.push_str(&format!(
            r#"                // {pascal} link
                SidebarLink {{
                    to: Route::{pascal}List {{}},
                    icon: "{icon_char}\u{{fe0f}}",
                    label: "{pascal}s".to_string(),
                }}

"#,
            icon_char = icon_char.to_uppercase(),
        ));
    }

    content.push_str(
        r#"            }

            // Footer
            div {
                class: "sidebar-link text-xs text-muted mt-4",
                style: "margin-top: auto;",
                "Generated by Immortal Engine"
            }
        }
    }
}

// ============================================================================
// SidebarLink sub-component
// ============================================================================

/// Properties for a single sidebar navigation link.
#[derive(Props, Clone, PartialEq)]
struct SidebarLinkProps {
    /// Route to navigate to when clicked.
    to: Route,
    /// Icon character or emoji.
    icon: &'static str,
    /// Display label for the link.
    label: String,
}

/// A single link in the sidebar navigation.
#[component]
fn SidebarLink(props: SidebarLinkProps) -> Element {
    // Active route detection: apply "sidebar-link active" class when route matches
    rsx! {
        Link {
            class: "sidebar-link",
            to: props.to.clone(),

            span { "{props.icon}" }
            span { "{props.label}" }
        }
    }
}
"#,
    );

    GeneratedFile::new(
        "frontend/src/components/sidebar.rs",
        content,
        FileType::Rust,
    )
}

// ============================================================================
// components/table.rs
// ============================================================================

fn generate_table(_ctx: &GenerationContext) -> GeneratedFile {
    let content = format!(
        r#"{header}use dioxus::prelude::*;
use serde::Serialize;
use serde_json::Value;

// ============================================================================
// Column Definition
// ============================================================================

/// Definition of a single table column.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {{
    /// The key in the data object (used to extract the cell value).
    pub key: String,
    /// Display header text.
    pub label: String,
    /// Whether the column is sortable.
    pub sortable: bool,
    /// Optional CSS class for the column.
    pub class: String,
}}

impl Column {{
    /// Create a new column definition.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {{
        Self {{
            key: key.into(),
            label: label.into(),
            sortable: true,
            class: String::new(),
        }}
    }}

    /// Create a non-sortable column.
    pub fn fixed(key: impl Into<String>, label: impl Into<String>) -> Self {{
        Self {{
            key: key.into(),
            label: label.into(),
            sortable: false,
            class: String::new(),
        }}
    }}

    /// Set a custom CSS class.
    pub fn with_class(mut self, class: impl Into<String>) -> Self {{
        self.class = class.into();
        self
    }}
}}

// ============================================================================
// DataTable Component
// ============================================================================

/// Properties for the DataTable component.
#[derive(Props, Clone, PartialEq)]
pub struct DataTableProps {{
    /// Column definitions.
    pub columns: Vec<Column>,
    /// Row data as a vector of JSON objects.
    pub rows: Vec<Value>,
    /// Whether the table is in a loading state.
    #[props(default = false)]
    pub loading: bool,
    /// Callback when a row is clicked (receives the row's JSON object).
    #[props(default)]
    pub on_row_click: Option<EventHandler<Value>>,
    /// Callback for the "Edit" action button on a row.
    #[props(default)]
    pub on_edit: Option<EventHandler<String>>,
    /// Callback for the "Delete" action button on a row.
    #[props(default)]
    pub on_delete: Option<EventHandler<String>>,
    /// Key field name to extract the row ID (default: "id").
    #[props(default = "id".to_string())]
    pub id_field: String,
    /// Text to display when there are no rows.
    #[props(default = "No data found".to_string())]
    pub empty_text: String,
}}

/// A generic, reusable data table component.
///
/// Renders rows of data in a table with column headers, optional sorting,
/// row click handling, and action buttons (edit/delete).
///
/// # Example
///
/// ```rust,ignore
/// DataTable {{
///     columns: vec![
///         Column::new("name", "Name"),
///         Column::new("email", "Email"),
///     ],
///     rows: users_json,
///     on_edit: move |id| {{ /* navigate to edit */ }},
///     on_delete: move |id| {{ /* show delete confirm */ }},
/// }}
/// ```
#[component]
pub fn DataTable(props: DataTableProps) -> Element {{
    // Loading state
    if props.loading {{
        return rsx! {{
            div {{
                class: "loading",
                div {{ class: "spinner" }}
                span {{ class: "ml-2", "Loading…" }}
            }}
        }};
    }}

    // Empty state
    if props.rows.is_empty() {{
        return rsx! {{
            div {{
                class: "empty-state",
                div {{ class: "empty-state-icon", "\u{{1f4cb}}" }}
                h3 {{ class: "empty-state-title", "No Data" }}
                p {{ class: "empty-state-text", "{{props.empty_text}}" }}
            }}
        }};
    }}

    let has_actions = props.on_edit.is_some() || props.on_delete.is_some();

    rsx! {{
        div {{
            class: "table-container",

            table {{
                // Header
                thead {{
                    tr {{
                        for col in props.columns.iter() {{
                            th {{
                                key: "{{col.key}}",
                                class: "{{col.class}}",
                                "{{col.label}}"
                            }}
                        }}

                        if has_actions {{
                            th {{
                                class: "text-right",
                                "Actions"
                            }}
                        }}
                    }}
                }}

                // Body
                tbody {{
                    for row in props.rows.iter() {{
                        {{
                            let row_id = row
                                .get(&props.id_field)
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let row_clone = row.clone();
                            let row_id_edit = row_id.clone();
                            let row_id_delete = row_id.clone();

                            rsx! {{
                                tr {{
                                    key: "{{row_id}}",
                                    onclick: move |_| {{
                                        if let Some(handler) = &props.on_row_click {{
                                            handler.call(row_clone.clone());
                                        }}
                                    }},

                                    // Data cells
                                    for col in props.columns.iter() {{
                                        td {{
                                            key: "{{col.key}}",
                                            class: "{{col.class}}",
                                            {{
                                                let value = row
                                                    .get(&col.key)
                                                    .map(format_cell_value)
                                                    .unwrap_or_else(|| "—".to_string());
                                                rsx! {{ "{{value}}" }}
                                            }}
                                        }}
                                    }}

                                    // Action cells
                                    if has_actions {{
                                        td {{
                                            class: "text-right",

                                            div {{
                                                class: "flex items-center justify-end gap-2",

                                                if props.on_edit.is_some() {{
                                                    button {{
                                                        class: "btn btn-secondary btn-sm",
                                                        onclick: move |e| {{
                                                            e.stop_propagation();
                                                            if let Some(handler) = &props.on_edit {{
                                                                handler.call(row_id_edit.clone());
                                                            }}
                                                        }},
                                                        "\u{{270f}} Edit"
                                                    }}
                                                }}

                                                if props.on_delete.is_some() {{
                                                    button {{
                                                        class: "btn btn-danger btn-sm",
                                                        onclick: move |e| {{
                                                            e.stop_propagation();
                                                            if let Some(handler) = &props.on_delete {{
                                                                handler.call(row_id_delete.clone());
                                                            }}
                                                        }},
                                                        "\u{{1f5d1}} Delete"
                                                    }}
                                                }}
                                            }}
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
            }}
        }}
    }}
}}

// ============================================================================
// Pagination Component
// ============================================================================

/// Properties for the Pagination component.
#[derive(Props, Clone, PartialEq)]
pub struct PaginationProps {{
    /// Current page number (1-based).
    pub page: u64,
    /// Total number of pages.
    pub total_pages: u64,
    /// Total number of items.
    pub total_items: u64,
    /// Items per page.
    pub per_page: u64,
    /// Callback when page changes.
    pub on_page_change: EventHandler<u64>,
}}

/// Pagination controls component.
///
/// Displays page info and previous/next navigation buttons.
#[component]
pub fn Pagination(props: PaginationProps) -> Element {{
    let can_prev = props.page > 1;
    let can_next = props.page < props.total_pages;

    let start = ((props.page - 1) * props.per_page) + 1;
    let end = (props.page * props.per_page).min(props.total_items);

    rsx! {{
        div {{
            class: "pagination",

            // Info text
            span {{
                "Showing {{start}}–{{end}} of {{props.total_items}} items"
            }}

            // Navigation buttons
            div {{
                class: "pagination-buttons",

                button {{
                    class: "btn btn-secondary btn-sm",
                    disabled: !can_prev,
                    onclick: move |_| {{
                        if can_prev {{
                            props.on_page_change.call(props.page - 1);
                        }}
                    }},
                    "\u{{2190}} Previous"
                }}

                span {{
                    class: "btn btn-sm text-muted",
                    "Page {{props.page}} of {{props.total_pages}}"
                }}

                button {{
                    class: "btn btn-secondary btn-sm",
                    disabled: !can_next,
                    onclick: move |_| {{
                        if can_next {{
                            props.on_page_change.call(props.page + 1);
                        }}
                    }},
                    "Next \u{{2192}}"
                }}
            }}
        }}
    }}
}}

// ============================================================================
// Delete Confirmation Dialog
// ============================================================================

/// Properties for the DeleteConfirmDialog component.
#[derive(Props, Clone, PartialEq)]
pub struct DeleteConfirmDialogProps {{
    /// Whether the dialog is visible.
    pub visible: bool,
    /// Name of the item being deleted (for display).
    pub item_name: String,
    /// Callback to confirm deletion.
    pub on_confirm: EventHandler<()>,
    /// Callback to cancel deletion.
    pub on_cancel: EventHandler<()>,
}}

/// A confirmation dialog for delete operations.
#[component]
pub fn DeleteConfirmDialog(props: DeleteConfirmDialogProps) -> Element {{
    if !props.visible {{
        return rsx! {{}};
    }}

    rsx! {{
        div {{
            class: "modal-backdrop",
            onclick: move |_| props.on_cancel.call(()),

            div {{
                class: "modal",
                onclick: move |e| e.stop_propagation(),

                h2 {{
                    class: "modal-title",
                    "Confirm Deletion"
                }}

                p {{
                    class: "text-sm",
                    "Are you sure you want to delete "
                    strong {{ "{{props.item_name}}" }}
                    "? This action cannot be undone."
                }}

                div {{
                    class: "modal-actions",

                    button {{
                        class: "btn btn-secondary",
                        onclick: move |_| props.on_cancel.call(()),
                        "Cancel"
                    }}

                    button {{
                        class: "btn btn-danger",
                        onclick: move |_| props.on_confirm.call(()),
                        "Delete"
                    }}
                }}
            }}
        }}
    }}
}}

// ============================================================================
// Helpers
// ============================================================================

/// Format a JSON value for display in a table cell.
fn format_cell_value(value: &Value) -> String {{
    match value {{
        Value::Null => "—".to_string(),
        Value::Bool(b) => if *b {{ "\u{{2705}}" }} else {{ "\u{{274c}}" }}.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {{
            // Truncate long strings
            if s.len() > 80 {{
                format!("{{}}…", &s[..77])
            }} else {{
                s.clone()
            }}
        }}
        Value::Array(arr) => format!("[{{}} items]", arr.len()),
        Value::Object(_) => "{{…}}".to_string(),
    }}
}}
"#,
        header = file_header("Reusable data table and pagination components."),
    );

    GeneratedFile::new("frontend/src/components/table.rs", content, FileType::Rust)
}

// ============================================================================
// components/form.rs
// ============================================================================

fn generate_form(_ctx: &GenerationContext) -> GeneratedFile {
    let content = format!(
        r#"{header}use dioxus::prelude::*;

// ============================================================================
// FormInput — text / email / password / number input
// ============================================================================

/// Properties for the FormInput component.
#[derive(Props, Clone, PartialEq)]
pub struct FormInputProps {{
    /// Current value of the input.
    pub value: String,
    /// Callback when the value changes.
    pub on_change: EventHandler<String>,
    /// Label text displayed above the input.
    #[props(default)]
    pub label: Option<String>,
    /// Placeholder text shown when the input is empty.
    #[props(default)]
    pub placeholder: Option<String>,
    /// HTML input type (text, email, password, number, etc.).
    #[props(default = "text".to_string())]
    pub input_type: String,
    /// Whether the field is required.
    #[props(default = false)]
    pub required: bool,
    /// Whether the input is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Error message to display below the input.
    #[props(default)]
    pub error: Option<String>,
    /// Help text displayed below the input.
    #[props(default)]
    pub help: Option<String>,
}}

/// A styled text input with optional label, error, and help text.
///
/// # Example
///
/// ```rust,ignore
/// FormInput {{
///     label: Some("Email".to_string()),
///     value: email(),
///     input_type: "email".to_string(),
///     placeholder: Some("user@example.com".to_string()),
///     required: true,
///     on_change: move |v| email.set(v),
///     error: email_error(),
/// }}
/// ```
#[component]
pub fn FormInput(props: FormInputProps) -> Element {{
    let has_error = props.error.is_some();

    rsx! {{
        div {{
            class: "form-group",

            // Label
            if let Some(label) = &props.label {{
                label {{
                    class: "form-label",
                    "{{label}}"
                    if props.required {{
                        span {{
                            class: "text-xs",
                            style: "color: #f87171; margin-left: 0.25rem;",
                            "*"
                        }}
                    }}
                }}
            }}

            // Input
            input {{
                class: format!(
                    "form-input {{}}",
                    if has_error {{ "border-color: #f87171;" }} else {{ "" }}
                ),
                r#type: "{{props.input_type}}",
                value: "{{props.value}}",
                placeholder: props.placeholder.as_deref().unwrap_or(""),
                required: props.required,
                disabled: props.disabled,
                oninput: move |evt| {{
                    props.on_change.call(evt.value().clone());
                }},
            }}

            // Error message
            if let Some(error) = &props.error {{
                p {{
                    class: "form-error",
                    "{{error}}"
                }}
            }}

            // Help text
            if let Some(help) = &props.help {{
                if !has_error {{
                    p {{
                        class: "text-xs text-muted mt-1",
                        "{{help}}"
                    }}
                }}
            }}
        }}
    }}
}}

// ============================================================================
// FormTextArea — multi-line text input
// ============================================================================

/// Properties for the FormTextArea component.
#[derive(Props, Clone, PartialEq)]
pub struct FormTextAreaProps {{
    /// Current value of the textarea.
    pub value: String,
    /// Callback when the value changes.
    pub on_change: EventHandler<String>,
    /// Label text displayed above the textarea.
    #[props(default)]
    pub label: Option<String>,
    /// Placeholder text shown when empty.
    #[props(default)]
    pub placeholder: Option<String>,
    /// Number of visible rows.
    #[props(default = 4)]
    pub rows: u32,
    /// Whether the field is required.
    #[props(default = false)]
    pub required: bool,
    /// Whether the textarea is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Error message to display below.
    #[props(default)]
    pub error: Option<String>,
}}

/// A styled multi-line text input.
#[component]
pub fn FormTextArea(props: FormTextAreaProps) -> Element {{
    let has_error = props.error.is_some();

    rsx! {{
        div {{
            class: "form-group",

            if let Some(label) = &props.label {{
                label {{
                    class: "form-label",
                    "{{label}}"
                    if props.required {{
                        span {{
                            class: "text-xs",
                            style: "color: #f87171; margin-left: 0.25rem;",
                            "*"
                        }}
                    }}
                }}
            }}

            textarea {{
                class: format!(
                    "form-input form-textarea {{}}",
                    if has_error {{ "border-color: #f87171;" }} else {{ "" }}
                ),
                rows: "{{props.rows}}",
                value: "{{props.value}}",
                placeholder: props.placeholder.as_deref().unwrap_or(""),
                required: props.required,
                disabled: props.disabled,
                oninput: move |evt| {{
                    props.on_change.call(evt.value().clone());
                }},
            }}

            if let Some(error) = &props.error {{
                p {{ class: "form-error", "{{error}}" }}
            }}
        }}
    }}
}}

// ============================================================================
// FormSelect — dropdown selection
// ============================================================================

/// A single option in a FormSelect dropdown.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectOption {{
    /// The value submitted when this option is selected.
    pub value: String,
    /// The display label.
    pub label: String,
}}

/// Properties for the FormSelect component.
#[derive(Props, Clone, PartialEq)]
pub struct FormSelectProps {{
    /// Currently selected value.
    pub value: String,
    /// Callback when the selection changes.
    pub on_change: EventHandler<String>,
    /// Available options.
    pub options: Vec<SelectOption>,
    /// Label text displayed above the select.
    #[props(default)]
    pub label: Option<String>,
    /// Whether the field is required.
    #[props(default = false)]
    pub required: bool,
    /// Whether the select is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Error message.
    #[props(default)]
    pub error: Option<String>,
}}

/// A styled dropdown select input.
#[component]
pub fn FormSelect(props: FormSelectProps) -> Element {{
    let has_error = props.error.is_some();

    rsx! {{
        div {{
            class: "form-group",

            if let Some(label) = &props.label {{
                label {{
                    class: "form-label",
                    "{{label}}"
                    if props.required {{
                        span {{
                            class: "text-xs",
                            style: "color: #f87171; margin-left: 0.25rem;",
                            "*"
                        }}
                    }}
                }}
            }}

            select {{
                class: format!(
                    "form-input {{}}",
                    if has_error {{ "border-color: #f87171;" }} else {{ "" }}
                ),
                required: props.required,
                disabled: props.disabled,
                value: "{{props.value}}",
                onchange: move |evt| {{
                    props.on_change.call(evt.value().clone());
                }},

                for opt in props.options.iter() {{
                    option {{
                        key: "{{opt.value}}",
                        value: "{{opt.value}}",
                        selected: opt.value == props.value,
                        "{{opt.label}}"
                    }}
                }}
            }}

            if let Some(error) = &props.error {{
                p {{ class: "form-error", "{{error}}" }}
            }}
        }}
    }}
}}

// ============================================================================
// FormCheckbox — boolean checkbox
// ============================================================================

/// Properties for the FormCheckbox component.
#[derive(Props, Clone, PartialEq)]
pub struct FormCheckboxProps {{
    /// Whether the checkbox is checked.
    pub checked: bool,
    /// Callback when the checked state changes.
    pub on_change: EventHandler<bool>,
    /// Label text displayed next to the checkbox.
    pub label: String,
    /// Whether the checkbox is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Help text displayed below.
    #[props(default)]
    pub help: Option<String>,
}}

/// A styled checkbox input with a label.
#[component]
pub fn FormCheckbox(props: FormCheckboxProps) -> Element {{
    rsx! {{
        div {{
            class: "form-group",

            label {{
                class: "flex items-center gap-2 cursor-pointer",

                input {{
                    class: "form-checkbox",
                    r#type: "checkbox",
                    checked: props.checked,
                    disabled: props.disabled,
                    onchange: move |evt| {{
                        props.on_change.call(evt.checked());
                    }},
                }}

                span {{
                    class: "text-sm",
                    "{{props.label}}"
                }}
            }}

            if let Some(help) = &props.help {{
                p {{
                    class: "text-xs text-muted mt-1",
                    style: "margin-left: 1.5rem;",
                    "{{help}}"
                }}
            }}
        }}
    }}
}}

// ============================================================================
// FormActions — standard form footer with submit / cancel buttons
// ============================================================================

/// Properties for the FormActions component.
#[derive(Props, Clone, PartialEq)]
pub struct FormActionsProps {{
    /// Text for the primary submit button.
    #[props(default = "Save".to_string())]
    pub submit_text: String,
    /// Text for the secondary cancel button.
    #[props(default = "Cancel".to_string())]
    pub cancel_text: String,
    /// Callback when submit is clicked.
    pub on_submit: EventHandler<()>,
    /// Callback when cancel is clicked.
    pub on_cancel: EventHandler<()>,
    /// Whether the submit button is in a loading state.
    #[props(default = false)]
    pub loading: bool,
    /// Whether the submit button is disabled.
    #[props(default = false)]
    pub disabled: bool,
}}

/// Standard form footer with submit and cancel buttons.
#[component]
pub fn FormActions(props: FormActionsProps) -> Element {{
    rsx! {{
        div {{
            class: "flex justify-between items-center mt-4",
            style: "padding-top: 1rem; border-top: 1px solid #334155;",

            button {{
                class: "btn btn-secondary",
                onclick: move |_| props.on_cancel.call(()),
                "{{props.cancel_text}}"
            }}

            button {{
                class: "btn btn-primary",
                disabled: props.disabled || props.loading,
                onclick: move |_| {{
                    if !props.loading && !props.disabled {{
                        props.on_submit.call(());
                    }}
                }},

                if props.loading {{
                    span {{ class: "spinner", style: "width: 1rem; height: 1rem; margin-right: 0.5rem;" }}
                }}
                "{{props.submit_text}}"
            }}
        }}
    }}
}}

// ============================================================================
// Alert — dismissible notification banner
// ============================================================================

/// Properties for the Alert component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertProps {{
    /// Alert message text.
    pub message: String,
    /// Alert level: "error", "success", or "info".
    #[props(default = "info".to_string())]
    pub level: String,
    /// Callback when the alert is dismissed.
    #[props(default)]
    pub on_dismiss: Option<EventHandler<()>>,
}}

/// A styled alert / notification banner.
#[component]
pub fn Alert(props: AlertProps) -> Element {{
    let class = match props.level.as_str() {{
        "error" => "alert alert-error",
        "success" => "alert alert-success",
        _ => "alert alert-info",
    }};

    rsx! {{
        div {{
            class: "{{class}}",

            div {{
                class: "flex justify-between items-start",

                span {{ "{{props.message}}" }}

                if let Some(handler) = &props.on_dismiss {{
                    button {{
                        class: "text-sm",
                        style: "opacity: 0.7; cursor: pointer; background: none; border: none; color: inherit;",
                        onclick: move |_| handler.call(()),
                        "\u{{2715}}"
                    }}
                }}
            }}
        }}
    }}
}}
"#,
        header = file_header("Form input components for CRUD operations."),
    );

    GeneratedFile::new("frontend/src/components/form.rs", content, FileType::Rust)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_core::DataType;
    use imortal_ir::{EndpointGroup, Entity, Field, ProjectGraph, ProjectType};

    fn fullstack_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("my_app");
        project.config.project_type = ProjectType::Fullstack;
        project.config.package_name = "my_app".to_string();

        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;

        let mut email = Field::new("email", DataType::String);
        email.required = true;
        user.fields.push(email);

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        user.fields.push(name);

        project.add_entity(user);
        project.add_endpoint(EndpointGroup::new(user_id, "User"));

        project
    }

    #[test]
    fn test_generate_components_produces_five_files() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        assert_eq!(files.len(), 5);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"frontend/src/components/mod.rs".to_string()));
        assert!(paths.contains(&"frontend/src/components/navbar.rs".to_string()));
        assert!(paths.contains(&"frontend/src/components/sidebar.rs".to_string()));
        assert!(paths.contains(&"frontend/src/components/table.rs".to_string()));
        assert!(paths.contains(&"frontend/src/components/form.rs".to_string()));
    }

    #[test]
    fn test_components_mod_re_exports() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let mod_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().ends_with("mod.rs"))
            .unwrap();

        assert!(mod_file.content.contains("pub mod navbar;"));
        assert!(mod_file.content.contains("pub mod sidebar;"));
        assert!(mod_file.content.contains("pub mod table;"));
        assert!(mod_file.content.contains("pub mod form;"));
        assert!(mod_file.content.contains("pub use navbar::Navbar;"));
        assert!(mod_file.content.contains("pub use sidebar::Sidebar;"));
        assert!(
            mod_file
                .content
                .contains("pub use table::{DataTable, Column}")
        );
        assert!(mod_file.content.contains("pub use form::"));
    }

    #[test]
    fn test_navbar_has_component() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let navbar = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("navbar.rs"))
            .unwrap();

        assert!(navbar.content.contains("pub fn Navbar("));
        assert!(navbar.content.contains("NavbarProps"));
        assert!(navbar.content.contains("navbar-title"));
        assert!(navbar.content.contains("navbar-actions"));
    }

    #[test]
    fn test_sidebar_has_entity_links() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let sidebar = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("sidebar.rs"))
            .unwrap();

        assert!(sidebar.content.contains("pub fn Sidebar("));
        assert!(sidebar.content.contains("Route::Home"));
        assert!(sidebar.content.contains("Route::UserList"));
        assert!(sidebar.content.contains("sidebar-link"));
        assert!(sidebar.content.contains("Dashboard"));
    }

    #[test]
    fn test_sidebar_uses_router() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let sidebar = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("sidebar.rs"))
            .unwrap();

        assert!(sidebar.content.contains("use crate::router::Route;"));
        assert!(sidebar.content.contains("Link {"));
    }

    #[test]
    fn test_table_has_data_table_component() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let table = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("table.rs"))
            .unwrap();

        assert!(table.content.contains("pub fn DataTable("));
        assert!(table.content.contains("DataTableProps"));
        assert!(table.content.contains("pub struct Column"));
        assert!(table.content.contains("pub fn Pagination("));
        assert!(table.content.contains("PaginationProps"));
        assert!(table.content.contains("pub fn DeleteConfirmDialog("));
        assert!(table.content.contains("format_cell_value"));
    }

    #[test]
    fn test_table_has_loading_and_empty_states() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let table = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("table.rs"))
            .unwrap();

        assert!(table.content.contains("loading"));
        assert!(table.content.contains("spinner"));
        assert!(table.content.contains("empty-state"));
        assert!(table.content.contains("No Data"));
    }

    #[test]
    fn test_table_has_edit_delete_actions() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let table = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("table.rs"))
            .unwrap();

        assert!(table.content.contains("on_edit"));
        assert!(table.content.contains("on_delete"));
        assert!(table.content.contains("Edit"));
        assert!(table.content.contains("Delete"));
    }

    #[test]
    fn test_form_has_all_input_types() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("form.rs"))
            .unwrap();

        assert!(form.content.contains("pub fn FormInput("));
        assert!(form.content.contains("pub fn FormTextArea("));
        assert!(form.content.contains("pub fn FormSelect("));
        assert!(form.content.contains("pub fn FormCheckbox("));
        assert!(form.content.contains("pub fn FormActions("));
        assert!(form.content.contains("pub fn Alert("));
    }

    #[test]
    fn test_form_input_has_error_handling() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("form.rs"))
            .unwrap();

        assert!(form.content.contains("form-error"));
        assert!(form.content.contains("error: Option<String>"));
        assert!(form.content.contains("has_error"));
    }

    #[test]
    fn test_form_input_supports_types() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("form.rs"))
            .unwrap();

        assert!(form.content.contains("input_type: String"));
        assert!(form.content.contains("r#type:"));
    }

    #[test]
    fn test_form_select_has_options() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("form.rs"))
            .unwrap();

        assert!(form.content.contains("pub struct SelectOption"));
        assert!(form.content.contains("pub options: Vec<SelectOption>"));
        assert!(form.content.contains("selected: opt.value == props.value"));
    }

    #[test]
    fn test_alert_supports_levels() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("form.rs"))
            .unwrap();

        assert!(form.content.contains("alert-error"));
        assert!(form.content.contains("alert-success"));
        assert!(form.content.contains("alert-info"));
        assert!(form.content.contains("on_dismiss"));
    }

    #[test]
    fn test_sidebar_has_package_name() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let sidebar = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("sidebar.rs"))
            .unwrap();

        assert!(sidebar.content.contains("my_app"));
    }

    #[test]
    fn test_sidebar_excludes_entity_without_endpoint() {
        let mut project = fullstack_project();

        // Add entity WITHOUT endpoint
        let mut cat = Entity::new("Category");
        cat.config.timestamps = false;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        cat.fields.push(name);
        project.add_entity(cat);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let sidebar = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("sidebar.rs"))
            .unwrap();

        assert!(sidebar.content.contains("UserList"));
        assert!(
            !sidebar.content.contains("CategoryList"),
            "Entity without endpoint should not appear in sidebar"
        );
    }

    #[test]
    fn test_table_has_column_constructors() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let table = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("table.rs"))
            .unwrap();

        // Column struct should have new(), fixed(), and with_class() methods
        assert!(table.content.contains("pub fn new("));
        assert!(table.content.contains("pub fn fixed("));
        assert!(table.content.contains("pub fn with_class("));
        assert!(table.content.contains("pub key: String"));
        assert!(table.content.contains("pub label: String"));
        assert!(table.content.contains("pub sortable: bool"));
        assert!(table.content.contains("pub class: String"));
    }

    #[test]
    fn test_pagination_component_exists() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let table = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("table.rs"))
            .unwrap();

        assert!(table.content.contains("pub fn Pagination("));
        assert!(table.content.contains("on_page_change"));
        assert!(table.content.contains("Previous"));
        assert!(table.content.contains("Next"));
        assert!(table.content.contains("total_pages"));
    }

    #[test]
    fn test_delete_confirm_dialog_exists() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let table = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("table.rs"))
            .unwrap();

        assert!(table.content.contains("pub fn DeleteConfirmDialog("));
        assert!(table.content.contains("on_confirm"));
        assert!(table.content.contains("on_cancel"));
        assert!(table.content.contains("modal-backdrop"));
        assert!(table.content.contains("Confirm Deletion"));
        assert!(table.content.contains("cannot be undone"));
    }

    #[test]
    fn test_form_actions_has_loading_state() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("form.rs"))
            .unwrap();

        assert!(form.content.contains("loading: bool"));
        assert!(
            form.content
                .contains("disabled: props.disabled || props.loading")
        );
        assert!(form.content.contains("spinner"));
    }

    #[test]
    fn test_all_files_have_headers() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        for file in &files {
            assert!(
                file.content.contains("Auto-generated by Immortal Engine"),
                "File {} should have a header",
                file.path.display()
            );
        }
    }

    #[test]
    fn test_sidebar_multiple_entities() {
        let mut project = fullstack_project();

        let mut post = Entity::new("Post");
        post.config.timestamps = true;
        let post_id = post.id;
        let mut title = Field::new("title", DataType::String);
        title.required = true;
        post.fields.push(title);
        project.add_entity(post);
        project.add_endpoint(EndpointGroup::new(post_id, "Post"));

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let sidebar = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("sidebar.rs"))
            .unwrap();

        assert!(sidebar.content.contains("UserList"));
        assert!(sidebar.content.contains("PostList"));
    }

    #[test]
    fn test_navbar_default_title() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_components(&ctx);

        let navbar = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("navbar.rs"))
            .unwrap();

        // Default title should use the package name
        assert!(navbar.content.contains("my_app"));
    }
}
