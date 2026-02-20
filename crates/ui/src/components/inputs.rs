//! # Input Components
//!
//! Reusable form input components for the Immortal Engine UI.
//!
//! This module provides styled, accessible input components including:
//! - **TextInput**: Single-line text input
//! - **TextArea**: Multi-line text input
//! - **NumberInput**: Numeric input with optional min/max
//! - **Select**: Dropdown selection
//! - **Checkbox**: Boolean checkbox
//! - **Toggle**: Switch-style toggle
//! - **ColorPicker**: Color selection input
//!
//! All components follow consistent styling with Tailwind CSS and
//! support common accessibility features.
//!

use dioxus::prelude::*;

// ============================================================================
// Text Input Component
// ============================================================================

/// Properties for TextInput component
#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
    /// Input value
    pub value: String,

    /// Label text (optional)
    #[props(default)]
    pub label: Option<String>,

    /// Placeholder text
    #[props(default)]
    pub placeholder: Option<String>,

    /// Help text shown below input
    #[props(default)]
    pub help_text: Option<String>,

    /// Error message (shows error state)
    #[props(default)]
    pub error: Option<String>,

    /// Whether the input is required
    #[props(default = false)]
    pub required: bool,

    /// Whether the input is disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Whether the input is readonly
    #[props(default = false)]
    pub readonly: bool,

    /// Input type (text, email, password, etc.)
    #[props(default = "text".to_string())]
    pub input_type: String,

    /// Maximum length
    #[props(default)]
    pub max_length: Option<usize>,

    /// Prefix icon or text
    #[props(default)]
    pub prefix: Option<String>,

    /// Suffix icon or text
    #[props(default)]
    pub suffix: Option<String>,

    /// Additional CSS classes
    #[props(default)]
    pub class: Option<String>,

    /// Change handler
    #[props(default)]
    pub on_change: EventHandler<String>,

    /// Blur handler
    #[props(default)]
    pub on_blur: EventHandler<String>,

    /// Focus handler
    #[props(default)]
    pub on_focus: EventHandler<()>,

    /// Enter key handler
    #[props(default)]
    pub on_enter: EventHandler<String>,
}

/// Single-line text input component
#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    let has_error = props.error.is_some();

    let input_class = build_input_class(has_error, props.disabled, &props.class);
    let wrapper_class = if props.prefix.is_some() || props.suffix.is_some() {
        "relative flex items-center"
    } else {
        "relative"
    };

    rsx! {
        div {
            class: "input-group",

            // Label
            if let Some(label) = &props.label {
                label {
                    class: "block text-sm font-medium text-slate-300 mb-1.5",
                    "{label}"
                    if props.required {
                        span { class: "text-rose-400 ml-0.5", "*" }
                    }
                }
            }

            // Input wrapper
            div {
                class: "{wrapper_class}",

                // Prefix
                if let Some(prefix) = &props.prefix {
                    span {
                        class: "absolute left-3 text-slate-400 text-sm pointer-events-none",
                        "{prefix}"
                    }
                }

                // Input
                input {
                    class: "{input_class}",
                    class: if props.prefix.is_some() { "pl-8" } else { "" },
                    class: if props.suffix.is_some() { "pr-8" } else { "" },
                    r#type: "{props.input_type}",
                    value: "{props.value}",
                    placeholder: props.placeholder.as_deref().unwrap_or(""),
                    disabled: props.disabled,
                    readonly: props.readonly,
                    maxlength: props.max_length.map(|l| l.to_string()),
                    oninput: move |e| props.on_change.call(e.value()),
                    onblur: {
                        let value = props.value.clone();
                        move |_| props.on_blur.call(value.clone())
                    },
                    onfocus: move |_| props.on_focus.call(()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            props.on_enter.call(props.value.clone());
                        }
                    },
                }

                // Suffix
                if let Some(suffix) = &props.suffix {
                    span {
                        class: "absolute right-3 text-slate-400 text-sm pointer-events-none",
                        "{suffix}"
                    }
                }
            }

            // Help text or error
            if let Some(error) = &props.error {
                p {
                    class: "mt-1 text-xs text-rose-400",
                    "{error}"
                }
            } else if let Some(help) = &props.help_text {
                p {
                    class: "mt-1 text-xs text-slate-500",
                    "{help}"
                }
            }
        }
    }
}

// ============================================================================
// Text Area Component
// ============================================================================

/// Properties for TextArea component
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    /// Input value
    pub value: String,

    /// Label text
    #[props(default)]
    pub label: Option<String>,

    /// Placeholder text
    #[props(default)]
    pub placeholder: Option<String>,

    /// Help text
    #[props(default)]
    pub help_text: Option<String>,

    /// Error message
    #[props(default)]
    pub error: Option<String>,

    /// Number of visible rows
    #[props(default = 3)]
    pub rows: usize,

    /// Whether required
    #[props(default = false)]
    pub required: bool,

    /// Whether disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Whether readonly
    #[props(default = false)]
    pub readonly: bool,

    /// Maximum length
    #[props(default)]
    pub max_length: Option<usize>,

    /// Whether to show character count
    #[props(default = false)]
    pub show_count: bool,

    /// Whether to allow resize
    #[props(default = true)]
    pub resizable: bool,

    /// Additional CSS classes
    #[props(default)]
    pub class: Option<String>,

    /// Change handler
    #[props(default)]
    pub on_change: EventHandler<String>,

    /// Blur handler
    #[props(default)]
    pub on_blur: EventHandler<String>,
}

/// Multi-line text input component
#[component]
pub fn TextArea(props: TextAreaProps) -> Element {
    let has_error = props.error.is_some();
    let char_count = props.value.len();

    let textarea_class =
        build_textarea_class(has_error, props.disabled, props.resizable, &props.class);

    rsx! {
        div {
            class: "input-group",

            // Label
            if let Some(label) = &props.label {
                label {
                    class: "block text-sm font-medium text-slate-300 mb-1.5",
                    "{label}"
                    if props.required {
                        span { class: "text-rose-400 ml-0.5", "*" }
                    }
                }
            }

            // Textarea
            textarea {
                class: "{textarea_class}",
                rows: "{props.rows}",
                placeholder: props.placeholder.as_deref().unwrap_or(""),
                disabled: props.disabled,
                readonly: props.readonly,
                maxlength: props.max_length.map(|l| l.to_string()),
                oninput: move |e| props.on_change.call(e.value()),
                onblur: {
                    let value = props.value.clone();
                    move |_| props.on_blur.call(value.clone())
                },
                "{props.value}"
            }

            // Footer (help text / error / character count)
            div {
                class: "flex justify-between items-center mt-1",

                // Help text or error
                if let Some(error) = &props.error {
                    p {
                        class: "text-xs text-rose-400",
                        "{error}"
                    }
                } else if let Some(help) = &props.help_text {
                    p {
                        class: "text-xs text-slate-500",
                        "{help}"
                    }
                } else {
                    span {}
                }

                // Character count
                if props.show_count {
                    span {
                        class: "text-xs text-slate-500",
                        if let Some(max) = props.max_length {
                            "{char_count}/{max}"
                        } else {
                            "{char_count}"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Number Input Component
// ============================================================================

/// Properties for NumberInput component
#[derive(Props, Clone, PartialEq)]
pub struct NumberInputProps {
    /// Input value
    pub value: f64,

    /// Label text
    #[props(default)]
    pub label: Option<String>,

    /// Placeholder text
    #[props(default)]
    pub placeholder: Option<String>,

    /// Help text
    #[props(default)]
    pub help_text: Option<String>,

    /// Error message
    #[props(default)]
    pub error: Option<String>,

    /// Minimum value
    #[props(default)]
    pub min: Option<f64>,

    /// Maximum value
    #[props(default)]
    pub max: Option<f64>,

    /// Step value
    #[props(default = 1.0)]
    pub step: f64,

    /// Whether required
    #[props(default = false)]
    pub required: bool,

    /// Whether disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Whether to show increment/decrement buttons
    #[props(default = true)]
    pub show_controls: bool,

    /// Unit suffix (e.g., "px", "%", "ms")
    #[props(default)]
    pub unit: Option<String>,

    /// Additional CSS classes
    #[props(default)]
    pub class: Option<String>,

    /// Change handler
    #[props(default)]
    pub on_change: EventHandler<f64>,
}

/// Numeric input component with optional controls
#[component]
pub fn NumberInput(props: NumberInputProps) -> Element {
    let has_error = props.error.is_some();
    let input_class = build_input_class(has_error, props.disabled, &props.class);

    let increment = {
        let current = props.value;
        let step = props.step;
        let max = props.max;
        let on_change = props.on_change.clone();
        move |_| {
            let new_value = current + step;
            let clamped = if let Some(max) = max {
                new_value.min(max)
            } else {
                new_value
            };
            on_change.call(clamped);
        }
    };

    let decrement = {
        let current = props.value;
        let step = props.step;
        let min = props.min;
        let on_change = props.on_change.clone();
        move |_| {
            let new_value = current - step;
            let clamped = if let Some(min) = min {
                new_value.max(min)
            } else {
                new_value
            };
            on_change.call(clamped);
        }
    };

    rsx! {
        div {
            class: "input-group",

            // Label
            if let Some(label) = &props.label {
                label {
                    class: "block text-sm font-medium text-slate-300 mb-1.5",
                    "{label}"
                    if props.required {
                        span { class: "text-rose-400 ml-0.5", "*" }
                    }
                }
            }

            // Input with controls
            div {
                class: "relative flex items-center",

                // Decrement button
                if props.show_controls {
                    button {
                        class: "absolute left-0 h-full px-3 text-slate-400 hover:text-slate-200 hover:bg-slate-700/50 rounded-l-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                        r#type: "button",
                        disabled: props.disabled || props.min.is_some_and(|m| props.value <= m),
                        onclick: decrement,
                        "−"
                    }
                }

                // Input
                input {
                    class: "{input_class}",
                    class: if props.show_controls { "text-center px-10" } else { "" },
                    class: if props.unit.is_some() { "pr-12" } else { "" },
                    r#type: "number",
                    value: "{props.value}",
                    placeholder: props.placeholder.as_deref().unwrap_or(""),
                    disabled: props.disabled,
                    min: props.min.map(|v| v.to_string()),
                    max: props.max.map(|v| v.to_string()),
                    step: "{props.step}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<f64>() {
                            let clamped = clamp_value(v, props.min, props.max);
                            props.on_change.call(clamped);
                        }
                    },
                }

                // Unit suffix
                if let Some(unit) = &props.unit {
                    span {
                        class: "absolute right-10 text-slate-400 text-sm pointer-events-none",
                        "{unit}"
                    }
                }

                // Increment button
                if props.show_controls {
                    button {
                        class: "absolute right-0 h-full px-3 text-slate-400 hover:text-slate-200 hover:bg-slate-700/50 rounded-r-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                        r#type: "button",
                        disabled: props.disabled || props.max.is_some_and(|m| props.value >= m),
                        onclick: increment,
                        "+"
                    }
                }
            }

            // Help text or error
            if let Some(error) = &props.error {
                p {
                    class: "mt-1 text-xs text-rose-400",
                    "{error}"
                }
            } else if let Some(help) = &props.help_text {
                p {
                    class: "mt-1 text-xs text-slate-500",
                    "{help}"
                }
            }
        }
    }
}

// ============================================================================
// Select Component
// ============================================================================

/// A single option for the Select component
#[derive(Clone, PartialEq, Debug)]
pub struct SelectOption {
    /// Option value
    pub value: String,
    /// Display label
    pub label: String,
    /// Whether disabled
    pub disabled: bool,
}

impl SelectOption {
    /// Create a new select option
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Create a disabled option
    pub fn disabled(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: true,
        }
    }
}

/// Properties for Select component
#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    /// Selected value
    pub value: String,

    /// Available options
    pub options: Vec<SelectOption>,

    /// Label text
    #[props(default)]
    pub label: Option<String>,

    /// Placeholder (shown when no selection)
    #[props(default)]
    pub placeholder: Option<String>,

    /// Help text
    #[props(default)]
    pub help_text: Option<String>,

    /// Error message
    #[props(default)]
    pub error: Option<String>,

    /// Whether required
    #[props(default = false)]
    pub required: bool,

    /// Whether disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Additional CSS classes
    #[props(default)]
    pub class: Option<String>,

    /// Change handler
    #[props(default)]
    pub on_change: EventHandler<String>,
}

/// Dropdown select component
#[component]
pub fn Select(props: SelectProps) -> Element {
    let has_error = props.error.is_some();

    // Build border color based on error state
    let border_color = if has_error {
        "border-color: rgb(244 63 94);"
    } else {
        "border-color: rgb(51 65 85);"
    };

    let disabled_style = if props.disabled {
        "opacity: 0.5; cursor: not-allowed;"
    } else {
        "cursor: pointer;"
    };

    rsx! {
        div {
            class: "input-group",

            // Label
            if let Some(label) = &props.label {
                label {
                    class: "block text-sm font-medium text-slate-300 mb-1.5",
                    "{label}"
                    if props.required {
                        span { class: "text-rose-400 ml-0.5", "*" }
                    }
                }
            }

            // Select wrapper
            div {
                class: "relative",

                select {
                    class: "w-full rounded-lg text-sm transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500/30",
                    style: "
                        padding: 0.5rem 2.5rem 0.5rem 0.75rem;
                        background-color: rgb(30 41 59);
                        color: rgb(241 245 249);
                        border: 1px solid;
                        {border_color}
                        {disabled_style}
                        -webkit-appearance: none;
                        -moz-appearance: none;
                        appearance: none;
                        background-image: url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%2394a3b8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E\");
                        background-repeat: no-repeat;
                        background-position: right 0.75rem center;
                        background-size: 1rem;
                    ",
                    disabled: props.disabled,
                    onchange: move |e| props.on_change.call(e.value()),

                    // Placeholder option
                    if let Some(placeholder) = &props.placeholder {
                        option {
                            value: "",
                            disabled: true,
                            selected: props.value.is_empty(),
                            style: "background-color: rgb(30 41 59); color: rgb(148 163 184);",
                            "{placeholder}"
                        }
                    }

                    // Options
                    for option in &props.options {
                        option {
                            key: "{option.value}",
                            value: "{option.value}",
                            disabled: option.disabled,
                            selected: props.value == option.value,
                            style: "background-color: rgb(30 41 59); color: rgb(241 245 249);",
                            "{option.label}"
                        }
                    }
                }
            }

            // Help text or error
            if let Some(error) = &props.error {
                p {
                    class: "mt-1 text-xs text-rose-400",
                    "{error}"
                }
            } else if let Some(help) = &props.help_text {
                p {
                    class: "mt-1 text-xs text-slate-500",
                    "{help}"
                }
            }
        }
    }
}

// ============================================================================
// Checkbox Component
// ============================================================================

/// Properties for Checkbox component
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    /// Whether checked
    pub checked: bool,

    /// Label text
    #[props(default)]
    pub label: Option<String>,

    /// Help text
    #[props(default)]
    pub help_text: Option<String>,

    /// Whether disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Whether indeterminate (partial selection)
    #[props(default = false)]
    pub indeterminate: bool,

    /// Additional CSS classes
    #[props(default)]
    pub class: Option<String>,

    /// Change handler
    #[props(default)]
    pub on_change: EventHandler<bool>,
}

/// Checkbox input component
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let checkbox_class = build_checkbox_class(props.disabled);

    rsx! {
        label {
            class: "checkbox-wrapper inline-flex items-start gap-2 cursor-pointer",
            class: if props.disabled { "opacity-50 cursor-not-allowed" } else { "" },

            // Checkbox input
            div {
                class: "relative flex items-center justify-center mt-0.5",

                input {
                    class: "sr-only peer",
                    r#type: "checkbox",
                    checked: props.checked,
                    disabled: props.disabled,
                    onchange: move |_| {
                        if !props.disabled {
                            props.on_change.call(!props.checked);
                        }
                    },
                }

                // Custom checkbox visual
                div {
                    class: "{checkbox_class}",

                    // Checkmark
                    if props.checked && !props.indeterminate {
                        svg {
                            class: "w-3 h-3 text-white",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.5",
                            view_box: "0 0 24 24",
                            path {
                                d: "M5 13l4 4L19 7",
                            }
                        }
                    }

                    // Indeterminate mark
                    if props.indeterminate {
                        div {
                            class: "w-2 h-0.5 bg-white rounded-full",
                        }
                    }
                }
            }

            // Label and help text
            if props.label.is_some() || props.help_text.is_some() {
                div {
                    class: "flex flex-col",

                    if let Some(label) = &props.label {
                        span {
                            class: "text-sm text-slate-200",
                            "{label}"
                        }
                    }

                    if let Some(help) = &props.help_text {
                        span {
                            class: "text-xs text-slate-500 mt-0.5",
                            "{help}"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Toggle Component
// ============================================================================

/// Properties for Toggle component
#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    /// Whether on
    pub checked: bool,

    /// Label text
    #[props(default)]
    pub label: Option<String>,

    /// Help text
    #[props(default)]
    pub help_text: Option<String>,

    /// Whether disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Size variant
    #[props(default = ToggleSize::Medium)]
    pub size: ToggleSize,

    /// Additional CSS classes
    #[props(default)]
    pub class: Option<String>,

    /// Change handler
    #[props(default)]
    pub on_change: EventHandler<bool>,
}

/// Toggle size variants
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ToggleSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ToggleSize {
    fn track_class(&self) -> &'static str {
        match self {
            ToggleSize::Small => "w-8 h-4",
            ToggleSize::Medium => "w-10 h-5",
            ToggleSize::Large => "w-12 h-6",
        }
    }

    fn thumb_class(&self) -> &'static str {
        match self {
            ToggleSize::Small => "w-3 h-3",
            ToggleSize::Medium => "w-4 h-4",
            ToggleSize::Large => "w-5 h-5",
        }
    }

    fn translate_class(&self) -> &'static str {
        match self {
            ToggleSize::Small => "translate-x-4",
            ToggleSize::Medium => "translate-x-5",
            ToggleSize::Large => "translate-x-6",
        }
    }
}

/// Toggle switch component - styled as a clickable card/button
#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    // Card background color based on state
    let card_bg = if props.checked {
        "bg-indigo-600/10 border-indigo-500/50"
    } else {
        "bg-slate-800/50 border-slate-600/50"
    };

    let hover_class = if props.disabled {
        ""
    } else {
        "hover:bg-slate-700/50 hover:border-slate-500"
    };

    let disabled_class = if props.disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    let handle_click = move |_| {
        if !props.disabled {
            props.on_change.call(!props.checked);
        }
    };

    rsx! {
        div {
            class: "toggle-card flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-all select-none {card_bg} {hover_class} {disabled_class}",
            onclick: handle_click,

            // Checkbox visual
            div {
                class: "flex-shrink-0 w-5 h-5 rounded border-2 flex items-center justify-center transition-colors",
                class: if props.checked { "bg-indigo-600 border-indigo-600" } else { "bg-transparent border-slate-500" },

                // Checkmark
                if props.checked {
                    svg {
                        class: "w-3 h-3 text-white",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "3",
                        view_box: "0 0 24 24",
                        path {
                            d: "M5 13l4 4L19 7",
                        }
                    }
                }
            }

            // Label and help text
            if props.label.is_some() || props.help_text.is_some() {
                div {
                    class: "flex flex-col min-w-0 flex-1",

                    if let Some(label) = &props.label {
                        span {
                            class: "text-sm font-medium text-slate-200 leading-tight",
                            "{label}"
                        }
                    }

                    if let Some(help) = &props.help_text {
                        span {
                            class: "text-xs text-slate-400 mt-0.5 leading-tight",
                            "{help}"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Form Group Component
// ============================================================================

/// Properties for FormGroup component
#[derive(Props, Clone, PartialEq)]
pub struct FormGroupProps {
    /// Group label
    #[props(default)]
    pub label: Option<String>,

    /// Group description
    #[props(default)]
    pub description: Option<String>,

    /// Whether the group is required
    #[props(default = false)]
    pub required: bool,

    /// Children
    pub children: Element,
}

/// Form group wrapper component
#[component]
pub fn FormGroup(props: FormGroupProps) -> Element {
    rsx! {
        div {
            class: "form-group space-y-1.5",

            // Label
            if let Some(label) = &props.label {
                label {
                    class: "block text-sm font-medium text-slate-300",
                    "{label}"
                    if props.required {
                        span { class: "text-rose-400 ml-0.5", "*" }
                    }
                }
            }

            // Description
            if let Some(desc) = &props.description {
                p {
                    class: "text-xs text-slate-500",
                    "{desc}"
                }
            }

            // Children
            {props.children}
        }
    }
}

// ============================================================================
// Button Group Component
// ============================================================================

/// Properties for ButtonGroup component
#[derive(Props, Clone, PartialEq)]
pub struct ButtonGroupProps {
    /// Selected value
    pub value: String,

    /// Available options
    pub options: Vec<SelectOption>,

    /// Whether disabled
    #[props(default = false)]
    pub disabled: bool,

    /// Change handler
    #[props(default)]
    pub on_change: EventHandler<String>,
}

/// Button group for mutually exclusive options
#[component]
pub fn ButtonGroup(props: ButtonGroupProps) -> Element {
    rsx! {
        div {
            class: "button-group inline-flex rounded-lg overflow-hidden border border-slate-700",

            for (i, option) in props.options.iter().enumerate() {
                button {
                    key: "{option.value}",
                    class: "px-3 py-1.5 text-sm transition-colors",
                    class: if props.value == option.value {
                        "bg-indigo-600 text-white"
                    } else {
                        "bg-slate-800 text-slate-300 hover:bg-slate-700"
                    },
                    class: if i > 0 { "border-l border-slate-700" } else { "" },
                    disabled: props.disabled || option.disabled,
                    onclick: {
                        let value = option.value.clone();
                        move |_| props.on_change.call(value.clone())
                    },
                    "{option.label}"
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Build input class string
fn build_input_class(has_error: bool, disabled: bool, extra: &Option<String>) -> String {
    let mut classes = vec![
        "w-full",
        "px-3",
        "py-2",
        "bg-slate-800",
        "border",
        "rounded-lg",
        "text-sm",
        "text-slate-100",
        "placeholder-slate-500",
        "transition-colors",
        "focus:outline-none",
        "focus:ring-2",
    ];

    if has_error {
        classes.push("border-rose-500");
        classes.push("focus:ring-rose-500/30");
        classes.push("focus:border-rose-500");
    } else {
        classes.push("border-slate-700");
        classes.push("focus:ring-indigo-500/30");
        classes.push("focus:border-indigo-500");
    }

    if disabled {
        classes.push("opacity-50");
        classes.push("cursor-not-allowed");
    }

    let mut result = classes.join(" ");
    if let Some(extra) = extra {
        result.push(' ');
        result.push_str(extra);
    }

    result
}

/// Build textarea class string
fn build_textarea_class(
    has_error: bool,
    disabled: bool,
    resizable: bool,
    extra: &Option<String>,
) -> String {
    let mut class = build_input_class(has_error, disabled, extra);

    if !resizable {
        class.push_str(" resize-none");
    } else {
        class.push_str(" resize-y");
    }

    class
}

/// Build select class string (kept for backwards compatibility but not used by Select component)
#[allow(dead_code)]
fn build_select_class(has_error: bool, disabled: bool, extra: &Option<String>) -> String {
    let mut class = build_input_class(has_error, disabled, extra);
    class.push_str(" appearance-none pr-10 cursor-pointer");
    class
}

/// Build checkbox class string
fn build_checkbox_class(disabled: bool) -> String {
    let mut classes = vec![
        "w-4",
        "h-4",
        "rounded",
        "border-2",
        "transition-colors",
        "flex",
        "items-center",
        "justify-center",
        "peer-checked:bg-indigo-600",
        "peer-checked:border-indigo-600",
        "peer-focus:ring-2",
        "peer-focus:ring-indigo-500/30",
    ];

    if disabled {
        classes.push("border-slate-600");
        classes.push("bg-slate-700");
    } else {
        classes.push("border-slate-500");
        classes.push("bg-slate-800");
        classes.push("hover:border-slate-400");
    }

    classes.join(" ")
}

/// Clamp a value between optional min and max
fn clamp_value(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    let mut result = value;
    if let Some(min) = min {
        result = result.max(min);
    }
    if let Some(max) = max {
        result = result.min(max);
    }
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_input_class() {
        let class = build_input_class(false, false, &None);
        assert!(class.contains("border-slate-700"));
        assert!(!class.contains("border-rose-500"));
        assert!(!class.contains("opacity-50"));
    }

    #[test]
    fn test_build_input_class_error() {
        let class = build_input_class(true, false, &None);
        assert!(class.contains("border-rose-500"));
    }

    #[test]
    fn test_build_input_class_disabled() {
        let class = build_input_class(false, true, &None);
        assert!(class.contains("opacity-50"));
        assert!(class.contains("cursor-not-allowed"));
    }

    #[test]
    fn test_build_textarea_class_resizable() {
        let class = build_textarea_class(false, false, true, &None);
        assert!(class.contains("resize-y"));
    }

    #[test]
    fn test_build_textarea_class_not_resizable() {
        let class = build_textarea_class(false, false, false, &None);
        assert!(class.contains("resize-none"));
    }

    #[test]
    fn test_select_option_new() {
        let opt = SelectOption::new("val", "Label");
        assert_eq!(opt.value, "val");
        assert_eq!(opt.label, "Label");
        assert!(!opt.disabled);
    }

    #[test]
    fn test_select_option_disabled() {
        let opt = SelectOption::disabled("val", "Label");
        assert!(opt.disabled);
    }

    #[test]
    fn test_clamp_value() {
        assert_eq!(clamp_value(5.0, Some(0.0), Some(10.0)), 5.0);
        assert_eq!(clamp_value(-5.0, Some(0.0), Some(10.0)), 0.0);
        assert_eq!(clamp_value(15.0, Some(0.0), Some(10.0)), 10.0);
        assert_eq!(clamp_value(5.0, None, None), 5.0);
    }

    #[test]
    fn test_toggle_size_classes() {
        assert!(ToggleSize::Small.track_class().contains("w-8"));
        assert!(ToggleSize::Medium.track_class().contains("w-10"));
        assert!(ToggleSize::Large.track_class().contains("w-12"));
    }
}
