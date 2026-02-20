//! # Validation Editor Component
//!
//! A component for configuring field validation rules in the Immortal Engine.
//!
//! ## Features
//!
//! - Add, edit, and remove validation rules
//! - Support for common validation types (required, min/max length, pattern, etc.)
//! - Custom error messages
//! - Real-time validation preview
//! - Drag-to-reorder validations
//!

use dioxus::prelude::*;
use imortal_core::types::Validation;

// ============================================================================
// Types
// ============================================================================

/// Validation type for UI selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationKind {
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
    Custom,
}

impl ValidationKind {
    /// Get all available validation kinds
    pub fn all() -> Vec<Self> {
        vec![
            Self::Required,
            Self::MinLength,
            Self::MaxLength,
            Self::Min,
            Self::Max,
            Self::Pattern,
            Self::Email,
            Self::Url,
            Self::Uuid,
            Self::Phone,
            Self::Custom,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Required => "Required",
            Self::MinLength => "Minimum Length",
            Self::MaxLength => "Maximum Length",
            Self::Min => "Minimum Value",
            Self::Max => "Maximum Value",
            Self::Pattern => "Pattern (Regex)",
            Self::Email => "Email Format",
            Self::Url => "URL Format",
            Self::Uuid => "UUID Format",
            Self::Phone => "Phone Format",
            Self::Custom => "Custom Validation",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Required => "Field must have a value",
            Self::MinLength => "Minimum number of characters",
            Self::MaxLength => "Maximum number of characters",
            Self::Min => "Minimum numeric value",
            Self::Max => "Maximum numeric value",
            Self::Pattern => "Must match a regular expression",
            Self::Email => "Must be a valid email address",
            Self::Url => "Must be a valid URL",
            Self::Uuid => "Must be a valid UUID",
            Self::Phone => "Must be a valid phone number",
            Self::Custom => "Custom validation expression",
        }
    }

    /// Get icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Required => "❗",
            Self::MinLength => "📏",
            Self::MaxLength => "📐",
            Self::Min => "⬇️",
            Self::Max => "⬆️",
            Self::Pattern => "🔤",
            Self::Email => "📧",
            Self::Url => "🔗",
            Self::Uuid => "🔑",
            Self::Phone => "📱",
            Self::Custom => "⚙️",
        }
    }

    /// Check if this validation needs a numeric value
    pub fn needs_numeric_value(&self) -> bool {
        matches!(
            self,
            Self::MinLength | Self::MaxLength | Self::Min | Self::Max
        )
    }

    /// Check if this validation needs a string value (pattern/custom)
    pub fn needs_string_value(&self) -> bool {
        matches!(self, Self::Pattern | Self::Custom)
    }

    /// Check if this validation needs a custom message
    pub fn needs_message(&self) -> bool {
        matches!(self, Self::Pattern | Self::Custom)
    }

    /// Get from a Validation enum
    pub fn from_validation(v: &Validation) -> Self {
        match v {
            Validation::Required => Self::Required,
            Validation::MinLength(_) => Self::MinLength,
            Validation::MaxLength(_) => Self::MaxLength,
            Validation::Min(_) => Self::Min,
            Validation::Max(_) => Self::Max,
            Validation::Pattern { .. } => Self::Pattern,
            Validation::Email => Self::Email,
            Validation::Url => Self::Url,
            Validation::Uuid => Self::Uuid,
            Validation::Phone => Self::Phone,
            Validation::OneOf(_) => Self::Custom,
            Validation::Custom { .. } => Self::Custom,
        }
    }
}

/// UI state for a single validation rule
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationState {
    pub id: usize,
    pub kind: ValidationKind,
    pub numeric_value: i64,
    pub float_value: f64,
    pub string_value: String,
    pub message: String,
    pub is_expanded: bool,
}

impl ValidationState {
    /// Create a new validation state
    pub fn new(id: usize, kind: ValidationKind) -> Self {
        Self {
            id,
            kind,
            numeric_value: match kind {
                ValidationKind::MinLength => 1,
                ValidationKind::MaxLength => 255,
                _ => 0,
            },
            float_value: match kind {
                ValidationKind::Min => 0.0,
                ValidationKind::Max => 100.0,
                _ => 0.0,
            },
            string_value: String::new(),
            message: String::new(),
            is_expanded: true,
        }
    }

    /// Create from a Validation enum
    pub fn from_validation(id: usize, v: &Validation) -> Self {
        let kind = ValidationKind::from_validation(v);
        let (numeric_value, float_value, string_value, message) = match v {
            Validation::Required => (0, 0.0, String::new(), String::new()),
            Validation::MinLength(n) => (*n as i64, 0.0, String::new(), String::new()),
            Validation::MaxLength(n) => (*n as i64, 0.0, String::new(), String::new()),
            Validation::Min(n) => (0, *n, String::new(), String::new()),
            Validation::Max(n) => (0, *n, String::new(), String::new()),
            Validation::Pattern { regex, message } => (0, 0.0, regex.clone(), message.clone()),
            Validation::Email => (0, 0.0, String::new(), String::new()),
            Validation::Url => (0, 0.0, String::new(), String::new()),
            Validation::Uuid => (0, 0.0, String::new(), String::new()),
            Validation::Phone => (0, 0.0, String::new(), String::new()),
            Validation::OneOf(values) => (0, 0.0, values.join(", "), String::new()),
            Validation::Custom { name, expression } => (0, 0.0, expression.clone(), name.clone()),
        };

        Self {
            id,
            kind,
            numeric_value,
            float_value,
            string_value,
            message,
            is_expanded: false,
        }
    }

    /// Convert to Validation enum
    pub fn to_validation(&self) -> Option<Validation> {
        Some(match self.kind {
            ValidationKind::Required => Validation::Required,
            ValidationKind::MinLength => {
                if self.numeric_value < 0 {
                    return None;
                }
                Validation::MinLength(self.numeric_value as usize)
            }
            ValidationKind::MaxLength => {
                if self.numeric_value < 0 {
                    return None;
                }
                Validation::MaxLength(self.numeric_value as usize)
            }
            ValidationKind::Min => Validation::Min(self.float_value),
            ValidationKind::Max => Validation::Max(self.float_value),
            ValidationKind::Pattern => {
                if self.string_value.is_empty() {
                    return None;
                }
                Validation::Pattern {
                    regex: self.string_value.clone(),
                    message: if self.message.is_empty() {
                        "Invalid format".to_string()
                    } else {
                        self.message.clone()
                    },
                }
            }
            ValidationKind::Email => Validation::Email,
            ValidationKind::Url => Validation::Url,
            ValidationKind::Uuid => Validation::Uuid,
            ValidationKind::Phone => Validation::Phone,
            ValidationKind::Custom => {
                if self.string_value.is_empty() {
                    return None;
                }
                Validation::Custom {
                    name: if self.message.is_empty() {
                        "custom".to_string()
                    } else {
                        self.message.clone()
                    },
                    expression: self.string_value.clone(),
                }
            }
        })
    }

    /// Validate this validation state
    pub fn validate(&self) -> Option<String> {
        match self.kind {
            ValidationKind::MinLength | ValidationKind::MaxLength => {
                if self.numeric_value < 0 {
                    return Some("Value must be non-negative".to_string());
                }
            }
            ValidationKind::Pattern => {
                if self.string_value.is_empty() {
                    return Some("Pattern regex is required".to_string());
                }
                // Validate regex syntax
                if regex::Regex::new(&self.string_value).is_err() {
                    return Some("Invalid regex pattern".to_string());
                }
            }
            ValidationKind::Custom => {
                if self.string_value.is_empty() {
                    return Some("Expression is required".to_string());
                }
            }
            _ => {}
        }
        None
    }

    /// Get default error message for this validation
    pub fn default_error_message(&self) -> String {
        match self.kind {
            ValidationKind::Required => "This field is required".to_string(),
            ValidationKind::MinLength => {
                format!("Must be at least {} characters", self.numeric_value)
            }
            ValidationKind::MaxLength => {
                format!("Must be at most {} characters", self.numeric_value)
            }
            ValidationKind::Min => format!("Must be at least {}", self.float_value),
            ValidationKind::Max => format!("Must be at most {}", self.float_value),
            ValidationKind::Pattern => "Invalid format".to_string(),
            ValidationKind::Email => "Must be a valid email address".to_string(),
            ValidationKind::Url => "Must be a valid URL".to_string(),
            ValidationKind::Uuid => "Must be a valid UUID".to_string(),
            ValidationKind::Phone => "Must be a valid phone number".to_string(),
            ValidationKind::Custom => "Validation failed".to_string(),
        }
    }
}

// ============================================================================
// Component Props
// ============================================================================

#[derive(Props, Clone, PartialEq)]
pub struct ValidationEditorProps {
    /// Current validations
    pub validations: Vec<Validation>,

    /// Whether the editor is disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Whether to show in compact mode
    #[props(default = false)]
    pub compact: bool,

    /// Maximum number of validations allowed
    #[props(default = 10)]
    pub max_validations: usize,

    /// Callback when validations change
    pub on_change: EventHandler<Vec<Validation>>,
}

// ============================================================================
// Main Component
// ============================================================================

/// Validation editor for configuring field validation rules
#[component]
pub fn ValidationEditor(props: ValidationEditorProps) -> Element {
    // Convert validations to state
    let initial_states: Vec<ValidationState> = props
        .validations
        .iter()
        .enumerate()
        .map(|(i, v)| ValidationState::from_validation(i, v))
        .collect();

    let mut validation_states = use_signal(|| initial_states);
    let mut next_id = use_signal(|| props.validations.len());
    let mut show_add_menu = use_signal(|| false);

    // Emit changes when states change
    let emit_changes = move |states: &Vec<ValidationState>| {
        let validations: Vec<Validation> =
            states.iter().filter_map(|s| s.to_validation()).collect();
        props.on_change.call(validations);
    };

    // Add new validation
    let mut add_validation = move |kind: ValidationKind| {
        let id = *next_id.read();
        next_id.set(id + 1);

        let mut states = validation_states.write();
        states.push(ValidationState::new(id, kind));
        emit_changes(&states);
        show_add_menu.set(false);
    };

    // Remove validation
    let mut remove_validation = move |id: usize| {
        let mut states = validation_states.write();
        states.retain(|s| s.id != id);
        emit_changes(&states);
    };

    // Update validation
    let mut update_validation = move |id: usize, new_state: ValidationState| {
        let mut states = validation_states.write();
        if let Some(state) = states.iter_mut().find(|s| s.id == id) {
            *state = new_state;
        }
        emit_changes(&states);
    };

    // Toggle expansion
    let mut toggle_expansion = move |id: usize| {
        let mut states = validation_states.write();
        if let Some(state) = states.iter_mut().find(|s| s.id == id) {
            state.is_expanded = !state.is_expanded;
        }
    };

    // Move validation up
    let mut move_up = move |id: usize| {
        let mut states = validation_states.write();
        if let Some(index) = states.iter().position(|s| s.id == id) {
            if index > 0 {
                states.swap(index, index - 1);
                emit_changes(&states);
            }
        }
    };

    // Move validation down
    let mut move_down = move |id: usize| {
        let mut states = validation_states.write();
        if let Some(index) = states.iter().position(|s| s.id == id) {
            if index < states.len() - 1 {
                states.swap(index, index + 1);
                emit_changes(&states);
            }
        }
    };

    let states: Vec<ValidationState> = validation_states.read().clone();
    let can_add = states.len() < props.max_validations;
    let states_len = states.len();

    rsx! {
        div {
            class: "validation-editor space-y-3",

            // Header
            div {
                class: "flex items-center justify-between",

                h4 {
                    class: "text-sm font-medium text-slate-300",
                    "Validation Rules"
                    span {
                        class: "ml-2 text-xs text-slate-500",
                        "({states_len}/{props.max_validations})"
                    }
                }

                // Add button with dropdown
                div {
                    class: "relative",

                    button {
                        r#type: "button",
                        class: format!(
                            "px-3 py-1.5 text-sm rounded-lg transition-colors flex items-center gap-1 {}",
                            if can_add && !props.disabled {
                                "bg-indigo-600 hover:bg-indigo-700 text-white"
                            } else {
                                "bg-slate-700 text-slate-500 cursor-not-allowed"
                            }
                        ),
                        disabled: !can_add || props.disabled,
                        onclick: move |_| {
                            let current = *show_add_menu.read();
                            show_add_menu.set(!current);
                        },

                        span { "+" }
                        span { "Add Rule" }
                    }

                    // Add menu dropdown
                    if *show_add_menu.read() && can_add {
                        div {
                            class: "absolute right-0 mt-1 w-64 bg-slate-800 border border-slate-700 rounded-lg shadow-xl z-50 overflow-hidden",

                            div {
                                class: "max-h-80 overflow-y-auto",

                                for kind in ValidationKind::all() {
                                    button {
                                        r#type: "button",
                                        class: "w-full px-3 py-2 text-left hover:bg-slate-700 transition-colors flex items-center gap-3",
                                        onclick: move |_| add_validation(kind),

                                        span { class: "text-lg", "{kind.icon()}" }
                                        div {
                                            div { class: "text-sm font-medium text-white", "{kind.display_name()}" }
                                            div { class: "text-xs text-slate-500", "{kind.description()}" }
                                        }
                                    }
                                }
                            }
                        }

                        // Backdrop
                        div {
                            class: "fixed inset-0 z-40",
                            onclick: move |_| show_add_menu.set(false),
                        }
                    }
                }
            }

            // Validation list
            if states.is_empty() {
                div {
                    class: "text-center py-8 bg-slate-800/50 rounded-lg border border-slate-700 border-dashed",

                    div { class: "text-3xl mb-2", "📋" }
                    p { class: "text-slate-400 text-sm", "No validation rules configured" }
                    p { class: "text-slate-500 text-xs mt-1", "Click \"Add Rule\" to add validation constraints" }
                }
            } else {
                div {
                    class: "space-y-2",

                    {states.iter().enumerate().map(|(index, state)| {
                        let state_id = state.id;
                        let state_clone = state.clone();
                        let total = states.len();
                        rsx! {
                            ValidationRow {
                                key: "{state_id}",
                                state: state_clone,
                                index: index,
                                total: total,
                                disabled: props.disabled,
                                compact: props.compact,
                                on_update: move |new_state: ValidationState| update_validation(state_id, new_state),
                                on_remove: move |_| remove_validation(state_id),
                                on_toggle: move |_| toggle_expansion(state_id),
                                on_move_up: move |_| move_up(state_id),
                                on_move_down: move |_| move_down(state_id),
                            }
                        }
                    })}
                }
            }

            // Validation summary
            if !states.is_empty() {
                div {
                    class: "text-xs text-slate-500 pt-2 border-t border-slate-700",

                    "Active rules: "
                    {states.iter().enumerate().map(|(i, state)| {
                        let is_last = i >= states.len() - 1;
                        rsx! {
                            span {
                                key: "{i}",
                                class: "inline-flex items-center gap-1 px-1.5 py-0.5 bg-slate-700 rounded text-slate-300 mr-1",
                                "{state.kind.icon()}"
                                "{state.kind.display_name()}"
                            }
                            if !is_last {
                                span { " " }
                            }
                        }
                    })}
                }
            }
        }
    }
}

// ============================================================================
// Sub-Components
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct ValidationRowProps {
    state: ValidationState,
    index: usize,
    total: usize,
    disabled: bool,
    compact: bool,
    on_update: EventHandler<ValidationState>,
    on_remove: EventHandler<()>,
    on_toggle: EventHandler<()>,
    on_move_up: EventHandler<()>,
    on_move_down: EventHandler<()>,
}

#[component]
fn ValidationRow(props: ValidationRowProps) -> Element {
    let state = props.state.clone();
    let validation_error = state.validate();
    let has_error = validation_error.is_some();
    let state_for_value = state.clone();
    let state_for_pattern = state.clone();
    let state_for_message = state.clone();
    let state_for_letters = state.clone();
    let state_for_digits = state.clone();
    let state_for_slug = state.clone();
    let state_is_expanded = state.is_expanded;
    let state_kind = state.kind;
    let state_numeric_value = state.numeric_value;
    let state_float_value = state.float_value;
    let state_string_value = state.string_value.clone();
    let state_message = state.message.clone();

    rsx! {
        div {
            class: format!(
                "bg-slate-800 rounded-lg border transition-colors {}",
                if has_error { "border-red-500/50" } else { "border-slate-700" }
            ),

            // Header (always visible)
            div {
                class: "flex items-center gap-2 px-3 py-2",

                // Drag handle / reorder buttons
                div {
                    class: "flex flex-col gap-0.5",

                    button {
                        r#type: "button",
                        class: format!(
                            "p-0.5 rounded text-slate-500 transition-colors {}",
                            if props.index > 0 && !props.disabled { "hover:text-slate-300 hover:bg-slate-700" } else { "opacity-30" }
                        ),
                        disabled: props.index == 0 || props.disabled,
                        onclick: move |_| props.on_move_up.call(()),
                        title: "Move up",
                        "▲"
                    }
                    button {
                        r#type: "button",
                        class: format!(
                            "p-0.5 rounded text-slate-500 transition-colors {}",
                            if props.index < props.total - 1 && !props.disabled { "hover:text-slate-300 hover:bg-slate-700" } else { "opacity-30" }
                        ),
                        disabled: props.index >= props.total - 1 || props.disabled,
                        onclick: move |_| props.on_move_down.call(()),
                        title: "Move down",
                        "▼"
                    }
                }

                // Icon
                span { class: "text-lg", "{state_kind.icon()}" }

                // Name and summary
                div {
                    class: "flex-1 min-w-0",

                    div {
                        class: "flex items-center gap-2",

                        span {
                            class: "font-medium text-white text-sm",
                            "{state_kind.display_name()}"
                        }

                        // Value preview
                        if state_kind.needs_numeric_value() {
                            span {
                                class: "text-xs px-1.5 py-0.5 bg-indigo-500/20 text-indigo-300 rounded",
                                if matches!(state_kind, ValidationKind::Min | ValidationKind::Max) {
                                    "{state_float_value}"
                                } else {
                                    "{state_numeric_value}"
                                }
                            }
                        }

                        // Error indicator
                        if has_error {
                            span {
                                class: "text-xs text-red-400",
                                title: validation_error.clone().unwrap_or_default(),
                                "⚠️"
                            }
                        }
                    }

                    // Preview of generated message
                    if !props.compact {
                        div {
                            class: "text-xs text-slate-500 truncate",
                            "{props.state.default_error_message()}"
                        }
                    }
                }

                // Actions
                div {
                    class: "flex items-center gap-1",

                    // Expand/collapse
                    button {
                        r#type: "button",
                        class: "p-1.5 rounded text-slate-400 hover:text-slate-300 hover:bg-slate-700 transition-colors",
                        onclick: move |_| props.on_toggle.call(()),
                        title: if state_is_expanded { "Collapse" } else { "Expand" },

                        span {
                            class: format!("transition-transform {}", if state_is_expanded { "rotate-180" } else { "" }),
                            "▼"
                        }
                    }

                    // Remove
                    button {
                        r#type: "button",
                        class: "p-1.5 rounded text-red-400 hover:text-red-300 hover:bg-red-500/20 transition-colors",
                        disabled: props.disabled,
                        onclick: move |_| props.on_remove.call(()),
                        title: "Remove validation",
                        "✕"
                    }
                }
            }

            // Expanded content
            if state_is_expanded {
                div {
                    class: "px-3 pb-3 pt-1 border-t border-slate-700 space-y-3",

                    // Numeric value input
                    if state_kind.needs_numeric_value() {
                        div {
                            class: "flex items-center gap-3",

                            label {
                                class: "text-sm text-slate-400 w-20",
                                "Value:"
                            }
                            input {
                                r#type: "number",
                                class: "flex-1 px-3 py-1.5 bg-slate-700 border border-slate-600 rounded text-sm text-white focus:outline-none focus:border-indigo-500",
                                value: "{state_numeric_value}",
                                onchange: move |e| {
                                    let mut new_state = state_for_value.clone();
                                    if matches!(state_for_value.kind, ValidationKind::Min | ValidationKind::Max) {
                                        new_state.float_value = e.value().parse().unwrap_or(0.0);
                                    } else {
                                        new_state.numeric_value = e.value().parse().unwrap_or(0);
                                    }
                                    props.on_update.call(new_state);
                                },
                            }
                        }
                    }

                    // String value input (pattern/custom)
                    if state_kind.needs_string_value() {
                        div {
                            class: "flex items-start gap-3",

                            label {
                                class: "text-sm text-slate-400 w-20 pt-1.5",
                                if matches!(state.kind, ValidationKind::Pattern) { "Pattern:" } else { "Expression:" }
                            }
                            div {
                                class: "flex-1",
                                input {
                                    r#type: "text",
                                    class: "w-full px-3 py-1.5 bg-slate-700 border border-slate-600 rounded text-sm text-white font-mono focus:outline-none focus:border-indigo-500",
                                    placeholder: if matches!(state_kind, ValidationKind::Pattern) { "^[a-zA-Z0-9]+$" } else { "value.len() > 0" },
                                    value: "{state_string_value}",
                                    onchange: move |e| {
                                        let mut new_state = state_for_pattern.clone();
                                        new_state.string_value = e.value();
                                        props.on_update.call(new_state);
                                    },
                                }

                                // Regex helper for patterns
                                if matches!(state_kind, ValidationKind::Pattern) {
                                    div {
                                        class: "mt-1 text-xs text-slate-500",
                                        "Common patterns: "
                                        button {
                                            r#type: "button",
                                            class: "text-indigo-400 hover:text-indigo-300 mr-2",
                                            onclick: move |_| {
                                                let mut new_state = state_for_letters.clone();
                                                new_state.string_value = r"^[a-zA-Z]+$".to_string();
                                                props.on_update.call(new_state);
                                            },
                                            "letters"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "text-indigo-400 hover:text-indigo-300 mr-2",
                                            onclick: move |_| {
                                                let mut new_state = state_for_digits.clone();
                                                new_state.string_value = r"^[0-9]+$".to_string();
                                                props.on_update.call(new_state);
                                            },
                                            "digits"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "text-indigo-400 hover:text-indigo-300 mr-2",
                                            onclick: move |_| {
                                                let mut new_state = state_for_slug.clone();
                                                new_state.string_value = r"^[a-zA-Z0-9_-]+$".to_string();
                                                props.on_update.call(new_state);
                                            },
                                            "slug"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Custom message input
                    if state_kind.needs_message() {
                        div {
                            class: "flex items-center gap-3",

                            label {
                                class: "text-sm text-slate-400 w-20",
                                "Message:"
                            }
                            input {
                                r#type: "text",
                                class: "flex-1 px-3 py-1.5 bg-slate-700 border border-slate-600 rounded text-sm text-white focus:outline-none focus:border-indigo-500",
                                placeholder: "Error message when validation fails...",
                                value: "{state_message}",
                                onchange: move |e| {
                                    let mut new_state = state_for_message.clone();
                                    new_state.message = e.value();
                                    props.on_update.call(new_state);
                                },
                            }
                        }
                    }

                    // Error display
                    if let Some(error) = validation_error {
                        div {
                            class: "text-xs text-red-400 flex items-center gap-1",
                            "⚠️ {error}"
                        }
                    }

                    // Preview
                    div {
                        class: "text-xs text-slate-500 pt-2 border-t border-slate-700",
                        "Generated error: "
                        span { class: "text-slate-400 italic", "\"{state.default_error_message()}\"" }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Convert a list of Validation to ValidationState
pub fn validations_to_states(validations: &[Validation]) -> Vec<ValidationState> {
    validations
        .iter()
        .enumerate()
        .map(|(i, v)| ValidationState::from_validation(i, v))
        .collect()
}

/// Convert a list of ValidationState to Validation
pub fn states_to_validations(states: &[ValidationState]) -> Vec<Validation> {
    states.iter().filter_map(|s| s.to_validation()).collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_kind_all() {
        let kinds = ValidationKind::all();
        assert!(!kinds.is_empty());
        assert!(kinds.contains(&ValidationKind::Required));
        assert!(kinds.contains(&ValidationKind::Email));
    }

    #[test]
    fn test_validation_kind_display() {
        assert_eq!(ValidationKind::Required.display_name(), "Required");
        assert_eq!(ValidationKind::MinLength.display_name(), "Minimum Length");
    }

    #[test]
    fn test_validation_kind_needs_value() {
        assert!(ValidationKind::MinLength.needs_numeric_value());
        assert!(ValidationKind::MaxLength.needs_numeric_value());
        assert!(ValidationKind::Min.needs_numeric_value());
        assert!(ValidationKind::Max.needs_numeric_value());
        assert!(!ValidationKind::Required.needs_numeric_value());
        assert!(!ValidationKind::Email.needs_numeric_value());
    }

    #[test]
    fn test_validation_kind_needs_string() {
        assert!(ValidationKind::Pattern.needs_string_value());
        assert!(ValidationKind::Custom.needs_string_value());
        assert!(!ValidationKind::Required.needs_string_value());
        assert!(!ValidationKind::MinLength.needs_string_value());
    }

    #[test]
    fn test_validation_state_new() {
        let state = ValidationState::new(0, ValidationKind::Required);
        assert_eq!(state.kind, ValidationKind::Required);
        assert!(state.is_expanded);
    }

    #[test]
    fn test_validation_state_to_validation() {
        let state = ValidationState::new(0, ValidationKind::Required);
        let validation = state.to_validation();
        assert!(validation.is_some());
        assert!(matches!(validation.unwrap(), Validation::Required));

        let mut min_state = ValidationState::new(1, ValidationKind::MinLength);
        min_state.numeric_value = 5;
        let validation = min_state.to_validation();
        assert!(validation.is_some());
        assert!(matches!(validation.unwrap(), Validation::MinLength(5)));
    }

    #[test]
    fn test_validation_state_from_validation() {
        let state = ValidationState::from_validation(0, &Validation::Required);
        assert_eq!(state.kind, ValidationKind::Required);

        let state = ValidationState::from_validation(1, &Validation::MinLength(10));
        assert_eq!(state.kind, ValidationKind::MinLength);
        assert_eq!(state.numeric_value, 10);

        let state = ValidationState::from_validation(
            2,
            &Validation::Pattern {
                regex: "^[a-z]+$".to_string(),
                message: "Letters only".to_string(),
            },
        );
        assert_eq!(state.kind, ValidationKind::Pattern);
        assert_eq!(state.string_value, "^[a-z]+$");
        assert_eq!(state.message, "Letters only");
    }

    #[test]
    fn test_validation_state_validate() {
        let state = ValidationState::new(0, ValidationKind::Required);
        assert!(state.validate().is_none());

        let mut state = ValidationState::new(1, ValidationKind::MinLength);
        state.numeric_value = -5;
        assert!(state.validate().is_some());

        let mut state = ValidationState::new(2, ValidationKind::Pattern);
        state.string_value = String::new();
        assert!(state.validate().is_some());

        state.string_value = "^[a-z]+$".to_string();
        assert!(state.validate().is_none());
    }

    #[test]
    fn test_validations_to_states() {
        let validations = vec![
            Validation::Required,
            Validation::MinLength(5),
            Validation::Email,
        ];
        let states = validations_to_states(&validations);
        assert_eq!(states.len(), 3);
        assert_eq!(states[0].kind, ValidationKind::Required);
        assert_eq!(states[1].kind, ValidationKind::MinLength);
        assert_eq!(states[2].kind, ValidationKind::Email);
    }

    #[test]
    fn test_states_to_validations() {
        let states = vec![
            ValidationState::new(0, ValidationKind::Required),
            ValidationState::new(1, ValidationKind::Email),
        ];
        let validations = states_to_validations(&states);
        assert_eq!(validations.len(), 2);
    }

    #[test]
    fn test_default_error_message() {
        let state = ValidationState::new(0, ValidationKind::Required);
        assert_eq!(state.default_error_message(), "This field is required");

        let mut state = ValidationState::new(1, ValidationKind::MinLength);
        state.numeric_value = 5;
        assert_eq!(
            state.default_error_message(),
            "Must be at least 5 characters"
        );

        let state = ValidationState::new(2, ValidationKind::Email);
        assert_eq!(
            state.default_error_message(),
            "Must be a valid email address"
        );
    }
}
