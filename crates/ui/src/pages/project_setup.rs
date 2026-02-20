//! Project Setup Page Component
//!
//! This page allows users to configure their project settings including
//! project metadata, database configuration, authentication options,
//! and other project-wide settings.

use dioxus::prelude::*;

use crate::file_ops;
use crate::state::{APP_STATE, Page, StatusLevel};
use imortal_ir::{AuthStrategy, DatabaseConfig, DatabaseType, ProjectConfig, ProjectType};

// ============================================================================
// Project Setup Page Component
// ============================================================================

/// Project setup/configuration page
#[component]
pub fn ProjectSetupPage() -> Element {
    // Get current project config from state
    let state = APP_STATE.read();
    let project = state.project.as_ref();

    let (initial_name, initial_desc, initial_config) = match project {
        Some(p) => (
            p.meta.name.clone(),
            p.meta.description.clone().unwrap_or_default(),
            p.config.clone(),
        ),
        None => (
            "New Project".to_string(),
            String::new(),
            ProjectConfig::default(),
        ),
    };
    drop(state);

    // Form state signals
    let mut project_name = use_signal(|| initial_name);
    let mut project_description = use_signal(|| initial_desc);
    let mut project_type = use_signal(|| initial_config.project_type);
    let mut database_type = use_signal(|| initial_config.database);
    let mut db_host = use_signal(|| initial_config.db_config.host.clone());
    let mut db_port = use_signal(|| initial_config.db_config.port);
    let mut db_username = use_signal(|| initial_config.db_config.username.clone());
    let mut db_password = use_signal(|| initial_config.db_config.password.clone());
    let mut db_name = use_signal(|| initial_config.db_config.database_name.clone());
    let mut db_max_conn = use_signal(|| initial_config.db_config.max_connections);
    let mut db_min_conn = use_signal(|| initial_config.db_config.min_connections);
    let mut db_ssl = use_signal(|| initial_config.db_config.ssl_enabled);
    let mut auth_enabled = use_signal(|| initial_config.auth.enabled);
    let mut auth_strategy = use_signal(|| initial_config.auth.strategy);
    let mut token_expiry_hours = use_signal(|| initial_config.auth.token_expiry_hours);
    let mut package_name = use_signal(|| initial_config.package_name.clone());

    // Project save location
    let initial_path = {
        let s = APP_STATE.read();
        s.project_path.clone()
    };
    let mut project_location: Signal<Option<std::path::PathBuf>> = use_signal(|| initial_path);

    // Connection test state
    let mut test_status: Signal<Option<(bool, String)>> = use_signal(|| None);
    let mut test_loading = use_signal(|| false);

    // Create database state
    let mut create_db_status: Signal<Option<(bool, String)>> = use_signal(|| None);
    let mut create_db_loading = use_signal(|| false);

    // Setup validation warnings
    let mut setup_warnings: Signal<Vec<String>> = use_signal(Vec::new);
    let mut show_warnings_dialog = use_signal(|| false);
    let mut proceed_after_warnings = use_signal(|| false);

    // Derived state for package name suggestion
    let suggested_package = use_memo(move || {
        project_name
            .read()
            .to_lowercase()
            .replace(' ', "_")
            .replace('-', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
    });

    // Save configuration helper function
    let do_save = move || {
        let mut state = APP_STATE.write();

        // Check if project exists
        if state.project.is_none() {
            return;
        }

        // Create snapshot from immutable reference first (before mutable borrow)
        let snapshot = state
            .project
            .as_ref()
            .and_then(|p| crate::state::HistorySnapshot::new("Update project config", p).ok());

        // Push snapshot to history
        if let Some(snap) = snapshot {
            state.history.push(snap);
        }

        // Now safely get mutable reference to project
        if let Some(project) = state.project.as_mut() {
            // Update project metadata
            project.meta.name = project_name.read().clone();
            project.meta.description =
                Some(project_description.read().clone()).filter(|s| !s.is_empty());
            project.touch();

            // Update project config
            project.config.project_type = *project_type.read();
            project.config.database = *database_type.read();
            project.config.db_config = DatabaseConfig {
                host: db_host.read().clone(),
                port: *db_port.read(),
                username: db_username.read().clone(),
                password: db_password.read().clone(),
                database_name: db_name.read().clone(),
                max_connections: *db_max_conn.read(),
                min_connections: *db_min_conn.read(),
                connect_timeout_secs: 30,
                idle_timeout_secs: 600,
                ssl_enabled: *db_ssl.read(),
            };
            project.config.auth.enabled = *auth_enabled.read();
            project.config.auth.strategy = *auth_strategy.read();
            project.config.auth.token_expiry_hours = *token_expiry_hours.read();
            project.config.package_name = package_name.read().clone();
        }

        state.mark_dirty();
        state
            .ui
            .set_status("Project configuration saved", StatusLevel::Success);
    };

    // Validate setup before proceeding
    let validate_setup = move || -> Vec<String> {
        let mut warnings = Vec::new();
        let current_db_type = *database_type.read();
        let name = project_name.read().clone();
        let pkg = package_name.read().clone();

        // Project name check
        if name.trim().is_empty() {
            warnings.push("Project name is empty.".to_string());
        }

        // Package name check
        if pkg.trim().is_empty() {
            warnings.push("Package name is empty.".to_string());
        }

        // Database connection checks
        match current_db_type {
            DatabaseType::SQLite => {
                if db_name.read().trim().is_empty() {
                    warnings.push("SQLite database file name is not set.".to_string());
                }
            }
            _ => {
                if db_host.read().trim().is_empty() {
                    warnings.push("Database host is not configured.".to_string());
                }
                if *db_port.read() == 0 {
                    warnings.push("Database port is not set.".to_string());
                }
                if db_username.read().trim().is_empty() {
                    warnings.push("Database username is not set.".to_string());
                }
                if db_name.read().trim().is_empty() {
                    warnings.push("Database name is not set.".to_string());
                }
                if db_password.read().trim().is_empty() {
                    warnings.push("Database password is empty — connection may fail.".to_string());
                }
            }
        }

        // Connection test check
        let test_result = test_status.read().clone();
        match test_result {
            None => {
                warnings.push(
                    "Database connection has not been tested. Click \"Test Connection\" to verify."
                        .to_string(),
                );
            }
            Some((false, _)) => {
                warnings.push(
                    "Database connection test failed. Fix the connection before proceeding."
                        .to_string(),
                );
            }
            Some((true, _)) => {
                // Connection test passed — no warning
            }
        }

        // Project location check
        if project_location.read().is_none() {
            warnings.push(
                "Project has not been saved yet. Set a project location or click Save first."
                    .to_string(),
            );
        }

        warnings
    };

    // Handle proceed after warnings
    if *proceed_after_warnings.read() {
        proceed_after_warnings.set(false);
        show_warnings_dialog.set(false);
        do_save();
        APP_STATE.write().ui.navigate(Page::EntityDesign);
    }

    rsx! {
        div {
            class: "project-setup-page h-full overflow-auto",

            div {
                class: "max-w-3xl mx-auto p-8",

                // Page Header
                PageHeader {}

                // Setup warnings dialog
                if *show_warnings_dialog.read() {
                    div {
                        class: "fixed inset-0 z-50 flex items-center justify-center",

                        // Backdrop
                        div {
                            class: "absolute inset-0 bg-black/60",
                            onclick: move |_| show_warnings_dialog.set(false),
                        }

                        // Dialog
                        div {
                            class: "relative bg-slate-800 rounded-xl shadow-xl border border-slate-700 p-6 max-w-lg w-full mx-4",
                            onclick: move |e| e.stop_propagation(),

                            // Header
                            div {
                                class: "flex items-center gap-3 mb-4",
                                span { class: "text-2xl", "⚠️" }
                                h2 { class: "text-xl font-bold text-amber-400", "Setup Warnings" }
                            }

                            p {
                                class: "text-sm text-slate-300 mb-4",
                                "The following issues were found with your project configuration:"
                            }

                            // Warning list
                            div {
                                class: "space-y-2 mb-6 max-h-60 overflow-y-auto",

                                for (i, warning) in setup_warnings.read().iter().enumerate() {
                                    div {
                                        key: "{i}",
                                        class: "flex items-start gap-2 p-3 bg-amber-900/20 border border-amber-700/30 rounded-lg text-sm",

                                        span { class: "text-amber-400 mt-0.5 flex-shrink-0", "•" }
                                        span { class: "text-amber-200", "{warning}" }
                                    }
                                }
                            }

                            // Actions
                            div {
                                class: "flex justify-end gap-3",

                                button {
                                    class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg transition-colors text-sm",
                                    onclick: move |_| show_warnings_dialog.set(false),
                                    "Go Back & Fix"
                                }

                                button {
                                    class: "px-4 py-2 bg-amber-600 hover:bg-amber-700 rounded-lg transition-colors text-sm font-medium",
                                    onclick: move |_| {
                                        proceed_after_warnings.set(true);
                                    },
                                    "Continue Anyway →"
                                }
                            }
                        }
                    }
                }

                // Form Sections
                form {
                    class: "space-y-8",
                    onsubmit: move |e| {
                        e.prevent_default();
                    },

                    // Project Information Section
                    FormSection {
                        title: "Project Information",
                        description: "Basic information about your project",

                        // Project Name
                        FormField {
                            label: "Project Name",
                            required: true,

                            input {
                                class: "input",
                                r#type: "text",
                                value: "{project_name}",
                                placeholder: "My Awesome API",
                                oninput: move |e| project_name.set(e.value()),
                            }
                        }

                        // Description
                        FormField {
                            label: "Description",
                            required: false,

                            textarea {
                                class: "input min-h-[80px] resize-y",
                                placeholder: "A brief description of your project...",
                                value: "{project_description}",
                                oninput: move |e| project_description.set(e.value()),
                            }
                        }

                        // Package Name
                        FormField {
                            label: "Package Name",
                            required: true,
                            hint: "Used for Cargo.toml and module names",

                            div {
                                class: "flex gap-2",

                                input {
                                    class: "input flex-1",
                                    r#type: "text",
                                    value: "{package_name}",
                                    placeholder: "my_awesome_api",
                                    oninput: move |e| package_name.set(e.value()),
                                }

                                button {
                                    class: "btn btn-secondary text-sm",
                                    r#type: "button",
                                    onclick: move |_| {
                                        package_name.set(suggested_package.read().clone());
                                    },
                                    "Auto"
                                }
                            }
                        }

                        // Project Location
                        FormField {
                            label: "Project Location",
                            required: false,
                            hint: "Where the project file (.ieng) is saved on disk",

                            div {
                                class: "flex gap-2 items-center",

                                div {
                                    class: "flex-1 px-4 py-2.5 bg-slate-800 border border-slate-700 rounded-lg text-sm font-mono truncate min-h-[40px] flex items-center",

                                    if let Some(path) = project_location.read().as_ref() {
                                        span {
                                            class: "text-slate-200",
                                            title: "{path.display()}",
                                            "{path.display()}"
                                        }
                                    } else {
                                        span {
                                            class: "text-slate-500 italic",
                                            "Not saved yet — click Browse or Save to set location"
                                        }
                                    }
                                }

                                button {
                                    class: "btn btn-secondary text-sm whitespace-nowrap",
                                    r#type: "button",
                                    onclick: move |_| {
                                        let current_name = project_name.read().clone();
                                        spawn(async move {
                                            let result = file_ops::show_save_dialog(
                                                Some(&current_name),
                                                None,
                                            ).await;

                                            if let Some(path) = result {
                                                // Update both local state and global state
                                                project_location.set(Some(path.clone()));
                                                let mut state = APP_STATE.write();
                                                state.project_path = Some(path.clone());

                                                // Also save the project immediately
                                                if let Some(project) = &state.project {
                                                    let project_clone = project.clone();
                                                    let path_clone = path.clone();
                                                    drop(state);

                                                    match imortal_ir::save_project(&project_clone, &path_clone) {
                                                        Ok(_) => {
                                                            let mut state = APP_STATE.write();
                                                            state.mark_saved(Some(path_clone.clone()));
                                                            state.ui.set_status(
                                                                format!("Project saved to {}", path_clone.display()),
                                                                StatusLevel::Success,
                                                            );
                                                        }
                                                        Err(e) => {
                                                            APP_STATE.write().ui.set_status(
                                                                format!("Failed to save: {}", e),
                                                                StatusLevel::Error,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    },
                                    "📁 Browse"
                                }

                                if project_location.read().is_some() {
                                    button {
                                        class: "btn btn-secondary text-sm whitespace-nowrap",
                                        r#type: "button",
                                        title: "Open containing folder",
                                        onclick: move |_| {
                                            if let Some(path) = project_location.read().as_ref() {
                                                if let Some(parent) = path.parent() {
                                                    let parent_str = parent.to_string_lossy().to_string();
                                                    spawn(async move {
                                                        let _ = tokio::process::Command::new("xdg-open")
                                                            .arg(&parent_str)
                                                            .spawn();
                                                    });
                                                }
                                            }
                                        },
                                        "📂 Open Folder"
                                    }
                                }
                            }
                        }
                    }

                    // Project Type Section
                    FormSection {
                        title: "Project Type",
                        description: "Choose what kind of project to generate",

                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                            ProjectTypeCard {
                                selected: *project_type.read() == ProjectType::RestApi,
                                icon: "🔌",
                                title: "REST API",
                                description: "Backend-only API with Axum and SeaORM. Perfect for microservices or when you have a separate frontend.",
                                onclick: move |_| project_type.set(ProjectType::RestApi),
                            }

                            ProjectTypeCard {
                                selected: *project_type.read() == ProjectType::Fullstack,
                                icon: "🖥️",
                                title: "Fullstack",
                                description: "Complete application with Dioxus frontend and Axum backend. Includes CRUD pages for all entities.",
                                onclick: move |_| project_type.set(ProjectType::Fullstack),
                            }
                        }
                    }

                    // Database Section
                    FormSection {
                        title: "Database",
                        description: "Select your database backend",

                        div {
                            class: "grid grid-cols-1 md:grid-cols-3 gap-4",

                            DatabaseCard {
                                selected: *database_type.read() == DatabaseType::PostgreSQL,
                                icon: "🐘",
                                name: "PostgreSQL",
                                description: "Recommended for production",
                                onclick: move |_| {
                                    database_type.set(DatabaseType::PostgreSQL);
                                    db_port.set(5432);
                                    db_username.set("postgres".to_string());
                                    test_status.set(None);
                                },
                            }

                            DatabaseCard {
                                selected: *database_type.read() == DatabaseType::MySQL,
                                icon: "🐬",
                                name: "MySQL",
                                description: "Popular relational database",
                                onclick: move |_| {
                                    database_type.set(DatabaseType::MySQL);
                                    db_port.set(3306);
                                    db_username.set("root".to_string());
                                    test_status.set(None);
                                },
                            }

                            DatabaseCard {
                                selected: *database_type.read() == DatabaseType::SQLite,
                                icon: "📁",
                                name: "SQLite",
                                description: "Great for development",
                                onclick: move |_| {
                                    database_type.set(DatabaseType::SQLite);
                                    db_port.set(0);
                                    db_host.set(String::new());
                                    db_username.set(String::new());
                                    db_password.set(String::new());
                                    test_status.set(None);
                                },
                            }
                        }
                    }

                    // Database Connection Details
                    FormSection {
                        title: "Database Connection",
                        description: "Configure connection details for your database",

                        if *database_type.read() == DatabaseType::SQLite {
                            // SQLite only needs a database name (file path)
                            FormField {
                                label: "Database File Name",
                                required: true,
                                hint: "The SQLite file will be created as ./[name].db in the project directory",

                                input {
                                    class: "input",
                                    r#type: "text",
                                    value: "{db_name}",
                                    placeholder: "my_app",
                                    oninput: move |e| {
                                        db_name.set(e.value());
                                        test_status.set(None);
                                    },
                                }
                            }
                        } else {
                            // PostgreSQL / MySQL need full connection details
                            div {
                                class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                                // Host
                                FormField {
                                    label: "Host",
                                    required: true,

                                    input {
                                        class: "input",
                                        r#type: "text",
                                        value: "{db_host}",
                                        placeholder: "localhost",
                                        oninput: move |e| {
                                            db_host.set(e.value());
                                            test_status.set(None);
                                        },
                                    }
                                }

                                // Port
                                FormField {
                                    label: "Port",
                                    required: true,

                                    input {
                                        class: "input",
                                        r#type: "number",
                                        min: "1",
                                        max: "65535",
                                        value: "{db_port}",
                                        oninput: move |e| {
                                            if let Ok(p) = e.value().parse::<u16>() {
                                                db_port.set(p);
                                                test_status.set(None);
                                            }
                                        },
                                    }
                                }

                                // Username
                                FormField {
                                    label: "Username",
                                    required: true,

                                    input {
                                        class: "input",
                                        r#type: "text",
                                        value: "{db_username}",
                                        placeholder: "postgres",
                                        oninput: move |e| {
                                            db_username.set(e.value());
                                            test_status.set(None);
                                        },
                                    }
                                }

                                // Password
                                FormField {
                                    label: "Password",
                                    required: false,
                                    hint: "Leave empty if no password is set",

                                    input {
                                        class: "input",
                                        r#type: "password",
                                        value: "{db_password}",
                                        placeholder: "••••••••",
                                        oninput: move |e| {
                                            db_password.set(e.value());
                                            test_status.set(None);
                                        },
                                    }
                                }

                                // Database Name
                                FormField {
                                    label: "Database Name",
                                    required: true,

                                    input {
                                        class: "input",
                                        r#type: "text",
                                        value: "{db_name}",
                                        placeholder: "my_app",
                                        oninput: move |e| {
                                            db_name.set(e.value());
                                            test_status.set(None);
                                        },
                                    }
                                }
                            }

                            // Pool settings (collapsible / advanced)
                            div {
                                class: "mt-4 p-4 bg-slate-800/30 rounded-lg border border-slate-700",

                                h4 {
                                    class: "font-medium mb-3 text-sm text-slate-300",
                                    "Connection Pool"
                                }

                                div {
                                    class: "grid grid-cols-2 md:grid-cols-3 gap-4",

                                    FormField {
                                        label: "Max Connections",
                                        required: false,

                                        input {
                                            class: "input",
                                            r#type: "number",
                                            min: "1",
                                            max: "200",
                                            value: "{db_max_conn}",
                                            oninput: move |e| {
                                                if let Ok(n) = e.value().parse::<u32>() {
                                                    db_max_conn.set(n);
                                                }
                                            },
                                        }
                                    }

                                    FormField {
                                        label: "Min Connections",
                                        required: false,

                                        input {
                                            class: "input",
                                            r#type: "number",
                                            min: "0",
                                            max: "50",
                                            value: "{db_min_conn}",
                                            oninput: move |e| {
                                                if let Ok(n) = e.value().parse::<u32>() {
                                                    db_min_conn.set(n);
                                                }
                                            },
                                        }
                                    }

                                    // SSL toggle
                                    div {
                                        class: "flex items-end pb-2",

                                        label {
                                            class: "flex items-center gap-2 cursor-pointer",

                                            input {
                                                r#type: "checkbox",
                                                class: "w-4 h-4 accent-indigo-500",
                                                checked: *db_ssl.read(),
                                                onchange: move |e| db_ssl.set(e.checked()),
                                            }

                                            span {
                                                class: "text-sm text-slate-300",
                                                "Enable SSL/TLS"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Connection URL preview + Test button
                        div {
                            class: "mt-4 p-4 bg-slate-900/50 rounded-lg border border-slate-700",

                            div {
                                class: "flex items-center justify-between mb-2",

                                h4 {
                                    class: "text-sm font-medium text-slate-400",
                                    "Connection URL"
                                }

                                // Action buttons
                                div {
                                    class: "flex items-center gap-2",

                                // Create Database button
                                button {
                                    class: format!(
                                        "px-3 py-1.5 text-sm font-medium rounded transition-colors {}",
                                        if *create_db_loading.read() {
                                            "bg-slate-700 text-slate-400 cursor-wait"
                                        } else {
                                            "bg-indigo-600 hover:bg-indigo-700 text-white cursor-pointer"
                                        }
                                    ),
                                    r#type: "button",
                                    disabled: *create_db_loading.read(),
                                    onclick: move |_| {
                                        let current_db_type = *database_type.read();
                                        let host = db_host.read().clone();
                                        let port = *db_port.read();
                                        let user = db_username.read().clone();
                                        let pass = db_password.read().clone();
                                        let dbname = db_name.read().clone();

                                        if dbname.trim().is_empty() {
                                            create_db_status.set(Some((false, "Database name cannot be empty.".to_string())));
                                            return;
                                        }

                                        create_db_loading.set(true);
                                        create_db_status.set(None);

                                        spawn(async move {
                                            let result = match current_db_type {
                                                DatabaseType::SQLite => {
                                                    // SQLite creates the file automatically — nothing to do
                                                    Ok("SQLite database will be created automatically when the app starts.".to_string())
                                                }
                                                DatabaseType::PostgreSQL => {
                                                    // Use psql to create the database
                                                    // --no-password (-w) prevents interactive prompt
                                                    let mut cmd = tokio::process::Command::new("psql");
                                                    cmd.arg("-h").arg(&host)
                                                       .arg("-p").arg(port.to_string())
                                                       .arg("-U").arg(&user)
                                                       .arg("-w")  // never prompt for password
                                                       .arg("-c").arg(format!("CREATE DATABASE \"{}\";", dbname))
                                                       .env("PGPASSWORD", &pass)
                                                       .stdin(std::process::Stdio::null());

                                                    match cmd.output().await {
                                                        Ok(output) => {
                                                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                                                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                                                            if output.status.success() {
                                                                Ok(format!("Database \"{}\" created successfully!", dbname))
                                                            } else if stderr.contains("already exists") {
                                                                Ok(format!("Database \"{}\" already exists — ready to use.", dbname))
                                                            } else {
                                                                Err(stderr.trim().to_string())
                                                            }
                                                        }
                                                        Err(e) => {
                                                            if e.kind() == std::io::ErrorKind::NotFound {
                                                                Err("'psql' command not found. Install postgresql client tools:\n  sudo dnf install postgresql".to_string())
                                                            } else {
                                                                Err(format!("Failed to run psql: {}", e))
                                                            }
                                                        }
                                                    }
                                                }
                                                DatabaseType::MySQL => {
                                                    // Use mysql CLI to create the database
                                                    let mut cmd = tokio::process::Command::new("mysql");
                                                    cmd.arg("-h").arg(&host)
                                                       .arg("-P").arg(port.to_string())
                                                       .arg("-u").arg(&user)
                                                       .arg("-e").arg(format!("CREATE DATABASE IF NOT EXISTS `{}`;", dbname))
                                                       .stdin(std::process::Stdio::null());

                                                    if !pass.is_empty() {
                                                        cmd.arg(format!("-p{}", pass));
                                                    }

                                                    match cmd.output().await {
                                                        Ok(output) => {
                                                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                                                            if output.status.success() {
                                                                Ok(format!("Database \"{}\" created successfully!", dbname))
                                                            } else {
                                                                Err(stderr.trim().to_string())
                                                            }
                                                        }
                                                        Err(e) => {
                                                            if e.kind() == std::io::ErrorKind::NotFound {
                                                                Err("'mysql' command not found. Install MySQL client tools:\n  sudo dnf install mysql".to_string())
                                                            } else {
                                                                Err(format!("Failed to run mysql: {}", e))
                                                            }
                                                        }
                                                    }
                                                }
                                            };

                                            match result {
                                                Ok(msg) => create_db_status.set(Some((true, msg))),
                                                Err(msg) => create_db_status.set(Some((false, msg))),
                                            }

                                            create_db_loading.set(false);
                                        });
                                    },

                                    if *create_db_loading.read() {
                                        "⏳ Creating…"
                                    } else {
                                        "🗄️ Create Database"
                                    }
                                }

                                // Test Connection button
                                // Uses psql/mysql CLI to actually authenticate — not just TCP
                                button {
                                    class: format!(
                                        "px-3 py-1.5 text-sm font-medium rounded transition-colors {}",
                                        if *test_loading.read() {
                                            "bg-slate-700 text-slate-400 cursor-wait"
                                        } else {
                                            "bg-emerald-600 hover:bg-emerald-700 text-white cursor-pointer"
                                        }
                                    ),
                                    r#type: "button",
                                    disabled: *test_loading.read(),
                                    onclick: move |_| {
                                        let current_db_type = *database_type.read();
                                        let host = db_host.read().clone();
                                        let port = *db_port.read();
                                        let user = db_username.read().clone();
                                        let pass = db_password.read().clone();
                                        let dbname = db_name.read().clone();

                                        test_loading.set(true);
                                        test_status.set(None);

                                        spawn(async move {
                                            let result = match current_db_type {
                                                DatabaseType::SQLite => {
                                                    if dbname.trim().is_empty() {
                                                        Err("Database file name cannot be empty.".to_string())
                                                    } else {
                                                        Ok("SQLite database file will be created on first run.".to_string())
                                                    }
                                                }
                                                DatabaseType::PostgreSQL => {
                                                    // Use psql to authenticate and run SELECT 1
                                                    // This verifies: server reachable, port correct,
                                                    // credentials valid, database exists
                                                    let mut cmd = tokio::process::Command::new("psql");
                                                    cmd.arg("-h").arg(&host)
                                                       .arg("-p").arg(port.to_string())
                                                       .arg("-U").arg(&user)
                                                       .arg("-d").arg(&dbname)
                                                       .arg("-w")  // never prompt for password
                                                       .arg("-c").arg("SELECT 1;")
                                                       .env("PGPASSWORD", &pass)
                                                       .stdin(std::process::Stdio::null());

                                                    match tokio::time::timeout(
                                                        std::time::Duration::from_secs(10),
                                                        cmd.output(),
                                                    ).await {
                                                        Ok(Ok(output)) => {
                                                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                                                            if output.status.success() {
                                                                Ok(format!(
                                                                    "Connected to PostgreSQL at {}:{} — database \"{}\" is accessible with user \"{}\".",
                                                                    host, port, dbname, user
                                                                ))
                                                            } else if stderr.contains("password authentication failed") {
                                                                Err(format!("Authentication failed: wrong password for user \"{}\".", user))
                                                            } else if stderr.contains("does not exist") && stderr.contains("database") {
                                                                Err(format!(
                                                                    "Database \"{}\" does not exist. Click \"Create Database\" first.",
                                                                    dbname
                                                                ))
                                                            } else if stderr.contains("could not connect") || stderr.contains("Connection refused") {
                                                                Err(format!(
                                                                    "Cannot reach PostgreSQL server at {}:{}. Is it running?",
                                                                    host, port
                                                                ))
                                                            } else if stderr.contains("role") && stderr.contains("does not exist") {
                                                                Err(format!("User \"{}\" does not exist on the server.", user))
                                                            } else {
                                                                Err(format!("Connection failed: {}", stderr.trim()))
                                                            }
                                                        }
                                                        Ok(Err(e)) => {
                                                            if e.kind() == std::io::ErrorKind::NotFound {
                                                                Err("'psql' not found. Install it with: sudo dnf install postgresql".to_string())
                                                            } else {
                                                                Err(format!("Failed to run psql: {}", e))
                                                            }
                                                        }
                                                        Err(_) => {
                                                            Err(format!("Connection to {}:{} timed out after 10 seconds.", host, port))
                                                        }
                                                    }
                                                }
                                                DatabaseType::MySQL => {
                                                    // Use mysql CLI to authenticate
                                                    let mut cmd = tokio::process::Command::new("mysql");
                                                    cmd.arg("-h").arg(&host)
                                                       .arg("-P").arg(port.to_string())
                                                       .arg("-u").arg(&user)
                                                       .arg(&dbname)
                                                       .arg("-e").arg("SELECT 1;")
                                                       .arg("--connect-timeout=10")
                                                       .stdin(std::process::Stdio::null());

                                                    if !pass.is_empty() {
                                                        cmd.arg(format!("-p{}", pass));
                                                    }

                                                    match tokio::time::timeout(
                                                        std::time::Duration::from_secs(15),
                                                        cmd.output(),
                                                    ).await {
                                                        Ok(Ok(output)) => {
                                                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                                                            if output.status.success() {
                                                                Ok(format!(
                                                                    "Connected to MySQL at {}:{} — database \"{}\" is accessible with user \"{}\".",
                                                                    host, port, dbname, user
                                                                ))
                                                            } else if stderr.contains("Access denied") {
                                                                Err(format!("Authentication failed: wrong password for user \"{}\".", user))
                                                            } else if stderr.contains("Unknown database") {
                                                                Err(format!(
                                                                    "Database \"{}\" does not exist. Click \"Create Database\" first.",
                                                                    dbname
                                                                ))
                                                            } else if stderr.contains("Can't connect") {
                                                                Err(format!(
                                                                    "Cannot reach MySQL server at {}:{}. Is it running?",
                                                                    host, port
                                                                ))
                                                            } else {
                                                                Err(format!("Connection failed: {}", stderr.trim()))
                                                            }
                                                        }
                                                        Ok(Err(e)) => {
                                                            if e.kind() == std::io::ErrorKind::NotFound {
                                                                Err("'mysql' not found. Install it with: sudo dnf install mysql".to_string())
                                                            } else {
                                                                Err(format!("Failed to run mysql: {}", e))
                                                            }
                                                        }
                                                        Err(_) => {
                                                            Err(format!("Connection to {}:{} timed out after 10 seconds.", host, port))
                                                        }
                                                    }
                                                }
                                            };

                                            match result {
                                                Ok(msg) => test_status.set(Some((true, msg))),
                                                Err(msg) => test_status.set(Some((false, msg))),
                                            }

                                            test_loading.set(false);
                                        });
                                    },

                                    if *test_loading.read() {
                                        "⏳ Testing…"
                                    } else {
                                        "🔌 Test Connection"
                                    }
                                }

                                } // close action buttons div
                            }

                            // Display the connection URL (password masked)
                            {
                                let db_cfg = DatabaseConfig {
                                    host: db_host.read().clone(),
                                    port: *db_port.read(),
                                    username: db_username.read().clone(),
                                    password: db_password.read().clone(),
                                    database_name: db_name.read().clone(),
                                    max_connections: *db_max_conn.read(),
                                    min_connections: *db_min_conn.read(),
                                    connect_timeout_secs: 30,
                                    idle_timeout_secs: 600,
                                    ssl_enabled: *db_ssl.read(),
                                };
                                let display_url = db_cfg.display_url(*database_type.read());
                                rsx! {
                                    p {
                                        class: "font-mono text-sm text-slate-300 bg-slate-800 px-3 py-2 rounded select-all break-all",
                                        "{display_url}"
                                    }
                                }
                            }

                            // Test result
                            if let Some((success, message)) = test_status.read().as_ref() {
                                div {
                                    class: format!(
                                        "mt-3 p-3 rounded-lg text-sm {}",
                                        if *success {
                                            "bg-emerald-900/30 border border-emerald-700 text-emerald-300"
                                        } else {
                                            "bg-red-900/30 border border-red-700 text-red-300"
                                        }
                                    ),

                                    div {
                                        class: "flex items-start gap-2",

                                        span {
                                            if *success { "✅" } else { "❌" }
                                        }

                                        span {
                                            "{message}"
                                        }
                                    }

                                    if !*success {
                                        p {
                                            class: "mt-2 text-xs text-slate-500",
                                            "Make sure the database server is running and the host/port are correct."
                                        }
                                    }
                                }
                            }

                            // Create database result
                            if let Some((success, message)) = create_db_status.read().as_ref() {
                                div {
                                    class: format!(
                                        "mt-3 p-3 rounded-lg text-sm {}",
                                        if *success {
                                            "bg-indigo-900/30 border border-indigo-700 text-indigo-300"
                                        } else {
                                            "bg-red-900/30 border border-red-700 text-red-300"
                                        }
                                    ),

                                    div {
                                        class: "flex items-start gap-2",

                                        span {
                                            if *success { "✅" } else { "❌" }
                                        }

                                        span {
                                            class: "whitespace-pre-wrap",
                                            "{message}"
                                        }
                                    }
                                }
                            }

                            p {
                                class: "mt-2 text-xs text-slate-500",
                                "This URL will be written to the generated .env.example file. "
                                "Use \"Create Database\" to create it on the server, then \"Test Connection\" to verify."
                            }
                        }
                    }

                    // Authentication Section
                    FormSection {
                        title: "Authentication",
                        description: "Configure user authentication for your API",

                        // Enable Auth Toggle
                        div {
                            class: "flex items-center justify-between p-4 bg-slate-800/50 rounded-lg mb-4",

                            div {
                                h4 { class: "font-medium", "Enable Authentication" }
                                p { class: "text-sm text-slate-400", "Add JWT-based authentication to your API" }
                            }

                            ToggleSwitch {
                                enabled: *auth_enabled.read(),
                                onchange: move |enabled| auth_enabled.set(enabled),
                            }
                        }

                        // Auth Strategy (shown when auth is enabled)
                        if *auth_enabled.read() {
                            div {
                                class: "grid grid-cols-1 md:grid-cols-2 gap-4 mt-4",

                                AuthStrategyCard {
                                    selected: *auth_strategy.read() == AuthStrategy::Jwt,
                                    icon: "🔑",
                                    name: "JWT Tokens",
                                    description: "Stateless authentication with JSON Web Tokens. Best for APIs and SPAs.",
                                    onclick: move |_| auth_strategy.set(AuthStrategy::Jwt),
                                }

                                AuthStrategyCard {
                                    selected: *auth_strategy.read() == AuthStrategy::Session,
                                    icon: "🍪",
                                    name: "Session-based",
                                    description: "Traditional session cookies. Best for server-rendered applications.",
                                    onclick: move |_| auth_strategy.set(AuthStrategy::Session),
                                }
                            }

                            // JWT Options
                            if *auth_strategy.read() == AuthStrategy::Jwt {
                                div {
                                    class: "mt-4 p-4 bg-slate-800/30 rounded-lg border border-slate-700",

                                    h4 {
                                        class: "font-medium mb-3 text-sm",
                                        "JWT Configuration"
                                    }

                                    // Token Expiry — editable
                                    div {
                                        class: "mb-4",
                                        label {
                                            class: "block text-slate-400 mb-1 text-sm",
                                            "Token Expiry (hours)"
                                        }
                                        div {
                                            class: "flex items-center gap-3",
                                            input {
                                                class: "w-32 px-3 py-1.5 bg-slate-700 border border-slate-600 rounded font-mono text-sm text-slate-200 focus:outline-none focus:border-indigo-500",
                                                r#type: "number",
                                                min: "1",
                                                max: "8760",
                                                value: "{token_expiry_hours}",
                                                oninput: move |e| {
                                                    if let Ok(hours) = e.value().parse::<u32>() {
                                                        token_expiry_hours.set(hours);
                                                    }
                                                },
                                            }
                                            span {
                                                class: "text-xs text-slate-500",
                                                "How long each token remains valid before the user must re-authenticate."
                                            }
                                        }
                                    }

                                    // Environment variable (static display)
                                    div {
                                        class: "mb-4 p-3 bg-slate-900/50 rounded-lg",
                                        p { class: "text-xs text-slate-500 mb-1", "Environment Variable" }
                                        p {
                                            class: "font-mono text-sm text-slate-300",
                                            "JWT_SECRET"
                                        }
                                        p {
                                            class: "text-xs text-slate-500 mt-1",
                                            "Set this in your .env file to a long, random string. It is used to sign and verify tokens."
                                        }
                                    }

                                    // JWT Token Composition — what data goes into the token
                                    div {
                                        class: "p-3 bg-slate-900/50 rounded-lg",

                                        h5 {
                                            class: "text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2",
                                            "Token Payload (Claims)"
                                        }

                                        p {
                                            class: "text-xs text-slate-500 mb-3",
                                            "Each JWT token is signed with HMAC-SHA256 and encodes the following data:"
                                        }

                                        div {
                                            class: "space-y-2 text-sm",

                                            div {
                                                class: "flex items-start gap-3 p-2 rounded bg-slate-800/50",
                                                span { class: "font-mono text-indigo-400 min-w-[60px]", "sub" }
                                                div {
                                                    p { class: "text-slate-300", "User ID" }
                                                    p { class: "text-xs text-slate-500", "The authenticated user's unique identifier (UUID)" }
                                                }
                                            }

                                            div {
                                                class: "flex items-start gap-3 p-2 rounded bg-slate-800/50",
                                                span { class: "font-mono text-indigo-400 min-w-[60px]", "email" }
                                                div {
                                                    p { class: "text-slate-300", "User Email" }
                                                    p { class: "text-xs text-slate-500", "The user's email address at time of token creation" }
                                                }
                                            }

                                            div {
                                                class: "flex items-start gap-3 p-2 rounded bg-slate-800/50",
                                                span { class: "font-mono text-indigo-400 min-w-[60px]", "roles" }
                                                div {
                                                    p { class: "text-slate-300", "User Roles" }
                                                    p { class: "text-xs text-slate-500", "List of roles assigned to the user (e.g. [\"admin\", \"editor\"])" }
                                                }
                                            }

                                            div {
                                                class: "flex items-start gap-3 p-2 rounded bg-slate-800/50",
                                                span { class: "font-mono text-amber-400 min-w-[60px]", "iat" }
                                                div {
                                                    p { class: "text-slate-300", "Issued At" }
                                                    p { class: "text-xs text-slate-500", "Timestamp when the token was created (seconds since Unix epoch)" }
                                                }
                                            }

                                            div {
                                                class: "flex items-start gap-3 p-2 rounded bg-slate-800/50",
                                                span { class: "font-mono text-amber-400 min-w-[60px]", "exp" }
                                                div {
                                                    p { class: "text-slate-300", "Expires At" }
                                                    p { class: "text-xs text-slate-500", "Timestamp when the token becomes invalid (iat + expiry hours)" }
                                                }
                                            }
                                        }

                                        p {
                                            class: "mt-3 text-xs text-slate-600",
                                            "The token is verified on every authenticated request. Expired tokens are rejected with 401 Unauthorized."
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Actions
                    div {
                        class: "flex justify-between items-center pt-6 border-t border-slate-700",

                        button {
                            class: "btn btn-secondary",
                            r#type: "button",
                            onclick: move |_| {
                                do_save();
                            },
                            span { class: "mr-2", "💾" }
                            "Save Configuration"
                        }

                        button {
                            class: "btn btn-primary",
                            r#type: "button",
                            onclick: move |_| {
                                let warnings = validate_setup();
                                if warnings.is_empty() {
                                    // No warnings — save and proceed
                                    do_save();
                                    APP_STATE.write().ui.navigate(Page::EntityDesign);
                                } else {
                                    // Show warnings dialog
                                    setup_warnings.set(warnings);
                                    show_warnings_dialog.set(true);
                                }
                            },
                            span { class: "mr-2", "→" }
                            "Continue to Entity Design"
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Page Header
// ============================================================================

#[component]
fn PageHeader() -> Element {
    rsx! {
        div {
            class: "mb-8",

            h1 {
                class: "text-2xl font-bold mb-2",
                "⚙️ Project Setup"
            }

            p {
                class: "text-slate-400",
                "Configure your project settings before designing entities. These settings determine what code will be generated."
            }
        }
    }
}

// ============================================================================
// Form Section Component
// ============================================================================

#[component]
fn FormSection(title: &'static str, description: &'static str, children: Element) -> Element {
    rsx! {
        section {
            class: "bg-slate-800/30 rounded-xl p-6 border border-slate-700/50",

            // Section Header
            div {
                class: "mb-6",

                h2 {
                    class: "text-lg font-semibold",
                    "{title}"
                }

                p {
                    class: "text-sm text-slate-400 mt-1",
                    "{description}"
                }
            }

            // Section Content
            div {
                class: "space-y-4",
                {children}
            }
        }
    }
}

// ============================================================================
// Form Field Component
// ============================================================================

#[component]
fn FormField(
    label: &'static str,
    required: bool,
    #[props(default = "")] hint: &'static str,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "form-field",

            label {
                class: "label flex items-center gap-2",
                "{label}"
                if required {
                    span { class: "text-rose-400 text-xs", "*" }
                }
            }

            {children}

            if !hint.is_empty() {
                p {
                    class: "text-xs text-slate-500 mt-1",
                    "{hint}"
                }
            }
        }
    }
}

// ============================================================================
// Project Type Card
// ============================================================================

#[component]
fn ProjectTypeCard(
    selected: bool,
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let border_class = if selected {
        "border-indigo-500 bg-indigo-500/10"
    } else {
        "border-slate-700 hover:border-slate-600"
    };

    rsx! {
        button {
            class: "text-left p-4 rounded-lg border-2 transition-all duration-200 {border_class}",
            r#type: "button",
            onclick: move |e| onclick.call(e),

            div {
                class: "flex items-start gap-3",

                span {
                    class: "text-2xl",
                    "{icon}"
                }

                div {
                    h3 {
                        class: "font-semibold",
                        class: if selected { "text-indigo-300" } else { "text-slate-200" },
                        "{title}"
                    }

                    p {
                        class: "text-sm text-slate-400 mt-1",
                        "{description}"
                    }
                }
            }

            if selected {
                div {
                    class: "mt-3 text-xs text-indigo-400 flex items-center gap-1",
                    "✓ Selected"
                }
            }
        }
    }
}

// ============================================================================
// Database Card
// ============================================================================

#[component]
fn DatabaseCard(
    selected: bool,
    icon: &'static str,
    name: &'static str,
    description: &'static str,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let border_class = if selected {
        "border-indigo-500 bg-indigo-500/10"
    } else {
        "border-slate-700 hover:border-slate-600"
    };

    rsx! {
        button {
            class: "text-center p-4 rounded-lg border-2 transition-all duration-200 {border_class}",
            r#type: "button",
            onclick: move |e| onclick.call(e),

            span {
                class: "text-3xl block mb-2",
                "{icon}"
            }

            h3 {
                class: "font-semibold",
                class: if selected { "text-indigo-300" } else { "text-slate-200" },
                "{name}"
            }

            p {
                class: "text-xs text-slate-500 mt-1",
                "{description}"
            }
        }
    }
}

// ============================================================================
// Auth Strategy Card
// ============================================================================

#[component]
fn AuthStrategyCard(
    selected: bool,
    icon: &'static str,
    name: &'static str,
    description: &'static str,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let border_class = if selected {
        "border-green-500 bg-green-500/10"
    } else {
        "border-slate-700 hover:border-slate-600"
    };

    rsx! {
        button {
            class: "text-left p-4 rounded-lg border-2 transition-all duration-200 {border_class}",
            r#type: "button",
            onclick: move |e| onclick.call(e),

            div {
                class: "flex items-start gap-3",

                span {
                    class: "text-2xl",
                    "{icon}"
                }

                div {
                    h3 {
                        class: "font-semibold",
                        class: if selected { "text-green-300" } else { "text-slate-200" },
                        "{name}"
                    }

                    p {
                        class: "text-sm text-slate-400 mt-1",
                        "{description}"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Toggle Switch Component
// ============================================================================

#[component]
fn ToggleSwitch(enabled: bool, onchange: EventHandler<bool>) -> Element {
    let bg_class = if enabled {
        "bg-indigo-600"
    } else {
        "bg-slate-600"
    };

    // Position thumb: off = 2px from left, on = 2px from right (26px from left for 48px track, 16px thumb)
    let thumb_style = if enabled { "left: 28px;" } else { "left: 4px;" };

    rsx! {
        button {
            class: "relative w-12 h-6 rounded-full transition-colors duration-200 {bg_class}",
            r#type: "button",
            role: "switch",
            aria_checked: "{enabled}",
            onclick: move |_| onchange.call(!enabled),

            span {
                class: "absolute top-1 w-4 h-4 bg-white rounded-full shadow-md",
                style: "transition: left 0.2s ease; {thumb_style}",
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
    fn test_project_setup_compiles() {
        // Basic compilation test
        assert!(true);
    }
}
