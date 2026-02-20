//! # Data Type Selector Component
//!
//! An enhanced data type selector with visual grouping and type information.
//!
//! ## Features
//!
//! - Visual grouping of data types by category
//! - Type icons and descriptions
//! - Search/filter functionality
//! - Optional and Array modifiers
//! - Keyboard navigation support
//!

use dioxus::prelude::*;
use imortal_core::types::DataType;

// ============================================================================
// Types
// ============================================================================

/// Category of data types for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataTypeCategory {
    Text,
    Numeric,
    Temporal,
    Boolean,
    Binary,
    Complex,
}

impl DataTypeCategory {
    /// Get the display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            DataTypeCategory::Text => "Text",
            DataTypeCategory::Numeric => "Numeric",
            DataTypeCategory::Temporal => "Date & Time",
            DataTypeCategory::Boolean => "Boolean",
            DataTypeCategory::Binary => "Binary",
            DataTypeCategory::Complex => "Complex",
        }
    }

    /// Get the icon for this category
    pub fn icon(&self) -> &'static str {
        match self {
            DataTypeCategory::Text => "📝",
            DataTypeCategory::Numeric => "🔢",
            DataTypeCategory::Temporal => "📅",
            DataTypeCategory::Boolean => "✓",
            DataTypeCategory::Binary => "📦",
            DataTypeCategory::Complex => "🔗",
        }
    }
}

/// Information about a data type
#[derive(Debug, Clone, PartialEq)]
pub struct DataTypeInfo {
    pub data_type: DataType,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub category: DataTypeCategory,
    pub rust_type: &'static str,
    pub sql_type: &'static str,
}

impl DataTypeInfo {
    const fn new(
        data_type: DataType,
        name: &'static str,
        description: &'static str,
        icon: &'static str,
        category: DataTypeCategory,
        rust_type: &'static str,
        sql_type: &'static str,
    ) -> Self {
        Self {
            data_type,
            name,
            description,
            icon,
            category,
            rust_type,
            sql_type,
        }
    }
}

/// Get all available data types with their information
pub fn get_all_data_types() -> Vec<DataTypeInfo> {
    vec![
        // Text types
        DataTypeInfo::new(
            DataType::String,
            "String",
            "Variable-length text up to 255 characters",
            "📝",
            DataTypeCategory::Text,
            "String",
            "VARCHAR(255)",
        ),
        DataTypeInfo::new(
            DataType::Text,
            "Text",
            "Long-form text content without length limit",
            "📄",
            DataTypeCategory::Text,
            "String",
            "TEXT",
        ),
        // Numeric types
        DataTypeInfo::new(
            DataType::Int32,
            "Integer",
            "32-bit signed integer (-2B to +2B)",
            "🔢",
            DataTypeCategory::Numeric,
            "i32",
            "INTEGER",
        ),
        DataTypeInfo::new(
            DataType::Int64,
            "Big Integer",
            "64-bit signed integer for large numbers",
            "🔢",
            DataTypeCategory::Numeric,
            "i64",
            "BIGINT",
        ),
        DataTypeInfo::new(
            DataType::Float32,
            "Float",
            "32-bit floating point number",
            "🔢",
            DataTypeCategory::Numeric,
            "f32",
            "REAL",
        ),
        DataTypeInfo::new(
            DataType::Float64,
            "Double",
            "64-bit double precision floating point",
            "🔢",
            DataTypeCategory::Numeric,
            "f64",
            "DOUBLE PRECISION",
        ),
        // Temporal types
        DataTypeInfo::new(
            DataType::DateTime,
            "DateTime",
            "Date and time with timezone",
            "📅",
            DataTypeCategory::Temporal,
            "DateTime<Utc>",
            "TIMESTAMP WITH TIME ZONE",
        ),
        DataTypeInfo::new(
            DataType::Date,
            "Date",
            "Date without time component",
            "📆",
            DataTypeCategory::Temporal,
            "NaiveDate",
            "DATE",
        ),
        DataTypeInfo::new(
            DataType::Time,
            "Time",
            "Time without date component",
            "🕐",
            DataTypeCategory::Temporal,
            "NaiveTime",
            "TIME",
        ),
        // Boolean type
        DataTypeInfo::new(
            DataType::Bool,
            "Boolean",
            "True or false value",
            "✓",
            DataTypeCategory::Boolean,
            "bool",
            "BOOLEAN",
        ),
        // Binary types
        DataTypeInfo::new(
            DataType::Uuid,
            "UUID",
            "Universally unique identifier",
            "🔑",
            DataTypeCategory::Binary,
            "Uuid",
            "UUID",
        ),
        DataTypeInfo::new(
            DataType::Bytes,
            "Bytes",
            "Binary data (files, images, etc.)",
            "📦",
            DataTypeCategory::Binary,
            "Vec<u8>",
            "BYTEA",
        ),
        // Complex types
        DataTypeInfo::new(
            DataType::Json,
            "JSON",
            "JSON/JSONB structured data",
            "{ }",
            DataTypeCategory::Complex,
            "serde_json::Value",
            "JSONB",
        ),
    ]
}

// ============================================================================
// Component Props
// ============================================================================

#[derive(Props, Clone, PartialEq)]
pub struct DataTypeSelectorProps {
    /// Currently selected data type
    pub value: DataType,

    /// Whether the type is optional (nullable)
    #[props(default = false)]
    pub is_optional: bool,

    /// Whether the type is an array
    #[props(default = false)]
    pub is_array: bool,

    /// Optional label for the selector
    #[props(default)]
    pub label: Option<String>,

    /// Optional help text
    #[props(default)]
    pub help_text: Option<String>,

    /// Whether the selector is disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Whether to show type modifiers (Optional, Array)
    #[props(default = true)]
    pub show_modifiers: bool,

    /// Whether to show compact view
    #[props(default = false)]
    pub compact: bool,

    /// Callback when data type changes
    pub on_change: EventHandler<DataType>,

    /// Callback when optional modifier changes
    #[props(default)]
    pub on_optional_change: EventHandler<bool>,

    /// Callback when array modifier changes
    #[props(default)]
    pub on_array_change: EventHandler<bool>,
}

// ============================================================================
// Main Component
// ============================================================================

/// Enhanced data type selector with visual grouping
#[component]
pub fn DataTypeSelector(props: DataTypeSelectorProps) -> Element {
    let mut is_open = use_signal(|| false);
    let mut search_query = use_signal(|| String::new());
    let mut highlighted_index = use_signal(|| 0usize);

    // Get all data types (stored in signal for reactive access)
    let all_types = use_signal(|| get_all_data_types());

    // Filter types based on search query
    let filtered_types = use_memo(move || {
        let query = search_query.read().to_lowercase();
        let types = all_types.read();
        if query.is_empty() {
            types.clone()
        } else {
            types
                .iter()
                .filter(|t| {
                    t.name.to_lowercase().contains(&query)
                        || t.description.to_lowercase().contains(&query)
                        || t.rust_type.to_lowercase().contains(&query)
                })
                .cloned()
                .collect()
        }
    });

    // Group types by category
    let grouped_types = use_memo(move || {
        let types = filtered_types.read();
        let mut groups: Vec<(DataTypeCategory, Vec<DataTypeInfo>)> = Vec::new();

        for category in [
            DataTypeCategory::Text,
            DataTypeCategory::Numeric,
            DataTypeCategory::Temporal,
            DataTypeCategory::Boolean,
            DataTypeCategory::Binary,
            DataTypeCategory::Complex,
        ] {
            let category_types: Vec<_> = types
                .iter()
                .filter(|t| t.category == category)
                .cloned()
                .collect();

            if !category_types.is_empty() {
                groups.push((category, category_types));
            }
        }

        groups
    });

    // Get current type info
    let current_type_info = {
        let types = all_types.read();
        types
            .iter()
            .find(|t| std::mem::discriminant(&t.data_type) == std::mem::discriminant(&props.value))
            .cloned()
    };

    // Handle type selection
    let mut handle_select = move |data_type: DataType| {
        props.on_change.call(data_type);
        is_open.set(false);
        search_query.set(String::new());
    };

    // Handle keyboard navigation
    let handle_keydown = move |e: KeyboardEvent| {
        let types = filtered_types.read();
        let type_count = types.len();

        match e.key() {
            Key::ArrowDown => {
                e.prevent_default();
                let current = *highlighted_index.read();
                highlighted_index.set((current + 1) % type_count.max(1));
            }
            Key::ArrowUp => {
                e.prevent_default();
                let current = *highlighted_index.read();
                highlighted_index.set(
                    current
                        .checked_sub(1)
                        .unwrap_or(type_count.saturating_sub(1)),
                );
            }
            Key::Escape => {
                e.prevent_default();
                is_open.set(false);
                search_query.set(String::new());
            }
            _ => {}
        }
    };

    // Build the display value
    let display_value = current_type_info
        .as_ref()
        .map(|t| {
            let mut name = t.name.to_string();
            if props.is_array {
                name = format!("[{}]", name);
            }
            if props.is_optional {
                name = format!("{}?", name);
            }
            name
        })
        .unwrap_or_else(|| "Select type...".to_string());

    let display_icon = current_type_info.as_ref().map(|t| t.icon).unwrap_or("❓");

    rsx! {
        div {
            class: "data-type-selector relative",

            // Label
            if let Some(label) = &props.label {
                label {
                    class: "block text-sm font-medium text-slate-300 mb-1.5",
                    "{label}"
                }
            }

            // Main button/trigger
            button {
                r#type: "button",
                class: format!(
                    "w-full flex items-center gap-2 px-3 py-2 bg-slate-700 border rounded-lg text-left transition-colors {} {}",
                    if *is_open.read() { "border-indigo-500 ring-1 ring-indigo-500" } else { "border-slate-600 hover:border-slate-500" },
                    if props.disabled { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" }
                ),
                disabled: props.disabled,
                onclick: move |_| {
                    if !props.disabled {
                        let current = *is_open.read();
                        is_open.set(!current);
                    }
                },

                // Type icon
                span { class: "text-lg", "{display_icon}" }

                // Type name
                span { class: "flex-1 text-white", "{display_value}" }

                // Chevron
                span {
                    class: format!(
                        "text-slate-400 transition-transform {}",
                        if *is_open.read() { "rotate-180" } else { "" }
                    ),
                    "▼"
                }
            }

            // Type modifiers
            if props.show_modifiers {
                div {
                    class: "flex items-center gap-4 mt-2",

                    // Optional modifier
                    label {
                        class: "flex items-center gap-2 text-sm text-slate-400 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "w-4 h-4 rounded bg-slate-700 border-slate-600 text-indigo-500 focus:ring-indigo-500",
                            checked: props.is_optional,
                            disabled: props.disabled,
                            onchange: move |e| {
                                props.on_optional_change.call(e.checked());
                            },
                        }
                        span { "Optional (nullable)" }
                    }

                    // Array modifier
                    label {
                        class: "flex items-center gap-2 text-sm text-slate-400 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "w-4 h-4 rounded bg-slate-700 border-slate-600 text-indigo-500 focus:ring-indigo-500",
                            checked: props.is_array,
                            disabled: props.disabled,
                            onchange: move |e| {
                                props.on_array_change.call(e.checked());
                            },
                        }
                        span { "Array (multiple)" }
                    }
                }
            }

            // Help text
            if let Some(help) = &props.help_text {
                p {
                    class: "mt-1 text-xs text-slate-500",
                    "{help}"
                }
            }

            // Dropdown panel
            if *is_open.read() {
                div {
                    class: "absolute z-50 w-full mt-1 bg-slate-800 border border-slate-700 rounded-lg shadow-xl overflow-hidden",
                    style: "max-height: 400px;",

                    // Search input
                    div {
                        class: "p-2 border-b border-slate-700",
                        input {
                            r#type: "text",
                            class: "w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded text-sm text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500",
                            placeholder: "Search types...",
                            value: "{search_query}",
                            autofocus: true,
                            oninput: move |e| {
                                search_query.set(e.value());
                                highlighted_index.set(0);
                            },
                            onkeydown: handle_keydown,
                        }
                    }

                    // Type list (scrollable)
                    div {
                        class: "overflow-y-auto",
                        style: "max-height: 320px;",

                        if filtered_types.read().is_empty() {
                            div {
                                class: "p-4 text-center text-slate-500",
                                "No matching types found"
                            }
                        } else {
                            for (category, types) in grouped_types.read().iter() {
                                // Category header
                                div {
                                    class: "sticky top-0 px-3 py-1.5 bg-slate-750 text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-2",
                                    span { "{category.icon()}" }
                                    "{category.display_name()}"
                                }

                                // Types in category
                                for type_info in types.iter() {
                                    TypeOptionButton {
                                        type_info: type_info.clone(),
                                        selected: std::mem::discriminant(&type_info.data_type) == std::mem::discriminant(&props.value),
                                        compact: props.compact,
                                        on_change: props.on_change.clone(),
                                        is_open: is_open,
                                        search_query: search_query,
                                    }
                                }
                            }
                        }
                    }

                    // Type info footer
                    if let Some(info) = &current_type_info {
                        div {
                            class: "p-2 border-t border-slate-700 bg-slate-750",
                            div {
                                class: "flex items-center justify-between text-xs",
                                span {
                                    class: "text-slate-400",
                                    "Rust: "
                                    span { class: "font-mono text-indigo-300", "{info.rust_type}" }
                                }
                                span {
                                    class: "text-slate-400",
                                    "SQL: "
                                    span { class: "font-mono text-green-300", "{info.sql_type}" }
                                }
                            }
                        }
                    }
                }

                // Backdrop to close dropdown
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| {
                        is_open.set(false);
                        search_query.set(String::new());
                    },
                }
            }
        }
    }
}

// ============================================================================
// Sub-Components
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct TypeOptionButtonProps {
    type_info: DataTypeInfo,
    selected: bool,
    compact: bool,
    on_change: EventHandler<DataType>,
    is_open: Signal<bool>,
    search_query: Signal<String>,
}

#[component]
fn TypeOptionButton(props: TypeOptionButtonProps) -> Element {
    let info = props.type_info.clone();
    let mut is_open = props.is_open;
    let mut search_query = props.search_query;

    rsx! {
        button {
            r#type: "button",
            class: format!(
                "w-full px-3 py-2 flex items-start gap-3 text-left transition-colors hover:bg-slate-700 {}",
                if props.selected { "bg-indigo-500/20 border-l-2 border-indigo-500" } else { "" }
            ),
            onclick: move |_| {
                props.on_change.call(info.data_type.clone());
                is_open.set(false);
                search_query.set(String::new());
            },

            // Icon
            span {
                class: "text-lg flex-shrink-0",
                "{props.type_info.icon}"
            }

            // Content
            div {
                class: "flex-1 min-w-0",

                // Name
                div {
                    class: format!(
                        "font-medium {}",
                        if props.selected { "text-indigo-300" } else { "text-white" }
                    ),
                    "{props.type_info.name}"
                }

                // Description (if not compact)
                if !props.compact {
                    div {
                        class: "text-xs text-slate-500 truncate mt-0.5",
                        "{props.type_info.description}"
                    }
                }
            }

            // Selected indicator
            if props.selected {
                span {
                    class: "text-indigo-400 flex-shrink-0",
                    "✓"
                }
            }
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the base type (unwrapping Optional/Array)
pub fn get_base_type(data_type: &DataType) -> DataType {
    match data_type {
        DataType::Optional(inner) => get_base_type(inner),
        DataType::Array(inner) => get_base_type(inner),
        other => other.clone(),
    }
}

/// Check if a data type is optional
pub fn is_optional_type(data_type: &DataType) -> bool {
    matches!(data_type, DataType::Optional(_))
}

/// Check if a data type is an array
pub fn is_array_type(data_type: &DataType) -> bool {
    match data_type {
        DataType::Array(_) => true,
        DataType::Optional(inner) => is_array_type(inner),
        _ => false,
    }
}

/// Wrap a data type with Optional
pub fn make_optional(data_type: DataType) -> DataType {
    DataType::Optional(Box::new(data_type))
}

/// Wrap a data type with Array
pub fn make_array(data_type: DataType) -> DataType {
    DataType::Array(Box::new(data_type))
}

/// Build a data type with modifiers
pub fn build_type_with_modifiers(base: DataType, optional: bool, array: bool) -> DataType {
    let mut dt = base;

    if array {
        dt = DataType::Array(Box::new(dt));
    }

    if optional {
        dt = DataType::Optional(Box::new(dt));
    }

    dt
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_data_types() {
        let types = get_all_data_types();
        assert!(!types.is_empty());

        // Check that all primitive types are included
        let has_string = types
            .iter()
            .any(|t| matches!(t.data_type, DataType::String));
        let has_int32 = types.iter().any(|t| matches!(t.data_type, DataType::Int32));
        let has_bool = types.iter().any(|t| matches!(t.data_type, DataType::Bool));
        let has_uuid = types.iter().any(|t| matches!(t.data_type, DataType::Uuid));

        assert!(has_string);
        assert!(has_int32);
        assert!(has_bool);
        assert!(has_uuid);
    }

    #[test]
    fn test_data_type_category() {
        assert_eq!(DataTypeCategory::Text.display_name(), "Text");
        assert_eq!(DataTypeCategory::Numeric.display_name(), "Numeric");
        assert_eq!(DataTypeCategory::Temporal.display_name(), "Date & Time");
    }

    #[test]
    fn test_get_base_type() {
        assert_eq!(get_base_type(&DataType::String), DataType::String);
        assert_eq!(
            get_base_type(&DataType::Optional(Box::new(DataType::String))),
            DataType::String
        );
        assert_eq!(
            get_base_type(&DataType::Array(Box::new(DataType::Int32))),
            DataType::Int32
        );
    }

    #[test]
    fn test_is_optional_type() {
        assert!(!is_optional_type(&DataType::String));
        assert!(is_optional_type(&DataType::Optional(Box::new(
            DataType::String
        ))));
    }

    #[test]
    fn test_is_array_type() {
        assert!(!is_array_type(&DataType::String));
        assert!(is_array_type(&DataType::Array(Box::new(DataType::String))));
        assert!(is_array_type(&DataType::Optional(Box::new(
            DataType::Array(Box::new(DataType::String))
        ))));
    }

    #[test]
    fn test_build_type_with_modifiers() {
        let base = DataType::String;

        // No modifiers
        let dt = build_type_with_modifiers(base.clone(), false, false);
        assert!(matches!(dt, DataType::String));

        // Optional only
        let dt = build_type_with_modifiers(base.clone(), true, false);
        assert!(matches!(dt, DataType::Optional(_)));

        // Array only
        let dt = build_type_with_modifiers(base.clone(), false, true);
        assert!(matches!(dt, DataType::Array(_)));

        // Both
        let dt = build_type_with_modifiers(base.clone(), true, true);
        assert!(matches!(dt, DataType::Optional(_)));
        if let DataType::Optional(inner) = dt {
            assert!(matches!(*inner, DataType::Array(_)));
        }
    }
}
