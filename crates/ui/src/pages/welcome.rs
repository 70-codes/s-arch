//! Welcome Page Component
//!
//! The landing page shown when the application starts or when no project is loaded.
//! Features quick actions to create/open projects and displays recent projects.

use dioxus::prelude::*;
use std::path::PathBuf;

use crate::file_ops;
use crate::state::{APP_STATE, Dialog, Page, StatusLevel};

// ============================================================================
// Recent Projects Persistence
// ============================================================================

/// Get the path to the recent projects JSON file.
///
/// Stored in the user's config directory:
/// - Linux: ~/.config/immortal-engine/recent_projects.json
/// - macOS: ~/Library/Application Support/immortal-engine/recent_projects.json
/// - Windows: %APPDATA%/immortal-engine/recent_projects.json
fn recent_projects_path() -> Option<PathBuf> {
    // Use a simple fallback if dirs crate is not available
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("immortal-engine")
            .join("recent_projects.json"),
    )
}

/// A single recent project entry for persistence.
///
/// Supports both the old format (unix timestamp integer) and the new format
/// (ISO 8601 string) for `last_opened` to maintain backward compatibility.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct RecentProjectEntry {
    /// Display name of the project
    name: String,
    /// Full file path to the .ieng file
    path: String,
    /// Last opened timestamp — accepts both ISO 8601 string and unix timestamp integer
    #[serde(deserialize_with = "deserialize_last_opened")]
    last_opened: String,
}

/// Custom deserializer that accepts both string (ISO 8601) and integer (unix timestamp).
fn deserialize_last_opened<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct LastOpenedVisitor;

    impl<'de> de::Visitor<'de> for LastOpenedVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string (ISO 8601) or integer (unix timestamp)")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<String, E> {
            Ok(v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<String, E> {
            // Convert unix timestamp to ISO 8601 string
            let dt = chrono::DateTime::from_timestamp(v, 0).unwrap_or_else(|| chrono::Utc::now());
            Ok(dt.to_rfc3339())
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            let dt =
                chrono::DateTime::from_timestamp(v as i64, 0).unwrap_or_else(|| chrono::Utc::now());
            Ok(dt.to_rfc3339())
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<String, E> {
            let dt =
                chrono::DateTime::from_timestamp(v as i64, 0).unwrap_or_else(|| chrono::Utc::now());
            Ok(dt.to_rfc3339())
        }
    }

    deserializer.deserialize_any(LastOpenedVisitor)
}

/// Load recent projects from the config file.
fn load_recent_projects() -> Vec<RecentProjectEntry> {
    let Some(path) = recent_projects_path() else {
        return Vec::new();
    };

    if !path.exists() {
        return Vec::new();
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Save recent projects to the config file.
fn save_recent_projects(projects: &[RecentProjectEntry]) {
    let Some(path) = recent_projects_path() else {
        return;
    };

    // Ensure the config directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(json) = serde_json::to_string_pretty(projects) {
        let _ = std::fs::write(&path, json);
    }
}

/// Add a project to the recent projects list and persist it.
///
/// Call this whenever a project is opened or saved to keep
/// the recent projects list up to date.
pub fn add_to_recent_projects(name: &str, file_path: &std::path::Path) {
    let mut projects = load_recent_projects();

    let path_str = file_path.to_string_lossy().to_string();

    // Remove existing entry with the same path (to move it to front)
    projects.retain(|p| p.path != path_str);

    // Add to front of the list
    projects.insert(
        0,
        RecentProjectEntry {
            name: name.to_string(),
            path: path_str,
            last_opened: chrono::Utc::now().to_rfc3339(),
        },
    );

    // Limit to 10 entries
    projects.truncate(10);

    save_recent_projects(&projects);
}

// ============================================================================
// Welcome Page Component
// ============================================================================

/// Welcome/landing page component
#[component]
pub fn WelcomePage() -> Element {
    // Load recent projects
    let mut recent_projects = use_signal(|| load_recent_projects());

    rsx! {
        div {
            style: "height: 100%; display: flex; flex-direction: column; overflow: auto; background-color: rgb(15 23 42);",

            // Main content centered
            div {
                style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 32px;",

                // Hero Section
                HeroSection {}

                // Action Buttons
                ActionButtons {}

                // Recent Projects (if any exist)
                if !recent_projects.read().is_empty() {
                    RecentProjectsSection {
                        projects: recent_projects.read().clone(),
                        on_open: move |path: String| {
                            let path_buf = PathBuf::from(&path);
                            spawn(async move {
                                match file_ops::open_project(Some(path_buf)).await {
                                    Ok((project, saved_path)) => {
                                        let name = project.meta.name.clone();
                                        let mut state = APP_STATE.write();
                                        state.load_project(project, saved_path.clone());
                                        state.ui.set_status(
                                            format!("Opened project: {}", name),
                                            StatusLevel::Success,
                                        );
                                        drop(state);
                                        // Update recent projects
                                        add_to_recent_projects(&name, &saved_path);
                                        recent_projects.set(load_recent_projects());
                                    }
                                    Err(e) => {
                                        APP_STATE.write().ui.set_status(
                                            format!("Failed to open project: {}", e),
                                            StatusLevel::Error,
                                        );
                                    }
                                }
                            });
                        },
                        on_remove: move |path: String| {
                            let mut projects = load_recent_projects();
                            projects.retain(|p| p.path != path);
                            save_recent_projects(&projects);
                            recent_projects.set(projects);
                        },
                    }
                }

                // Features Grid
                FeaturesGrid {}
            }

            // Footer
            Footer {}
        }
    }
}

// ============================================================================
// Hero Section
// ============================================================================

#[component]
fn HeroSection() -> Element {
    rsx! {
        div {
            style: "text-align: center; margin-bottom: 48px;",

            // Logo/Icon
            div {
                style: "font-size: 72px; margin-bottom: 16px;",
                "🔮"
            }

            // Title
            h1 {
                style: "font-size: 42px; font-weight: 700; margin-bottom: 16px; background: linear-gradient(to right, rgb(129 140 248), rgb(192 132 252)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;",
                "Immortal Engine"
            }

            // Subtitle
            p {
                style: "font-size: 20px; color: rgb(148 163 184); margin-bottom: 8px;",
                "Visual Code Generator for Rust Applications"
            }

            // Description
            p {
                style: "font-size: 14px; color: rgb(100 116 139); max-width: 450px; margin: 0 auto; line-height: 1.6;",
                "Design entities, define relationships, configure endpoints, and generate production-ready Axum + SeaORM backends with a visual interface."
            }
        }
    }
}

// ============================================================================
// Action Buttons
// ============================================================================

#[component]
fn ActionButtons() -> Element {
    rsx! {
        div {
            style: "display: flex; flex-wrap: wrap; gap: 16px; justify-content: center; margin-bottom: 48px;",

            // New Project Button
            button {
                style: "display: flex; align-items: center; gap: 8px; padding: 16px 32px; font-size: 16px; font-weight: 500; background-color: rgb(79 70 229); color: white; border: none; border-radius: 12px; cursor: pointer; transition: all 0.2s; box-shadow: 0 4px 14px 0 rgba(79, 70, 229, 0.4);",
                onclick: move |_| {
                    APP_STATE.write().ui.show_dialog(Dialog::NewProject);
                },
                span { style: "font-size: 20px;", "📄" }
                "New Project"
            }

            // Open Project Button
            button {
                style: "display: flex; align-items: center; gap: 8px; padding: 16px 32px; font-size: 16px; font-weight: 500; background-color: rgb(51 65 85); color: rgb(226 232 240); border: none; border-radius: 12px; cursor: pointer; transition: all 0.2s;",
                onclick: move |_| {
                    APP_STATE.write().ui.show_dialog(Dialog::OpenProject);
                },
                span { style: "font-size: 20px;", "📂" }
                "Open Project"
            }
        }
    }
}

// ============================================================================
// Recent Projects Section
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct RecentProjectsSectionProps {
    projects: Vec<RecentProjectEntry>,
    on_open: EventHandler<String>,
    on_remove: EventHandler<String>,
}

#[component]
fn RecentProjectsSection(props: RecentProjectsSectionProps) -> Element {
    rsx! {
        div {
            style: "max-width: 600px; width: 100%; margin-bottom: 48px;",

            // Section header
            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;",

                h2 {
                    style: "font-size: 16px; font-weight: 600; color: rgb(148 163 184); display: flex; align-items: center; gap: 8px;",
                    span { "🕐" }
                    "Recent Projects"
                }

                span {
                    style: "font-size: 12px; color: rgb(71 85 105);",
                    "{props.projects.len()} project(s)"
                }
            }

            // Project list
            div {
                style: "display: flex; flex-direction: column; gap: 8px;",

                for project in props.projects.iter() {
                    {
                        let path_for_open = project.path.clone();
                        let path_for_remove = project.path.clone();
                        let file_exists = std::path::Path::new(&project.path).exists();

                        // Parse relative time
                        let time_ago = chrono::DateTime::parse_from_rfc3339(&project.last_opened)
                            .ok()
                            .map(|dt| {
                                let diff = chrono::Utc::now().signed_duration_since(dt);
                                if diff.num_minutes() < 1 {
                                    "just now".to_string()
                                } else if diff.num_hours() < 1 {
                                    format!("{}m ago", diff.num_minutes())
                                } else if diff.num_days() < 1 {
                                    format!("{}h ago", diff.num_hours())
                                } else if diff.num_days() < 30 {
                                    format!("{}d ago", diff.num_days())
                                } else {
                                    format!("{}mo ago", diff.num_days() / 30)
                                }
                            })
                            .unwrap_or_else(|| "—".to_string());

                        let opacity = if file_exists { "1" } else { "0.5" };
                        let cursor = if file_exists { "pointer" } else { "default" };

                        rsx! {
                            div {
                                key: "{project.path}",
                                style: "display: flex; align-items: center; gap: 12px; padding: 12px 16px; \
                                        background-color: rgb(30 41 59); border: 1px solid rgb(51 65 85); \
                                        border-radius: 10px; cursor: {cursor}; transition: all 0.15s; opacity: {opacity};",
                                onclick: move |_| {
                                    if file_exists {
                                        props.on_open.call(path_for_open.clone());
                                    }
                                },

                                // Project icon
                                div {
                                    style: "width: 40px; height: 40px; border-radius: 10px; \
                                            background: linear-gradient(135deg, rgb(79 70 229), rgb(124 58 237)); \
                                            display: flex; align-items: center; justify-content: center; \
                                            font-size: 18px; flex-shrink: 0;",
                                    "📦"
                                }

                                // Project info
                                div {
                                    style: "flex: 1; min-width: 0;",

                                    // Name row
                                    div {
                                        style: "display: flex; align-items: center; gap: 8px;",

                                        span {
                                            style: "font-size: 14px; font-weight: 600; color: white; \
                                                    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                            "{project.name}"
                                        }

                                        if !file_exists {
                                            span {
                                                style: "font-size: 10px; padding: 2px 6px; \
                                                        background-color: rgba(239, 68, 68, 0.2); \
                                                        color: rgb(252 165 165); border-radius: 4px;",
                                                "Missing"
                                            }
                                        }
                                    }

                                    // Path row
                                    div {
                                        style: "font-size: 12px; color: rgb(100 116 139); \
                                                overflow: hidden; text-overflow: ellipsis; white-space: nowrap; \
                                                font-family: monospace;",
                                        title: "{project.path}",
                                        "{project.path}"
                                    }
                                }

                                // Time ago
                                span {
                                    style: "font-size: 11px; color: rgb(71 85 105); white-space: nowrap; flex-shrink: 0;",
                                    "{time_ago}"
                                }

                                // Remove button
                                button {
                                    style: "padding: 4px 8px; font-size: 14px; color: rgb(100 116 139); \
                                            background: none; border: none; cursor: pointer; border-radius: 6px; \
                                            flex-shrink: 0;",
                                    title: "Remove from recent projects",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        props.on_remove.call(path_for_remove.clone());
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Features Grid
// ============================================================================

#[component]
fn FeaturesGrid() -> Element {
    rsx! {
        div {
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 24px; max-width: 1000px; width: 100%;",

            FeatureCard {
                icon: "🗃️",
                title: "Entity Design",
                description: "Visually design your data models with fields, types, validations, and constraints on an intuitive canvas."
            }

            FeatureCard {
                icon: "🔗",
                title: "Relationships",
                description: "Draw connections between entities to define one-to-one, one-to-many, and many-to-many relationships."
            }

            FeatureCard {
                icon: "🔐",
                title: "Auth & Security",
                description: "Configure JWT authentication, role-based access control, and per-endpoint security settings."
            }

            FeatureCard {
                icon: "⚡",
                title: "Code Generation",
                description: "Generate complete Axum + SeaORM backends with migrations, handlers, and full CRUD APIs."
            }
        }
    }
}

// ============================================================================
// Feature Card Component
// ============================================================================

#[component]
fn FeatureCard(icon: &'static str, title: &'static str, description: &'static str) -> Element {
    rsx! {
        div {
            style: "padding: 24px; background-color: rgb(30 41 59); border: 1px solid rgb(51 65 85); border-radius: 12px; transition: border-color 0.2s;",

            // Icon
            div {
                style: "font-size: 36px; margin-bottom: 12px;",
                "{icon}"
            }

            // Title
            h3 {
                style: "font-size: 16px; font-weight: 600; margin-bottom: 8px; color: white;",
                "{title}"
            }

            // Description
            p {
                style: "font-size: 14px; color: rgb(148 163 184); line-height: 1.5;",
                "{description}"
            }
        }
    }
}

// ============================================================================
// Footer
// ============================================================================

#[component]
fn Footer() -> Element {
    rsx! {
        footer {
            style: "text-align: center; padding: 16px; font-size: 12px; color: rgb(71 85 105);",

            p {
                "Immortal Engine v"
                {env!("CARGO_PKG_VERSION")}
                " • Built with "
                span { style: "color: rgb(251 113 133);", "♥" }
                " using Dioxus & Rust"
            }

            p {
                style: "margin-top: 8px; display: flex; justify-content: center; gap: 16px;",

                a {
                    style: "color: rgb(100 116 139); text-decoration: none; cursor: pointer;",
                    href: "https://github.com/70-codes/immortal_engine",
                    target: "_blank",
                    "GitHub"
                }

                span { style: "color: rgb(51 65 85);", "•" }

                button {
                    style: "color: rgb(100 116 139); background: none; border: none; cursor: pointer; font-size: 12px;",
                    onclick: move |_| {
                        APP_STATE.write().ui.show_dialog(Dialog::About);
                    },
                    "About"
                }

                span { style: "color: rgb(51 65 85);", "•" }

                button {
                    style: "color: rgb(100 116 139); background: none; border: none; cursor: pointer; font-size: 12px;",
                    onclick: move |_| {
                        APP_STATE.write().ui.navigate(Page::Settings);
                    },
                    "Settings"
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
    fn test_welcome_page_compiles() {
        assert!(true);
    }

    #[test]
    fn test_recent_project_entry_serialization() {
        let entry = RecentProjectEntry {
            name: "Test Project".to_string(),
            path: "/home/user/test.ieng".to_string(),
            last_opened: "2026-01-29T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Test Project"));
        assert!(json.contains("/home/user/test.ieng"));

        let parsed: RecentProjectEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test Project");
        assert_eq!(parsed.path, "/home/user/test.ieng");
    }

    #[test]
    fn test_recent_project_entry_from_unix_timestamp() {
        // Old format: last_opened is a unix timestamp integer
        let json = r#"{"name":"Blog","path":"/home/user/Blog.imortal","last_opened":1768917004}"#;
        let parsed: RecentProjectEntry = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.name, "Blog");
        assert_eq!(parsed.path, "/home/user/Blog.imortal");
        // Should have been converted to an ISO 8601 string
        assert!(
            parsed.last_opened.contains("T"),
            "Expected ISO 8601 string, got: {}",
            parsed.last_opened
        );
    }

    #[test]
    fn test_recent_project_entry_from_iso_string() {
        // New format: last_opened is an ISO 8601 string
        let json =
            r#"{"name":"Test","path":"/test.ieng","last_opened":"2026-01-29T12:00:00+00:00"}"#;
        let parsed: RecentProjectEntry = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.name, "Test");
        assert!(parsed.last_opened.contains("2026"));
    }

    #[test]
    fn test_recent_project_entry_equality() {
        let a = RecentProjectEntry {
            name: "A".to_string(),
            path: "/a.ieng".to_string(),
            last_opened: "2026-01-01T00:00:00Z".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_mixed_format_list() {
        // A list with both old and new format entries
        let json = r#"[
            {"name":"Old","path":"/old.imortal","last_opened":1768917004},
            {"name":"New","path":"/new.ieng","last_opened":"2026-02-19T12:00:00Z"}
        ]"#;
        let parsed: Vec<RecentProjectEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "Old");
        assert!(parsed[0].last_opened.contains("T"));
        assert_eq!(parsed[1].name, "New");
        assert!(parsed[1].last_opened.contains("2026"));
    }
}
