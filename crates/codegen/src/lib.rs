//! # Immortal Codegen
//!
//! Code generation engine for Immortal Engine.
//!
//! This crate is responsible for generating production-ready Rust code
//! from the project's intermediate representation (IR).
//!
//! ## Features
//!
//! - **Model Generation**: SeaORM entity generation from IR entities
//! - **Handler Generation**: Axum request handlers for CRUD operations
//! - **Router Generation**: API route configuration
//! - **Migration Generation**: SQL migrations for database schema
//! - **Auth Generation**: JWT authentication middleware and handlers
//! - **Frontend Generation**: Dioxus components for fullstack projects
//!

// ============================================================================
// Modules
// ============================================================================

pub mod context;
pub mod frontend;
pub mod generator;
pub mod migrations;
pub mod rust;

// ============================================================================
// Re-exports
// ============================================================================

pub use context::{EntityInfo, GenerationContext};
pub use generator::{GenerationSummary, Generator, generate, generate_to_dir, summarize};

use imortal_core::{EngineError, EngineResult};
use imortal_ir::ProjectGraph;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================================
// GeneratorConfig
// ============================================================================

/// Configuration for the code generator
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Output directory for generated code
    pub output_dir: PathBuf,

    /// Whether to generate tests
    pub generate_tests: bool,

    /// Whether to generate documentation comments
    pub generate_docs: bool,

    /// Whether to generate migrations
    pub generate_migrations: bool,

    /// Whether to format generated code with rustfmt
    pub format_code: bool,

    /// Whether to overwrite existing files
    pub overwrite: bool,

    /// Custom options
    pub options: HashMap<String, String>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./generated"),
            generate_tests: true,
            generate_docs: true,
            generate_migrations: true,
            format_code: true,
            overwrite: false,
            options: HashMap::new(),
        }
    }
}

impl GeneratorConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Disable test generation
    pub fn without_tests(mut self) -> Self {
        self.generate_tests = false;
        self
    }

    /// Disable documentation generation
    pub fn without_docs(mut self) -> Self {
        self.generate_docs = false;
        self
    }

    /// Disable migration generation
    pub fn without_migrations(mut self) -> Self {
        self.generate_migrations = false;
        self
    }

    /// Allow overwriting existing files
    pub fn allow_overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    /// Set a custom option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

// ============================================================================
// GeneratedFile
// ============================================================================

/// Represents a single generated file
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Relative path from output directory
    pub path: PathBuf,

    /// File content
    pub content: String,

    /// File type for categorization
    pub file_type: FileType,
}

impl GeneratedFile {
    /// Create a new generated file
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>, file_type: FileType) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
            file_type,
        }
    }

    /// Create a Rust source file
    pub fn rust(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self::new(path, content, FileType::Rust)
    }

    /// Create a SQL migration file
    pub fn sql(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self::new(path, content, FileType::Sql)
    }

    /// Create a TOML config file
    pub fn toml(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self::new(path, content, FileType::Toml)
    }

    /// Get the file extension
    pub fn extension(&self) -> &str {
        self.file_type.extension()
    }
}

/// Type of generated file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Rust,
    Sql,
    Toml,
    Markdown,
    Env,
    Other,
}

impl FileType {
    /// Get the file extension for this type
    pub fn extension(&self) -> &str {
        match self {
            FileType::Rust => "rs",
            FileType::Sql => "sql",
            FileType::Toml => "toml",
            FileType::Markdown => "md",
            FileType::Env => "env",
            FileType::Other => "txt",
        }
    }
}

// ============================================================================
// GeneratedProject
// ============================================================================

/// Collection of all generated files for a project
#[derive(Debug, Clone, Default)]
pub struct GeneratedProject {
    /// Project name
    pub name: String,

    /// All generated files
    pub files: Vec<GeneratedFile>,

    /// Warnings generated during code generation
    pub warnings: Vec<String>,
}

impl GeneratedProject {
    /// Create a new generated project
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            files: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add a file to the project
    pub fn add_file(&mut self, file: GeneratedFile) {
        self.files.push(file);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Get the number of files
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get files by type
    pub fn files_by_type(&self, file_type: FileType) -> Vec<&GeneratedFile> {
        self.files
            .iter()
            .filter(|f| f.file_type == file_type)
            .collect()
    }

    /// Write all files to disk
    pub fn write_to_disk(&self, base_dir: impl AsRef<Path>) -> EngineResult<()> {
        let base_dir = base_dir.as_ref();

        for file in &self.files {
            let full_path = base_dir.join(&file.path);

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| EngineError::DirectoryCreate {
                    path: parent.to_path_buf(),
                    message: e.to_string(),
                })?;
            }

            // Write file
            std::fs::write(&full_path, &file.content).map_err(|e| EngineError::FileWrite {
                path: full_path.clone(),
                message: e.to_string(),
            })?;
        }

        Ok(())
    }
}

// ============================================================================
// CodeGenerator
// ============================================================================

/// Main code generator (legacy alias for [`Generator`]).
///
/// **Deprecated**: Use [`Generator`] instead. This type is kept for backward
/// compatibility with existing code that references `CodeGenerator`.
#[derive(Debug)]
pub struct CodeGenerator {
    inner: Generator,
}

impl CodeGenerator {
    /// Create a new code generator with the given configuration
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            inner: Generator::new(config),
        }
    }

    /// Create a code generator with default configuration
    pub fn with_defaults() -> Self {
        Self::new(GeneratorConfig::default())
    }

    /// Generate code from a project graph.
    ///
    /// This now delegates to the full [`Generator`] pipeline which produces
    /// complete Rust source files, SQL migrations, and project scaffolding.
    pub fn generate(&self, project: &ProjectGraph) -> EngineResult<GeneratedProject> {
        self.inner.generate(project)
    }

    /// Get the configuration
    pub fn config(&self) -> &GeneratorConfig {
        self.inner.config()
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_config_default() {
        let config = GeneratorConfig::default();
        assert!(config.generate_tests);
        assert!(config.generate_docs);
        assert!(!config.overwrite);
    }

    #[test]
    fn test_generator_config_builder() {
        let config = GeneratorConfig::new()
            .with_output_dir("/tmp/output")
            .without_tests()
            .allow_overwrite();

        assert_eq!(config.output_dir, PathBuf::from("/tmp/output"));
        assert!(!config.generate_tests);
        assert!(config.overwrite);
    }

    #[test]
    fn test_generated_file() {
        let file = GeneratedFile::rust("src/main.rs", "fn main() {}");
        assert_eq!(file.extension(), "rs");
        assert_eq!(file.file_type, FileType::Rust);
    }

    #[test]
    fn test_generated_project() {
        let mut project = GeneratedProject::new("test");
        project.add_file(GeneratedFile::rust("src/main.rs", "fn main() {}"));
        project.add_file(GeneratedFile::toml("Cargo.toml", "[package]"));

        assert_eq!(project.file_count(), 2);
        assert_eq!(project.files_by_type(FileType::Rust).len(), 1);
    }

    #[test]
    fn test_code_generator() {
        let generator = CodeGenerator::with_defaults();
        let project = ProjectGraph::new("test");

        let result = generator.generate(&project);
        assert!(result.is_ok());

        // The new generator should produce actual files, not an empty project
        let output = result.unwrap();
        assert!(output.file_count() > 0, "Generator should produce files");
    }

    #[test]
    fn test_new_generator_api() {
        let generator = Generator::with_defaults();
        let project = ProjectGraph::new("test");

        let result = generator.generate(&project);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.file_count() > 0);
    }

    #[test]
    fn test_standalone_generate_function() {
        let project = ProjectGraph::new("test");
        let result = generate(&project);
        assert!(result.is_ok());
        assert!(result.unwrap().file_count() > 0);
    }

    #[test]
    fn test_generation_summary() {
        let project = ProjectGraph::new("test");
        let output = generate(&project).unwrap();
        let summary = summarize(&output);

        assert_eq!(summary.project_name, "my_app");
        assert!(summary.total_files > 0);
    }
}
