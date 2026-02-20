//! Serialization and deserialization for Immortal Engine projects
//!
//! This module provides functionality for saving and loading project files,
//! including JSON serialization, file I/O, and schema version migration.

use crate::{ProjectGraph, SCHEMA_VERSION};
use imortal_core::{EngineError, EngineResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ============================================================================
// Constants
// ============================================================================

/// File extension for Immortal Engine projects
pub const PROJECT_EXTENSION: &str = "ieng";

/// Magic bytes for binary format (future use)
pub const MAGIC_BYTES: &[u8] = b"IENG";

// ============================================================================
// Project File Wrapper
// ============================================================================

/// Wrapper for project files that includes version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    /// Schema version for migration purposes
    pub schema_version: u32,

    /// The project data
    pub project: ProjectGraph,

    /// File format version (for future binary formats)
    #[serde(default)]
    pub format_version: u32,
}

impl ProjectFile {
    /// Create a new project file from a project graph
    pub fn new(project: ProjectGraph) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            project,
            format_version: 1,
        }
    }

    /// Check if migration is needed
    pub fn needs_migration(&self) -> bool {
        self.schema_version < SCHEMA_VERSION
    }

    /// Migrate to the latest schema version
    pub fn migrate(&mut self) -> EngineResult<()> {
        while self.schema_version < SCHEMA_VERSION {
            self.migrate_one_version()?;
        }
        Ok(())
    }

    /// Migrate one version at a time
    fn migrate_one_version(&mut self) -> EngineResult<()> {
        match self.schema_version {
            // Add migration logic for each version here
            // 1 => { /* migrate from v1 to v2 */ self.schema_version = 2; }
            _ => {
                // No migration needed or unknown version
                self.schema_version = SCHEMA_VERSION;
            }
        }
        Ok(())
    }
}

// ============================================================================
// Save Functions
// ============================================================================

/// Save a project to a file
///
/// # Arguments
///
/// * `project` - The project graph to save
/// * `path` - The path to save to
///
/// # Example
///
/// ```rust,ignore
/// use imortal_ir::{ProjectGraph, save_project};
///
/// let project = ProjectGraph::new("My Project");
/// save_project(&project, "my_project.ieng").unwrap();
/// ```
pub fn save_project(project: &ProjectGraph, path: impl AsRef<Path>) -> EngineResult<()> {
    let path = path.as_ref();
    let file = ProjectFile::new(project.clone());

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&file).map_err(|e| EngineError::FileWrite {
        path: path.to_path_buf(),
        message: format!("Failed to serialize project: {}", e),
    })?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| EngineError::DirectoryCreate {
                path: parent.to_path_buf(),
                message: e.to_string(),
            })?;
        }
    }

    // Write to file
    std::fs::write(path, json).map_err(|e| EngineError::FileWrite {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    Ok(())
}

/// Save a project to a JSON string
pub fn save_project_to_string(project: &ProjectGraph) -> EngineResult<String> {
    let file = ProjectFile::new(project.clone());
    serde_json::to_string_pretty(&file)
        .map_err(|e| EngineError::CodeGeneration(format!("Failed to serialize project: {}", e)))
}

/// Save a project to a compact JSON string (no pretty printing)
pub fn save_project_to_compact_string(project: &ProjectGraph) -> EngineResult<String> {
    let file = ProjectFile::new(project.clone());
    serde_json::to_string(&file)
        .map_err(|e| EngineError::CodeGeneration(format!("Failed to serialize project: {}", e)))
}

// ============================================================================
// Load Functions
// ============================================================================

/// Load a project from a file
///
/// # Arguments
///
/// * `path` - The path to load from
///
/// # Example
///
/// ```rust,ignore
/// use imortal_ir::load_project;
///
/// let project = load_project("my_project.ieng").unwrap();
/// println!("Loaded project: {}", project.meta.name);
/// ```
pub fn load_project(path: impl AsRef<Path>) -> EngineResult<ProjectGraph> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(EngineError::ProjectNotFound(path.to_path_buf()));
    }

    // Read file contents
    let json = std::fs::read_to_string(path).map_err(|e| EngineError::FileRead {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    // Parse and potentially migrate
    load_project_from_string(&json).map_err(|e| match e {
        EngineError::JsonSerialization(je) => EngineError::FileRead {
            path: path.to_path_buf(),
            message: format!("Invalid project file format: {}", je),
        },
        other => other,
    })
}

/// Load a project from a JSON string
pub fn load_project_from_string(json: &str) -> EngineResult<ProjectGraph> {
    // Try to parse as ProjectFile first
    if let Ok(mut file) = serde_json::from_str::<ProjectFile>(json) {
        if file.needs_migration() {
            file.migrate()?;
        }
        return Ok(file.project);
    }

    // Try to parse as raw ProjectGraph (for backwards compatibility)
    let project: ProjectGraph = serde_json::from_str(json)?;
    Ok(project)
}

/// Load a project from bytes
pub fn load_project_from_bytes(bytes: &[u8]) -> EngineResult<ProjectGraph> {
    let json = std::str::from_utf8(bytes)
        .map_err(|e| EngineError::InvalidProjectFormat(format!("Invalid UTF-8: {}", e)))?;
    load_project_from_string(json)
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Check if a file is a valid Immortal Engine project file
pub fn is_project_file(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if let Some(ext) = path.extension() {
        if ext != PROJECT_EXTENSION {
            return false;
        }
    } else {
        return false;
    }

    // Try to load and validate
    load_project(path).is_ok()
}

/// Get the default file name for a project
pub fn default_file_name(project_name: &str) -> String {
    let safe_name: String = project_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    format!("{}.{}", safe_name.to_lowercase(), PROJECT_EXTENSION)
}

/// Ensure a path has the correct extension
pub fn ensure_extension(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();

    if path.extension().map_or(true, |e| e != PROJECT_EXTENSION) {
        let mut new_path = path.to_path_buf();
        new_path.set_extension(PROJECT_EXTENSION);
        new_path
    } else {
        path.to_path_buf()
    }
}

/// Create a backup of a project file before overwriting
pub fn backup_project(path: impl AsRef<Path>) -> EngineResult<Option<PathBuf>> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(None);
    }

    let backup_path = path.with_extension(format!("{}.backup", PROJECT_EXTENSION));

    std::fs::copy(path, &backup_path).map_err(|e| EngineError::FileWrite {
        path: backup_path.clone(),
        message: format!("Failed to create backup: {}", e),
    })?;

    Ok(Some(backup_path))
}

/// Get project metadata without loading the full project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetaPreview {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub entity_count: usize,
    pub schema_version: u32,
}

/// Preview project metadata without loading the full graph
pub fn preview_project(path: impl AsRef<Path>) -> EngineResult<ProjectMetaPreview> {
    let project = load_project(path)?;

    Ok(ProjectMetaPreview {
        name: project.meta.name.clone(),
        description: project.meta.description.clone(),
        author: project.meta.author.clone(),
        version: project.meta.version.clone(),
        entity_count: project.entities.len(),
        schema_version: project.schema_version,
    })
}

// ============================================================================
// Recent Projects
// ============================================================================

/// Entry in the recent projects list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    /// Project name
    pub name: String,

    /// File path
    pub path: PathBuf,

    /// Last opened timestamp
    pub last_opened: chrono::DateTime<chrono::Utc>,
}

impl RecentProject {
    /// Create a new recent project entry
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            last_opened: chrono::Utc::now(),
        }
    }

    /// Check if the project file still exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

/// Manage recent projects list
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentProjectsList {
    /// Maximum number of recent projects to keep
    #[serde(skip)]
    pub max_entries: usize,

    /// Recent projects, most recent first
    pub projects: Vec<RecentProject>,
}

impl RecentProjectsList {
    /// Create a new recent projects list
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            projects: Vec::new(),
        }
    }

    /// Add or update a project in the list
    pub fn add(&mut self, name: impl Into<String>, path: impl Into<PathBuf>) {
        let path = path.into();
        self.projects.retain(|p| p.path != path);
        self.projects.insert(0, RecentProject::new(name, path));

        // Trim to max entries
        if self.projects.len() > self.max_entries {
            self.projects.truncate(self.max_entries);
        }
    }

    /// Remove a project from the list
    pub fn remove(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.projects.retain(|p| p.path != path);
    }

    /// Clean up entries for non-existent files
    pub fn cleanup(&mut self) {
        self.projects.retain(|p| p.exists());
    }

    /// Get the most recent project
    pub fn most_recent(&self) -> Option<&RecentProject> {
        self.projects.first()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.projects.len()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Entity;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_project() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.ieng");

        // Create a project
        let mut project = ProjectGraph::new("Test Project");
        project.add_entity(Entity::with_timestamps("User"));

        // Save
        save_project(&project, &path).unwrap();
        assert!(path.exists());

        // Load
        let loaded = load_project(&path).unwrap();
        assert_eq!(loaded.meta.name, "Test Project");
        assert_eq!(loaded.entity_count(), 1);
    }

    #[test]
    fn test_save_and_load_string() {
        let mut project = ProjectGraph::new("String Test");
        project.add_entity(Entity::new("Post"));

        // Save to string
        let json = save_project_to_string(&project).unwrap();
        assert!(json.contains("String Test"));

        // Load from string
        let loaded = load_project_from_string(&json).unwrap();
        assert_eq!(loaded.meta.name, "String Test");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_project("/nonexistent/path/project.ieng");
        assert!(result.is_err());

        if let Err(EngineError::ProjectNotFound(path)) = result {
            assert!(path.to_string_lossy().contains("nonexistent"));
        } else {
            panic!("Expected ProjectNotFound error");
        }
    }

    #[test]
    fn test_project_file_wrapper() {
        let project = ProjectGraph::new("Wrapper Test");
        let file = ProjectFile::new(project);

        assert_eq!(file.schema_version, SCHEMA_VERSION);
        assert!(!file.needs_migration());
    }

    #[test]
    fn test_default_file_name() {
        assert_eq!(default_file_name("My Project"), "my_project.ieng");
        assert_eq!(default_file_name("Test!@#$%"), "test_____.ieng");
        assert_eq!(default_file_name("simple"), "simple.ieng");
    }

    #[test]
    fn test_ensure_extension() {
        let path = ensure_extension("project");
        assert_eq!(path.extension().unwrap(), PROJECT_EXTENSION);

        let path = ensure_extension("project.ieng");
        assert_eq!(path.extension().unwrap(), PROJECT_EXTENSION);

        let path = ensure_extension("project.json");
        assert_eq!(path.extension().unwrap(), PROJECT_EXTENSION);
    }

    #[test]
    fn test_backup_project() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.ieng");

        // Create a project
        let project = ProjectGraph::new("Backup Test");
        save_project(&project, &path).unwrap();

        // Create backup
        let backup_path = backup_project(&path).unwrap();
        assert!(backup_path.is_some());
        assert!(backup_path.unwrap().exists());
    }

    #[test]
    fn test_backup_nonexistent() {
        let result = backup_project("/nonexistent/path.ieng").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_recent_projects_list() {
        let mut list = RecentProjectsList::new(3);

        list.add("Project 1", "/path/to/project1.ieng");
        list.add("Project 2", "/path/to/project2.ieng");
        list.add("Project 3", "/path/to/project3.ieng");

        assert_eq!(list.len(), 3);
        assert_eq!(list.most_recent().unwrap().name, "Project 3");

        // Adding a fourth should remove the oldest
        list.add("Project 4", "/path/to/project4.ieng");
        assert_eq!(list.len(), 3);
        assert!(!list.projects.iter().any(|p| p.name == "Project 1"));
    }

    #[test]
    fn test_recent_projects_update() {
        let mut list = RecentProjectsList::new(5);

        list.add("Project 1", "/path/to/project1.ieng");
        list.add("Project 2", "/path/to/project2.ieng");

        // Re-adding an existing project should move it to the top
        list.add("Project 1 Updated", "/path/to/project1.ieng");

        assert_eq!(list.len(), 2);
        assert_eq!(list.most_recent().unwrap().name, "Project 1 Updated");
    }

    #[test]
    fn test_preview_project() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("preview.ieng");

        // Create a project with some entities
        let mut project = ProjectGraph::new("Preview Test");
        project.meta.description = Some("A test project".to_string());
        project.meta.author = Some("Test Author".to_string());
        project.add_entity(Entity::new("User"));
        project.add_entity(Entity::new("Post"));
        save_project(&project, &path).unwrap();

        // Preview
        let preview = preview_project(&path).unwrap();
        assert_eq!(preview.name, "Preview Test");
        assert_eq!(preview.description, Some("A test project".to_string()));
        assert_eq!(preview.author, Some("Test Author".to_string()));
        assert_eq!(preview.entity_count, 2);
    }

    #[test]
    fn test_is_project_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("valid.ieng");

        // Create a valid project file
        let project = ProjectGraph::new("Valid");
        save_project(&project, &path).unwrap();

        assert!(is_project_file(&path));

        // Invalid extension
        let invalid_path = temp_dir.path().join("invalid.txt");
        std::fs::write(&invalid_path, "not a project").unwrap();
        assert!(!is_project_file(&invalid_path));
    }

    #[test]
    fn test_compact_string() {
        let project = ProjectGraph::new("Compact Test");

        let pretty = save_project_to_string(&project).unwrap();
        let compact = save_project_to_compact_string(&project).unwrap();

        // Compact should be shorter (no whitespace/newlines)
        assert!(compact.len() < pretty.len());

        // Both should be valid
        let from_pretty = load_project_from_string(&pretty).unwrap();
        let from_compact = load_project_from_string(&compact).unwrap();

        assert_eq!(from_pretty.meta.name, from_compact.meta.name);
    }
}
