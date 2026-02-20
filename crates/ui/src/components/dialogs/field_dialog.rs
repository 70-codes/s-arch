//! # Field Dialog Component
//!
//! Dialog for creating and editing fields within entities in the Immortal Engine.
//!
//! ## Features
//!
//! - Create new fields with name, data type, and constraints
//! - Edit existing fields
//! - Configure field options (required, unique, indexed)
//! - Set default values
//! - Configure validations (min/max length, patterns, etc.)
//! - Foreign key configuration
//! - UI hints for form rendering
//!

use dioxus::prelude::*;
use imortal_core::types::{DataType, EntityId, FieldId, ReferentialAction, Validation};
use imortal_ir::field::{DefaultValue, Field, ForeignKeyRef, UiHints, WidgetType};

use crate::components::inputs::{NumberInput, Select, SelectOption, TextArea, TextInput, Toggle};
use crate::state::{APP_STATE, StatusLevel};

// ============================================================================
// Types
// ============================================================================

/// Mode for the field dialog
#[derive(Debug, Clone, PartialEq)]
pub enum FieldDialogMode {
    /// Create a new field
    Create,
    /// Edit an existing field
    Edit(FieldId),
}

/// Form state for field editing
#[derive(Debug, Clone)]
struct FieldFormState {
    // Basic properties
    name: String,
    column_name: String,
    description: String,

    // Data type
    data_type: DataType,
    is_optional: bool,
    is_array: bool,

    // Constraints
    required: bool,
    unique: bool,
    indexed: bool,

    // Default value
    has_default: bool,
    default_type: DefaultValueType,
    default_string: String,
    default_number: f64,
    default_bool: bool,

    // Validations
    validations: Vec<ValidationConfig>,

    // Foreign key
    is_foreign_key: bool,
    fk_entity_name: String,
    fk_field_name: String,
    fk_on_delete: ReferentialAction,
    fk_on_update: ReferentialAction,

    // UI hints
    label: String,
    placeholder: String,
    help_text: String,
    widget_type: WidgetType,
    hidden: bool,
    readonly: bool,
    secret: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum DefaultValueType {
    Null,
    String,
    Number,
    Bool,
    Now,
    Uuid,
    Expression,
}

// ============================================================================
// Field Templates — presets for common field types
// ============================================================================

/// Pre-configured field templates for common use cases.
///
/// Selecting a template pre-fills the field dialog with sensible defaults,
/// saving the user from configuring every setting manually.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldTemplate {
    /// Plain text field (no preset — blank form)
    Custom,
    /// Password field: secret, hashed, min-length validation
    Password,
    /// Email field: unique, indexed, email validation
    Email,
    /// Phone number field: phone validation
    Phone,
    /// URL/website field: URL validation
    Url,
    /// Username field: unique, indexed, pattern validation
    Username,
    /// Status/enum field: string with common values
    Status,
    /// Slug field: unique, indexed, URL-safe pattern
    Slug,
    /// Boolean toggle (e.g. is_active, is_verified)
    BooleanFlag,
    /// Integer counter (e.g. view_count, sort_order)
    Counter,
    /// Price / monetary amount (float with 2 decimal places)
    Price,
    /// Long text / content field
    RichText,
    /// JSON data field
    JsonData,
    /// Foreign key reference to another entity
    ForeignKey,
}

impl FieldTemplate {
    /// All available templates in display order.
    fn all() -> &'static [FieldTemplate] {
        &[
            FieldTemplate::Custom,
            FieldTemplate::Password,
            FieldTemplate::Email,
            FieldTemplate::Username,
            FieldTemplate::Phone,
            FieldTemplate::Url,
            FieldTemplate::Status,
            FieldTemplate::Slug,
            FieldTemplate::BooleanFlag,
            FieldTemplate::Counter,
            FieldTemplate::Price,
            FieldTemplate::RichText,
            FieldTemplate::JsonData,
            FieldTemplate::ForeignKey,
        ]
    }

    /// Display name for the template.
    fn label(&self) -> &'static str {
        match self {
            FieldTemplate::Custom => "Custom",
            FieldTemplate::Password => "Password",
            FieldTemplate::Email => "Email",
            FieldTemplate::Phone => "Phone",
            FieldTemplate::Url => "URL",
            FieldTemplate::Username => "Username",
            FieldTemplate::Status => "Status",
            FieldTemplate::Slug => "Slug",
            FieldTemplate::BooleanFlag => "Boolean",
            FieldTemplate::Counter => "Counter",
            FieldTemplate::Price => "Price",
            FieldTemplate::RichText => "Rich Text",
            FieldTemplate::JsonData => "JSON",
            FieldTemplate::ForeignKey => "Foreign Key",
        }
    }

    /// Icon/emoji for the template.
    fn icon(&self) -> &'static str {
        match self {
            FieldTemplate::Custom => "✏️",
            FieldTemplate::Password => "🔒",
            FieldTemplate::Email => "📧",
            FieldTemplate::Phone => "📱",
            FieldTemplate::Url => "🔗",
            FieldTemplate::Username => "👤",
            FieldTemplate::Status => "🏷️",
            FieldTemplate::Slug => "🔤",
            FieldTemplate::BooleanFlag => "✅",
            FieldTemplate::Counter => "🔢",
            FieldTemplate::Price => "💰",
            FieldTemplate::RichText => "📝",
            FieldTemplate::JsonData => "{ }",
            FieldTemplate::ForeignKey => "🔑",
        }
    }

    /// Short description of what the template configures.
    fn description(&self) -> &'static str {
        match self {
            FieldTemplate::Custom => "Blank field — configure everything manually",
            FieldTemplate::Password => "Hashed password (secret, min 8 chars, bcrypt)",
            FieldTemplate::Email => "Email address (unique, indexed, validated)",
            FieldTemplate::Phone => "Phone number with validation",
            FieldTemplate::Url => "URL/website with validation",
            FieldTemplate::Username => "Unique username (indexed, alphanumeric)",
            FieldTemplate::Status => "Status string (e.g. active, inactive, pending)",
            FieldTemplate::Slug => "URL-safe slug (unique, indexed, lowercase)",
            FieldTemplate::BooleanFlag => "True/false toggle (e.g. is_active)",
            FieldTemplate::Counter => "Integer counter (e.g. view_count)",
            FieldTemplate::Price => "Monetary amount (float64)",
            FieldTemplate::RichText => "Long-form text content",
            FieldTemplate::JsonData => "Arbitrary JSON data",
            FieldTemplate::ForeignKey => "Reference to another entity",
        }
    }

    /// Apply this template to a FieldFormState, pre-filling values.
    fn apply(&self) -> FieldFormState {
        match self {
            FieldTemplate::Custom => FieldFormState::default(),

            FieldTemplate::Password => FieldFormState {
                name: "password_hash".to_string(),
                description: "Hashed user password (bcrypt). Never stored in plain text."
                    .to_string(),
                data_type: DataType::String,
                required: true,
                secret: true,
                hidden: true,
                validations: vec![
                    ValidationConfig::new(ValidationType::MinLength).with_int(8),
                    ValidationConfig::new(ValidationType::MaxLength).with_int(128),
                ],
                widget_type: WidgetType::Password,
                help_text: "Minimum 8 characters. Will be hashed with bcrypt before storage."
                    .to_string(),
                ..Default::default()
            },

            FieldTemplate::Email => FieldFormState {
                name: "email".to_string(),
                description: "User email address".to_string(),
                data_type: DataType::String,
                required: true,
                unique: true,
                indexed: true,
                validations: vec![
                    ValidationConfig::new(ValidationType::Email),
                    ValidationConfig::new(ValidationType::MaxLength).with_int(255),
                ],
                widget_type: WidgetType::Email,
                placeholder: "user@example.com".to_string(),
                ..Default::default()
            },

            FieldTemplate::Phone => FieldFormState {
                name: "phone".to_string(),
                description: "Phone number".to_string(),
                data_type: DataType::String,
                required: false,
                validations: vec![ValidationConfig::new(ValidationType::Phone)],
                placeholder: "+1 234 567 8900".to_string(),
                ..Default::default()
            },

            FieldTemplate::Url => FieldFormState {
                name: "website".to_string(),
                description: "Website URL".to_string(),
                data_type: DataType::String,
                required: false,
                validations: vec![ValidationConfig::new(ValidationType::Url)],
                placeholder: "https://example.com".to_string(),
                ..Default::default()
            },

            FieldTemplate::Username => FieldFormState {
                name: "username".to_string(),
                description: "Unique username for identification".to_string(),
                data_type: DataType::String,
                required: true,
                unique: true,
                indexed: true,
                validations: vec![
                    ValidationConfig::new(ValidationType::MinLength).with_int(3),
                    ValidationConfig::new(ValidationType::MaxLength).with_int(50),
                    ValidationConfig::new(ValidationType::Pattern)
                        .with_string(r"^[a-zA-Z0-9_-]+$")
                        .with_message(
                            "Username can only contain letters, numbers, underscores, and hyphens",
                        ),
                ],
                placeholder: "john_doe".to_string(),
                ..Default::default()
            },

            FieldTemplate::Status => FieldFormState {
                name: "status".to_string(),
                description: "Current status".to_string(),
                data_type: DataType::String,
                required: true,
                indexed: true,
                has_default: true,
                default_type: DefaultValueType::String,
                default_string: "active".to_string(),
                help_text: "Common values: active, inactive, pending, suspended".to_string(),
                ..Default::default()
            },

            FieldTemplate::Slug => FieldFormState {
                name: "slug".to_string(),
                description: "URL-friendly identifier".to_string(),
                data_type: DataType::String,
                required: true,
                unique: true,
                indexed: true,
                validations: vec![
                    ValidationConfig::new(ValidationType::Pattern)
                        .with_string(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
                        .with_message("Slug must be lowercase with hyphens only"),
                    ValidationConfig::new(ValidationType::MaxLength).with_int(200),
                ],
                placeholder: "my-blog-post".to_string(),
                ..Default::default()
            },

            FieldTemplate::BooleanFlag => FieldFormState {
                name: "is_active".to_string(),
                description: "Whether this record is active".to_string(),
                data_type: DataType::Bool,
                required: true,
                has_default: true,
                default_type: DefaultValueType::Bool,
                default_bool: true,
                widget_type: WidgetType::Checkbox,
                ..Default::default()
            },

            FieldTemplate::Counter => FieldFormState {
                name: "count".to_string(),
                description: "Integer counter".to_string(),
                data_type: DataType::Int32,
                required: true,
                has_default: true,
                default_type: DefaultValueType::Number,
                default_number: 0.0,
                widget_type: WidgetType::Number,
                ..Default::default()
            },

            FieldTemplate::Price => FieldFormState {
                name: "price".to_string(),
                description: "Monetary amount".to_string(),
                data_type: DataType::Float64,
                required: true,
                validations: vec![ValidationConfig::new(ValidationType::Min).with_float(0.0)],
                has_default: true,
                default_type: DefaultValueType::Number,
                default_number: 0.0,
                widget_type: WidgetType::Number,
                placeholder: "0.00".to_string(),
                ..Default::default()
            },

            FieldTemplate::RichText => FieldFormState {
                name: "content".to_string(),
                description: "Long-form text content".to_string(),
                data_type: DataType::Text,
                required: false,
                widget_type: WidgetType::RichText,
                ..Default::default()
            },

            FieldTemplate::JsonData => FieldFormState {
                name: "metadata".to_string(),
                description: "Arbitrary JSON data".to_string(),
                data_type: DataType::Json,
                required: false,
                has_default: true,
                default_type: DefaultValueType::Expression,
                default_string: "'{}'".to_string(),
                ..Default::default()
            },

            FieldTemplate::ForeignKey => FieldFormState {
                name: "ref_id".to_string(),
                description: "Reference to another entity".to_string(),
                data_type: DataType::Uuid,
                required: true,
                indexed: true,
                is_foreign_key: true,
                fk_field_name: "id".to_string(),
                ..Default::default()
            },
        }
    }
}

impl ValidationConfig {
    /// Builder: set integer value (for MinLength, MaxLength)
    fn with_int(mut self, val: i64) -> Self {
        self.value_int = val;
        self
    }

    /// Builder: set float value (for Min, Max)
    fn with_float(mut self, val: f64) -> Self {
        self.value_float = val;
        self
    }

    /// Builder: set string value (for Pattern)
    fn with_string(mut self, val: &str) -> Self {
        self.value_string = val.to_string();
        self
    }

    /// Builder: set message (for Pattern)
    fn with_message(mut self, msg: &str) -> Self {
        self.message = msg.to_string();
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ValidationConfig {
    validation_type: ValidationType,
    value_int: i64,
    value_float: f64,
    value_string: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq)]
enum ValidationType {
    Required,
    MinLength,
    MaxLength,
    Min,
    Max,
    Pattern,
    Email,
    Url,
    Uuid,
    Phone,
}

impl Default for FieldFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            column_name: String::new(),
            description: String::new(),
            data_type: DataType::String,
            is_optional: false,
            is_array: false,
            required: false,
            unique: false,
            indexed: false,
            has_default: false,
            default_type: DefaultValueType::Null,
            default_string: String::new(),
            default_number: 0.0,
            default_bool: false,
            validations: Vec::new(),
            is_foreign_key: false,
            fk_entity_name: String::new(),
            fk_field_name: "id".to_string(),
            fk_on_delete: ReferentialAction::Cascade,
            fk_on_update: ReferentialAction::Cascade,
            label: String::new(),
            placeholder: String::new(),
            help_text: String::new(),
            widget_type: WidgetType::Text,
            hidden: false,
            readonly: false,
            secret: false,
        }
    }
}

impl FieldFormState {
    /// Create form state from an existing field
    fn from_field(field: &Field) -> Self {
        // Extract base data type (unwrap Optional/Array if needed)
        let (base_type, is_optional, is_array) = match &field.data_type {
            DataType::Optional(inner) => match inner.as_ref() {
                DataType::Array(arr_inner) => (arr_inner.as_ref().clone(), true, true),
                other => (other.clone(), true, false),
            },
            DataType::Array(inner) => (inner.as_ref().clone(), false, true),
            other => (other.clone(), false, false),
        };

        // Parse default value
        let (has_default, default_type, default_string, default_number, default_bool) =
            if let Some(def) = &field.default_value {
                match def {
                    DefaultValue::Null => (true, DefaultValueType::Null, String::new(), 0.0, false),
                    DefaultValue::String(s) => {
                        (true, DefaultValueType::String, s.clone(), 0.0, false)
                    }
                    DefaultValue::Int(i) => (
                        true,
                        DefaultValueType::Number,
                        String::new(),
                        *i as f64,
                        false,
                    ),
                    DefaultValue::Float(f) => {
                        (true, DefaultValueType::Number, String::new(), *f, false)
                    }
                    DefaultValue::Bool(b) => (true, DefaultValueType::Bool, String::new(), 0.0, *b),
                    DefaultValue::Now => (true, DefaultValueType::Now, String::new(), 0.0, false),
                    DefaultValue::Uuid => (true, DefaultValueType::Uuid, String::new(), 0.0, false),
                    DefaultValue::Expression(e) => {
                        (true, DefaultValueType::Expression, e.clone(), 0.0, false)
                    }
                    DefaultValue::EmptyArray | DefaultValue::EmptyObject => {
                        (true, DefaultValueType::Null, String::new(), 0.0, false)
                    }
                }
            } else {
                (false, DefaultValueType::Null, String::new(), 0.0, false)
            };

        // Parse validations
        let validations = field
            .validations
            .iter()
            .filter_map(|v| ValidationConfig::from_validation(v))
            .collect();

        // Parse foreign key
        let (is_fk, fk_entity_name, fk_field_name, fk_on_delete, fk_on_update) =
            if let Some(fk_ref) = &field.foreign_key_ref {
                (
                    true,
                    fk_ref.entity_name.clone(),
                    fk_ref.field_name.clone(),
                    fk_ref.on_delete.clone(),
                    fk_ref.on_update.clone(),
                )
            } else {
                (
                    false,
                    String::new(),
                    "id".to_string(),
                    ReferentialAction::Cascade,
                    ReferentialAction::Cascade,
                )
            };

        Self {
            name: field.name.clone(),
            column_name: field.column_name.clone(),
            description: field.description.clone().unwrap_or_default(),
            data_type: base_type,
            is_optional,
            is_array,
            required: field.required,
            unique: field.unique,
            indexed: field.indexed,
            has_default,
            default_type,
            default_string,
            default_number,
            default_bool,
            validations,
            is_foreign_key: field.is_foreign_key,
            fk_entity_name,
            fk_field_name,
            fk_on_delete,
            fk_on_update,
            label: field.ui_hints.label.clone().unwrap_or_default(),
            placeholder: field.ui_hints.placeholder.clone().unwrap_or_default(),
            help_text: field.ui_hints.help_text.clone().unwrap_or_default(),
            widget_type: field.ui_hints.widget.clone().unwrap_or_default(),
            hidden: field.hidden,
            readonly: field.readonly,
            secret: field.secret,
        }
    }

    /// Build the final data type with Optional/Array wrappers
    fn build_data_type(&self) -> DataType {
        let mut dt = self.data_type.clone();

        if self.is_array {
            dt = DataType::Array(Box::new(dt));
        }

        if self.is_optional {
            dt = DataType::Optional(Box::new(dt));
        }

        dt
    }

    /// Build the default value
    fn build_default_value(&self) -> Option<DefaultValue> {
        if !self.has_default {
            return None;
        }

        Some(match self.default_type {
            DefaultValueType::Null => DefaultValue::Null,
            DefaultValueType::String => DefaultValue::String(self.default_string.clone()),
            DefaultValueType::Number => {
                if self.default_number.fract() == 0.0 {
                    DefaultValue::Int(self.default_number as i64)
                } else {
                    DefaultValue::Float(self.default_number)
                }
            }
            DefaultValueType::Bool => DefaultValue::Bool(self.default_bool),
            DefaultValueType::Now => DefaultValue::Now,
            DefaultValueType::Uuid => DefaultValue::Uuid,
            DefaultValueType::Expression => DefaultValue::Expression(self.default_string.clone()),
        })
    }

    /// Build validations list
    fn build_validations(&self) -> Vec<Validation> {
        self.validations
            .iter()
            .filter_map(|v| v.to_validation())
            .collect()
    }

    /// Build foreign key reference
    fn build_foreign_key_ref(&self) -> Option<ForeignKeyRef> {
        if !self.is_foreign_key || self.fk_entity_name.is_empty() {
            return None;
        }

        Some(
            ForeignKeyRef::new(&self.fk_entity_name)
                .with_field(&self.fk_field_name)
                .on_delete(self.fk_on_delete.clone())
                .on_update(self.fk_on_update.clone()),
        )
    }

    /// Build UI hints
    fn build_ui_hints(&self) -> UiHints {
        let mut hints = UiHints::new();

        if !self.label.is_empty() {
            hints.label = Some(self.label.clone());
        }
        if !self.placeholder.is_empty() {
            hints.placeholder = Some(self.placeholder.clone());
        }
        if !self.help_text.is_empty() {
            hints.help_text = Some(self.help_text.clone());
        }
        hints.widget = Some(self.widget_type.clone());

        hints
    }

    /// Validate the form and return errors if any
    fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate name
        if self.name.trim().is_empty() {
            errors.push("Field name is required".to_string());
        } else if !is_valid_identifier(&self.name) {
            errors
                .push("Field name must be a valid identifier (snake_case recommended)".to_string());
        }

        // Validate column name if provided
        if !self.column_name.is_empty() && !is_valid_identifier(&self.column_name) {
            errors.push("Column name must be a valid SQL identifier".to_string());
        }

        // Validate foreign key config
        if self.is_foreign_key && self.fk_entity_name.is_empty() {
            errors.push("Foreign key entity name is required".to_string());
        }

        // Validate validations
        for (i, v) in self.validations.iter().enumerate() {
            if let Some(err) = v.validate() {
                errors.push(format!("Validation {}: {}", i + 1, err));
            }
        }

        errors
    }

    /// Check if the form is valid
    fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

impl ValidationConfig {
    fn new(validation_type: ValidationType) -> Self {
        Self {
            validation_type,
            value_int: 0,
            value_float: 0.0,
            value_string: String::new(),
            message: String::new(),
        }
    }

    fn from_validation(v: &Validation) -> Option<Self> {
        Some(match v {
            Validation::Required => Self::new(ValidationType::Required),
            Validation::MinLength(len) => {
                let mut config = Self::new(ValidationType::MinLength);
                config.value_int = *len as i64;
                config
            }
            Validation::MaxLength(len) => {
                let mut config = Self::new(ValidationType::MaxLength);
                config.value_int = *len as i64;
                config
            }
            Validation::Min(val) => {
                let mut config = Self::new(ValidationType::Min);
                config.value_float = *val;
                config
            }
            Validation::Max(val) => {
                let mut config = Self::new(ValidationType::Max);
                config.value_float = *val;
                config
            }
            Validation::Pattern { regex, message } => {
                let mut config = Self::new(ValidationType::Pattern);
                config.value_string = regex.clone();
                config.message = message.clone();
                config
            }
            Validation::Email => Self::new(ValidationType::Email),
            Validation::Url => Self::new(ValidationType::Url),
            Validation::Uuid => Self::new(ValidationType::Uuid),
            Validation::Phone => Self::new(ValidationType::Phone),
            _ => return None, // Skip unsupported validations
        })
    }

    fn to_validation(&self) -> Option<Validation> {
        Some(match self.validation_type {
            ValidationType::Required => Validation::Required,
            ValidationType::MinLength => Validation::MinLength(self.value_int as usize),
            ValidationType::MaxLength => Validation::MaxLength(self.value_int as usize),
            ValidationType::Min => Validation::Min(self.value_float),
            ValidationType::Max => Validation::Max(self.value_float),
            ValidationType::Pattern => Validation::Pattern {
                regex: self.value_string.clone(),
                message: self.message.clone(),
            },
            ValidationType::Email => Validation::Email,
            ValidationType::Url => Validation::Url,
            ValidationType::Uuid => Validation::Uuid,
            ValidationType::Phone => Validation::Phone,
        })
    }

    fn validate(&self) -> Option<String> {
        match self.validation_type {
            ValidationType::MinLength | ValidationType::MaxLength => {
                if self.value_int < 0 {
                    return Some("Length must be non-negative".to_string());
                }
            }
            ValidationType::Pattern => {
                if self.value_string.is_empty() {
                    return Some("Pattern regex is required".to_string());
                }
            }
            _ => {}
        }
        None
    }

    fn display_name(&self) -> &'static str {
        match self.validation_type {
            ValidationType::Required => "Required",
            ValidationType::MinLength => "Min Length",
            ValidationType::MaxLength => "Max Length",
            ValidationType::Min => "Min Value",
            ValidationType::Max => "Max Value",
            ValidationType::Pattern => "Pattern (Regex)",
            ValidationType::Email => "Email",
            ValidationType::Url => "URL",
            ValidationType::Uuid => "UUID",
            ValidationType::Phone => "Phone",
        }
    }
}

// ============================================================================
// Component Props
// ============================================================================

#[derive(Props, Clone, PartialEq)]
pub struct FieldDialogProps {
    /// Entity ID that this field belongs to
    pub entity_id: EntityId,

    /// Dialog mode (create or edit)
    pub mode: FieldDialogMode,

    /// Optional callback when field is created/updated
    #[props(default)]
    pub on_save: EventHandler<FieldId>,

    /// Optional callback when dialog is cancelled
    #[props(default)]
    pub on_cancel: EventHandler<()>,
}

// ============================================================================
// Main Component
// ============================================================================

/// Field creation and editing dialog
#[component]
pub fn FieldDialog(props: FieldDialogProps) -> Element {
    // Initialize form state based on mode
    let initial_state = match &props.mode {
        FieldDialogMode::Create => FieldFormState::default(),
        FieldDialogMode::Edit(field_id) => {
            let state = APP_STATE.read();
            state
                .project
                .as_ref()
                .and_then(|p| p.entities.get(&props.entity_id))
                .and_then(|e| e.get_field(*field_id))
                .map(FieldFormState::from_field)
                .unwrap_or_default()
        }
    };

    let mut form_state = use_signal(|| initial_state);
    let mut errors = use_signal(Vec::<String>::new);
    let mut is_saving = use_signal(|| false);
    let mut active_tab = use_signal(|| "basic");
    let mut selected_template = use_signal(|| FieldTemplate::Custom);
    let is_create_mode = matches!(props.mode, FieldDialogMode::Create);

    // Get list of entities for foreign key dropdown
    let entity_options = use_memo(move || {
        let state = APP_STATE.read();
        state
            .project
            .as_ref()
            .map(|p| {
                p.entities
                    .values()
                    .map(|e| SelectOption::new(&e.name, &e.name))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    // Auto-generate column name from field name
    let auto_column_name = use_memo(move || {
        let state = form_state.read();
        if state.column_name.is_empty() {
            to_snake_case(&state.name)
        } else {
            state.column_name.clone()
        }
    });
    let mode_for_unique = props.mode.clone();
    let mode_for_save = props.mode.clone();
    let mode_for_title = props.mode.clone();
    let entity_id_for_unique = props.entity_id;
    let entity_id_for_save = props.entity_id;

    // Check if name is unique within the entity (for new fields)
    let name_is_unique = use_memo(move || {
        let state = form_state.read();
        let app_state = APP_STATE.read();

        if let Some(project) = &app_state.project {
            if let Some(entity) = project.entities.get(&entity_id_for_unique) {
                let edit_id = if let FieldDialogMode::Edit(id) = &mode_for_unique {
                    Some(*id)
                } else {
                    None
                };

                // Check if any other field has the same name
                return !entity.fields.iter().any(|f| {
                    f.name.to_lowercase() == state.name.to_lowercase() && Some(f.id) != edit_id
                });
            }
        }
        true
    });

    // Handle form submission
    let mut handle_save = move |_| {
        // Validate form
        let validation_errors = form_state.read().validate();
        if !validation_errors.is_empty() {
            errors.set(validation_errors);
            return;
        }

        // Check for unique name
        if !*name_is_unique.read() {
            errors.set(vec![
                "A field with this name already exists in this entity".to_string(),
            ]);
            return;
        }

        is_saving.set(true);
        errors.set(Vec::new());

        let state = form_state.read();
        let entity_id = entity_id_for_save;

        let field_id = match &mode_for_save {
            FieldDialogMode::Create => {
                // Create new field
                let mut field = Field::new(&state.name, state.build_data_type());

                // Set column name
                field.column_name = if state.column_name.is_empty() {
                    to_snake_case(&state.name)
                } else {
                    state.column_name.clone()
                };

                // Set description
                if !state.description.is_empty() {
                    field.description = Some(state.description.clone());
                }

                // Set constraints
                field.required = state.required;
                field.unique = state.unique;
                field.indexed = state.indexed;

                // Set default value
                field.default_value = state.build_default_value();

                // Set validations
                field.validations = state.build_validations();

                // Set foreign key
                field.is_foreign_key = state.is_foreign_key;
                field.foreign_key_ref = state.build_foreign_key_ref();

                // Set UI hints
                field.ui_hints = state.build_ui_hints();
                field.hidden = state.hidden;
                field.readonly = state.readonly;
                field.secret = state.secret;

                let id = field.id;

                // Add to entity
                let mut app_state = APP_STATE.write();
                if let Some(project) = &mut app_state.project {
                    if let Some(entity) = project.entities.get_mut(&entity_id) {
                        entity.add_field(field);
                        entity.touch();
                    }
                }
                app_state.is_dirty = true;
                app_state.selection.field = Some((entity_id, id));
                app_state.ui.close_dialog();
                app_state.ui.set_status(
                    &format!("Created field '{}'", state.name),
                    StatusLevel::Success,
                );
                drop(app_state);

                // Save to history
                APP_STATE.write().save_to_history("Create field");

                id
            }
            FieldDialogMode::Edit(field_id) => {
                // Update existing field
                let mut app_state = APP_STATE.write();
                if let Some(project) = &mut app_state.project {
                    if let Some(entity) = project.entities.get_mut(&entity_id) {
                        if let Some(field) = entity.get_field_mut(*field_id) {
                            field.name = state.name.clone();
                            field.column_name = if state.column_name.is_empty() {
                                to_snake_case(&state.name)
                            } else {
                                state.column_name.clone()
                            };
                            field.description = if state.description.is_empty() {
                                None
                            } else {
                                Some(state.description.clone())
                            };
                            field.data_type = state.build_data_type();
                            field.required = state.required;
                            field.unique = state.unique;
                            field.indexed = state.indexed;
                            field.default_value = state.build_default_value();
                            field.validations = state.build_validations();
                            field.is_foreign_key = state.is_foreign_key;
                            field.foreign_key_ref = state.build_foreign_key_ref();
                            field.ui_hints = state.build_ui_hints();
                            field.hidden = state.hidden;
                            field.readonly = state.readonly;
                            field.secret = state.secret;
                        }
                        entity.touch();
                    }
                }
                app_state.is_dirty = true;
                app_state.ui.close_dialog();
                app_state.ui.set_status(
                    &format!("Updated field '{}'", state.name),
                    StatusLevel::Success,
                );
                drop(app_state);

                // Save to history
                APP_STATE.write().save_to_history("Update field");

                *field_id
            }
        };

        is_saving.set(false);
        props.on_save.call(field_id);
    };

    // Handle cancel
    let handle_cancel = move |_| {
        APP_STATE.write().ui.close_dialog();
        props.on_cancel.call(());
    };

    // Build data type options
    let data_type_options = get_data_type_options();
    let widget_type_options = get_widget_type_options();
    let referential_action_options = get_referential_action_options();
    let default_type_options = get_default_type_options();
    let validation_type_options = get_validation_type_options();

    // Determine dialog title
    let title = match &mode_for_title {
        FieldDialogMode::Create => "Create New Field",
        FieldDialogMode::Edit(_) => "Edit Field",
    };

    let save_button_text = match &mode_for_title {
        FieldDialogMode::Create => "Create Field",
        FieldDialogMode::Edit(_) => "Save Changes",
    };

    let form = form_state.read();
    let error_list = errors.read();
    let saving = *is_saving.read();
    let current_tab = *active_tab.read();

    rsx! {
        div {
            class: "field-dialog p-6 max-h-[85vh] overflow-hidden flex flex-col",
            style: "width: 600px;",

            // Header
            div {
                class: "flex items-center gap-3 mb-4",
                span { class: "text-2xl", "📝" }
                h2 { class: "text-xl font-bold", "{title}" }
            }

            // Template picker (only shown in Create mode)
            if is_create_mode {
                div {
                    class: "mb-4",

                    div {
                        class: "flex items-center gap-2 mb-2",
                        span { class: "text-xs font-semibold text-slate-400 uppercase tracking-wider", "Quick Templates" }
                        span { class: "text-xs text-slate-600", "— click to pre-fill" }
                    }

                    div {
                        class: "flex flex-wrap gap-1.5",

                        for template in FieldTemplate::all().iter() {
                            {
                                let tmpl = *template;
                                let is_selected = *selected_template.read() == tmpl;
                                rsx! {
                                    button {
                                        key: "{tmpl.label()}",
                                        r#type: "button",
                                        class: format!(
                                            "px-2.5 py-1.5 rounded-lg text-xs font-medium transition-all flex items-center gap-1.5 {}",
                                            if is_selected {
                                                "bg-indigo-600 text-white ring-2 ring-indigo-400/30"
                                            } else {
                                                "bg-slate-700 text-slate-300 hover:bg-slate-600 hover:text-white"
                                            }
                                        ),
                                        title: tmpl.description(),
                                        onclick: move |_| {
                                            selected_template.set(tmpl);
                                            // Apply template — pre-fill form with preset values
                                            let new_state = tmpl.apply();
                                            form_state.set(new_state);
                                            errors.set(Vec::new());
                                            // Jump to basic tab to see the changes
                                            active_tab.set("basic");
                                        },

                                        span { "{tmpl.icon()}" }
                                        "{tmpl.label()}"
                                    }
                                }
                            }
                        }
                    }

                    // Show description of selected template
                    if *selected_template.read() != FieldTemplate::Custom {
                        div {
                            class: "mt-2 px-3 py-2 bg-indigo-900/20 border border-indigo-700/30 rounded-lg text-xs text-indigo-300 flex items-center gap-2",

                            span { "💡" }
                            span { "{selected_template.read().description()}" }
                            span { class: "text-indigo-500 ml-1", "— you can customize all fields below" }
                        }
                    }
                }
            }

            // Error messages
            if !error_list.is_empty() {
                div {
                    class: "mb-4 p-3 bg-red-500/20 border border-red-500/50 rounded-lg",
                    ul {
                        class: "text-red-300 text-sm list-disc list-inside",
                        for error in error_list.iter() {
                            li { "{error}" }
                        }
                    }
                }
            }

            // Tabs
            div {
                class: "flex border-b border-slate-700 mb-4",

                TabButton {
                    label: "Basic",
                    active: current_tab == "basic",
                    onclick: move |_| active_tab.set("basic"),
                }
                TabButton {
                    label: "Constraints",
                    active: current_tab == "constraints",
                    onclick: move |_| active_tab.set("constraints"),
                }
                TabButton {
                    label: "Validations",
                    active: current_tab == "validations",
                    onclick: move |_| active_tab.set("validations"),
                }
                TabButton {
                    label: "Foreign Key",
                    active: current_tab == "fk",
                    onclick: move |_| active_tab.set("fk"),
                }
                TabButton {
                    label: "UI Hints",
                    active: current_tab == "ui",
                    onclick: move |_| active_tab.set("ui"),
                }
            }

            // Form content (scrollable)
            div {
                class: "flex-1 overflow-y-auto pr-2",

                form {
                    class: "space-y-4",
                    onsubmit: move |e| {
                        e.prevent_default();
                    },

                    // Basic Tab
                    div {
                        class: if current_tab == "basic" { "space-y-4" } else { "hidden" },

                        // Field name
                        TextInput {
                            value: form.name.clone(),
                            label: "Field Name",
                            placeholder: "e.g., email, created_at, user_id",
                            help_text: "Use snake_case for field names",
                            required: true,
                            error: if !name_is_unique.read().clone() {
                                Some("A field with this name already exists".to_string())
                            } else {
                                None
                            },
                            on_change: move |value: String| {
                                form_state.write().name = value;
                            },
                        }

                        // Column name
                        TextInput {
                            value: form.column_name.clone(),
                            label: "Column Name",
                            placeholder: auto_column_name.read().clone(),
                            help_text: "Database column name (auto-generated if empty)",
                            on_change: move |value: String| {
                                form_state.write().column_name = value;
                            },
                        }

                        // Data type
                        Select {
                            value: data_type_to_string(&form.data_type),
                            options: data_type_options.clone(),
                            label: "Data Type",
                            help_text: "The type of data this field stores",
                            on_change: move |value: String| {
                                form_state.write().data_type = string_to_data_type(&value);
                            },
                        }

                        // Type modifiers
                        div {
                            class: "grid grid-cols-2 gap-4",

                            Toggle {
                                checked: form.is_optional,
                                label: "Optional (Nullable)",
                                help_text: "Allow NULL values",
                                on_change: move |checked: bool| {
                                    form_state.write().is_optional = checked;
                                },
                            }

                            Toggle {
                                checked: form.is_array,
                                label: "Array",
                                help_text: "Multiple values (list)",
                                on_change: move |checked: bool| {
                                    form_state.write().is_array = checked;
                                },
                            }
                        }

                        // Description
                        TextArea {
                            value: form.description.clone(),
                            label: "Description",
                            placeholder: "Describe what this field represents...",
                            rows: 2,
                            on_change: move |value: String| {
                                form_state.write().description = value;
                            },
                        }
                    }

                    // Constraints Tab
                    div {
                        class: if current_tab == "constraints" { "space-y-4" } else { "hidden" },

                        div {
                            class: "grid grid-cols-2 gap-4",

                            Toggle {
                                checked: form.required,
                                label: "Required",
                                help_text: "NOT NULL constraint",
                                on_change: move |checked: bool| {
                                    form_state.write().required = checked;
                                },
                            }

                            Toggle {
                                checked: form.unique,
                                label: "Unique",
                                help_text: "UNIQUE constraint",
                                on_change: move |checked: bool| {
                                    form_state.write().unique = checked;
                                },
                            }

                            Toggle {
                                checked: form.indexed,
                                label: "Indexed",
                                help_text: "Create database index",
                                on_change: move |checked: bool| {
                                    form_state.write().indexed = checked;
                                },
                            }
                        }

                        // Default value section
                        div {
                            class: "pt-4 border-t border-slate-700 space-y-4",

                            h4 {
                                class: "text-sm font-medium text-slate-300",
                                "Default Value"
                            }

                            Toggle {
                                checked: form.has_default,
                                label: "Has Default Value",
                                on_change: move |checked: bool| {
                                    form_state.write().has_default = checked;
                                },
                            }

                            if form.has_default {
                                Select {
                                    value: default_type_to_string(&form.default_type),
                                    options: default_type_options.clone(),
                                    label: "Default Type",
                                    on_change: move |value: String| {
                                        form_state.write().default_type = string_to_default_type(&value);
                                    },
                                }

                                // Conditional default value input
                                match form.default_type {
                                    DefaultValueType::String | DefaultValueType::Expression => rsx! {
                                        TextInput {
                                            value: form.default_string.clone(),
                                            label: if matches!(form.default_type, DefaultValueType::Expression) { "Expression" } else { "Default String" },
                                            placeholder: if matches!(form.default_type, DefaultValueType::Expression) { "SQL expression..." } else { "Default value..." },
                                            on_change: move |value: String| {
                                                form_state.write().default_string = value;
                                            },
                                        }
                                    },
                                    DefaultValueType::Number => rsx! {
                                        NumberInput {
                                            value: form.default_number,
                                            label: "Default Number",
                                            on_change: move |value: f64| {
                                                form_state.write().default_number = value;
                                            },
                                        }
                                    },
                                    DefaultValueType::Bool => rsx! {
                                        Toggle {
                                            checked: form.default_bool,
                                            label: "Default Boolean Value",
                                            on_change: move |checked: bool| {
                                                form_state.write().default_bool = checked;
                                            },
                                        }
                                    },
                                    _ => rsx! {},
                                }
                            }
                        }
                    }

                    // Validations Tab
                    div {
                        class: if current_tab == "validations" { "space-y-4" } else { "hidden" },

                        div {
                            class: "flex items-center justify-between mb-2",
                            h4 {
                                class: "text-sm font-medium text-slate-300",
                                "Field Validations"
                            }
                            button {
                                r#type: "button",
                                class: "px-3 py-1 text-sm bg-indigo-600 hover:bg-indigo-700 rounded transition-colors",
                                onclick: move |_| {
                                    form_state.write().validations.push(ValidationConfig::new(ValidationType::Required));
                                },
                                "+ Add Validation"
                            }
                        }

                        if form.validations.is_empty() {
                            div {
                                class: "text-center py-8 text-slate-500",
                                p { "No validations configured" }
                                p { class: "text-sm", "Click \"Add Validation\" to add constraints" }
                            }
                        } else {
                            for (i, validation) in form.validations.iter().enumerate() {
                                ValidationRow {
                                    index: i,
                                    config: validation.clone(),
                                    options: validation_type_options.clone(),
                                    on_change: move |new_config: ValidationConfig| {
                                        form_state.write().validations[i] = new_config;
                                    },
                                    on_remove: move |_| {
                                        form_state.write().validations.remove(i);
                                    },
                                }
                            }
                        }
                    }

                    // Foreign Key Tab
                    div {
                        class: if current_tab == "fk" { "space-y-4" } else { "hidden" },

                        Toggle {
                            checked: form.is_foreign_key,
                            label: "Is Foreign Key",
                            help_text: "Reference another entity's field",
                            on_change: move |checked: bool| {
                                form_state.write().is_foreign_key = checked;
                            },
                        }

                        if form.is_foreign_key {
                            Select {
                                value: form.fk_entity_name.clone(),
                                options: entity_options.read().clone(),
                                label: "Referenced Entity",
                                placeholder: "Select entity...",
                                required: true,
                                on_change: move |value: String| {
                                    form_state.write().fk_entity_name = value;
                                },
                            }

                            TextInput {
                                value: form.fk_field_name.clone(),
                                label: "Referenced Field",
                                placeholder: "id",
                                help_text: "Usually 'id' (the primary key)",
                                on_change: move |value: String| {
                                    form_state.write().fk_field_name = value;
                                },
                            }

                            div {
                                class: "grid grid-cols-2 gap-4",

                                Select {
                                    value: referential_action_to_string(&form.fk_on_delete),
                                    options: referential_action_options.clone(),
                                    label: "On Delete",
                                    help_text: "Action when parent is deleted",
                                    on_change: move |value: String| {
                                        form_state.write().fk_on_delete = string_to_referential_action(&value);
                                    },
                                }

                                Select {
                                    value: referential_action_to_string(&form.fk_on_update),
                                    options: referential_action_options.clone(),
                                    label: "On Update",
                                    help_text: "Action when parent is updated",
                                    on_change: move |value: String| {
                                        form_state.write().fk_on_update = string_to_referential_action(&value);
                                    },
                                }
                            }
                        }
                    }

                    // UI Hints Tab
                    div {
                        class: if current_tab == "ui" { "space-y-4" } else { "hidden" },

                        TextInput {
                            value: form.label.clone(),
                            label: "Display Label",
                            placeholder: "Human-readable label",
                            help_text: "Label shown in forms (auto-generated if empty)",
                            on_change: move |value: String| {
                                form_state.write().label = value;
                            },
                        }

                        TextInput {
                            value: form.placeholder.clone(),
                            label: "Placeholder",
                            placeholder: "Placeholder text...",
                            on_change: move |value: String| {
                                form_state.write().placeholder = value;
                            },
                        }

                        TextArea {
                            value: form.help_text.clone(),
                            label: "Help Text",
                            placeholder: "Additional guidance for users...",
                            rows: 2,
                            on_change: move |value: String| {
                                form_state.write().help_text = value;
                            },
                        }

                        Select {
                            value: widget_type_to_string(&form.widget_type),
                            options: widget_type_options.clone(),
                            label: "Widget Type",
                            help_text: "Form input type for this field",
                            on_change: move |value: String| {
                                form_state.write().widget_type = string_to_widget_type(&value);
                            },
                        }

                        div {
                            class: "grid grid-cols-3 gap-4 pt-4 border-t border-slate-700",

                            Toggle {
                                checked: form.hidden,
                                label: "Hidden",
                                help_text: "Hide from default views",
                                on_change: move |checked: bool| {
                                    form_state.write().hidden = checked;
                                },
                            }

                            Toggle {
                                checked: form.readonly,
                                label: "Read-only",
                                help_text: "Cannot be edited",
                                on_change: move |checked: bool| {
                                    form_state.write().readonly = checked;
                                },
                            }

                            Toggle {
                                checked: form.secret,
                                label: "Secret",
                                help_text: "Mask in responses",
                                on_change: move |checked: bool| {
                                    form_state.write().secret = checked;
                                },
                            }
                        }
                    }
                }
            }

            // Actions (fixed at bottom)
            div {
                class: "flex justify-end gap-3 pt-4 border-t border-slate-700 mt-4",

                button {
                    r#type: "button",
                    class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg transition-colors",
                    disabled: saving,
                    onclick: handle_cancel,
                    "Cancel"
                }

                button {
                    r#type: "button",
                    class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-700 rounded-lg transition-colors flex items-center gap-2",
                    disabled: saving || !form.is_valid(),
                    onclick: move |_| handle_save(()),

                    if saving {
                        span { class: "animate-spin", "⏳" }
                        "Saving..."
                    } else {
                        span { "✓" }
                        "{save_button_text}"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Sub-Components
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct TabButtonProps {
    label: &'static str,
    active: bool,
    onclick: EventHandler<()>,
}

#[component]
fn TabButton(props: TabButtonProps) -> Element {
    let class = if props.active {
        "px-4 py-2 text-sm font-medium text-indigo-400 border-b-2 border-indigo-400"
    } else {
        "px-4 py-2 text-sm font-medium text-slate-400 hover:text-slate-300 border-b-2 border-transparent"
    };

    rsx! {
        button {
            r#type: "button",
            class: class,
            onclick: move |_| props.onclick.call(()),
            "{props.label}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ValidationRowProps {
    index: usize,
    config: ValidationConfig,
    options: Vec<SelectOption>,
    on_change: EventHandler<ValidationConfig>,
    on_remove: EventHandler<()>,
}

#[component]
fn ValidationRow(props: ValidationRowProps) -> Element {
    let config = props.config.clone();
    let needs_value = matches!(
        config.validation_type,
        ValidationType::MinLength
            | ValidationType::MaxLength
            | ValidationType::Min
            | ValidationType::Max
    );
    let needs_pattern = matches!(config.validation_type, ValidationType::Pattern);
    let config_for_type = config.clone();
    let config_for_length = config.clone();
    let config_for_float = config.clone();
    let config_for_pattern = config.clone();
    let config_for_message = config.clone();

    rsx! {
        div {
            class: "p-3 bg-slate-800 rounded-lg border border-slate-700 space-y-3",

            div {
                class: "flex items-center gap-3",

                // Validation type selector
                select {
                    class: "flex-1 px-3 py-2 bg-slate-700 border border-slate-600 rounded text-sm text-white focus:outline-none focus:border-indigo-500 appearance-none cursor-pointer",
                    value: validation_type_to_string(&config.validation_type),
                    onchange: move |e| {
                        let mut new_config = config_for_type.clone();
                        new_config.validation_type = string_to_validation_type(&e.value());
                        props.on_change.call(new_config);
                    },

                    for opt in props.options.iter() {
                        option {
                            value: opt.value.clone(),
                            "{opt.label}"
                        }
                    }
                }

                // Remove button
                button {
                    r#type: "button",
                    class: "p-2 text-red-400 hover:text-red-300 hover:bg-red-500/20 rounded transition-colors",
                    onclick: move |_| props.on_remove.call(()),
                    "✕"
                }
            }

            // Value input for validations that need it
            if needs_value {
                div {
                    class: "flex items-center gap-2",
                    label {
                        class: "text-sm text-slate-400 w-16",
                        "Value:"
                    }
                    // Use different input based on whether it's a length (int) or value (float) validation
                    if matches!(config.validation_type, ValidationType::MinLength | ValidationType::MaxLength) {
                        input {
                            r#type: "number",
                            class: "flex-1 px-3 py-1.5 bg-slate-700 border border-slate-600 rounded text-sm text-white focus:outline-none focus:border-indigo-500",
                            value: "{config.value_int}",
                            onchange: move |e| {
                                let mut new_config = config_for_length.clone();
                                new_config.value_int = e.value().parse().unwrap_or(0);
                                props.on_change.call(new_config);
                            },
                        }
                    } else {
                        input {
                            r#type: "number",
                            step: "any",
                            class: "flex-1 px-3 py-1.5 bg-slate-700 border border-slate-600 rounded text-sm text-white focus:outline-none focus:border-indigo-500",
                            value: "{config.value_float}",
                            onchange: move |e| {
                                let mut new_config = config_for_float.clone();
                                new_config.value_float = e.value().parse().unwrap_or(0.0);
                                props.on_change.call(new_config);
                            },
                        }
                    }
                }
            }

            // Pattern input
            if needs_pattern {
                div {
                    class: "space-y-2",
                    div {
                        class: "flex items-center gap-2",
                        label {
                            class: "text-sm text-slate-400 w-16",
                            "Regex:"
                        }
                        input {
                            r#type: "text",
                            class: "flex-1 px-3 py-1.5 bg-slate-700 border border-slate-600 rounded text-sm text-white font-mono focus:outline-none focus:border-indigo-500 placeholder-slate-500",
                            placeholder: "^[a-zA-Z]+$",
                            value: "{config.value_string}",
                            onchange: move |e| {
                                let mut new_config = config_for_pattern.clone();
                                new_config.value_string = e.value();
                                props.on_change.call(new_config);
                            },
                        }
                    }
                    div {
                        class: "flex items-center gap-2",
                        label {
                            class: "text-sm text-slate-400 w-16",
                            "Message:"
                        }
                        input {
                            r#type: "text",
                            class: "flex-1 px-3 py-1.5 bg-slate-700 border border-slate-600 rounded text-sm text-white focus:outline-none focus:border-indigo-500 placeholder-slate-500",
                            placeholder: "Error message...",
                            value: "{config.message}",
                            onchange: move |e| {
                                let mut new_config = config_for_message.clone();
                                new_config.message = e.value();
                                props.on_change.call(new_config);
                            },
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

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;
    let mut prev_is_separator = true;

    for c in s.chars() {
        if c.is_uppercase() {
            if !prev_is_separator && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
            prev_is_upper = true;
        } else if c == '-' || c == ' ' || c == '_' {
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
            prev_is_separator = true;
            prev_is_upper = false;
        } else {
            result.push(c);
            prev_is_upper = false;
            prev_is_separator = false;
        }
    }

    result
}

/// Check if a string is a valid identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be alphabetic or underscore
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

// Data type conversion functions
fn get_data_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("string", "String"),
        SelectOption::new("text", "Text (Long)"),
        SelectOption::new("int32", "Integer (32-bit)"),
        SelectOption::new("int64", "Integer (64-bit)"),
        SelectOption::new("float32", "Float (32-bit)"),
        SelectOption::new("float64", "Float (64-bit)"),
        SelectOption::new("bool", "Boolean"),
        SelectOption::new("uuid", "UUID"),
        SelectOption::new("datetime", "DateTime"),
        SelectOption::new("date", "Date"),
        SelectOption::new("time", "Time"),
        SelectOption::new("bytes", "Binary (Bytes)"),
        SelectOption::new("json", "JSON"),
    ]
}

fn data_type_to_string(dt: &DataType) -> String {
    match dt {
        DataType::String => "string".to_string(),
        DataType::Text => "text".to_string(),
        DataType::Int32 => "int32".to_string(),
        DataType::Int64 => "int64".to_string(),
        DataType::Float32 => "float32".to_string(),
        DataType::Float64 => "float64".to_string(),
        DataType::Bool => "bool".to_string(),
        DataType::Uuid => "uuid".to_string(),
        DataType::DateTime => "datetime".to_string(),
        DataType::Date => "date".to_string(),
        DataType::Time => "time".to_string(),
        DataType::Bytes => "bytes".to_string(),
        DataType::Json => "json".to_string(),
        DataType::Optional(inner) => data_type_to_string(inner),
        DataType::Array(inner) => data_type_to_string(inner),
        DataType::Reference { .. } => "uuid".to_string(),
        DataType::Enum { .. } => "string".to_string(),
    }
}

fn string_to_data_type(s: &str) -> DataType {
    match s {
        "string" => DataType::String,
        "text" => DataType::Text,
        "int32" => DataType::Int32,
        "int64" => DataType::Int64,
        "float32" => DataType::Float32,
        "float64" => DataType::Float64,
        "bool" => DataType::Bool,
        "uuid" => DataType::Uuid,
        "datetime" => DataType::DateTime,
        "date" => DataType::Date,
        "time" => DataType::Time,
        "bytes" => DataType::Bytes,
        "json" => DataType::Json,
        _ => DataType::String,
    }
}

// Widget type conversion functions
fn get_widget_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("text", "Text Input"),
        SelectOption::new("textarea", "Text Area"),
        SelectOption::new("number", "Number"),
        SelectOption::new("email", "Email"),
        SelectOption::new("password", "Password"),
        SelectOption::new("url", "URL"),
        SelectOption::new("phone", "Phone"),
        SelectOption::new("select", "Select (Dropdown)"),
        SelectOption::new("checkbox", "Checkbox"),
        SelectOption::new("toggle", "Toggle"),
        SelectOption::new("date", "Date Picker"),
        SelectOption::new("time", "Time Picker"),
        SelectOption::new("datetime", "DateTime Picker"),
        SelectOption::new("color", "Color Picker"),
        SelectOption::new("file", "File Upload"),
        SelectOption::new("image", "Image Upload"),
        SelectOption::new("richtext", "Rich Text Editor"),
        SelectOption::new("markdown", "Markdown Editor"),
        SelectOption::new("code", "Code Editor"),
        SelectOption::new("json", "JSON Editor"),
        SelectOption::new("hidden", "Hidden"),
        SelectOption::new("readonly", "Read-only Display"),
    ]
}

fn widget_type_to_string(wt: &WidgetType) -> String {
    match wt {
        WidgetType::Text => "text".to_string(),
        WidgetType::TextArea => "textarea".to_string(),
        WidgetType::Number => "number".to_string(),
        WidgetType::Email => "email".to_string(),
        WidgetType::Password => "password".to_string(),
        WidgetType::Url => "url".to_string(),
        WidgetType::Phone => "phone".to_string(),
        WidgetType::Select => "select".to_string(),
        WidgetType::MultiSelect => "select".to_string(),
        WidgetType::Checkbox => "checkbox".to_string(),
        WidgetType::Radio => "checkbox".to_string(),
        WidgetType::Toggle => "toggle".to_string(),
        WidgetType::Date => "date".to_string(),
        WidgetType::Time => "time".to_string(),
        WidgetType::DateTime => "datetime".to_string(),
        WidgetType::Color => "color".to_string(),
        WidgetType::File => "file".to_string(),
        WidgetType::Image => "image".to_string(),
        WidgetType::RichText => "richtext".to_string(),
        WidgetType::Markdown => "markdown".to_string(),
        WidgetType::Code => "code".to_string(),
        WidgetType::Json => "json".to_string(),
        WidgetType::Slider => "number".to_string(),
        WidgetType::Range => "number".to_string(),
        WidgetType::Rating => "number".to_string(),
        WidgetType::Hidden => "hidden".to_string(),
        WidgetType::Readonly => "readonly".to_string(),
    }
}

fn string_to_widget_type(s: &str) -> WidgetType {
    match s {
        "text" => WidgetType::Text,
        "textarea" => WidgetType::TextArea,
        "number" => WidgetType::Number,
        "email" => WidgetType::Email,
        "password" => WidgetType::Password,
        "url" => WidgetType::Url,
        "phone" => WidgetType::Phone,
        "select" => WidgetType::Select,
        "checkbox" => WidgetType::Checkbox,
        "toggle" => WidgetType::Toggle,
        "date" => WidgetType::Date,
        "time" => WidgetType::Time,
        "datetime" => WidgetType::DateTime,
        "color" => WidgetType::Color,
        "file" => WidgetType::File,
        "image" => WidgetType::Image,
        "richtext" => WidgetType::RichText,
        "markdown" => WidgetType::Markdown,
        "code" => WidgetType::Code,
        "json" => WidgetType::Json,
        "hidden" => WidgetType::Hidden,
        "readonly" => WidgetType::Readonly,
        _ => WidgetType::Text,
    }
}

// Referential action conversion functions
fn get_referential_action_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("cascade", "CASCADE"),
        SelectOption::new("set_null", "SET NULL"),
        SelectOption::new("restrict", "RESTRICT"),
        SelectOption::new("no_action", "NO ACTION"),
        SelectOption::new("set_default", "SET DEFAULT"),
    ]
}

fn referential_action_to_string(ra: &ReferentialAction) -> String {
    match ra {
        ReferentialAction::Cascade => "cascade".to_string(),
        ReferentialAction::SetNull => "set_null".to_string(),
        ReferentialAction::Restrict => "restrict".to_string(),
        ReferentialAction::NoAction => "no_action".to_string(),
        ReferentialAction::SetDefault => "set_default".to_string(),
    }
}

fn string_to_referential_action(s: &str) -> ReferentialAction {
    match s {
        "cascade" => ReferentialAction::Cascade,
        "set_null" => ReferentialAction::SetNull,
        "restrict" => ReferentialAction::Restrict,
        "no_action" => ReferentialAction::NoAction,
        "set_default" => ReferentialAction::SetDefault,
        _ => ReferentialAction::Cascade,
    }
}

// Default value type conversion functions
fn get_default_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("null", "NULL"),
        SelectOption::new("string", "String Value"),
        SelectOption::new("number", "Number Value"),
        SelectOption::new("bool", "Boolean Value"),
        SelectOption::new("now", "Current Timestamp (NOW)"),
        SelectOption::new("uuid", "Generate UUID"),
        SelectOption::new("expression", "SQL Expression"),
    ]
}

fn default_type_to_string(dt: &DefaultValueType) -> String {
    match dt {
        DefaultValueType::Null => "null".to_string(),
        DefaultValueType::String => "string".to_string(),
        DefaultValueType::Number => "number".to_string(),
        DefaultValueType::Bool => "bool".to_string(),
        DefaultValueType::Now => "now".to_string(),
        DefaultValueType::Uuid => "uuid".to_string(),
        DefaultValueType::Expression => "expression".to_string(),
    }
}

fn string_to_default_type(s: &str) -> DefaultValueType {
    match s {
        "null" => DefaultValueType::Null,
        "string" => DefaultValueType::String,
        "number" => DefaultValueType::Number,
        "bool" => DefaultValueType::Bool,
        "now" => DefaultValueType::Now,
        "uuid" => DefaultValueType::Uuid,
        "expression" => DefaultValueType::Expression,
        _ => DefaultValueType::Null,
    }
}

// Validation type conversion functions
fn get_validation_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("required", "Required"),
        SelectOption::new("min_length", "Min Length"),
        SelectOption::new("max_length", "Max Length"),
        SelectOption::new("min", "Min Value"),
        SelectOption::new("max", "Max Value"),
        SelectOption::new("pattern", "Pattern (Regex)"),
        SelectOption::new("email", "Email Format"),
        SelectOption::new("url", "URL Format"),
        SelectOption::new("uuid", "UUID Format"),
        SelectOption::new("phone", "Phone Format"),
    ]
}

fn validation_type_to_string(vt: &ValidationType) -> String {
    match vt {
        ValidationType::Required => "required".to_string(),
        ValidationType::MinLength => "min_length".to_string(),
        ValidationType::MaxLength => "max_length".to_string(),
        ValidationType::Min => "min".to_string(),
        ValidationType::Max => "max".to_string(),
        ValidationType::Pattern => "pattern".to_string(),
        ValidationType::Email => "email".to_string(),
        ValidationType::Url => "url".to_string(),
        ValidationType::Uuid => "uuid".to_string(),
        ValidationType::Phone => "phone".to_string(),
    }
}

fn string_to_validation_type(s: &str) -> ValidationType {
    match s {
        "required" => ValidationType::Required,
        "min_length" => ValidationType::MinLength,
        "max_length" => ValidationType::MaxLength,
        "min" => ValidationType::Min,
        "max" => ValidationType::Max,
        "pattern" => ValidationType::Pattern,
        "email" => ValidationType::Email,
        "url" => ValidationType::Url,
        "uuid" => ValidationType::Uuid,
        "phone" => ValidationType::Phone,
        _ => ValidationType::Required,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("userName"), "user_name");
        assert_eq!(to_snake_case("UserName"), "user_name");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("XMLParser"), "xmlparser");
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("user_name"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("field1"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123field"));
        assert!(!is_valid_identifier("field-name"));
    }

    #[test]
    fn test_data_type_conversion() {
        assert_eq!(data_type_to_string(&DataType::String), "string");
        assert_eq!(data_type_to_string(&DataType::Int32), "int32");
        assert_eq!(string_to_data_type("string"), DataType::String);
        assert_eq!(string_to_data_type("int32"), DataType::Int32);
    }

    #[test]
    fn test_widget_type_conversion() {
        assert_eq!(widget_type_to_string(&WidgetType::Text), "text");
        assert_eq!(widget_type_to_string(&WidgetType::Password), "password");
        assert_eq!(string_to_widget_type("text"), WidgetType::Text);
        assert_eq!(string_to_widget_type("password"), WidgetType::Password);
    }

    #[test]
    fn test_referential_action_conversion() {
        assert_eq!(
            referential_action_to_string(&ReferentialAction::Cascade),
            "cascade"
        );
        assert_eq!(
            referential_action_to_string(&ReferentialAction::SetNull),
            "set_null"
        );
        assert_eq!(
            string_to_referential_action("cascade"),
            ReferentialAction::Cascade
        );
        assert_eq!(
            string_to_referential_action("set_null"),
            ReferentialAction::SetNull
        );
    }

    #[test]
    fn test_default_type_conversion() {
        assert_eq!(default_type_to_string(&DefaultValueType::Null), "null");
        assert_eq!(default_type_to_string(&DefaultValueType::Now), "now");
        assert_eq!(string_to_default_type("null"), DefaultValueType::Null);
        assert_eq!(string_to_default_type("now"), DefaultValueType::Now);
    }

    #[test]
    fn test_validation_type_conversion() {
        assert_eq!(
            validation_type_to_string(&ValidationType::Required),
            "required"
        );
        assert_eq!(validation_type_to_string(&ValidationType::Email), "email");
        assert_eq!(
            string_to_validation_type("required"),
            ValidationType::Required
        );
        assert_eq!(string_to_validation_type("email"), ValidationType::Email);
    }

    #[test]
    fn test_form_state_default() {
        let state = FieldFormState::default();
        assert!(state.name.is_empty());
        assert!(!state.required);
        assert!(!state.unique);
        assert!(!state.has_default);
        assert!(!state.is_foreign_key);
    }

    #[test]
    fn test_form_state_validation() {
        let mut state = FieldFormState::default();

        // Empty name should fail
        assert!(!state.is_valid());

        // Valid name should pass
        state.name = "email".to_string();
        assert!(state.is_valid());

        // Invalid name should fail
        state.name = "123invalid".to_string();
        assert!(!state.is_valid());

        // FK without entity name should fail
        state.name = "user_id".to_string();
        state.is_foreign_key = true;
        assert!(!state.is_valid());

        state.fk_entity_name = "User".to_string();
        assert!(state.is_valid());
    }

    #[test]
    fn test_validation_config() {
        let config = ValidationConfig::new(ValidationType::MinLength);
        assert_eq!(config.validation_type, ValidationType::MinLength);
        assert_eq!(config.value_int, 0);

        let validation = config.to_validation();
        assert!(validation.is_some());
        assert!(matches!(validation.unwrap(), Validation::MinLength(0)));
    }

    #[test]
    fn test_build_data_type() {
        let mut state = FieldFormState::default();
        state.data_type = DataType::String;

        // Base type
        assert_eq!(state.build_data_type(), DataType::String);

        // Optional type
        state.is_optional = true;
        assert!(matches!(state.build_data_type(), DataType::Optional(_)));

        // Array type
        state.is_optional = false;
        state.is_array = true;
        assert!(matches!(state.build_data_type(), DataType::Array(_)));

        // Optional array
        state.is_optional = true;
        let dt = state.build_data_type();
        assert!(matches!(dt, DataType::Optional(_)));
    }
}
