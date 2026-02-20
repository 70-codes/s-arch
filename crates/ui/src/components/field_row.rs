//! # Field Row Component
//!
//! Displays a single field within an entity card on the visual canvas.
//!
//! The field row shows:
//! - Field name with appropriate styling
//! - Data type badge
//! - Attribute indicators (required, unique, indexed, PK, FK)
//! - Selection state
//!

use dioxus::prelude::*;
use imortal_core::types::{DataType, FieldId};
use imortal_ir::field::Field;

// ============================================================================
// Field Row Component
// ============================================================================

/// Properties for the FieldRow component
#[derive(Props, Clone, PartialEq)]
pub struct FieldRowProps {
    /// The field to display
    pub field: Field,

    /// Whether this field is currently selected
    #[props(default = false)]
    pub selected: bool,

    /// Whether the entity card is collapsed (show minimal info)
    #[props(default = false)]
    pub collapsed: bool,

    /// Whether the field is in edit mode
    #[props(default = false)]
    pub editing: bool,

    /// Callback when field is clicked
    #[props(default)]
    pub on_click: EventHandler<FieldId>,

    /// Callback when field is double-clicked (for editing)
    #[props(default)]
    pub on_double_click: EventHandler<FieldId>,

    /// Callback when field is right-clicked (context menu)
    #[props(default)]
    pub on_context_menu: EventHandler<(FieldId, MouseEvent)>,
}

/// Field row component for displaying a single field in an entity card
#[component]
pub fn FieldRow(props: FieldRowProps) -> Element {
    let field = &props.field;
    let selected = props.selected;
    let collapsed = props.collapsed;
    let field_id = field.id;

    // Build class names
    let row_class = if selected {
        "field-row group flex items-center gap-2 px-2 py-1.5 cursor-pointer transition-colors bg-indigo-600/20 border-l-2 border-indigo-500"
    } else {
        "field-row group flex items-center gap-2 px-2 py-1.5 cursor-pointer transition-colors hover:bg-slate-700/50 border-l-2 border-transparent"
    };

    // Get type display info
    let _type_info = get_type_display_info(&field.data_type);

    rsx! {
        div {
            class: "{row_class}",
            onclick: move |_| props.on_click.call(field_id),
            ondoubleclick: move |_| props.on_double_click.call(field_id),
            oncontextmenu: move |e| {
                e.prevent_default();
                props.on_context_menu.call((field_id, e));
            },

            // Field name with indicators
            div {
                class: "flex-1 flex items-center gap-1.5 min-w-0",

                // Primary key indicator
                if field.is_primary_key {
                    span {
                        class: "text-amber-400 text-xs flex-shrink-0",
                        title: "Primary Key",
                        "🔑"
                    }
                }

                // Foreign key indicator
                if field.is_foreign_key {
                    span {
                        class: "text-blue-400 text-xs flex-shrink-0",
                        title: "Foreign Key",
                        "🔗"
                    }
                }

                // Field name
                span {
                    class: "text-sm truncate",
                    class: if field.required { "font-medium text-slate-100" } else { "text-slate-300" },
                    title: "{field.name}",
                    "{field.name}"
                }

                // Required indicator (asterisk)
                if field.required && !field.is_primary_key {
                    span {
                        class: "text-rose-400 text-xs flex-shrink-0",
                        title: "Required",
                        "*"
                    }
                }
            }

            // Attribute badges (only show when not collapsed)
            if !collapsed {
                div {
                    class: "flex items-center gap-1 flex-shrink-0",

                    // Unique badge
                    if field.unique && !field.is_primary_key {
                        FieldBadge {
                            text: "U",
                            color: BadgeColor::Purple,
                            title: "Unique",
                        }
                    }

                    // Indexed badge
                    if field.indexed && !field.is_primary_key && !field.unique {
                        FieldBadge {
                            text: "I",
                            color: BadgeColor::Cyan,
                            title: "Indexed",
                        }
                    }

                    // Secret/sensitive badge
                    if field.secret {
                        FieldBadge {
                            text: "🔒",
                            color: BadgeColor::Rose,
                            title: "Secret/Sensitive",
                        }
                    }
                }
            }

            // Data type badge
            TypeBadge {
                data_type: field.data_type.clone(),
                compact: collapsed,
            }
        }
    }
}

// ============================================================================
// Type Badge Component
// ============================================================================

/// Properties for TypeBadge component
#[derive(Props, Clone, PartialEq)]
struct TypeBadgeProps {
    /// The data type to display
    data_type: DataType,

    /// Whether to show compact version
    #[props(default = false)]
    compact: bool,
}

/// Badge showing the data type of a field
#[component]
fn TypeBadge(props: TypeBadgeProps) -> Element {
    let info = get_type_display_info(&props.data_type);

    let class = format!(
        "text-xs px-1.5 py-0.5 rounded {} {}",
        info.bg_class, info.text_class
    );

    let display_text = if props.compact {
        info.short_name
    } else {
        info.name
    };

    rsx! {
        span {
            class: "{class}",
            title: "{info.full_name}",
            "{display_text}"
        }
    }
}

// ============================================================================
// Field Badge Component
// ============================================================================

/// Badge colors for field attributes
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum BadgeColor {
    Blue,
    Green,
    Purple,
    Amber,
    Rose,
    Cyan,
    Slate,
}

impl BadgeColor {
    fn classes(&self) -> (&'static str, &'static str) {
        match self {
            BadgeColor::Blue => ("bg-blue-500/20", "text-blue-300"),
            BadgeColor::Green => ("bg-green-500/20", "text-green-300"),
            BadgeColor::Purple => ("bg-purple-500/20", "text-purple-300"),
            BadgeColor::Amber => ("bg-amber-500/20", "text-amber-300"),
            BadgeColor::Rose => ("bg-rose-500/20", "text-rose-300"),
            BadgeColor::Cyan => ("bg-cyan-500/20", "text-cyan-300"),
            BadgeColor::Slate => ("bg-slate-500/20", "text-slate-300"),
        }
    }
}

/// Properties for FieldBadge component
#[derive(Props, Clone, PartialEq)]
struct FieldBadgeProps {
    /// Badge text
    text: &'static str,

    /// Badge color
    color: BadgeColor,

    /// Tooltip title
    title: &'static str,
}

/// Small badge for field attributes
#[component]
fn FieldBadge(props: FieldBadgeProps) -> Element {
    let (bg, text) = props.color.classes();
    let class = format!(
        "text-[10px] px-1 py-0.5 rounded font-medium {} {}",
        bg, text
    );

    rsx! {
        span {
            class: "{class}",
            title: "{props.title}",
            "{props.text}"
        }
    }
}

// ============================================================================
// Type Display Information
// ============================================================================

/// Display information for a data type
struct TypeDisplayInfo {
    /// Full type name
    full_name: String,
    /// Display name
    name: &'static str,
    /// Short name for compact display
    short_name: &'static str,
    /// Background color class
    bg_class: &'static str,
    /// Text color class
    text_class: &'static str,
}

/// Get display information for a data type
fn get_type_display_info(data_type: &DataType) -> TypeDisplayInfo {
    match data_type {
        DataType::String => TypeDisplayInfo {
            full_name: "String (VARCHAR)".to_string(),
            name: "String",
            short_name: "Str",
            bg_class: "bg-emerald-500/20",
            text_class: "text-emerald-300",
        },
        DataType::Text => TypeDisplayInfo {
            full_name: "Text (TEXT/CLOB)".to_string(),
            name: "Text",
            short_name: "Txt",
            bg_class: "bg-emerald-500/20",
            text_class: "text-emerald-300",
        },
        DataType::Int32 => TypeDisplayInfo {
            full_name: "32-bit Integer".to_string(),
            name: "Int32",
            short_name: "i32",
            bg_class: "bg-blue-500/20",
            text_class: "text-blue-300",
        },
        DataType::Int64 => TypeDisplayInfo {
            full_name: "64-bit Integer".to_string(),
            name: "Int64",
            short_name: "i64",
            bg_class: "bg-blue-500/20",
            text_class: "text-blue-300",
        },
        DataType::Float32 => TypeDisplayInfo {
            full_name: "32-bit Float".to_string(),
            name: "Float32",
            short_name: "f32",
            bg_class: "bg-sky-500/20",
            text_class: "text-sky-300",
        },
        DataType::Float64 => TypeDisplayInfo {
            full_name: "64-bit Float".to_string(),
            name: "Float64",
            short_name: "f64",
            bg_class: "bg-sky-500/20",
            text_class: "text-sky-300",
        },
        DataType::Bool => TypeDisplayInfo {
            full_name: "Boolean".to_string(),
            name: "Bool",
            short_name: "bool",
            bg_class: "bg-violet-500/20",
            text_class: "text-violet-300",
        },
        DataType::Uuid => TypeDisplayInfo {
            full_name: "UUID".to_string(),
            name: "UUID",
            short_name: "UUID",
            bg_class: "bg-amber-500/20",
            text_class: "text-amber-300",
        },
        DataType::DateTime => TypeDisplayInfo {
            full_name: "DateTime (TIMESTAMP)".to_string(),
            name: "DateTime",
            short_name: "DT",
            bg_class: "bg-rose-500/20",
            text_class: "text-rose-300",
        },
        DataType::Date => TypeDisplayInfo {
            full_name: "Date".to_string(),
            name: "Date",
            short_name: "Date",
            bg_class: "bg-rose-500/20",
            text_class: "text-rose-300",
        },
        DataType::Time => TypeDisplayInfo {
            full_name: "Time".to_string(),
            name: "Time",
            short_name: "Time",
            bg_class: "bg-rose-500/20",
            text_class: "text-rose-300",
        },
        DataType::Bytes => TypeDisplayInfo {
            full_name: "Bytes (BYTEA/BLOB)".to_string(),
            name: "Bytes",
            short_name: "[]u8",
            bg_class: "bg-slate-500/20",
            text_class: "text-slate-300",
        },
        DataType::Json => TypeDisplayInfo {
            full_name: "JSON".to_string(),
            name: "JSON",
            short_name: "JSON",
            bg_class: "bg-orange-500/20",
            text_class: "text-orange-300",
        },
        DataType::Optional(inner) => {
            let inner_info = get_type_display_info(inner);
            TypeDisplayInfo {
                full_name: "Optional".to_string(),
                name: inner_info.name,
                short_name: inner_info.short_name,
                bg_class: inner_info.bg_class,
                text_class: inner_info.text_class,
            }
        }
        DataType::Array(inner) => {
            let inner_info = get_type_display_info(inner);
            TypeDisplayInfo {
                full_name: "Array".to_string(),
                name: inner_info.name,
                short_name: inner_info.short_name,
                bg_class: "bg-indigo-500/20",
                text_class: "text-indigo-300",
            }
        }
        DataType::Reference { entity_name, .. } => TypeDisplayInfo {
            full_name: entity_name.clone(),
            name: "Ref",
            short_name: "→",
            bg_class: "bg-pink-500/20",
            text_class: "text-pink-300",
        },
        DataType::Enum { name, .. } => TypeDisplayInfo {
            full_name: name.clone(),
            name: "Enum",
            short_name: "Enum",
            bg_class: "bg-teal-500/20",
            text_class: "text-teal-300",
        },
    }
}

// ============================================================================
// Compact Field Row (for collapsed entities)
// ============================================================================

/// Properties for CompactFieldRow
#[derive(Props, Clone, PartialEq)]
pub struct CompactFieldRowProps {
    /// The field to display
    pub field: Field,
}

/// Compact field row for collapsed entity cards
#[component]
pub fn CompactFieldRow(props: CompactFieldRowProps) -> Element {
    let field = &props.field;
    let type_info = get_type_display_info(&field.data_type);

    rsx! {
        div {
            class: "flex items-center gap-1 text-xs text-slate-400",

            // Key indicators
            if field.is_primary_key {
                span { class: "text-amber-400", "🔑" }
            }
            if field.is_foreign_key {
                span { class: "text-blue-400", "🔗" }
            }

            // Name
            span {
                class: "truncate",
                "{field.name}"
            }

            // Type (very compact)
            span {
                class: "text-slate-500",
                "({type_info.short_name})"
            }
        }
    }
}

// ============================================================================
// Field List Component
// ============================================================================

/// Properties for FieldList component
#[derive(Props, Clone, PartialEq)]
pub struct FieldListProps {
    /// List of fields to display
    pub fields: Vec<Field>,

    /// Currently selected field ID (if any)
    #[props(default)]
    pub selected_field: Option<FieldId>,

    /// Whether the list is in collapsed mode
    #[props(default = false)]
    pub collapsed: bool,

    /// Maximum fields to show (0 = all)
    #[props(default = 0)]
    pub max_visible: usize,

    /// Callback when a field is clicked
    #[props(default)]
    pub on_field_click: EventHandler<FieldId>,

    /// Callback when a field is double-clicked
    #[props(default)]
    pub on_field_double_click: EventHandler<FieldId>,

    /// Callback for context menu
    #[props(default)]
    pub on_field_context_menu: EventHandler<(FieldId, MouseEvent)>,
}

/// Component for displaying a list of fields
#[component]
pub fn FieldList(props: FieldListProps) -> Element {
    let fields = &props.fields;
    let max_visible = if props.max_visible == 0 {
        fields.len()
    } else {
        props.max_visible
    };

    let visible_fields: Vec<_> = fields.iter().take(max_visible).collect();
    let hidden_count = fields.len().saturating_sub(max_visible);

    rsx! {
        div {
            class: "field-list divide-y divide-slate-700/50",

            // Visible fields
            for field in visible_fields {
                FieldRow {
                    key: "{field.id}",
                    field: field.clone(),
                    selected: props.selected_field == Some(field.id),
                    collapsed: props.collapsed,
                    on_click: move |id| props.on_field_click.call(id),
                    on_double_click: move |id| props.on_field_double_click.call(id),
                    on_context_menu: move |(id, e)| props.on_field_context_menu.call((id, e)),
                }
            }

            // Hidden fields indicator
            if hidden_count > 0 {
                div {
                    class: "px-2 py-1 text-xs text-slate-500 text-center",
                    if hidden_count == 1 {
                        "+1 more field"
                    } else {
                        "+{hidden_count} more fields"
                    }
                }
            }

            // Empty state
            if fields.is_empty() {
                div {
                    class: "px-2 py-3 text-xs text-slate-500 text-center italic",
                    "No fields defined"
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
    fn test_type_display_info() {
        let info = get_type_display_info(&DataType::String);
        assert_eq!(info.name, "String");
        assert_eq!(info.short_name, "Str");

        let info = get_type_display_info(&DataType::Int64);
        assert_eq!(info.name, "Int64");
        assert_eq!(info.short_name, "i64");

        let info = get_type_display_info(&DataType::Bool);
        assert_eq!(info.name, "Bool");
        assert_eq!(info.short_name, "bool");
    }

    #[test]
    fn test_type_display_optional() {
        let optional_string = DataType::Optional(Box::new(DataType::String));
        let info = get_type_display_info(&optional_string);
        // Should use inner type's info
        assert_eq!(info.name, "String");
    }

    #[test]
    fn test_badge_colors() {
        let (bg, text) = BadgeColor::Blue.classes();
        assert!(bg.contains("blue"));
        assert!(text.contains("blue"));

        let (bg, text) = BadgeColor::Rose.classes();
        assert!(bg.contains("rose"));
        assert!(text.contains("rose"));
    }
}
