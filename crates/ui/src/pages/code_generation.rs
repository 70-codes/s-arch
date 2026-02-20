//! # Code Generation Page
//!
//! The Code Generation page allows users to:
//!
//! - Review a summary of what will be generated (entities, endpoints, auth, DB)
//! - Select an output directory for the generated project
//! - Run the code generator and see real-time progress
//! - View the list of generated files organized by category
//! - Open the output directory or copy the path
//! - See warnings and suggestions from the generator
//!
//! This page ties together the entire Immortal Engine workflow — it's the final
//! step where the visual design becomes a real, runnable Rust project.

use dioxus::prelude::*;
use std::path::PathBuf;

use imortal_codegen::{FileType, GeneratedProject, GenerationSummary, Generator, GeneratorConfig};
use imortal_ir::ProjectType;

use crate::file_ops;
use crate::state::{APP_STATE, StatusLevel};

// ============================================================================
// Code Generation Page Component
// ============================================================================

/// Main code generation page component.
#[component]
pub fn CodeGenerationPage() -> Element {
    // ── State ────────────────────────────────────────────────────────────
    let mut output_dir: Signal<Option<PathBuf>> = use_signal(|| None);
    let mut is_generating = use_signal(|| false);
    let mut generation_result: Signal<Option<Result<GenerationSummary, String>>> =
        use_signal(|| None);
    let mut generated_files: Signal<Vec<GeneratedFileInfo>> = use_signal(Vec::new);
    let mut warnings: Signal<Vec<String>> = use_signal(Vec::new);
    let mut show_files = use_signal(|| false);

    // ── Generator options ────────────────────────────────────────────────
    let mut gen_tests = use_signal(|| true);
    let mut gen_docs = use_signal(|| true);
    let mut gen_migrations = use_signal(|| true);
    let mut overwrite = use_signal(|| true);

    // ── Read project info ────────────────────────────────────────────────
    let state = APP_STATE.read();
    let has_project = state.project.is_some();

    let project_summary = state.project.as_ref().map(|p| {
        let db_name = match p.config.database {
            imortal_ir::DatabaseType::PostgreSQL => "PostgreSQL",
            imortal_ir::DatabaseType::MySQL => "MySQL",
            imortal_ir::DatabaseType::SQLite => "SQLite",
        };
        let auth_str = if p.config.auth.enabled {
            match p.config.auth.strategy {
                imortal_ir::AuthStrategy::Jwt => "JWT",
                imortal_ir::AuthStrategy::Session => "Session",
                imortal_ir::AuthStrategy::ApiKey => "API Key",
                imortal_ir::AuthStrategy::None => "None",
            }
        } else {
            "None"
        };
        ProjectSummary {
            name: p.meta.name.clone(),
            package_name: p.config.package_name.clone(),
            project_type: p.config.project_type,
            database: db_name.to_string(),
            entity_count: p.entities.len(),
            relationship_count: p.relationships.len(),
            endpoint_count: p.endpoints.len(),
            auth_enabled: p.config.auth.enabled,
            auth_strategy: auth_str.to_string(),
            field_count: p.entities.values().map(|e| e.fields.len()).sum(),
        }
    });

    drop(state);

    // ── No project state ─────────────────────────────────────────────────
    if !has_project {
        return rsx! {
            div {
                class: "flex-1 flex items-center justify-center p-8",
                div {
                    class: "text-center max-w-md",
                    div {
                        class: "w-20 h-20 mx-auto mb-6 rounded-full bg-slate-800 flex items-center justify-center",
                        span { class: "text-4xl", "⚡" }
                    }
                    h2 {
                        class: "text-2xl font-bold text-white mb-2",
                        "No Project Open"
                    }
                    p {
                        class: "text-slate-400 mb-6",
                        "Open or create a project to generate code."
                    }
                }
            }
        };
    }

    let summary = project_summary.unwrap();

    // ── Select output directory handler ───────────────────────────────────
    let on_select_dir = move |_| {
        spawn(async move {
            if let Some(dir) = file_ops::show_export_directory_dialog().await {
                output_dir.set(Some(dir));
                // Clear previous results
                generation_result.set(None);
                generated_files.set(Vec::new());
                warnings.set(Vec::new());
            }
        });
    };

    // ── Generate handler ─────────────────────────────────────────────────
    let on_generate = move |_| {
        let dir = match output_dir.read().clone() {
            Some(d) => d,
            None => return,
        };

        is_generating.set(true);
        generation_result.set(None);
        generated_files.set(Vec::new());
        warnings.set(Vec::new());

        spawn(async move {
            // Read project from state
            let state = APP_STATE.read();
            let project = match &state.project {
                Some(p) => p.clone(),
                None => {
                    is_generating.set(false);
                    generation_result.set(Some(Err("No project loaded".to_string())));
                    return;
                }
            };
            drop(state);

            // Build generator config
            let config = GeneratorConfig::new().with_output_dir(&dir);

            let mut config = config;
            if !*gen_tests.peek() {
                config = config.without_tests();
            }
            if !*gen_docs.peek() {
                config = config.without_docs();
            }
            if !*gen_migrations.peek() {
                config = config.without_migrations();
            }
            if *overwrite.peek() {
                config = config.allow_overwrite();
            }

            // Run generator
            let generator = Generator::new(config);
            match generator.generate(&project) {
                Ok(output) => {
                    // Write files to disk
                    match output.write_to_disk(&dir) {
                        Ok(_) => {
                            let summary = imortal_codegen::summarize(&output);

                            // Collect file info for display
                            let files: Vec<GeneratedFileInfo> = output
                                .files
                                .iter()
                                .map(|f| GeneratedFileInfo {
                                    path: f.path.to_string_lossy().to_string(),
                                    file_type: format!("{:?}", f.file_type),
                                    size: f.content.len(),
                                    category: categorize_file(&f.path.to_string_lossy()),
                                })
                                .collect();

                            generated_files.set(files);
                            warnings.set(output.warnings.clone());
                            generation_result.set(Some(Ok(summary)));

                            // Update status bar
                            APP_STATE.write().ui.set_status(
                                format!(
                                    "Generated {} files to {}",
                                    output.file_count(),
                                    dir.display()
                                ),
                                StatusLevel::Success,
                            );
                        }
                        Err(e) => {
                            generation_result
                                .set(Some(Err(format!("Failed to write files: {}", e))));
                            APP_STATE.write().ui.set_status(
                                format!("Code generation failed: {}", e),
                                StatusLevel::Error,
                            );
                        }
                    }
                }
                Err(e) => {
                    generation_result.set(Some(Err(format!("Generation failed: {}", e))));
                    APP_STATE
                        .write()
                        .ui
                        .set_status(format!("Code generation failed: {}", e), StatusLevel::Error);
                }
            }

            is_generating.set(false);
        });
    };

    // ── Open output directory ────────────────────────────────────────────
    let on_open_dir = move |_| {
        if let Some(dir) = output_dir.read().as_ref() {
            let dir_str = dir.to_string_lossy().to_string();
            spawn(async move {
                let _ = tokio::process::Command::new("xdg-open")
                    .arg(&dir_str)
                    .spawn();
            });
        }
    };

    // ── Derived state ────────────────────────────────────────────────────
    let has_output_dir = output_dir.read().is_some();
    let can_generate = has_output_dir && !*is_generating.read();
    let has_result = generation_result.read().is_some();
    let is_success = generation_result
        .read()
        .as_ref()
        .map(|r| r.is_ok())
        .unwrap_or(false);

    // ── Render ───────────────────────────────────────────────────────────
    rsx! {
        div {
            class: "code-generation-page h-full overflow-auto",

            div {
                class: "max-w-4xl mx-auto p-8",

                // ── Page Header ──────────────────────────────────────────
                div {
                    class: "mb-8",
                    div {
                        class: "flex items-center gap-3 mb-2",
                        span { class: "text-3xl", "⚡" }
                        h1 {
                            class: "text-2xl font-bold text-white",
                            "Code Generation"
                        }
                    }
                    p {
                        class: "text-slate-400",
                        "Generate a complete, production-ready Rust project from your visual design."
                    }
                }

                // ── Project Summary Card ─────────────────────────────────
                div {
                    class: "bg-slate-800 rounded-xl border border-slate-700 p-6 mb-6",

                    h2 {
                        class: "text-lg font-semibold text-white mb-4 flex items-center gap-2",
                        span { "📊" }
                        "Project Summary"
                    }

                    div {
                        class: "grid grid-cols-2 md:grid-cols-4 gap-4",

                        SummaryCard {
                            icon: "📦",
                            label: "Entities",
                            value: summary.entity_count.to_string(),
                            color: "indigo",
                        }

                        SummaryCard {
                            icon: "🔗",
                            label: "Relationships",
                            value: summary.relationship_count.to_string(),
                            color: "emerald",
                        }

                        SummaryCard {
                            icon: "🌐",
                            label: "Endpoints",
                            value: summary.endpoint_count.to_string(),
                            color: "amber",
                        }

                        SummaryCard {
                            icon: "📝",
                            label: "Fields",
                            value: summary.field_count.to_string(),
                            color: "cyan",
                        }
                    }

                    // Project details row
                    div {
                        class: "mt-4 pt-4 border-t border-slate-700 flex flex-wrap gap-3",

                        DetailBadge {
                            label: "Project".to_string(),
                            value: summary.name.clone(),
                            color: "slate".to_string(),
                        }

                        DetailBadge {
                            label: "Package".to_string(),
                            value: summary.package_name.clone(),
                            color: "slate".to_string(),
                        }

                        DetailBadge {
                            label: "Type".to_string(),
                            value: if matches!(summary.project_type, ProjectType::Fullstack) {
                                "Fullstack".to_string()
                            } else {
                                "REST API".to_string()
                            },
                            color: "indigo".to_string(),
                        }

                        DetailBadge {
                            label: "Database".to_string(),
                            value: summary.database.clone(),
                            color: "emerald".to_string(),
                        }

                        {
                            let auth_color = if summary.auth_enabled { "amber" } else { "slate" };
                            rsx! {
                                DetailBadge {
                                    label: "Auth".to_string(),
                                    value: summary.auth_strategy.clone(),
                                    color: auth_color.to_string(),
                                }
                            }
                        }
                    }
                }

                // ── Generation Options ───────────────────────────────────
                div {
                    class: "bg-slate-800 rounded-xl border border-slate-700 p-6 mb-6",

                    h2 {
                        class: "text-lg font-semibold text-white mb-4 flex items-center gap-2",
                        span { "⚙️" }
                        "Generation Options"
                    }

                    div {
                        class: "grid grid-cols-2 md:grid-cols-4 gap-4",

                        OptionToggle {
                            label: "Generate Tests",
                            description: "API integration tests",
                            checked: *gen_tests.read(),
                            on_change: move |v: bool| gen_tests.set(v),
                        }

                        OptionToggle {
                            label: "Documentation",
                            description: "Doc comments on all items",
                            checked: *gen_docs.read(),
                            on_change: move |v: bool| gen_docs.set(v),
                        }

                        OptionToggle {
                            label: "Migrations",
                            description: "SQL migration files",
                            checked: *gen_migrations.read(),
                            on_change: move |v: bool| gen_migrations.set(v),
                        }

                        OptionToggle {
                            label: "Overwrite",
                            description: "Replace existing files",
                            checked: *overwrite.read(),
                            on_change: move |v: bool| overwrite.set(v),
                        }
                    }
                }

                // ── Output Directory ─────────────────────────────────────
                div {
                    class: "bg-slate-800 rounded-xl border border-slate-700 p-6 mb-6",

                    h2 {
                        class: "text-lg font-semibold text-white mb-4 flex items-center gap-2",
                        span { "📁" }
                        "Output Directory"
                    }

                    div {
                        class: "flex items-center gap-3",

                        // Directory display
                        div {
                            class: "flex-1 px-4 py-3 bg-slate-900 border border-slate-700 rounded-lg font-mono text-sm min-h-[48px] flex items-center",

                            if let Some(dir) = output_dir.read().as_ref() {
                                span {
                                    class: "text-slate-200 truncate",
                                    "{dir.display()}"
                                }
                            } else {
                                span {
                                    class: "text-slate-500 italic",
                                    "No directory selected — click Browse to choose where to generate"
                                }
                            }
                        }

                        // Browse button
                        button {
                            class: "px-4 py-3 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium transition-colors whitespace-nowrap",
                            onclick: on_select_dir,
                            "📂 Browse"
                        }

                        // Open folder button (if dir is set)
                        if has_output_dir {
                            button {
                                class: "px-3 py-3 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors",
                                title: "Open output directory",
                                onclick: on_open_dir,
                                "📂"
                            }
                        }
                    }

                    if has_output_dir {
                        p {
                            class: "mt-2 text-xs text-slate-500",
                            "The generated project will be written to this directory."
                        }
                    }
                }

                // ── Generate Button ──────────────────────────────────────
                div {
                    class: "mb-6",

                    button {
                        class: format!(
                            "w-full py-4 rounded-xl font-semibold text-lg transition-all flex items-center justify-center gap-3 {}",
                            if can_generate {
                                "bg-gradient-to-r from-green-600 to-emerald-600 hover:from-green-500 hover:to-emerald-500 text-white shadow-lg shadow-green-900/30 cursor-pointer"
                            } else if *is_generating.read() {
                                "bg-amber-600 text-white cursor-wait"
                            } else {
                                "bg-slate-700 text-slate-400 cursor-not-allowed"
                            }
                        ),
                        disabled: !can_generate,
                        onclick: on_generate,

                        if *is_generating.read() {
                            div {
                                class: "w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin",
                            }
                            "Generating…"
                        } else {
                            span { class: "text-xl", "🚀" }
                            "Generate Project"
                        }
                    }

                    if !has_output_dir {
                        p {
                            class: "mt-2 text-center text-sm text-slate-500",
                            "Select an output directory above before generating."
                        }
                    }
                }

                // ── Generation Result ────────────────────────────────────
                if has_result {
                    match generation_result.read().as_ref().unwrap() {
                        Ok(gen_summary) => rsx! {
                            // Success card
                            div {
                                class: "bg-emerald-900/20 border border-emerald-700/50 rounded-xl p-6 mb-6",

                                div {
                                    class: "flex items-center gap-3 mb-4",
                                    span { class: "text-3xl", "✅" }
                                    div {
                                        h2 {
                                            class: "text-xl font-bold text-emerald-300",
                                            "Code Generated Successfully!"
                                        }
                                        p {
                                            class: "text-sm text-emerald-400/70",
                                            "Your project is ready to build and run."
                                        }
                                    }
                                }

                                // Stats grid
                                {
                                    let total_files_str = gen_summary.total_files.to_string();
                                    let rust_files_str = gen_summary.rust_files.to_string();
                                    let sql_files_str = gen_summary.sql_files.to_string();
                                    let size_str = format_size(gen_summary.total_bytes);
                                    rsx! {
                                        div {
                                            class: "grid grid-cols-2 md:grid-cols-4 gap-3 mb-4",

                                            StatBox {
                                                label: "Total Files",
                                                value: total_files_str,
                                            }
                                            StatBox {
                                                label: "Rust Files",
                                                value: rust_files_str,
                                            }
                                            StatBox {
                                                label: "SQL Files",
                                                value: sql_files_str,
                                            }
                                            StatBox {
                                                label: "Total Size",
                                                value: size_str,
                                            }
                                        }
                                    }
                                }

                                // Action buttons
                                div {
                                    class: "flex flex-wrap gap-3 mt-4 pt-4 border-t border-emerald-700/30",

                                    button {
                                        class: "px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2",
                                        onclick: on_open_dir,
                                        span { "📂" }
                                        "Open in File Manager"
                                    }

                                    button {
                                        class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2",
                                        onclick: move |_| {
                                            let current = *show_files.read();
                                            show_files.set(!current);
                                        },
                                        span { "📋" }
                                        if *show_files.read() { "Hide Files" } else { "Show Files" }
                                    }

                                    // Copy next steps to clipboard
                                    button {
                                        class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2",
                                        title: "Show commands to run",
                                        onclick: move |_| {
                                            // Show next steps info
                                            APP_STATE.write().ui.set_status(
                                                "Next: cd into the output directory and run 'cargo run'".to_string(),
                                                StatusLevel::Info,
                                            );
                                        },
                                        span { "💡" }
                                        "Next Steps"
                                    }
                                }

                                // Next steps instructions
                                div {
                                    class: "mt-4 p-4 bg-slate-900/50 rounded-lg font-mono text-sm",

                                    p {
                                        class: "text-slate-400 mb-2 font-sans text-xs font-semibold uppercase tracking-wider",
                                        "Quick Start"
                                    }

                                    {
                                        let dir_display = output_dir.read().as_ref()
                                            .map(|d| d.display().to_string())
                                            .unwrap_or_else(|| "./generated".to_string());
                                        let cd_cmd = format!("$ cd {}", dir_display);
                                        rsx! {
                                            div {
                                                class: "space-y-1 text-slate-300",
                                                p { "{cd_cmd}" }
                                                p { "$ cp .env.example .env" }
                                                p {
                                                    class: "text-slate-500",
                                                    "$ # Edit .env with your database credentials"
                                                }
                                                p { "$ cargo build" }
                                                p { "$ cargo run" }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Err(error) => rsx! {
                            // Error card
                            div {
                                class: "bg-red-900/20 border border-red-700/50 rounded-xl p-6 mb-6",

                                div {
                                    class: "flex items-center gap-3 mb-3",
                                    span { class: "text-3xl", "❌" }
                                    h2 {
                                        class: "text-xl font-bold text-red-300",
                                        "Generation Failed"
                                    }
                                }

                                p {
                                    class: "text-red-400 text-sm bg-red-900/30 p-3 rounded-lg font-mono",
                                    "{error}"
                                }

                                button {
                                    class: "mt-4 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm transition-colors",
                                    onclick: move |_| {
                                        generation_result.set(None);
                                    },
                                    "Dismiss"
                                }
                            }
                        },
                    }
                }

                // ── Warnings ─────────────────────────────────────────────
                if !warnings.read().is_empty() {
                    div {
                        class: "bg-amber-900/20 border border-amber-700/50 rounded-xl p-6 mb-6",

                        h3 {
                            class: "text-sm font-semibold text-amber-300 mb-3 flex items-center gap-2",
                            span { "⚠️" }
                            "Warnings ({warnings.read().len()})"
                        }

                        div {
                            class: "space-y-2",

                            for (i, warning) in warnings.read().iter().enumerate() {
                                div {
                                    key: "{i}",
                                    class: "flex items-start gap-2 text-sm",
                                    span { class: "text-amber-500 mt-0.5 flex-shrink-0", "•" }
                                    span { class: "text-amber-200/80", "{warning}" }
                                }
                            }
                        }
                    }
                }

                // ── Generated Files List ─────────────────────────────────
                if *show_files.read() && !generated_files.read().is_empty() {
                    {
                        let files_snapshot: Vec<GeneratedFileInfo> = generated_files.read().clone();
                        let file_count = files_snapshot.len();
                        let categories = get_categories_owned(&files_snapshot);
                        rsx! {
                            div {
                                class: "bg-slate-800 rounded-xl border border-slate-700 overflow-hidden mb-6",

                                // Header
                                div {
                                    class: "px-4 py-3 border-b border-slate-700 flex items-center justify-between",

                                    h3 {
                                        class: "font-semibold text-white flex items-center gap-2",
                                        span { "📋" }
                                        "Generated Files ({file_count})"
                                    }
                                }

                                // File categories
                                div {
                                    class: "divide-y divide-slate-700",

                                    for (category, cat_files) in categories.iter() {
                                        {
                                            let cat_count = cat_files.len();
                                            rsx! {
                                                div {
                                                    // Category header
                                                    div {
                                                        class: "px-4 py-2 bg-slate-800/80 flex items-center justify-between",
                                                        span {
                                                            class: "text-xs font-semibold text-slate-400 uppercase tracking-wider",
                                                            "{category}"
                                                        }
                                                        span {
                                                            class: "text-xs text-slate-600",
                                                            "{cat_count} file(s)"
                                                        }
                                                    }

                                                    // Files in category
                                                    div {
                                                        class: "divide-y divide-slate-700/30",

                                                        for file in cat_files.iter() {
                                                            {
                                                                let icon = file_icon(&file.file_type);
                                                                let type_class = file_type_class(&file.file_type);
                                                                let size_str = format_size(file.size);
                                                                rsx! {
                                                                    div {
                                                                        class: "px-4 py-2 flex items-center gap-3 hover:bg-slate-700/20 transition-colors",

                                                                        span {
                                                                            class: "text-sm flex-shrink-0",
                                                                            "{icon}"
                                                                        }

                                                                        span {
                                                                            class: "font-mono text-sm text-slate-300 flex-1 truncate",
                                                                            "{file.path}"
                                                                        }

                                                                        span {
                                                                            class: "px-2 py-0.5 rounded text-xs font-medium {type_class}",
                                                                            "{file.file_type}"
                                                                        }

                                                                        span {
                                                                            class: "text-xs text-slate-600 min-w-[60px] text-right",
                                                                            "{size_str}"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── What Gets Generated ──────────────────────────────────
                if !has_result {
                    div {
                        class: "bg-slate-800/50 rounded-xl border border-slate-700 p-6 mb-6",

                        h2 {
                            class: "text-lg font-semibold text-white mb-4 flex items-center gap-2",
                            span { "📦" }
                            "What Gets Generated"
                        }

                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                            WhatItem {
                                icon: "🏗️",
                                title: "Project Scaffold",
                                description: "Cargo.toml, .env.example, .gitignore, README.md with setup instructions",
                            }

                            WhatItem {
                                icon: "📊",
                                title: "SeaORM Models",
                                description: "Entity structs, relations, DTOs (Create, Update, Response) with validators",
                            }

                            WhatItem {
                                icon: "🔌",
                                title: "Axum Handlers",
                                description: "CRUD handlers with pagination, validation, error handling, soft-delete",
                            }

                            WhatItem {
                                icon: "🛣️",
                                title: "Routes & Middleware",
                                description: "Router with per-entity routes, CORS, tracing, request logging, timeouts",
                            }

                            if summary.auth_enabled {
                                WhatItem {
                                    icon: "🔐",
                                    title: "JWT Authentication",
                                    description: "Claims, token create/verify, bcrypt password hashing, auth middleware",
                                }
                            }

                            WhatItem {
                                icon: "🗄️",
                                title: "SQL Migrations",
                                description: "CREATE TABLE statements with FK constraints, indexes, proper type mapping",
                            }

                            WhatItem {
                                icon: "🧪",
                                title: "Integration Tests",
                                description: "TestServer, per-entity CRUD tests, lifecycle tests, validation checks",
                            }

                            if matches!(summary.project_type, ProjectType::Fullstack) {
                                WhatItem {
                                    icon: "🖥️",
                                    title: "Dioxus Frontend",
                                    description: "Web UI with pages, components, API client, Tailwind CSS styling",
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
// Data Types
// ============================================================================

#[derive(Debug, Clone)]
struct ProjectSummary {
    name: String,
    package_name: String,
    project_type: ProjectType,
    database: String,
    entity_count: usize,
    relationship_count: usize,
    endpoint_count: usize,
    auth_enabled: bool,
    auth_strategy: String,
    field_count: usize,
}

#[derive(Debug, Clone)]
struct GeneratedFileInfo {
    path: String,
    file_type: String,
    size: usize,
    category: String,
}

// ============================================================================
// Sub-Components
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct SummaryCardProps {
    icon: &'static str,
    label: &'static str,
    value: String,
    color: &'static str,
}

#[component]
fn SummaryCard(props: SummaryCardProps) -> Element {
    let bg = match props.color {
        "indigo" => "bg-indigo-900/30 border-indigo-700/30",
        "emerald" => "bg-emerald-900/30 border-emerald-700/30",
        "amber" => "bg-amber-900/30 border-amber-700/30",
        "cyan" => "bg-cyan-900/30 border-cyan-700/30",
        _ => "bg-slate-700/30 border-slate-600/30",
    };

    let text = match props.color {
        "indigo" => "text-indigo-300",
        "emerald" => "text-emerald-300",
        "amber" => "text-amber-300",
        "cyan" => "text-cyan-300",
        _ => "text-slate-300",
    };

    rsx! {
        div {
            class: format!("p-4 rounded-lg border text-center {}", bg),

            span { class: "text-2xl", "{props.icon}" }
            div {
                class: format!("text-2xl font-bold mt-1 {}", text),
                "{props.value}"
            }
            div {
                class: "text-xs text-slate-500 mt-1",
                "{props.label}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DetailBadgeProps {
    label: String,
    value: String,
    color: String,
}

#[component]
fn DetailBadge(props: DetailBadgeProps) -> Element {
    let bg = match props.color.as_str() {
        "indigo" => "bg-indigo-900/30 text-indigo-300",
        "emerald" => "bg-emerald-900/30 text-emerald-300",
        "amber" => "bg-amber-900/30 text-amber-300",
        _ => "bg-slate-700 text-slate-300",
    };

    rsx! {
        span {
            class: "px-3 py-1 rounded-lg text-sm {bg}",
            span { class: "text-slate-500 mr-1", "{props.label}:" }
            span { class: "font-medium", "{props.value}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct OptionToggleProps {
    label: &'static str,
    description: &'static str,
    checked: bool,
    on_change: EventHandler<bool>,
}

#[component]
fn OptionToggle(props: OptionToggleProps) -> Element {
    rsx! {
        label {
            class: format!(
                "flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors {}",
                if props.checked {
                    "bg-indigo-900/20 border-indigo-700/30"
                } else {
                    "bg-slate-800/50 border-slate-700/50 opacity-60"
                }
            ),

            input {
                r#type: "checkbox",
                class: "mt-0.5 w-4 h-4 accent-indigo-500",
                checked: props.checked,
                onchange: move |e| props.on_change.call(e.checked()),
            }

            div {
                div {
                    class: "text-sm font-medium text-white",
                    "{props.label}"
                }
                div {
                    class: "text-xs text-slate-500",
                    "{props.description}"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct StatBoxProps {
    label: &'static str,
    value: String,
}

#[component]
fn StatBox(props: StatBoxProps) -> Element {
    rsx! {
        div {
            class: "p-3 bg-emerald-900/20 rounded-lg text-center",
            div {
                class: "text-xl font-bold text-emerald-300",
                "{props.value}"
            }
            div {
                class: "text-xs text-emerald-400/60",
                "{props.label}"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct WhatItemProps {
    icon: &'static str,
    title: &'static str,
    description: &'static str,
}

#[component]
fn WhatItem(props: WhatItemProps) -> Element {
    rsx! {
        div {
            class: "flex items-start gap-3 p-3 bg-slate-800 rounded-lg border border-slate-700",

            span { class: "text-xl flex-shrink-0 mt-0.5", "{props.icon}" }

            div {
                h4 {
                    class: "text-sm font-medium text-white",
                    "{props.title}"
                }
                p {
                    class: "text-xs text-slate-500 mt-0.5 leading-relaxed",
                    "{props.description}"
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format a byte size into a human-readable string.
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Categorize a file path into a group for the file list display.
fn categorize_file(path: &str) -> String {
    if path.starts_with("src/models/") {
        "Models (SeaORM)".to_string()
    } else if path.starts_with("src/handlers/") {
        "Handlers (Axum)".to_string()
    } else if path.starts_with("src/routes/") {
        "Routes".to_string()
    } else if path.starts_with("src/auth/") {
        "Authentication".to_string()
    } else if path.starts_with("migrations/") || path.starts_with("backend/migrations/") {
        "SQL Migrations".to_string()
    } else if path.starts_with("tests/") || path.starts_with("backend/tests/") {
        "Tests".to_string()
    } else if path.starts_with("frontend/") {
        "Frontend (Dioxus)".to_string()
    } else if path.starts_with("shared/") {
        "Shared Types".to_string()
    } else if path.starts_with("src/") || path.starts_with("backend/src/") {
        "Core Source".to_string()
    } else {
        "Project Files".to_string()
    }
}

/// Get unique categories and their files, sorted. Returns owned data to avoid lifetime issues in RSX.
fn get_categories_owned(files: &[GeneratedFileInfo]) -> Vec<(String, Vec<GeneratedFileInfo>)> {
    let mut categories: Vec<(String, Vec<GeneratedFileInfo>)> = Vec::new();

    for file in files {
        if let Some(cat) = categories.iter_mut().find(|(c, _)| c == &file.category) {
            cat.1.push(file.clone());
        } else {
            categories.push((file.category.clone(), vec![file.clone()]));
        }
    }

    // Sort categories in a logical order
    let order = [
        "Project Files",
        "Core Source",
        "Models (SeaORM)",
        "Handlers (Axum)",
        "Routes",
        "Authentication",
        "SQL Migrations",
        "Tests",
        "Shared Types",
        "Frontend (Dioxus)",
    ];

    categories.sort_by(|a, b| {
        let a_idx = order.iter().position(|&o| o == a.0).unwrap_or(99);
        let b_idx = order.iter().position(|&o| o == b.0).unwrap_or(99);
        a_idx.cmp(&b_idx)
    });

    categories
}

/// Get an icon for a file type.
fn file_icon(file_type: &str) -> &'static str {
    match file_type {
        "Rust" => "🦀",
        "Sql" => "🗄️",
        "Toml" => "⚙️",
        "Markdown" => "📝",
        "Env" => "🔧",
        _ => "📄",
    }
}

/// Get a CSS class for a file type badge.
fn file_type_class(file_type: &str) -> &'static str {
    match file_type {
        "Rust" => "bg-orange-900/30 text-orange-400",
        "Sql" => "bg-blue-900/30 text-blue-400",
        "Toml" => "bg-purple-900/30 text-purple-400",
        "Markdown" => "bg-slate-700 text-slate-400",
        "Env" => "bg-green-900/30 text-green-400",
        _ => "bg-slate-700 text-slate-400",
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(2048), "2.0 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_categorize_file() {
        assert_eq!(categorize_file("src/models/user.rs"), "Models (SeaORM)");
        assert_eq!(categorize_file("src/handlers/user.rs"), "Handlers (Axum)");
        assert_eq!(categorize_file("src/routes/api.rs"), "Routes");
        assert_eq!(categorize_file("src/auth/jwt.rs"), "Authentication");
        assert_eq!(
            categorize_file("migrations/001_users.sql"),
            "SQL Migrations"
        );
        assert_eq!(categorize_file("tests/api_tests.rs"), "Tests");
        assert_eq!(categorize_file("frontend/src/main.rs"), "Frontend (Dioxus)");
        assert_eq!(categorize_file("shared/src/lib.rs"), "Shared Types");
        assert_eq!(categorize_file("src/main.rs"), "Core Source");
        assert_eq!(categorize_file("Cargo.toml"), "Project Files");
        assert_eq!(categorize_file(".gitignore"), "Project Files");
    }

    #[test]
    fn test_file_icon() {
        assert_eq!(file_icon("Rust"), "🦀");
        assert_eq!(file_icon("Sql"), "🗄️");
        assert_eq!(file_icon("Toml"), "⚙️");
        assert_eq!(file_icon("Markdown"), "📝");
        assert_eq!(file_icon("Unknown"), "📄");
    }

    #[test]
    fn test_file_type_class() {
        assert!(file_type_class("Rust").contains("orange"));
        assert!(file_type_class("Sql").contains("blue"));
        assert!(file_type_class("Toml").contains("purple"));
    }

    #[test]
    fn test_get_categories_empty() {
        let files: Vec<GeneratedFileInfo> = Vec::new();
        let categories = get_categories_owned(&files);
        assert!(categories.is_empty());
    }

    #[test]
    fn test_get_categories_groups() {
        let files = vec![
            GeneratedFileInfo {
                path: "src/models/user.rs".to_string(),
                file_type: "Rust".to_string(),
                size: 100,
                category: "Models (SeaORM)".to_string(),
            },
            GeneratedFileInfo {
                path: "src/models/post.rs".to_string(),
                file_type: "Rust".to_string(),
                size: 80,
                category: "Models (SeaORM)".to_string(),
            },
            GeneratedFileInfo {
                path: "Cargo.toml".to_string(),
                file_type: "Toml".to_string(),
                size: 500,
                category: "Project Files".to_string(),
            },
        ];

        let categories = get_categories_owned(&files);
        assert_eq!(categories.len(), 2);

        // Project Files should come before Models in the sorted order
        assert_eq!(categories[0].0, "Project Files");
        assert_eq!(categories[1].0, "Models (SeaORM)");
        assert_eq!(categories[1].1.len(), 2);
    }
}
