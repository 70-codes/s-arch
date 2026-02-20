//! # Immortal UI
//!
//! Dioxus Desktop UI for Immortal Engine.
//!
//! This crate provides the visual editor interface for designing
//! and generating Rust applications.
//!
//! ## Features
//!
//! - Visual entity designer with drag-and-drop canvas
//! - Relationship drawing between entities
//! - Endpoint configuration with security settings
//! - Code generation preview and export
//!

// ============================================================================
// Modules
// ============================================================================

pub mod app;
pub mod components;
pub mod file_ops;
pub mod hooks;
pub mod pages;
pub mod state;

// ============================================================================
// Re-exports
// ============================================================================

// Re-export internal crates for convenience
pub use imortal_core;
pub use imortal_ir;

// Re-export main components
pub use app::App;
pub use file_ops::{
    RecentProject, RecentProjectsManager, open_project, save_project_as, save_project_to_file,
    show_export_directory_dialog, show_open_dialog, show_save_dialog,
};
pub use pages::{EndpointsPage, ProjectSetupPage, WelcomePage};
pub use state::{
    APP_STATE, AppState, CanvasState, ConnectionPort, DeleteTarget, Dialog, History,
    HistorySnapshot, Page, Selection, StatusLevel, StatusMessage, UiState, init_app_state,
};

// Re-export components
pub use components::{
    Canvas, CanvasToolbar, Checkbox, EndpointCard, EntityCard, FieldRow, GenerateEndpointsCard,
    NumberInput, PropertiesPanel, Select, SelectOption, TextArea, TextInput, Toggle,
};

// Re-export hooks
pub use hooks::{CanvasInteractions, DragState, PanState, use_canvas_interactions};

// ============================================================================
// Constants
// ============================================================================

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const NAME: &str = "Immortal Engine";

/// Application display title
pub const TITLE: &str = "Immortal Engine - Visual Code Generator";

/// CSS styles for the application
/// This is the compiled Tailwind CSS included at build time
const STYLES: &str = include_str!("../../../assets/styles/main.css");

// ============================================================================
// Launch Function
// ============================================================================

/// Launch the Immortal Engine desktop application
///
/// This is the main entry point for the Dioxus desktop app.
/// It initializes the application state and starts the UI.
///
/// # Example
///
/// ```rust,ignore
/// fn main() {
///     imortal_ui::launch();
/// }
/// ```
pub fn launch() {
    // Initialize logging for the UI
    tracing::info!("Starting {} v{}", NAME, VERSION);

    // Initialize application state
    init_app_state();

    // Build custom head with embedded CSS
    let custom_head = format!(r#"<style type="text/css">{}</style>"#, STYLES);

    // Configure and launch Dioxus desktop app
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title(TITLE)
                        .with_resizable(true)
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1400.0, 900.0))
                        .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0)),
                )
                .with_menu(None) // Disable default menu, we use custom toolbar
                .with_custom_head(custom_head),
        )
        .launch(App);
}

/// Launch with custom configuration
///
/// Allows specifying custom window size and title.
pub fn launch_with_config(title: &str, width: f64, height: f64) {
    tracing::info!("Starting {} v{} (custom config)", NAME, VERSION);

    init_app_state();

    let custom_head = format!(r#"<style type="text/css">{}</style>"#, STYLES);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title(title)
                        .with_resizable(true)
                        .with_inner_size(dioxus::desktop::LogicalSize::new(width, height))
                        .with_min_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0)),
                )
                .with_menu(None)
                .with_custom_head(custom_head),
        )
        .launch(App);
}

/// Get the embedded CSS styles
///
/// This can be used if you need to access the styles separately
pub fn get_styles() -> &'static str {
    STYLES
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_exists() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "Immortal Engine");
    }

    #[test]
    fn test_title() {
        assert!(TITLE.contains("Immortal Engine"));
    }

    #[test]
    fn test_styles_loaded() {
        // Verify CSS is loaded and contains expected content
        assert!(!STYLES.is_empty());
        assert!(STYLES.contains("tailwindcss"));
    }
}
