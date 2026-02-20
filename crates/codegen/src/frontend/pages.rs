//! # Page Generator
//!
//! Generates Dioxus page components for the frontend application.
//!
//! ## Generated Files
//!
//! - `frontend/src/pages/mod.rs` — module declarations and re-exports
//! - `frontend/src/pages/home.rs` — dashboard / home page
//! - `frontend/src/pages/{entity}_list.rs` — list page per entity (table + pagination)
//! - `frontend/src/pages/{entity}_form.rs` — create/edit form page per entity
//!
//! ## Architecture
//!
//! Each entity with configured endpoints gets a pair of pages:
//!
//! - **List page**: Fetches and displays all records in a `DataTable` with
//!   pagination, search, and delete functionality.
//! - **Form page**: A create/edit form with fields derived from the entity's
//!   `CreateDto` / `UpdateDto`. Detects create vs. edit mode from the route.
//!
//! Pages use the generated API client (`crate::api::client`) to communicate
//! with the backend and the shared DTOs from the `shared` crate.

use crate::context::{EntityInfo, GenerationContext};
use crate::rust::file_header;
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate all page files for the frontend application.
///
/// Produces:
/// - `frontend/src/pages/mod.rs`
/// - `frontend/src/pages/home.rs`
/// - One `{entity}_list.rs` and one `{entity}_form.rs` per entity with endpoints
pub fn generate_pages(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    let mut files = Vec::new();

    // pages/mod.rs
    files.push(generate_pages_mod(ctx));

    // pages/home.rs
    files.push(generate_home_page(ctx));

    // Per-entity pages
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);

        // Only generate pages for entities with endpoints
        if info.endpoint().is_none() {
            continue;
        }

        files.push(generate_list_page(&info, ctx));
        files.push(generate_form_page(&info, ctx));
    }

    files
}

// ============================================================================
// pages/mod.rs
// ============================================================================

fn generate_pages_mod(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(1024);

    content.push_str(&file_header(
        "Page components for the frontend application.",
    ));

    content.push_str("pub mod home;\n");

    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        if info.endpoint().is_none() {
            continue;
        }

        let snake = info.snake_name();
        content.push_str(&format!("pub mod {}_list;\n", snake));
        content.push_str(&format!("pub mod {}_form;\n", snake));
    }

    content.push('\n');

    // Re-exports
    content.push_str("// Re-exports for convenience\n");
    content.push_str("pub use home::HomePage;\n");

    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        if info.endpoint().is_none() {
            continue;
        }

        let snake = info.snake_name();
        let pascal = info.pascal_name();
        content.push_str(&format!("pub use {snake}_list::{pascal}ListPage;\n"));
        content.push_str(&format!("pub use {snake}_form::{pascal}FormPage;\n"));
    }

    GeneratedFile::new("frontend/src/pages/mod.rs", content, FileType::Rust)
}

// ============================================================================
// pages/home.rs — Dashboard / Home page
// ============================================================================

fn generate_home_page(ctx: &GenerationContext) -> GeneratedFile {
    let pkg = ctx.package_name();

    let mut content = String::with_capacity(4096);

    content.push_str(&file_header("Home / dashboard page."));

    content.push_str("use dioxus::prelude::*;\n\n");

    // Import router for navigation links
    content.push_str("use crate::router::Route;\n\n");

    content.push_str(&format!(
        r#"/// Home / dashboard page component.
///
/// Displays an overview of the application with quick-access cards for
/// each entity and summary statistics.
#[component]
pub fn HomePage() -> Element {{
    rsx! {{
        div {{
            // Page header
            div {{
                class: "mb-4",
                h1 {{
                    class: "card-title",
                    style: "font-size: 1.5rem;",
                    "Welcome to {pkg}"
                }}
                p {{
                    class: "text-muted text-sm mt-1",
                    "Manage your data using the navigation on the left, or use the quick links below."
                }}
            }}

            // Entity cards grid
            div {{
                class: "grid-2",

"#,
    ));

    // Generate a card for each entity with endpoints
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        if info.endpoint().is_none() {
            continue;
        }

        let pascal = info.pascal_name();
        let plural = info.plural_name();
        let field_count = entity.fields.len();
        let default_desc = format!("Manage {} records.", plural);
        let desc = entity.description.as_deref().unwrap_or(&default_desc);

        content.push_str(&format!(
            r#"                // {pascal} card
                div {{
                    class: "card",

                    div {{
                        class: "card-header",
                        h3 {{
                            class: "card-title",
                            "{pascal}s"
                        }}
                        span {{
                            class: "badge badge-info",
                            "{field_count} fields"
                        }}
                    }}

                    p {{
                        class: "text-sm text-muted mb-4",
                        "{desc}"
                    }}

                    div {{
                        class: "flex gap-2",

                        Link {{
                            class: "btn btn-primary btn-sm",
                            to: Route::{pascal}List {{}},
                            "View All"
                        }}

                        Link {{
                            class: "btn btn-secondary btn-sm",
                            to: Route::{pascal}New {{}},
                            "+ Create"
                        }}
                    }}
                }}

"#,
        ));
    }

    // If no entities have endpoints, show a helpful message
    if ctx
        .entities()
        .iter()
        .all(|e| EntityInfo::new(e, ctx).endpoint().is_none())
    {
        content.push_str(
            r#"                // No entities with endpoints
                div {
                    class: "card",
                    style: "grid-column: span 2;",

                    div {
                        class: "empty-state",
                        div { class: "empty-state-icon", "\u{1f4e6}" }
                        h3 { class: "empty-state-title", "No Entities Configured" }
                        p { class: "empty-state-text", "Add entities and configure endpoints to get started." }
                    }
                }
"#,
        );
    }

    content.push_str(
        r#"            }

            // Footer info
            div {
                class: "mt-4 text-xs text-muted text-center",
                style: "padding-top: 1rem; border-top: 1px solid #334155;",
                "Generated by Immortal Engine v2.0"
            }
        }
    }
}
"#,
    );

    GeneratedFile::new("frontend/src/pages/home.rs", content, FileType::Rust)
}

// ============================================================================
// Per-entity list page
// ============================================================================

fn generate_list_page(info: &EntityInfo, ctx: &GenerationContext) -> GeneratedFile {
    let snake = info.snake_name();
    let pascal = info.pascal_name();
    let plural = info.plural_name();
    let response_dto = GenerationContext::response_dto_name(&info.entity.name);
    let base_path = info.base_path();
    let path = format!("frontend/src/pages/{}_list.rs", snake);

    // Determine columns from response fields
    let response_fields = info.response_fields();

    let mut content = String::with_capacity(8192);

    content.push_str(&file_header(&format!(
        "{} list page — displays all {}s in a data table.",
        pascal, snake
    )));

    // ── Imports ──────────────────────────────────────────────────────────
    content.push_str("use dioxus::prelude::*;\n");
    content.push_str("use serde_json::{json, Value};\n\n");

    content.push_str("use crate::api::client::ApiClient;\n");
    content.push_str(
        "use crate::components::table::{DataTable, Column, Pagination, DeleteConfirmDialog};\n",
    );
    content.push_str("use crate::components::form::Alert;\n");
    content.push_str("use crate::router::Route;\n\n");

    // ── Component ────────────────────────────────────────────────────────
    content.push_str(&format!(
        r#"/// {pascal} list page component.
///
/// Fetches and displays all {snake}s in a paginated data table.
/// Supports:
/// - Pagination (page, per_page)
/// - Row click to view details
/// - Edit and delete actions per row
/// - Create new {snake} via button
#[component]
pub fn {pascal}ListPage() -> Element {{
    // State
    let mut rows = use_signal(Vec::<Value>::new);
    let mut loading = use_signal(|| true);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);
    let mut page = use_signal(|| 1u64);
    let mut total = use_signal(|| 0u64);
    let mut total_pages = use_signal(|| 0u64);
    let per_page = 20u64;

    // Delete confirmation state
    let mut delete_target: Signal<Option<String>> = use_signal(|| None);
    let mut delete_name = use_signal(|| String::new());
    let mut success_msg: Signal<Option<String>> = use_signal(|| None);

    let navigator = use_navigator();

    // Fetch data
    let current_page = *page.read();
    use_effect(move || {{
        spawn(async move {{
            loading.set(true);
            error_msg.set(None);

            let client = ApiClient::new();
            match client.list_{plural}(current_page, per_page).await {{
                Ok(response) => {{
                    // Convert items to JSON values for the generic table
                    let items: Vec<Value> = response.items
                        .into_iter()
                        .map(|item| serde_json::to_value(item).unwrap_or_default())
                        .collect();
                    rows.set(items);
                    total.set(response.total);
                    total_pages.set(response.total_pages);
                }}
                Err(e) => {{
                    error_msg.set(Some(format!("Failed to load {plural}: {{}}", e)));
                }}
            }}

            loading.set(false);
        }});
    }});

    // Column definitions
    let columns = vec![
"#,
    ));

    // Add columns based on response fields (limit to reasonable number)
    let display_fields: Vec<&imortal_ir::Field> = response_fields
        .iter()
        .filter(|f| !f.is_primary_key || f.name == "id")
        .take(6)
        .copied()
        .collect();

    for field in &display_fields {
        let field_name = GenerationContext::snake(&field.name);
        let label = field_name
            .replace('_', " ")
            .split(' ')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        if field.name == "id" {
            content.push_str(&format!(
                "        Column::new(\"{field_name}\", \"{label}\").with_class(\"font-mono text-xs\"),\n"
            ));
        } else {
            content.push_str(&format!(
                "        Column::new(\"{field_name}\", \"{label}\"),\n"
            ));
        }
    }

    content.push_str("    ];\n\n");

    // ── Render ────────────────────────────────────────────────────────────
    content.push_str(&format!(
        r#"    rsx! {{
        div {{
            // Page header
            div {{
                class: "card-header mb-4",

                h2 {{
                    class: "card-title",
                    "{pascal}s"
                }}

                div {{
                    class: "flex gap-2",

                    // Refresh button
                    button {{
                        class: "btn btn-secondary btn-sm",
                        onclick: move |_| {{
                            page.set(*page.read()); // trigger re-fetch
                        }},
                        "\u{{21bb}} Refresh"
                    }}

                    // Create button
                    Link {{
                        class: "btn btn-primary btn-sm",
                        to: Route::{pascal}New {{}},
                        "+ New {pascal}"
                    }}
                }}
            }}

            // Success message
            if let Some(msg) = success_msg.read().as_ref() {{
                Alert {{
                    message: msg.clone(),
                    level: "success".to_string(),
                    on_dismiss: move |_| success_msg.set(None),
                }}
            }}

            // Error message
            if let Some(msg) = error_msg.read().as_ref() {{
                Alert {{
                    message: msg.clone(),
                    level: "error".to_string(),
                    on_dismiss: move |_| error_msg.set(None),
                }}
            }}

            // Data table
            DataTable {{
                columns: columns,
                rows: rows.read().clone(),
                loading: *loading.read(),
                empty_text: "No {plural} found. Create one to get started.".to_string(),
                on_row_click: move |row: Value| {{
                    if let Some(id) = row.get("id").and_then(|v| v.as_str()) {{
                        // Navigate to edit page
                        navigator.push(Route::{pascal}Edit {{ id: id.to_string() }});
                    }}
                }},
                on_edit: move |id: String| {{
                    navigator.push(Route::{pascal}Edit {{ id }});
                }},
                on_delete: move |id: String| {{
                    delete_name.set(format!("{pascal} {{}}", &id[..8.min(id.len())]));
                    delete_target.set(Some(id));
                }},
            }}

            // Pagination
            if *total_pages.read() > 1 {{
                Pagination {{
                    page: *page.read(),
                    total_pages: *total_pages.read(),
                    total_items: *total.read(),
                    per_page: per_page,
                    on_page_change: move |new_page| page.set(new_page),
                }}
            }}

            // Delete confirmation dialog
            DeleteConfirmDialog {{
                visible: delete_target.read().is_some(),
                item_name: delete_name.read().clone(),
                on_cancel: move |_| {{
                    delete_target.set(None);
                }},
                on_confirm: move |_| {{
                    if let Some(id) = delete_target.read().clone() {{
                        delete_target.set(None);
                        spawn(async move {{
                            let client = ApiClient::new();
                            match client.delete_{snake}(&id).await {{
                                Ok(_) => {{
                                    success_msg.set(Some("{pascal} deleted successfully.".to_string()));
                                    // Trigger re-fetch
                                    page.set(*page.read());
                                }}
                                Err(e) => {{
                                    error_msg.set(Some(format!("Failed to delete: {{}}", e)));
                                }}
                            }}
                        }});
                    }}
                }},
            }}
        }}
    }}
}}
"#,
    ));

    GeneratedFile::new(path, content, FileType::Rust)
}

// ============================================================================
// Per-entity form page (create / edit)
// ============================================================================

fn generate_form_page(info: &EntityInfo, ctx: &GenerationContext) -> GeneratedFile {
    let snake = info.snake_name();
    let pascal = info.pascal_name();
    let plural = info.plural_name();
    let create_dto = GenerationContext::create_dto_name(&info.entity.name);
    let update_dto = GenerationContext::update_dto_name(&info.entity.name);
    let path = format!("frontend/src/pages/{}_form.rs", snake);

    let create_fields = info.create_fields();
    let update_fields = info.update_fields();

    let mut content = String::with_capacity(8192);

    content.push_str(&file_header(&format!(
        "{} form page — create and edit {}s.",
        pascal, snake
    )));

    // ── Imports ──────────────────────────────────────────────────────────
    content.push_str("use dioxus::prelude::*;\n");
    content.push_str("use serde_json::json;\n\n");

    content.push_str("use crate::api::client::ApiClient;\n");
    content.push_str("use crate::components::form::{FormInput, FormTextArea, FormCheckbox, FormActions, Alert};\n");
    content.push_str("use crate::router::Route;\n");
    content.push_str(&format!("use shared::{{{create_dto}, {update_dto}}};\n\n"));

    // ── Props ────────────────────────────────────────────────────────────
    content.push_str(&format!(
        r#"/// Properties for the {pascal} form page.
///
/// When `id` is `None`, the form operates in **create** mode.
/// When `id` is `Some(…)`, it operates in **edit** mode and fetches the
/// existing record to pre-populate the form.
#[derive(Props, Clone, PartialEq)]
pub struct {pascal}FormPageProps {{
    /// The ID of the {snake} to edit (None for create mode).
    #[props(default)]
    pub id: Option<String>,
}}

"#,
    ));

    // ── Component ────────────────────────────────────────────────────────
    content.push_str(&format!(
        r#"/// {pascal} create/edit form page component.
#[component]
pub fn {pascal}FormPage(props: {pascal}FormPageProps) -> Element {{
    let is_edit = props.id.is_some();
    let edit_id = props.id.clone();
    let navigator = use_navigator();

    // Form state
"#,
    ));

    // Generate signal for each create field
    for field in &create_fields {
        let field_name = GenerationContext::snake(&field.name);
        let default_value = form_default_value(&field.data_type);
        content.push_str(&format!(
            "    let mut {field_name} = use_signal(|| {default_value});\n"
        ));
    }

    content.push_str(
        r#"
    // UI state
    let mut loading = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);
    let mut fetching = use_signal(|| false);

"#,
    );

    // ── Load existing data for edit mode ─────────────────────────────────
    content.push_str(&format!(
        r#"    // Load existing record for edit mode
    let load_id = edit_id.clone();
    use_effect(move || {{
        if let Some(id) = load_id.clone() {{
            spawn(async move {{
                fetching.set(true);
                let client = ApiClient::new();
                match client.get_{snake}(&id).await {{
                    Ok(item) => {{
"#,
    ));

    // Set each field from the loaded item
    for field in &create_fields {
        let field_name = GenerationContext::snake(&field.name);
        let conversion = form_value_from_response(&field.data_type, &field_name);
        content.push_str(&format!(
            "                        {field_name}.set({conversion});\n"
        ));
    }

    content.push_str(
        r#"                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to load record: {}", e)));
                    }
                }
                fetching.set(false);
            });
        }
    });

"#,
    );

    // ── Submit handler ───────────────────────────────────────────────────
    content.push_str(&format!(
        r#"    // Handle form submission
    let submit_id = edit_id.clone();
    let on_submit = move |_| {{
        loading.set(true);
        error_msg.set(None);

        spawn(async move {{
            let client = ApiClient::new();

            let result = if let Some(id) = submit_id.clone() {{
                // Update existing
                let payload = {update_dto} {{
"#,
    ));

    // Build update payload
    for field in &update_fields {
        let field_name = GenerationContext::snake(&field.name);
        let to_option = form_value_to_option(&field.data_type, &field_name);
        content.push_str(&format!("                    {field_name}: {to_option},\n"));
    }

    // Fill in any remaining fields with Default
    content.push_str(&format!(
        r#"                    ..Default::default()
                }};
                client.update_{snake}(&id, &payload).await
            }} else {{
                // Create new
                let payload = {create_dto} {{
"#,
    ));

    // Build create payload
    for field in &create_fields {
        let field_name = GenerationContext::snake(&field.name);
        let to_value = form_value_to_dto(&field.data_type, &field_name, field.required);
        content.push_str(&format!("                    {field_name}: {to_value},\n"));
    }

    content.push_str(&format!(
        r#"                }};
                client.create_{snake}(&payload).await
            }};

            match result {{
                Ok(_) => {{
                    // Navigate back to list
                    navigator.push(Route::{pascal}List {{}});
                }}
                Err(e) => {{
                    error_msg.set(Some(format!("Failed to save: {{}}", e)));
                }}
            }}

            loading.set(false);
        }});
    }};

"#,
    ));

    // ── Render ────────────────────────────────────────────────────────────
    content.push_str(&format!(
        r#"    // Loading state while fetching existing record
    if *fetching.read() {{
        return rsx! {{
            div {{
                class: "loading",
                div {{ class: "spinner" }}
                span {{ class: "ml-2", "Loading {snake}…" }}
            }}
        }};
    }}

    rsx! {{
        div {{
            // Page header
            div {{
                class: "card-header mb-4",

                h2 {{
                    class: "card-title",
                    if is_edit {{ "Edit {pascal}" }} else {{ "Create {pascal}" }}
                }}

                Link {{
                    class: "btn btn-secondary btn-sm",
                    to: Route::{pascal}List {{}},
                    "\u{{2190}} Back to List"
                }}
            }}

            // Error message
            if let Some(msg) = error_msg.read().as_ref() {{
                Alert {{
                    message: msg.clone(),
                    level: "error".to_string(),
                    on_dismiss: move |_| error_msg.set(None),
                }}
            }}

            // Form card
            div {{
                class: "card",

"#,
    ));

    // Generate form fields
    for field in &create_fields {
        let field_name = GenerationContext::snake(&field.name);
        let label = field_name
            .replace('_', " ")
            .split(' ')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        let input_type = form_input_type(&field.data_type, &field.name);
        let placeholder = form_placeholder(&field.data_type, &field.name);

        match &field.data_type {
            imortal_core::DataType::Text => {
                content.push_str(&format!(
                    r#"                FormTextArea {{
                    label: Some("{label}".to_string()),
                    value: {field_name}.read().clone(),
                    placeholder: Some("{placeholder}".to_string()),
                    required: {required},
                    on_change: move |v: String| {field_name}.set(v),
                }}

"#,
                    required = field.required,
                ));
            }
            imortal_core::DataType::Bool => {
                content.push_str(&format!(
                    r#"                FormCheckbox {{
                    label: "{label}".to_string(),
                    checked: *{field_name}.read(),
                    on_change: move |v: bool| {field_name}.set(v),
                }}

"#,
                ));
            }
            _ => {
                content.push_str(&format!(
                    r#"                FormInput {{
                    label: Some("{label}".to_string()),
                    value: {field_name}.read().clone(),
                    input_type: "{input_type}".to_string(),
                    placeholder: Some("{placeholder}".to_string()),
                    required: {required},
                    on_change: move |v: String| {field_name}.set(v),
                }}

"#,
                    required = field.required,
                ));
            }
        }
    }

    // Form actions
    content.push_str(&format!(
        r#"                FormActions {{
                    submit_text: if is_edit {{ "Update {pascal}".to_string() }} else {{ "Create {pascal}".to_string() }},
                    cancel_text: "Cancel".to_string(),
                    loading: *loading.read(),
                    on_submit: on_submit,
                    on_cancel: move |_| {{
                        navigator.push(Route::{pascal}List {{}});
                    }},
                }}
            }}
        }}
    }}
}}
"#,
    ));

    GeneratedFile::new(path, content, FileType::Rust)
}

// ============================================================================
// Form helper functions
// ============================================================================

/// Get the default value for a form signal based on data type.
fn form_default_value(dt: &imortal_core::DataType) -> &'static str {
    use imortal_core::DataType;
    match dt {
        DataType::String | DataType::Text | DataType::Uuid => "String::new()",
        DataType::Int32 | DataType::Int64 => "String::new()",
        DataType::Float32 | DataType::Float64 => "String::new()",
        DataType::Bool => "false",
        DataType::DateTime | DataType::Date | DataType::Time => "String::new()",
        DataType::Json => "String::new()",
        DataType::Bytes => "String::new()",
        DataType::Optional(_) => "String::new()",
        DataType::Array(_) => "String::new()",
        DataType::Reference { .. } => "String::new()",
        DataType::Enum { .. } => "String::new()",
    }
}

/// Generate code to extract a field value from the API response into a signal.
fn form_value_from_response(dt: &imortal_core::DataType, field_name: &str) -> String {
    use imortal_core::DataType;
    match dt {
        DataType::String | DataType::Text => format!("item.{field_name}.clone()"),
        DataType::Int32 | DataType::Int64 => format!("item.{field_name}.to_string()"),
        DataType::Float32 | DataType::Float64 => format!("item.{field_name}.to_string()"),
        DataType::Bool => format!("item.{field_name}"),
        DataType::Uuid => format!("item.{field_name}.to_string()"),
        DataType::DateTime => format!("item.{field_name}.to_rfc3339()"),
        DataType::Date | DataType::Time => format!("item.{field_name}.to_string()"),
        DataType::Json => {
            format!("serde_json::to_string_pretty(&item.{field_name}).unwrap_or_default()")
        }
        DataType::Optional(inner) => {
            format!("item.{field_name}.map(|v| v.to_string()).unwrap_or_default()")
        }
        DataType::Reference { .. } => format!("item.{field_name}.to_string()"),
        _ => format!("item.{field_name}.to_string()"),
    }
}

/// Generate code to convert a form signal value into an Option<T> for UpdateDto.
fn form_value_to_option(dt: &imortal_core::DataType, field_name: &str) -> String {
    use imortal_core::DataType;
    match dt {
        DataType::String | DataType::Text => {
            format!("Some({field_name}.read().clone())")
        }
        DataType::Int32 => format!("{field_name}.read().parse::<i32>().ok()"),
        DataType::Int64 => format!("{field_name}.read().parse::<i64>().ok()"),
        DataType::Float32 => format!("{field_name}.read().parse::<f32>().ok()"),
        DataType::Float64 => format!("{field_name}.read().parse::<f64>().ok()"),
        DataType::Bool => format!("Some(*{field_name}.read())"),
        DataType::Uuid => format!("uuid::Uuid::parse_str(&{field_name}.read()).ok()"),
        _ => format!("Some({field_name}.read().clone())"),
    }
}

/// Generate code to convert a form signal value into the CreateDto field type.
fn form_value_to_dto(dt: &imortal_core::DataType, field_name: &str, required: bool) -> String {
    use imortal_core::DataType;
    match dt {
        DataType::String | DataType::Text => format!("{field_name}.read().clone()"),
        DataType::Int32 => {
            if required {
                format!("{field_name}.read().parse::<i32>().unwrap_or_default()")
            } else {
                format!("{field_name}.read().parse::<i32>().ok()")
            }
        }
        DataType::Int64 => {
            if required {
                format!("{field_name}.read().parse::<i64>().unwrap_or_default()")
            } else {
                format!("{field_name}.read().parse::<i64>().ok()")
            }
        }
        DataType::Float32 => {
            if required {
                format!("{field_name}.read().parse::<f32>().unwrap_or_default()")
            } else {
                format!("{field_name}.read().parse::<f32>().ok()")
            }
        }
        DataType::Float64 => {
            if required {
                format!("{field_name}.read().parse::<f64>().unwrap_or_default()")
            } else {
                format!("{field_name}.read().parse::<f64>().ok()")
            }
        }
        DataType::Bool => format!("*{field_name}.read()"),
        DataType::Uuid => {
            if required {
                format!("uuid::Uuid::parse_str(&{field_name}.read()).unwrap_or_default()")
            } else {
                format!("uuid::Uuid::parse_str(&{field_name}.read()).ok()")
            }
        }
        DataType::Optional(inner) => {
            let inner_conversion = form_value_to_dto(inner, field_name, true);
            format!(
                "if {field_name}.read().is_empty() {{ None }} else {{ Some({inner_conversion}) }}"
            )
        }
        DataType::Reference { .. } => {
            if required {
                format!("uuid::Uuid::parse_str(&{field_name}.read()).unwrap_or_default()")
            } else {
                format!("uuid::Uuid::parse_str(&{field_name}.read()).ok()")
            }
        }
        _ => format!("{field_name}.read().clone()"),
    }
}

/// Get the HTML input type for a data type.
fn form_input_type(dt: &imortal_core::DataType, field_name: &str) -> &'static str {
    use imortal_core::DataType;

    // Check field name for contextual type inference
    if field_name.contains("email") {
        return "email";
    }
    if field_name.contains("password") {
        return "password";
    }
    if field_name.contains("url") || field_name.contains("website") {
        return "url";
    }
    if field_name.contains("phone") || field_name.contains("tel") {
        return "tel";
    }

    match dt {
        DataType::String | DataType::Text => "text",
        DataType::Int32 | DataType::Int64 => "number",
        DataType::Float32 | DataType::Float64 => "number",
        DataType::Bool => "checkbox",
        DataType::Uuid => "text",
        DataType::DateTime => "datetime-local",
        DataType::Date => "date",
        DataType::Time => "time",
        DataType::Json => "text",
        DataType::Optional(inner) => form_input_type(inner, field_name),
        DataType::Reference { .. } => "text",
        _ => "text",
    }
}

/// Get a placeholder string for a form field.
fn form_placeholder(dt: &imortal_core::DataType, field_name: &str) -> String {
    if field_name.contains("email") {
        return "user@example.com".to_string();
    }
    if field_name.contains("password") {
        return "Enter password…".to_string();
    }
    if field_name.contains("url") || field_name.contains("website") {
        return "https://example.com".to_string();
    }
    if field_name.contains("phone") || field_name.contains("tel") {
        return "+1 234 567 8900".to_string();
    }

    use imortal_core::DataType;
    match dt {
        DataType::String | DataType::Text => {
            format!("Enter {}…", field_name.replace('_', " "))
        }
        DataType::Int32 | DataType::Int64 => "0".to_string(),
        DataType::Float32 | DataType::Float64 => "0.0".to_string(),
        DataType::Uuid => "00000000-0000-0000-0000-000000000000".to_string(),
        DataType::DateTime => "2026-01-29T12:00".to_string(),
        DataType::Date => "2026-01-29".to_string(),
        DataType::Time => "12:00:00".to_string(),
        DataType::Json => "{}".to_string(),
        DataType::Optional(inner) => form_placeholder(inner, field_name),
        DataType::Reference { entity_name, .. } => {
            format!("{} ID (UUID)", entity_name)
        }
        _ => format!("Enter {}…", field_name.replace('_', " ")),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_core::DataType;
    use imortal_ir::{EndpointGroup, Entity, Field, ProjectGraph, ProjectType};

    /// Helper: create a fullstack project with a User entity.
    fn fullstack_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("my_app");
        project.config.project_type = ProjectType::Fullstack;
        project.config.package_name = "my_app".to_string();

        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;

        let mut email = Field::new("email", DataType::String);
        email.required = true;
        email.unique = true;
        user.fields.push(email);

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        user.fields.push(name);

        let mut pw = Field::new("password_hash", DataType::String);
        pw.required = true;
        pw.secret = true;
        user.fields.push(pw);

        project.add_entity(user);
        project.add_endpoint(EndpointGroup::new(user_id, "User"));

        project
    }

    #[test]
    fn test_generate_pages_produces_files() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        // mod.rs + home.rs + user_list.rs + user_form.rs = 4
        assert_eq!(files.len(), 4);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"frontend/src/pages/mod.rs".to_string()));
        assert!(paths.contains(&"frontend/src/pages/home.rs".to_string()));
        assert!(paths.contains(&"frontend/src/pages/user_list.rs".to_string()));
        assert!(paths.contains(&"frontend/src/pages/user_form.rs".to_string()));
    }

    #[test]
    fn test_pages_mod_has_declarations() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let mod_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().ends_with("mod.rs"))
            .unwrap();

        assert!(mod_file.content.contains("pub mod home;"));
        assert!(mod_file.content.contains("pub mod user_list;"));
        assert!(mod_file.content.contains("pub mod user_form;"));
        assert!(mod_file.content.contains("pub use home::HomePage;"));
        assert!(
            mod_file
                .content
                .contains("pub use user_list::UserListPage;")
        );
        assert!(
            mod_file
                .content
                .contains("pub use user_form::UserFormPage;")
        );
    }

    #[test]
    fn test_home_page_has_entity_cards() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let home = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("home.rs"))
            .unwrap();

        assert!(home.content.contains("pub fn HomePage()"));
        assert!(home.content.contains("Users"));
        assert!(home.content.contains("Route::UserList"));
        assert!(home.content.contains("Route::UserNew"));
        assert!(home.content.contains("View All"));
        assert!(home.content.contains("+ Create"));
        assert!(home.content.contains("Welcome to my_app"));
    }

    #[test]
    fn test_home_page_shows_field_count() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let home = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("home.rs"))
            .unwrap();

        // User entity has id + email + name + password_hash = 4 fields
        assert!(home.content.contains("fields"));
    }

    #[test]
    fn test_list_page_has_table() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let list = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_list.rs"))
            .unwrap();

        assert!(list.content.contains("pub fn UserListPage()"));
        assert!(list.content.contains("DataTable"));
        assert!(list.content.contains("Column::new"));
        assert!(list.content.contains("Pagination"));
        assert!(list.content.contains("ApiClient::new()"));
        assert!(list.content.contains("list_users"));
    }

    #[test]
    fn test_list_page_has_pagination() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let list = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_list.rs"))
            .unwrap();

        assert!(list.content.contains("page"));
        assert!(list.content.contains("total_pages"));
        assert!(list.content.contains("per_page"));
        assert!(list.content.contains("on_page_change"));
    }

    #[test]
    fn test_list_page_has_delete_confirmation() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let list = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_list.rs"))
            .unwrap();

        assert!(list.content.contains("DeleteConfirmDialog"));
        assert!(list.content.contains("delete_target"));
        assert!(list.content.contains("delete_user"));
    }

    #[test]
    fn test_list_page_has_error_handling() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let list = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_list.rs"))
            .unwrap();

        assert!(list.content.contains("error_msg"));
        assert!(list.content.contains("Alert"));
        assert!(list.content.contains("success_msg"));
    }

    #[test]
    fn test_list_page_navigates_to_edit() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let list = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_list.rs"))
            .unwrap();

        assert!(list.content.contains("Route::UserEdit"));
        assert!(list.content.contains("on_edit"));
        assert!(list.content.contains("on_row_click"));
    }

    #[test]
    fn test_list_page_has_create_button() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let list = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_list.rs"))
            .unwrap();

        assert!(list.content.contains("Route::UserNew"));
        assert!(list.content.contains("+ New User"));
    }

    #[test]
    fn test_form_page_has_inputs() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_form.rs"))
            .unwrap();

        assert!(form.content.contains("pub fn UserFormPage("));
        assert!(form.content.contains("FormInput"));
        assert!(form.content.contains("FormActions"));
        assert!(form.content.contains("ApiClient::new()"));
    }

    #[test]
    fn test_form_page_handles_create_and_edit() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_form.rs"))
            .unwrap();

        assert!(form.content.contains("is_edit"));
        assert!(form.content.contains("Create User"));
        assert!(form.content.contains("Edit User"));
        assert!(form.content.contains("Update User"));
        assert!(form.content.contains("create_user"));
        assert!(form.content.contains("update_user"));
    }

    #[test]
    fn test_form_page_loads_existing_for_edit() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_form.rs"))
            .unwrap();

        assert!(form.content.contains("get_user"));
        assert!(form.content.contains("fetching"));
        assert!(form.content.contains("Loading user"));
    }

    #[test]
    fn test_form_page_navigates_back() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_form.rs"))
            .unwrap();

        assert!(form.content.contains("Route::UserList"));
        assert!(form.content.contains("Back to List"));
        assert!(form.content.contains("Cancel"));
    }

    #[test]
    fn test_form_page_has_error_handling() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_form.rs"))
            .unwrap();

        assert!(form.content.contains("error_msg"));
        assert!(form.content.contains("Alert"));
        assert!(form.content.contains("Failed to save"));
        assert!(form.content.contains("Failed to load"));
    }

    #[test]
    fn test_form_page_uses_shared_dtos() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_form.rs"))
            .unwrap();

        assert!(
            form.content
                .contains("use shared::{CreateUserDto, UpdateUserDto}")
        );
    }

    #[test]
    fn test_form_page_has_loading_state() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_form.rs"))
            .unwrap();

        assert!(form.content.contains("loading"));
        assert!(form.content.contains("FormActions"));
    }

    #[test]
    fn test_form_input_type_inference() {
        assert_eq!(form_input_type(&DataType::String, "email"), "email");
        assert_eq!(form_input_type(&DataType::String, "password"), "password");
        assert_eq!(form_input_type(&DataType::String, "website_url"), "url");
        assert_eq!(form_input_type(&DataType::String, "phone"), "tel");
        assert_eq!(form_input_type(&DataType::String, "name"), "text");
        assert_eq!(form_input_type(&DataType::Int32, "count"), "number");
        assert_eq!(form_input_type(&DataType::Float64, "price"), "number");
        assert_eq!(form_input_type(&DataType::Bool, "active"), "checkbox");
        assert_eq!(
            form_input_type(&DataType::DateTime, "created_at"),
            "datetime-local"
        );
        assert_eq!(form_input_type(&DataType::Date, "birthday"), "date");
        assert_eq!(form_input_type(&DataType::Time, "start_time"), "time");
    }

    #[test]
    fn test_form_placeholder_inference() {
        assert!(form_placeholder(&DataType::String, "email").contains("@example.com"));
        assert!(form_placeholder(&DataType::String, "password").contains("Enter password"));
        assert!(form_placeholder(&DataType::String, "name").contains("Enter name"));
        assert_eq!(form_placeholder(&DataType::Int32, "count"), "0");
        assert_eq!(form_placeholder(&DataType::Float64, "price"), "0.0");
        assert!(form_placeholder(&DataType::Uuid, "ref_id").contains("00000000"));
        assert!(form_placeholder(&DataType::DateTime, "ts").contains("2026"));
    }

    #[test]
    fn test_form_default_values() {
        assert_eq!(form_default_value(&DataType::String), "String::new()");
        assert_eq!(form_default_value(&DataType::Int32), "String::new()");
        assert_eq!(form_default_value(&DataType::Bool), "false");
        assert_eq!(form_default_value(&DataType::Uuid), "String::new()");
    }

    #[test]
    fn test_no_pages_for_entity_without_endpoint() {
        let mut project = fullstack_project();

        // Add entity WITHOUT endpoint
        let mut cat = Entity::new("Category");
        cat.config.timestamps = false;
        let mut cname = Field::new("name", DataType::String);
        cname.required = true;
        cat.fields.push(cname);
        project.add_entity(cat);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(paths.iter().any(|p| p.contains("user_list.rs")));
        assert!(paths.iter().any(|p| p.contains("user_form.rs")));
        assert!(!paths.iter().any(|p| p.contains("category_list.rs")));
        assert!(!paths.iter().any(|p| p.contains("category_form.rs")));
    }

    #[test]
    fn test_multiple_entities_generate_separate_pages() {
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
        let files = generate_pages(&ctx);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        // mod.rs + home.rs + user_list + user_form + post_list + post_form = 6
        assert_eq!(files.len(), 6);
        assert!(paths.iter().any(|p| p.contains("user_list.rs")));
        assert!(paths.iter().any(|p| p.contains("user_form.rs")));
        assert!(paths.iter().any(|p| p.contains("post_list.rs")));
        assert!(paths.iter().any(|p| p.contains("post_form.rs")));
    }

    #[test]
    fn test_list_page_columns_from_entity_fields() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let list = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_list.rs"))
            .unwrap();

        // Should have columns for the response fields
        assert!(list.content.contains("Column::new(\"id\""));
        assert!(list.content.contains("Column::new(\"email\""));
        assert!(list.content.contains("Column::new(\"name\""));
        // password_hash is secret, should NOT appear
        assert!(!list.content.contains("password_hash"));
    }

    #[test]
    fn test_form_page_email_field_uses_email_type() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_form.rs"))
            .unwrap();

        // Email field should use input_type "email"
        assert!(form.content.contains("\"email\""));
    }

    #[test]
    fn test_all_page_files_have_headers() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        for file in &files {
            assert!(
                file.content.contains("Auto-generated by Immortal Engine"),
                "File {} should have a header",
                file.path.display()
            );
        }
    }

    #[test]
    fn test_home_page_with_no_entities() {
        let mut project = ProjectGraph::new("empty");
        project.config.project_type = ProjectType::Fullstack;

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        // mod.rs + home.rs only
        assert_eq!(files.len(), 2);

        let home = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("home.rs"))
            .unwrap();

        assert!(home.content.contains("No Entities Configured"));
    }

    #[test]
    fn test_list_page_has_refresh_button() {
        let project = fullstack_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let list = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user_list.rs"))
            .unwrap();

        assert!(list.content.contains("Refresh"));
    }

    #[test]
    fn test_form_value_to_dto_string() {
        let result = form_value_to_dto(&DataType::String, "name", true);
        assert_eq!(result, "name.read().clone()");
    }

    #[test]
    fn test_form_value_to_dto_int() {
        let result = form_value_to_dto(&DataType::Int32, "count", true);
        assert!(result.contains("parse::<i32>()"));
        assert!(result.contains("unwrap_or_default()"));
    }

    #[test]
    fn test_form_value_to_dto_optional() {
        let result = form_value_to_dto(
            &DataType::Optional(Box::new(DataType::String)),
            "bio",
            false,
        );
        assert!(result.contains("is_empty()"));
        assert!(result.contains("None"));
        assert!(result.contains("Some"));
    }

    #[test]
    fn test_form_value_to_option_string() {
        let result = form_value_to_option(&DataType::String, "name");
        assert!(result.contains("Some("));
        assert!(result.contains("read().clone()"));
    }

    #[test]
    fn test_form_value_to_option_int() {
        let result = form_value_to_option(&DataType::Int32, "count");
        assert!(result.contains("parse::<i32>().ok()"));
    }

    #[test]
    fn test_form_value_from_response_string() {
        let result = form_value_from_response(&DataType::String, "name");
        assert_eq!(result, "item.name.clone()");
    }

    #[test]
    fn test_form_value_from_response_int() {
        let result = form_value_from_response(&DataType::Int32, "count");
        assert_eq!(result, "item.count.to_string()");
    }

    #[test]
    fn test_form_value_from_response_bool() {
        let result = form_value_from_response(&DataType::Bool, "active");
        assert_eq!(result, "item.active");
    }

    #[test]
    fn test_form_value_from_response_uuid() {
        let result = form_value_from_response(&DataType::Uuid, "ref_id");
        assert_eq!(result, "item.ref_id.to_string()");
    }

    #[test]
    fn test_text_field_uses_textarea() {
        let mut project = ProjectGraph::new("test");
        project.config.project_type = ProjectType::Fullstack;
        project.config.package_name = "test".to_string();

        let mut entity = Entity::new("Post");
        entity.config.timestamps = false;
        let entity_id = entity.id;

        let mut title = Field::new("title", DataType::String);
        title.required = true;
        entity.fields.push(title);

        let mut content_field = Field::new("content", DataType::Text);
        content_field.required = false;
        entity.fields.push(content_field);

        project.add_entity(entity);
        project.add_endpoint(EndpointGroup::new(entity_id, "Post"));

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("post_form.rs"))
            .unwrap();

        // Text field should use FormTextArea, not FormInput
        assert!(
            form.content.contains("FormTextArea"),
            "Text data type should render as FormTextArea"
        );
    }

    #[test]
    fn test_bool_field_uses_checkbox() {
        let mut project = ProjectGraph::new("test");
        project.config.project_type = ProjectType::Fullstack;
        project.config.package_name = "test".to_string();

        let mut entity = Entity::new("Setting");
        entity.config.timestamps = false;
        let entity_id = entity.id;

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);

        let mut active = Field::new("active", DataType::Bool);
        active.required = true;
        entity.fields.push(active);

        project.add_entity(entity);
        project.add_endpoint(EndpointGroup::new(entity_id, "Setting"));

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_pages(&ctx);

        let form = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("setting_form.rs"))
            .unwrap();

        assert!(
            form.content.contains("FormCheckbox"),
            "Bool data type should render as FormCheckbox"
        );
    }
}
