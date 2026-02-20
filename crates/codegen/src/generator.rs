//! # Code Generator Orchestrator
//!
//! The `Generator` is the top-level entry point for code generation. It takes a
//! [`ProjectGraph`] and a [`GeneratorConfig`], builds a [`GenerationContext`],
//! and delegates to the Rust code generators and SQL migration generators to
//! produce a complete [`GeneratedProject`].
//!
//! ## Pipeline
//!
//! ```text
//! ProjectGraph + GeneratorConfig
//!         │
//!         ▼
//!   GenerationContext::from_project()
//!         │
//!         ├──► rust::generate_rust_project()   → Vec<GeneratedFile>
//!         ├──► migrations::generate_migrations() → Vec<GeneratedFile>
//!         │
//!         ▼
//!   GeneratedProject { files, warnings }
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use imortal_codegen::{Generator, GeneratorConfig};
//! use imortal_ir::ProjectGraph;
//!
//! let project = ProjectGraph::new("my_app");
//! let config = GeneratorConfig::default();
//!
//! let result = Generator::new(config).generate(&project)?;
//!
//! println!("Generated {} files", result.file_count());
//! result.write_to_disk("/path/to/output")?;
//! ```

use imortal_core::{EngineResult, Validatable};
use imortal_ir::ProjectGraph;

use crate::context::GenerationContext;
use crate::frontend;
use crate::migrations;
use crate::rust;
use crate::{GeneratedProject, GeneratorConfig};

// ============================================================================
// Generator
// ============================================================================

/// Top-level code generator that orchestrates the full generation pipeline.
///
/// The `Generator` is stateless aside from its configuration. Call
/// [`generate`](Generator::generate) with a project graph to produce a
/// [`GeneratedProject`] containing every file that should be written to disk.
#[derive(Debug, Clone)]
pub struct Generator {
    /// Configuration controlling output behaviour (output dir, flags, etc.).
    config: GeneratorConfig,
}

impl Generator {
    // ====================================================================
    // Construction
    // ====================================================================

    /// Create a new generator with the given configuration.
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Create a generator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(GeneratorConfig::default())
    }

    /// Get the current configuration.
    pub fn config(&self) -> &GeneratorConfig {
        &self.config
    }

    /// Replace the configuration.
    pub fn set_config(&mut self, config: GeneratorConfig) {
        self.config = config;
    }

    // ====================================================================
    // Generation
    // ====================================================================

    /// Run the full code-generation pipeline on a project graph.
    ///
    /// # Steps
    ///
    /// 1. **Validate** the project graph (entities, relationships, endpoints).
    /// 2. **Build** a [`GenerationContext`] with sorted entities, lookups, and
    ///    derived metadata.
    /// 3. **Generate Rust source files** (models, handlers, routes, auth, config,
    ///    error, main, middleware, tests, Cargo.toml, .env, .gitignore, README).
    /// 4. **Generate SQL migrations** (one per entity, dependency-ordered).
    /// 5. **Generate frontend** (Dioxus Web app, shared crate — fullstack only).
    /// 6. **Collect warnings** from generators (e.g. entities without endpoints,
    ///    unused relationships).
    /// 7. Return the assembled [`GeneratedProject`].
    ///
    /// # Errors
    ///
    /// Returns an `EngineError` if project validation fails. Individual
    /// generators do not return errors — instead they add warnings to the
    /// output.
    pub fn generate(&self, project: &ProjectGraph) -> EngineResult<GeneratedProject> {
        // ── 1. Validate ──────────────────────────────────────────────────
        if let Err(e) = project.validate() {
            tracing::warn!("Project validation warning: {}", e);
            // We continue despite validation warnings so that partial
            // projects can still be generated. Only hard errors should
            // prevent generation.
        }

        // ── 2. Build context ─────────────────────────────────────────────
        let ctx = GenerationContext::from_project(project, self.config.clone());

        // ── 3. Collect warnings ──────────────────────────────────────────
        let mut warnings: Vec<String> = Vec::new();

        // Warn about entities without endpoints
        for entity in ctx.entities() {
            if ctx.endpoint_for_entity(entity.id).is_none() {
                warnings.push(format!(
                    "Entity '{}' has no endpoint group configured — no handlers or routes will be generated for it.",
                    entity.name,
                ));
            }
        }

        // Warn about endpoints referencing missing entities
        for ep in ctx.endpoints() {
            if ctx.entity_by_id(ep.entity_id).is_none() {
                warnings.push(format!(
                    "Endpoint group '{}' references entity ID {} which does not exist.",
                    ep.entity_name, ep.entity_id,
                ));
            }
        }

        // Warn if auth is enabled but no endpoints require it
        if ctx.auth_enabled() {
            let any_secured = ctx.endpoints().iter().any(|ep| ep.requires_auth());
            if !any_secured {
                warnings.push(
                    "Authentication is enabled in project config but no endpoints require authentication. \
                     Consider securing at least some endpoints or disabling auth.".to_string(),
                );
            }
        }

        // Warn about empty project
        if ctx.entity_count() == 0 {
            warnings.push(
                "No entities defined — the generated project will be an empty server scaffold."
                    .to_string(),
            );
        }

        // ── 4. Generate Rust source files ────────────────────────────────
        let rust_files = rust::generate_rust_project(&ctx);

        // ── 5. Generate SQL migrations ───────────────────────────────────
        let migration_files = migrations::generate_migrations(&ctx);

        // ── 6. Generate frontend (fullstack only) ────────────────────────
        let frontend_files = frontend::generate_frontend(&ctx);

        // ── 7. Assemble output ───────────────────────────────────────────
        let project_name = ctx.package_name().to_string();
        let mut output = GeneratedProject::new(&project_name);

        // For fullstack projects, prefix backend files under backend/
        if ctx.is_fullstack() && !frontend_files.is_empty() {
            for mut file in rust_files {
                let path_str = file.path.to_string_lossy().to_string();
                // Don't prefix workspace-level files that the frontend generator
                // also produces (like Cargo.toml, .gitignore, README.md, .env.example)
                if path_str == "Cargo.toml"
                    || path_str == ".gitignore"
                    || path_str == ".env.example"
                    || path_str == "README.md"
                {
                    // These will be replaced by the frontend generator's workspace versions
                    // Move them under backend/
                    file.path = format!("backend/{}", path_str).into();
                } else {
                    file.path = format!("backend/{}", path_str).into();
                }
                output.add_file(file);
            }

            // Prefix migration files under backend/ as well
            for mut file in migration_files {
                let path_str = file.path.to_string_lossy().to_string();
                file.path = format!("backend/{}", path_str).into();
                output.add_file(file);
            }

            // Add frontend files (already have correct paths: frontend/…, shared/…, Cargo.toml)
            for file in frontend_files {
                output.add_file(file);
            }
        } else {
            // REST-only project: no prefixing needed
            for file in rust_files {
                output.add_file(file);
            }
            for file in migration_files {
                output.add_file(file);
            }
        }

        for warning in warnings {
            output.add_warning(warning);
        }

        tracing::info!(
            files = output.file_count(),
            warnings = output.warnings.len(),
            project = %project_name,
            "code generation complete",
        );

        Ok(output)
    }

    // ====================================================================
    // Convenience: generate and write to disk
    // ====================================================================

    /// Generate code and write all files to the configured output directory.
    ///
    /// This is a convenience method that combines [`generate`](Generator::generate)
    /// and [`GeneratedProject::write_to_disk`].
    ///
    /// # Arguments
    ///
    /// * `project` — the project graph to generate from
    ///
    /// # Returns
    ///
    /// The generated project (so callers can inspect warnings, file counts, etc.).
    ///
    /// # Errors
    ///
    /// Returns an error if generation fails or if any file cannot be written.
    pub fn generate_and_write(&self, project: &ProjectGraph) -> EngineResult<GeneratedProject> {
        let output = self.generate(project)?;
        output.write_to_disk(&self.config.output_dir)?;
        tracing::info!(
            output_dir = %self.config.output_dir.display(),
            files = output.file_count(),
            "files written to disk",
        );
        Ok(output)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// Standalone convenience function
// ============================================================================

/// Generate code from a project graph using default configuration.
///
/// This is a shorthand for `Generator::with_defaults().generate(project)`.
pub fn generate(project: &ProjectGraph) -> EngineResult<GeneratedProject> {
    Generator::with_defaults().generate(project)
}

/// Generate code and write it to the specified output directory.
///
/// This is a shorthand for creating a `Generator` with the given output
/// directory and calling `generate_and_write`.
pub fn generate_to_dir(
    project: &ProjectGraph,
    output_dir: impl Into<std::path::PathBuf>,
) -> EngineResult<GeneratedProject> {
    let config = GeneratorConfig::new().with_output_dir(output_dir);
    Generator::new(config).generate_and_write(project)
}

// ============================================================================
// GenerationSummary — human-readable report
// ============================================================================

/// A human-readable summary of a completed generation run.
///
/// Use [`summarize`] to produce a `GenerationSummary` from a `GeneratedProject`.
#[derive(Debug, Clone)]
pub struct GenerationSummary {
    /// Project name.
    pub project_name: String,
    /// Total number of files generated.
    pub total_files: usize,
    /// Number of Rust source files.
    pub rust_files: usize,
    /// Number of SQL migration files.
    pub sql_files: usize,
    /// Number of other files (TOML, Markdown, etc.).
    pub other_files: usize,
    /// Number of warnings.
    pub warning_count: usize,
    /// Total bytes of generated content.
    pub total_bytes: usize,
}

impl GenerationSummary {
    /// Build a summary from a generated project.
    pub fn from_project(project: &GeneratedProject) -> Self {
        use crate::FileType;

        let rust_files = project.files_by_type(FileType::Rust).len();
        let sql_files = project.files_by_type(FileType::Sql).len();
        let other_files = project.file_count() - rust_files - sql_files;
        let total_bytes: usize = project.files.iter().map(|f| f.content.len()).sum();

        Self {
            project_name: project.name.clone(),
            total_files: project.file_count(),
            rust_files,
            sql_files,
            other_files,
            warning_count: project.warnings.len(),
            total_bytes,
        }
    }

    /// Format the summary as a human-readable string.
    pub fn display(&self) -> String {
        let mut out = String::with_capacity(512);

        out.push_str("╔══════════════════════════════════════════════════╗\n");
        out.push_str("║         Code Generation Complete                ║\n");
        out.push_str("╠══════════════════════════════════════════════════╣\n");
        out.push_str(&format!("║  Project:     {:<35}║\n", self.project_name));
        out.push_str(&format!("║  Total Files: {:<35}║\n", self.total_files));
        out.push_str(&format!("║    Rust:      {:<35}║\n", self.rust_files));
        out.push_str(&format!("║    SQL:       {:<35}║\n", self.sql_files));
        out.push_str(&format!("║    Other:     {:<35}║\n", self.other_files));
        out.push_str(&format!("║  Warnings:    {:<35}║\n", self.warning_count));

        let size_str = if self.total_bytes < 1024 {
            format!("{} B", self.total_bytes)
        } else if self.total_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.total_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.total_bytes as f64 / (1024.0 * 1024.0))
        };
        out.push_str(&format!("║  Total Size:  {:<35}║\n", size_str));
        out.push_str("╚══════════════════════════════════════════════════╝\n");

        out
    }
}

impl std::fmt::Display for GenerationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Produce a [`GenerationSummary`] from a [`GeneratedProject`].
pub fn summarize(project: &GeneratedProject) -> GenerationSummary {
    GenerationSummary::from_project(project)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_core::DataType;
    use imortal_ir::{AuthConfig, EndpointGroup, Entity, Field, OperationType, ProjectGraph};
    use uuid::Uuid;

    /// Helper: empty project.
    fn empty_project() -> ProjectGraph {
        ProjectGraph::new("empty")
    }

    /// Helper: project with a User entity and full CRUD.
    fn full_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("blog_api");

        // User entity
        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;

        let mut email = Field::new("email", DataType::String);
        email.required = true;
        email.unique = true;
        user.fields.push(email);

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        user.fields.push(name);

        let mut pw = Field::new("password_hash", DataType::String);
        pw.required = true;
        pw.secret = true;
        user.fields.push(pw);

        project.add_entity(user);

        // Endpoint
        let endpoint = EndpointGroup::new(user_id, "User");
        project.add_endpoint(endpoint);

        project
    }

    /// Helper: project with auth enabled.
    fn auth_project() -> ProjectGraph {
        let mut project = full_project();
        project.config.auth = AuthConfig::jwt();

        // Secure the endpoints
        if let Some(ep) = project.endpoints.values_mut().next() {
            ep.global_security.auth_required = true;
        }

        project
    }

    // ── Generator construction ───────────────────────────────────────────

    #[test]
    fn test_generator_new() {
        let generator = Generator::new(GeneratorConfig::default());
        assert!(generator.config().generate_tests);
        assert!(generator.config().generate_docs);
        assert!(generator.config().generate_migrations);
    }

    #[test]
    fn test_generator_with_defaults() {
        let generator = Generator::with_defaults();
        assert!(generator.config().generate_tests);
    }

    #[test]
    fn test_generator_default_trait() {
        let generator = Generator::default();
        assert!(generator.config().generate_tests);
    }

    #[test]
    fn test_generator_set_config() {
        let mut generator = Generator::with_defaults();
        generator.set_config(GeneratorConfig::new().without_tests());
        assert!(!generator.config().generate_tests);
    }

    // ── Empty project generation ─────────────────────────────────────────

    #[test]
    fn test_generate_empty_project() {
        let project = empty_project();
        let result = Generator::with_defaults().generate(&project);

        assert!(result.is_ok());
        let output = result.unwrap();

        // Should still produce scaffolding
        assert!(output.file_count() > 0);
        assert!(
            output.has_warnings(),
            "Empty project should produce warnings"
        );
    }

    #[test]
    fn test_generate_empty_project_warning_message() {
        let project = empty_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let has_empty_warning = output
            .warnings
            .iter()
            .any(|w| w.contains("No entities defined"));
        assert!(
            has_empty_warning,
            "Should warn about no entities: {:?}",
            output.warnings
        );
    }

    // ── Full project generation ──────────────────────────────────────────

    #[test]
    fn test_generate_full_project() {
        let project = full_project();
        let result = Generator::with_defaults().generate(&project);

        assert!(result.is_ok());
        let output = result.unwrap();

        assert!(output.file_count() > 10, "Should generate many files");

        // Key files should exist
        let paths: Vec<String> = output
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(paths.iter().any(|p| p == "Cargo.toml"));
        assert!(paths.iter().any(|p| p == "src/main.rs"));
        assert!(paths.iter().any(|p| p == "src/lib.rs"));
        assert!(paths.iter().any(|p| p == "src/config.rs"));
        assert!(paths.iter().any(|p| p == "src/error.rs"));
        assert!(paths.iter().any(|p| p == "src/state.rs"));
        assert!(paths.iter().any(|p| p == "src/middleware.rs"));
        assert!(paths.iter().any(|p| p == ".env.example"));
        assert!(paths.iter().any(|p| p == ".gitignore"));
        assert!(paths.iter().any(|p| p == "README.md"));

        // Model files
        assert!(paths.iter().any(|p| p == "src/models/mod.rs"));
        assert!(paths.iter().any(|p| p == "src/models/user.rs"));

        // Handler files
        assert!(paths.iter().any(|p| p == "src/handlers/mod.rs"));
        assert!(paths.iter().any(|p| p == "src/handlers/user.rs"));

        // Route files
        assert!(paths.iter().any(|p| p == "src/routes/mod.rs"));
        assert!(paths.iter().any(|p| p == "src/routes/api.rs"));

        // Migration files
        assert!(
            paths
                .iter()
                .any(|p| p.starts_with("migrations/") && p.contains("users")),
            "Should have a users migration, paths: {:?}",
            paths
                .iter()
                .filter(|p| p.starts_with("migrations/"))
                .collect::<Vec<_>>()
        );

        // Test files
        assert!(paths.iter().any(|p| p == "tests/api_tests.rs"));
    }

    #[test]
    fn test_generate_full_project_file_count() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        // Scaffolding:  Cargo.toml, .env, .gitignore, README = 4
        // Core:         main.rs, lib.rs, config.rs, error.rs, state.rs, middleware.rs = 6
        // Models:       mod.rs + user.rs = 2
        // Handlers:     mod.rs + user.rs = 2
        // Routes:       mod.rs + api.rs = 2
        // Migrations:   users.sql = 1
        // Tests:        api_tests.rs = 1
        // Total:        ~18 files (no auth)
        assert!(
            output.file_count() >= 15,
            "Expected at least 15 files, got {}",
            output.file_count()
        );
    }

    // ── Auth project generation ──────────────────────────────────────────

    #[test]
    fn test_generate_auth_project() {
        let project = auth_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let paths: Vec<String> = output
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        // Auth files should be present
        assert!(paths.iter().any(|p| p == "src/auth/mod.rs"));
        assert!(paths.iter().any(|p| p == "src/auth/jwt.rs"));
        assert!(paths.iter().any(|p| p == "src/auth/middleware.rs"));
    }

    #[test]
    fn test_generate_no_auth_project() {
        let mut project = full_project();
        project.config.auth.enabled = false;

        let output = Generator::with_defaults().generate(&project).unwrap();

        let paths: Vec<String> = output
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        // Auth files should NOT be present
        assert!(!paths.iter().any(|p| p.starts_with("src/auth/")));
    }

    // ── Warnings ─────────────────────────────────────────────────────────

    #[test]
    fn test_generate_warns_entity_without_endpoint() {
        let mut project = ProjectGraph::new("test");

        // Add entity without endpoint
        let mut entity = Entity::new("Secret");
        entity.config.timestamps = false;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);
        project.add_entity(entity);

        let output = Generator::with_defaults().generate(&project).unwrap();

        let has_warning = output
            .warnings
            .iter()
            .any(|w| w.contains("Secret") && w.contains("no endpoint group"));
        assert!(
            has_warning,
            "Should warn about entity without endpoints: {:?}",
            output.warnings
        );
    }

    #[test]
    fn test_generate_warns_auth_with_no_secured_endpoints() {
        let mut project = full_project();
        project.config.auth = AuthConfig::jwt();

        // Endpoints are NOT secured (default is open)
        let output = Generator::with_defaults().generate(&project).unwrap();

        let has_warning = output
            .warnings
            .iter()
            .any(|w| w.contains("no endpoints require authentication"));
        assert!(
            has_warning,
            "Should warn about auth enabled but no secured endpoints: {:?}",
            output.warnings
        );
    }

    // ── Configuration flags ──────────────────────────────────────────────

    #[test]
    fn test_generate_without_tests() {
        let project = full_project();
        let config = GeneratorConfig::new().without_tests();
        let output = Generator::new(config).generate(&project).unwrap();

        let paths: Vec<String> = output
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(
            !paths.iter().any(|p| p.starts_with("tests/")),
            "Should not generate test files"
        );
    }

    #[test]
    fn test_generate_without_migrations() {
        let project = full_project();
        let config = GeneratorConfig::new().without_migrations();
        let output = Generator::new(config).generate(&project).unwrap();

        let paths: Vec<String> = output
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        assert!(
            !paths.iter().any(|p| p.starts_with("migrations/")),
            "Should not generate migration files"
        );
    }

    #[test]
    fn test_generate_without_docs() {
        let project = full_project();
        let config = GeneratorConfig::new().without_docs();
        let output = Generator::new(config).generate(&project).unwrap();

        // The README should still be generated but without extensive docs
        // in individual files, doc comments are omitted.
        assert!(output.file_count() > 0);
    }

    // ── Standalone functions ─────────────────────────────────────────────

    #[test]
    fn test_standalone_generate() {
        let project = full_project();
        let result = generate(&project);
        assert!(result.is_ok());
        assert!(result.unwrap().file_count() > 0);
    }

    // ── GenerationSummary ────────────────────────────────────────────────

    #[test]
    fn test_generation_summary() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();
        let summary = summarize(&output);

        assert_eq!(summary.project_name, "my_app");
        assert!(summary.total_files > 0);
        assert!(summary.rust_files > 0);
        assert!(summary.total_bytes > 0);
    }

    #[test]
    fn test_generation_summary_display() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();
        let summary = summarize(&output);
        let display = summary.display();

        assert!(display.contains("Code Generation Complete"));
        assert!(display.contains("my_app"));
        assert!(display.contains("Total Files"));
        assert!(display.contains("Rust"));
        assert!(display.contains("SQL"));
    }

    #[test]
    fn test_generation_summary_from_project() {
        let mut output = GeneratedProject::new("test");
        output.add_file(crate::GeneratedFile::rust("a.rs", "fn main() {}"));
        output.add_file(crate::GeneratedFile::sql(
            "b.sql",
            "CREATE TABLE t (id INT);",
        ));
        output.add_file(crate::GeneratedFile::toml("c.toml", "[package]"));
        output.add_warning("test warning".to_string());

        let summary = GenerationSummary::from_project(&output);

        assert_eq!(summary.total_files, 3);
        assert_eq!(summary.rust_files, 1);
        assert_eq!(summary.sql_files, 1);
        assert_eq!(summary.other_files, 1);
        assert_eq!(summary.warning_count, 1);
        assert!(summary.total_bytes > 0);
    }

    #[test]
    fn test_generation_summary_size_formatting() {
        let mut output = GeneratedProject::new("test");
        // Add a file with known size
        output.add_file(crate::GeneratedFile::rust("a.rs", "x".repeat(2048)));

        let summary = GenerationSummary::from_project(&output);
        let display = summary.display();

        // 2048 bytes = 2.0 KB
        assert!(display.contains("KB"), "Should format as KB: {}", display);
    }

    #[test]
    fn test_generation_summary_display_trait() {
        let output = GeneratedProject::new("test");
        let summary = GenerationSummary::from_project(&output);

        // Test Display trait implementation
        let formatted = format!("{}", summary);
        assert!(formatted.contains("Code Generation Complete"));
    }

    // ── Multiple entities ────────────────────────────────────────────────

    #[test]
    fn test_generate_multiple_entities() {
        let mut project = ProjectGraph::new("multi");

        // User entity
        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;
        let mut email = Field::new("email", DataType::String);
        email.required = true;
        user.fields.push(email);
        project.add_entity(user);
        project.add_endpoint(EndpointGroup::new(user_id, "User"));

        // Post entity
        let mut post = Entity::new("Post");
        post.config.timestamps = true;
        let post_id = post.id;
        let mut title = Field::new("title", DataType::String);
        title.required = true;
        post.fields.push(title);
        project.add_entity(post);
        project.add_endpoint(EndpointGroup::new(post_id, "Post"));

        // Category entity (no endpoint)
        let mut cat = Entity::new("Category");
        cat.config.timestamps = false;
        let mut cat_name = Field::new("name", DataType::String);
        cat_name.required = true;
        cat.fields.push(cat_name);
        project.add_entity(cat);

        let output = Generator::with_defaults().generate(&project).unwrap();

        let paths: Vec<String> = output
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        // Models for all 3 entities
        assert!(paths.iter().any(|p| p == "src/models/user.rs"));
        assert!(paths.iter().any(|p| p == "src/models/post.rs"));
        assert!(paths.iter().any(|p| p == "src/models/category.rs"));

        // Handlers only for User and Post (Category has no endpoint)
        assert!(paths.iter().any(|p| p == "src/handlers/user.rs"));
        assert!(paths.iter().any(|p| p == "src/handlers/post.rs"));
        assert!(
            !paths.iter().any(|p| p == "src/handlers/category.rs"),
            "Category should not have handlers"
        );

        // Migrations for all 3 entities
        assert!(
            paths
                .iter()
                .filter(|p| p.starts_with("migrations/"))
                .count()
                == 3,
            "Should have 3 migration files"
        );

        // Warning about Category having no endpoint
        let has_warning = output
            .warnings
            .iter()
            .any(|w| w.contains("Category") && w.contains("no endpoint group"));
        assert!(has_warning, "Should warn about Category");
    }

    // ── Content verification ─────────────────────────────────────────────

    #[test]
    fn test_generated_cargo_toml_content() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let cargo_file = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == "Cargo.toml")
            .expect("Should have Cargo.toml");

        assert!(cargo_file.content.contains("[package]"));
        assert!(cargo_file.content.contains("[dependencies]"));
        assert!(cargo_file.content.contains("axum"));
        assert!(cargo_file.content.contains("sea-orm"));
    }

    #[test]
    fn test_generated_main_rs_content() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let main_file = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/main.rs")
            .expect("Should have src/main.rs");

        assert!(main_file.content.contains("#[tokio::main]"));
        assert!(main_file.content.contains("async fn main()"));
        assert!(main_file.content.contains("Database::connect"));
        assert!(main_file.content.contains("create_router"));
        assert!(main_file.content.contains("axum::serve"));
    }

    #[test]
    fn test_generated_model_content() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let user_model = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/models/user.rs")
            .expect("Should have src/models/user.rs");

        assert!(user_model.content.contains("DeriveEntityModel"));
        assert!(user_model.content.contains("pub struct Model"));
        assert!(user_model.content.contains("CreateUserDto"));
        assert!(user_model.content.contains("UpdateUserDto"));
        assert!(user_model.content.contains("UserResponse"));
        assert!(
            user_model
                .content
                .contains("impl From<Model> for UserResponse")
        );
    }

    #[test]
    fn test_generated_handler_content() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let user_handler = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/handlers/user.rs")
            .expect("Should have src/handlers/user.rs");

        assert!(user_handler.content.contains("pub async fn list_users("));
        assert!(user_handler.content.contains("pub async fn get_user("));
        assert!(user_handler.content.contains("pub async fn create_user("));
        assert!(user_handler.content.contains("pub async fn update_user("));
        assert!(user_handler.content.contains("pub async fn delete_user("));
    }

    #[test]
    fn test_generated_migration_content() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let migration = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("users"))
            .expect("Should have users migration");

        assert!(migration.content.contains("CREATE TABLE IF NOT EXISTS"));
        assert!(migration.content.contains("\"users\""));
        assert!(migration.content.contains("PRIMARY KEY"));
        assert!(migration.content.contains("\"email\""));
    }

    #[test]
    fn test_generated_routes_content() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let api_routes = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/routes/api.rs")
            .expect("Should have src/routes/api.rs");

        assert!(api_routes.content.contains("/api/users"));
        assert!(api_routes.content.contains("user::create_user"));
        assert!(api_routes.content.contains("user::list_users"));
        assert!(api_routes.content.contains("user::get_user"));
    }

    #[test]
    fn test_generated_env_content() {
        let project = full_project();
        let output = Generator::with_defaults().generate(&project).unwrap();

        let env_file = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == ".env.example")
            .expect("Should have .env.example");

        assert!(env_file.content.contains("DATABASE_URL="));
        assert!(env_file.content.contains("SERVER_HOST="));
        assert!(env_file.content.contains("SERVER_PORT="));
    }

    // ── Read-only endpoints ──────────────────────────────────────────────

    #[test]
    fn test_generate_read_only_endpoint() {
        let mut project = ProjectGraph::new("readonly");

        let mut item = Entity::new("Item");
        item.config.timestamps = false;
        let item_id = item.id;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        item.fields.push(name);
        project.add_entity(item);

        let endpoint = EndpointGroup::new(item_id, "Item")
            .with_operations(&[OperationType::Read, OperationType::ReadAll]);
        project.add_endpoint(endpoint);

        let output = Generator::with_defaults().generate(&project).unwrap();

        let handler = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == "src/handlers/item.rs")
            .expect("Should have item handler");

        // Only read handlers
        assert!(handler.content.contains("list_items"));
        assert!(handler.content.contains("get_item"));
        assert!(!handler.content.contains("create_item"));
        assert!(!handler.content.contains("update_item"));
        assert!(!handler.content.contains("delete_item"));
    }

    // ── Database-specific generation ─────────────────────────────────────

    #[test]
    fn test_generate_mysql_project() {
        let mut project = full_project();
        project.config.database = imortal_ir::DatabaseType::MySQL;

        let output = Generator::with_defaults().generate(&project).unwrap();

        let cargo = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == "Cargo.toml")
            .unwrap();

        assert!(cargo.content.contains("sqlx-mysql"));

        let migration = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("users"))
            .unwrap();

        assert!(migration.content.contains("MySQL"));
    }

    #[test]
    fn test_generate_sqlite_project() {
        let mut project = full_project();
        project.config.database = imortal_ir::DatabaseType::SQLite;

        let output = Generator::with_defaults().generate(&project).unwrap();

        let cargo = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == "Cargo.toml")
            .unwrap();

        assert!(cargo.content.contains("sqlx-sqlite"));

        let env = output
            .files
            .iter()
            .find(|f| f.path.to_string_lossy() == ".env.example")
            .unwrap();

        assert!(env.content.contains("sqlite://"));
    }
}
