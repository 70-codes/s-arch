//! Field definitions for entity properties
//!
//! This module contains the `Field` struct and related types for defining
//! the properties/columns of an entity in the Immortal Engine IR.

use imortal_core::{
    DataType, EngineError, EngineResult, ReferentialAction, Validatable, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Field
// ============================================================================

/// Represents a field within an entity (maps to a database column)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Unique identifier for this field
    pub id: Uuid,

    /// Field name (used in Rust code, typically snake_case)
    pub name: String,

    /// Database column name (typically snake_case)
    pub column_name: String,

    /// Data type of the field
    pub data_type: DataType,

    /// Whether the field is required (NOT NULL)
    pub required: bool,

    /// Whether the field must be unique
    pub unique: bool,

    /// Whether to create an index on this field
    pub indexed: bool,

    /// Default value for the field
    pub default_value: Option<DefaultValue>,

    /// Validation rules for the field
    pub validations: Vec<Validation>,

    /// Whether this is the primary key
    pub is_primary_key: bool,

    /// Whether this is a foreign key
    pub is_foreign_key: bool,

    /// Foreign key reference details (if is_foreign_key is true)
    pub foreign_key_ref: Option<ForeignKeyRef>,

    /// Human-readable description
    pub description: Option<String>,

    /// UI hints for rendering in forms and tables
    pub ui_hints: UiHints,

    /// Display order (lower numbers appear first)
    pub display_order: i32,

    /// Whether the field is hidden from default views
    pub hidden: bool,

    /// Whether the field is read-only
    pub readonly: bool,

    /// Whether this field is a secret (passwords, API keys)
    pub secret: bool,
}

impl Field {
    /// Create a new field with the given name and data type
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        let name = name.into();
        let column_name = to_snake_case(&name);

        Self {
            id: Uuid::new_v4(),
            name,
            column_name,
            data_type,
            required: false,
            unique: false,
            indexed: false,
            default_value: None,
            validations: Vec::new(),
            is_primary_key: false,
            is_foreign_key: false,
            foreign_key_ref: None,
            description: None,
            ui_hints: UiHints::default(),
            display_order: 0,
            hidden: false,
            readonly: false,
            secret: false,
        }
    }

    /// Create a UUID primary key field
    pub fn primary_key() -> Self {
        let mut field = Self::new("id", DataType::Uuid);
        field.is_primary_key = true;
        field.required = true;
        field.indexed = true;
        field.readonly = true;
        field.ui_hints.label = Some("ID".to_string());
        field
    }

    /// Create a created_at timestamp field
    pub fn created_at() -> Self {
        let mut field = Self::new("created_at", DataType::DateTime);
        field.required = true;
        field.default_value = Some(DefaultValue::Now);
        field.readonly = true;
        field.ui_hints.label = Some("Created At".to_string());
        field
    }

    /// Create an updated_at timestamp field
    pub fn updated_at() -> Self {
        let mut field = Self::new("updated_at", DataType::DateTime);
        field.required = true;
        field.default_value = Some(DefaultValue::Now);
        field.readonly = true;
        field.ui_hints.label = Some("Updated At".to_string());
        field
    }

    /// Create a foreign key field
    pub fn foreign_key(
        name: impl Into<String>,
        target_entity: impl Into<String>,
        target_field: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let target_entity = target_entity.into();
        let target_field = target_field.into();

        let mut field = Self::new(&name, DataType::Uuid);
        field.is_foreign_key = true;
        field.indexed = true;
        field.foreign_key_ref = Some(ForeignKeyRef {
            entity_id: Uuid::nil(), // Will be set when linking
            entity_name: target_entity,
            field_name: target_field,
            on_delete: ReferentialAction::Restrict,
            on_update: ReferentialAction::Cascade,
        });
        field
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Mark the field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Mark the field as unique
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self.indexed = true; // Unique fields are always indexed
        self
    }

    /// Mark the field as indexed
    pub fn indexed(mut self) -> Self {
        self.indexed = true;
        self
    }

    /// Set a default value
    pub fn with_default(mut self, default: DefaultValue) -> Self {
        self.default_value = Some(default);
        self
    }

    /// Add a validation rule
    pub fn with_validation(mut self, validation: Validation) -> Self {
        self.validations.push(validation);
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark the field as hidden
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Mark the field as read-only
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// Mark the field as secret (for passwords, API keys)
    pub fn secret(mut self) -> Self {
        self.secret = true;
        self.ui_hints.widget = Some(WidgetType::Password);
        self
    }

    /// Set the display order
    pub fn with_order(mut self, order: i32) -> Self {
        self.display_order = order;
        self
    }

    /// Set UI hints
    pub fn with_ui_hints(mut self, hints: UiHints) -> Self {
        self.ui_hints = hints;
        self
    }

    /// Set the label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.ui_hints.label = Some(label.into());
        self
    }

    /// Set the placeholder text
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.ui_hints.placeholder = Some(placeholder.into());
        self
    }

    /// Set the column name (if different from field name)
    pub fn with_column_name(mut self, column_name: impl Into<String>) -> Self {
        self.column_name = column_name.into();
        self
    }

    // ========================================================================
    // Utility methods
    // ========================================================================

    /// Get the effective data type (unwrapping Optional if not required)
    pub fn effective_type(&self) -> DataType {
        if self.required {
            self.data_type.clone()
        } else {
            DataType::Optional(Box::new(self.data_type.clone()))
        }
    }

    /// Check if this field is the primary key
    pub fn is_pk(&self) -> bool {
        self.is_primary_key
    }

    /// Check if this field is a foreign key
    pub fn is_fk(&self) -> bool {
        self.is_foreign_key
    }

    /// Check if this field has any validations
    pub fn has_validations(&self) -> bool {
        !self.validations.is_empty() || self.required
    }

    /// Get the display label (falls back to formatted field name)
    pub fn display_label(&self) -> String {
        self.ui_hints
            .label
            .clone()
            .unwrap_or_else(|| to_title_case(&self.name))
    }

    /// Check if this field should be included in create DTOs
    pub fn in_create_dto(&self) -> bool {
        !self.is_primary_key && !self.readonly && self.default_value.is_none()
    }

    /// Check if this field should be included in update DTOs
    pub fn in_update_dto(&self) -> bool {
        !self.is_primary_key && !self.readonly
    }

    /// Check if this field should be included in response DTOs
    pub fn in_response_dto(&self) -> bool {
        !self.secret
    }
}

impl Validatable for Field {
    fn validate(&self) -> EngineResult<()> {
        // Field name must not be empty
        if self.name.is_empty() {
            return Err(EngineError::validation("Field name cannot be empty"));
        }

        // Field name must be valid identifier
        if !is_valid_identifier(&self.name) {
            return Err(EngineError::validation(format!(
                "Field name '{}' is not a valid identifier",
                self.name
            )));
        }

        // Column name must not be empty
        if self.column_name.is_empty() {
            return Err(EngineError::validation("Column name cannot be empty"));
        }

        // Foreign key must have reference
        if self.is_foreign_key && self.foreign_key_ref.is_none() {
            return Err(EngineError::validation(
                "Foreign key field must have a reference",
            ));
        }

        Ok(())
    }
}

impl Default for Field {
    fn default() -> Self {
        Self::new("field", DataType::String)
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Field {}

impl std::hash::Hash for Field {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// ============================================================================
// DefaultValue
// ============================================================================

/// Default values for fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum DefaultValue {
    /// NULL value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Current timestamp (NOW())
    Now,
    /// Generate a new UUID
    Uuid,
    /// Custom SQL expression
    Expression(String),
    /// Empty array
    EmptyArray,
    /// Empty JSON object
    EmptyObject,
}

impl DefaultValue {
    /// Convert to SQL representation
    pub fn to_sql(&self, db: imortal_core::DatabaseType) -> String {
        match self {
            DefaultValue::Null => "NULL".to_string(),
            DefaultValue::Bool(v) => match db {
                imortal_core::DatabaseType::MySQL => if *v { "1" } else { "0" }.to_string(),
                _ => v.to_string().to_uppercase(),
            },
            DefaultValue::Int(v) => v.to_string(),
            DefaultValue::Float(v) => v.to_string(),
            DefaultValue::String(v) => format!("'{}'", v.replace('\'', "''")),
            DefaultValue::Now => match db {
                imortal_core::DatabaseType::SQLite => "CURRENT_TIMESTAMP".to_string(),
                _ => "NOW()".to_string(),
            },
            DefaultValue::Uuid => match db {
                imortal_core::DatabaseType::PostgreSQL => "gen_random_uuid()".to_string(),
                imortal_core::DatabaseType::MySQL => "(UUID())".to_string(),
                imortal_core::DatabaseType::SQLite => "(lower(hex(randomblob(16))))".to_string(),
            },
            DefaultValue::Expression(expr) => expr.clone(),
            DefaultValue::EmptyArray => match db {
                imortal_core::DatabaseType::PostgreSQL => "'{}'".to_string(),
                _ => "'[]'".to_string(),
            },
            DefaultValue::EmptyObject => "'{}'".to_string(),
        }
    }

    /// Convert to Rust representation
    pub fn to_rust(&self) -> String {
        match self {
            DefaultValue::Null => "None".to_string(),
            DefaultValue::Bool(v) => v.to_string(),
            DefaultValue::Int(v) => v.to_string(),
            DefaultValue::Float(v) => format!("{:.1}", v),
            DefaultValue::String(v) => format!("\"{}\".to_string()", v),
            DefaultValue::Now => "chrono::Utc::now()".to_string(),
            DefaultValue::Uuid => "uuid::Uuid::new_v4()".to_string(),
            DefaultValue::Expression(expr) => format!("/* {} */", expr),
            DefaultValue::EmptyArray => "vec![]".to_string(),
            DefaultValue::EmptyObject => "serde_json::json!({{}})".to_string(),
        }
    }
}

impl std::fmt::Display for DefaultValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultValue::Null => write!(f, "NULL"),
            DefaultValue::Bool(v) => write!(f, "{}", v),
            DefaultValue::Int(v) => write!(f, "{}", v),
            DefaultValue::Float(v) => write!(f, "{}", v),
            DefaultValue::String(v) => write!(f, "\"{}\"", v),
            DefaultValue::Now => write!(f, "NOW()"),
            DefaultValue::Uuid => write!(f, "UUID()"),
            DefaultValue::Expression(v) => write!(f, "{}", v),
            DefaultValue::EmptyArray => write!(f, "[]"),
            DefaultValue::EmptyObject => write!(f, "{{}}"),
        }
    }
}

// ============================================================================
// ForeignKeyRef
// ============================================================================

/// Foreign key reference details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    /// ID of the referenced entity
    pub entity_id: Uuid,

    /// Name of the referenced entity
    pub entity_name: String,

    /// Name of the referenced field (usually "id")
    pub field_name: String,

    /// Action on delete
    pub on_delete: ReferentialAction,

    /// Action on update
    pub on_update: ReferentialAction,
}

impl ForeignKeyRef {
    /// Create a new foreign key reference
    pub fn new(entity_name: impl Into<String>) -> Self {
        Self {
            entity_id: Uuid::nil(),
            entity_name: entity_name.into(),
            field_name: "id".to_string(),
            on_delete: ReferentialAction::Restrict,
            on_update: ReferentialAction::Cascade,
        }
    }

    /// Set the entity ID
    pub fn with_entity_id(mut self, id: Uuid) -> Self {
        self.entity_id = id;
        self
    }

    /// Set the field name
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field_name = field.into();
        self
    }

    /// Set the on delete action
    pub fn on_delete(mut self, action: ReferentialAction) -> Self {
        self.on_delete = action;
        self
    }

    /// Set the on update action
    pub fn on_update(mut self, action: ReferentialAction) -> Self {
        self.on_update = action;
        self
    }

    /// Generate SQL for the foreign key constraint
    pub fn to_sql(&self, field_name: &str, table_name: &str) -> String {
        format!(
            "CONSTRAINT fk_{table}_{field} FOREIGN KEY ({field}) REFERENCES {ref_table}({ref_field}) ON DELETE {on_delete} ON UPDATE {on_update}",
            table = table_name,
            field = field_name,
            ref_table = to_snake_case(&self.entity_name),
            ref_field = self.field_name,
            on_delete = self.on_delete.to_sql(),
            on_update = self.on_update.to_sql(),
        )
    }
}

// ============================================================================
// UiHints
// ============================================================================

/// UI hints for rendering fields in forms and tables
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHints {
    /// Display label (falls back to field name if None)
    pub label: Option<String>,

    /// Placeholder text for input
    pub placeholder: Option<String>,

    /// Help text / description shown below input
    pub help_text: Option<String>,

    /// Widget type for rendering
    pub widget: Option<WidgetType>,

    /// CSS class names for styling
    pub css_class: Option<String>,

    /// Icon name (for icon libraries)
    pub icon: Option<String>,

    /// Minimum width in pixels
    pub min_width: Option<u32>,

    /// Maximum width in pixels
    pub max_width: Option<u32>,

    /// Number of rows (for textarea)
    pub rows: Option<u32>,

    /// Whether to show in list/table views
    pub show_in_list: bool,

    /// Whether to show in detail views
    pub show_in_detail: bool,

    /// Whether to show in create forms
    pub show_in_create: bool,

    /// Whether to show in edit forms
    pub show_in_edit: bool,

    /// Whether the field is searchable
    pub searchable: bool,

    /// Whether the field is sortable in tables
    pub sortable: bool,

    /// Whether the field is filterable in tables
    pub filterable: bool,

    /// Format string for display (e.g., date format)
    pub format: Option<String>,

    /// Prefix text (e.g., "$" for currency)
    pub prefix: Option<String>,

    /// Suffix text (e.g., "kg" for weight)
    pub suffix: Option<String>,
}

impl UiHints {
    /// Create new UI hints with default values
    pub fn new() -> Self {
        Self {
            show_in_list: true,
            show_in_detail: true,
            show_in_create: true,
            show_in_edit: true,
            searchable: false,
            sortable: true,
            filterable: false,
            ..Default::default()
        }
    }

    /// Create UI hints for a primary key field
    pub fn for_primary_key() -> Self {
        Self {
            label: Some("ID".to_string()),
            show_in_list: false,
            show_in_create: false,
            show_in_edit: false,
            sortable: false,
            ..Self::new()
        }
    }

    /// Create UI hints for a timestamp field
    pub fn for_timestamp() -> Self {
        Self {
            widget: Some(WidgetType::DateTime),
            show_in_create: false,
            show_in_edit: false,
            format: Some("%Y-%m-%d %H:%M".to_string()),
            ..Self::new()
        }
    }

    /// Create UI hints for a password field
    pub fn for_password() -> Self {
        Self {
            label: Some("Password".to_string()),
            widget: Some(WidgetType::Password),
            show_in_list: false,
            show_in_detail: false,
            ..Self::new()
        }
    }

    /// Set the label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the placeholder
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set the widget type
    pub fn with_widget(mut self, widget: WidgetType) -> Self {
        self.widget = Some(widget);
        self
    }

    /// Set the help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help_text = Some(help.into());
        self
    }
}

// ============================================================================
// WidgetType
// ============================================================================

/// Widget types for form rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    /// Single-line text input
    Text,
    /// Multi-line text area
    TextArea,
    /// Numeric input
    Number,
    /// Email input with validation
    Email,
    /// Password input (masked)
    Password,
    /// URL input with validation
    Url,
    /// Phone number input
    Phone,
    /// Dropdown select
    Select,
    /// Multi-select
    MultiSelect,
    /// Checkbox
    Checkbox,
    /// Radio buttons
    Radio,
    /// Toggle switch
    Toggle,
    /// Date picker
    Date,
    /// Time picker
    Time,
    /// DateTime picker
    DateTime,
    /// Color picker
    Color,
    /// File upload
    File,
    /// Image upload
    Image,
    /// Rich text editor (WYSIWYG)
    RichText,
    /// Markdown editor
    Markdown,
    /// Code editor
    Code,
    /// JSON editor
    Json,
    /// Slider
    Slider,
    /// Range slider (min-max)
    Range,
    /// Rating (stars)
    Rating,
    /// Hidden field
    Hidden,
    /// Read-only display
    Readonly,
}

impl WidgetType {
    /// Get the default widget for a data type
    pub fn for_data_type(data_type: &DataType) -> Self {
        match data_type {
            DataType::String => WidgetType::Text,
            DataType::Text => WidgetType::TextArea,
            DataType::Int32 | DataType::Int64 => WidgetType::Number,
            DataType::Float32 | DataType::Float64 => WidgetType::Number,
            DataType::Bool => WidgetType::Checkbox,
            DataType::Uuid => WidgetType::Text,
            DataType::DateTime => WidgetType::DateTime,
            DataType::Date => WidgetType::Date,
            DataType::Time => WidgetType::Time,
            DataType::Bytes => WidgetType::File,
            DataType::Json => WidgetType::Json,
            DataType::Optional(inner) => WidgetType::for_data_type(inner),
            DataType::Array(_) => WidgetType::MultiSelect,
            DataType::Reference { .. } => WidgetType::Select,
            DataType::Enum { .. } => WidgetType::Select,
        }
    }

    /// Get the HTML input type attribute
    pub fn html_input_type(&self) -> &'static str {
        match self {
            WidgetType::Text | WidgetType::Readonly => "text",
            WidgetType::TextArea => "textarea",
            WidgetType::Number | WidgetType::Slider | WidgetType::Range => "number",
            WidgetType::Email => "email",
            WidgetType::Password => "password",
            WidgetType::Url => "url",
            WidgetType::Phone => "tel",
            WidgetType::Checkbox | WidgetType::Toggle => "checkbox",
            WidgetType::Radio => "radio",
            WidgetType::Date => "date",
            WidgetType::Time => "time",
            WidgetType::DateTime => "datetime-local",
            WidgetType::Color => "color",
            WidgetType::File | WidgetType::Image => "file",
            WidgetType::Hidden => "hidden",
            _ => "text",
        }
    }
}

impl Default for WidgetType {
    fn default() -> Self {
        WidgetType::Text
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a string to snake_case
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

/// Convert a string to Title Case
fn to_title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if a string is a valid Rust identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // First character must be letter or underscore
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_new() {
        let field = Field::new("email", DataType::String);
        assert_eq!(field.name, "email");
        assert_eq!(field.column_name, "email");
        assert_eq!(field.data_type, DataType::String);
        assert!(!field.required);
        assert!(!field.unique);
    }

    #[test]
    fn test_field_builder() {
        let field = Field::new("email", DataType::String)
            .required()
            .unique()
            .with_label("Email Address")
            .with_placeholder("user@example.com");

        assert!(field.required);
        assert!(field.unique);
        assert!(field.indexed); // unique implies indexed
        assert_eq!(field.ui_hints.label, Some("Email Address".to_string()));
        assert_eq!(
            field.ui_hints.placeholder,
            Some("user@example.com".to_string())
        );
    }

    #[test]
    fn test_primary_key_field() {
        let field = Field::primary_key();
        assert_eq!(field.name, "id");
        assert!(field.is_primary_key);
        assert!(field.required);
        assert!(field.readonly);
    }

    #[test]
    fn test_foreign_key_field() {
        let field = Field::foreign_key("user_id", "User", "id");
        assert_eq!(field.name, "user_id");
        assert!(field.is_foreign_key);
        assert!(field.indexed);
        assert!(field.foreign_key_ref.is_some());

        let fk_ref = field.foreign_key_ref.unwrap();
        assert_eq!(fk_ref.entity_name, "User");
        assert_eq!(fk_ref.field_name, "id");
    }

    #[test]
    fn test_field_validation() {
        let valid_field = Field::new("email", DataType::String);
        assert!(valid_field.validate().is_ok());

        let mut invalid_field = Field::new("", DataType::String);
        assert!(invalid_field.validate().is_err());

        invalid_field.name = "valid_name".to_string();
        invalid_field.is_foreign_key = true;
        assert!(invalid_field.validate().is_err()); // FK without reference
    }

    #[test]
    fn test_default_value_sql() {
        use imortal_core::DatabaseType;

        assert_eq!(DefaultValue::Null.to_sql(DatabaseType::PostgreSQL), "NULL");
        assert_eq!(
            DefaultValue::Bool(true).to_sql(DatabaseType::PostgreSQL),
            "TRUE"
        );
        assert_eq!(DefaultValue::Bool(true).to_sql(DatabaseType::MySQL), "1");
        assert_eq!(DefaultValue::Int(42).to_sql(DatabaseType::PostgreSQL), "42");
        assert_eq!(
            DefaultValue::String("test".to_string()).to_sql(DatabaseType::PostgreSQL),
            "'test'"
        );
        assert_eq!(DefaultValue::Now.to_sql(DatabaseType::PostgreSQL), "NOW()");
        assert_eq!(
            DefaultValue::Now.to_sql(DatabaseType::SQLite),
            "CURRENT_TIMESTAMP"
        );
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("userId"), "user_id");
        assert_eq!(to_snake_case("UserID"), "user_id");
        assert_eq!(to_snake_case("createdAt"), "created_at");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("user_id"), "User Id");
        assert_eq!(to_title_case("created_at"), "Created At");
        assert_eq!(to_title_case("email"), "Email");
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("user_id"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("User123"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123start"));
        assert!(!is_valid_identifier("with-dash"));
    }

    #[test]
    fn test_widget_type_for_data_type() {
        assert_eq!(
            WidgetType::for_data_type(&DataType::String),
            WidgetType::Text
        );
        assert_eq!(
            WidgetType::for_data_type(&DataType::Text),
            WidgetType::TextArea
        );
        assert_eq!(
            WidgetType::for_data_type(&DataType::Bool),
            WidgetType::Checkbox
        );
        assert_eq!(
            WidgetType::for_data_type(&DataType::DateTime),
            WidgetType::DateTime
        );
    }
}
