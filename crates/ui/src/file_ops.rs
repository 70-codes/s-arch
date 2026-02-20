//! File Operations for Immortal Engine
//!
//! This module provides file save/load functionality using the `rfd` crate
//! for native file dialogs. It integrates with the application state and
//! the IR serialization module.

use imortal_core::{EngineError, EngineResult};
use imortal_ir::{
    ProjectGraph,
    serialization::{PROJECT_EXTENSION, load_project, save_project},
};
use rfd::AsyncFileDialog;
use std::path::PathBuf;

// ============================================================================
// File Filter Constants
// ============================================================================

/// File extension for Immortal Engine projects
pub const FILE_EXTENSION: &str = PROJECT_EXTENSION;

/// Display name for file filter
pub const FILE_TYPE_NAME: &str = "Immortal Engine Project";

// ============================================================================
// File Dialog Functions
// ============================================================================

/// Open a file dialog to select a project file to open
///
/// Returns the selected file path, or None if the dialog was cancelled.
pub async fn show_open_dialog() -> Option<PathBuf> {
    let file = AsyncFileDialog::new()
        .set_title("Open Project")
        .add_filter(FILE_TYPE_NAME, &[FILE_EXTENSION.trim_start_matches('.')])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await?;

    Some(file.path().to_path_buf())
}

/// Open a file dialog to select where to save a project
///
/// If `starting_dir` is provided, the dialog opens in that directory.
/// If `default_name` is provided, it is used as the suggested file name
/// (with `.ieng` extension added automatically if missing).
///
/// Returns the selected file path (with `.ieng` extension guaranteed),
/// or None if the dialog was cancelled.
pub async fn show_save_dialog(
    default_name: Option<&str>,
    starting_dir: Option<&std::path::Path>,
) -> Option<PathBuf> {
    let mut dialog = AsyncFileDialog::new()
        .set_title("Save Project")
        .add_filter(FILE_TYPE_NAME, &[FILE_EXTENSION.trim_start_matches('.')]);

    // Set the starting directory if provided
    if let Some(dir) = starting_dir {
        // If it's a file path, use the parent directory
        let dir = if dir.is_file() {
            dir.parent().unwrap_or(dir)
        } else {
            dir
        };
        if dir.exists() {
            dialog = dialog.set_directory(dir);
        }
    }

    // Set default filename with correct extension
    if let Some(name) = default_name {
        let file_name = if name.ends_with(FILE_EXTENSION) {
            name.to_string()
        } else {
            format!(
                "{}{}",
                name.replace(' ', "_").to_lowercase(),
                FILE_EXTENSION
            )
        };
        dialog = dialog.set_file_name(&file_name);
    }

    let file = dialog.save_file().await?;

    // Ensure the returned path always has the .ieng extension
    Some(ensure_extension(file.path().to_path_buf()))
}

/// Open a file dialog to select an export directory
///
/// Returns the selected directory path, or None if the dialog was cancelled.
pub async fn show_export_directory_dialog() -> Option<PathBuf> {
    let folder = AsyncFileDialog::new()
        .set_title("Select Export Directory")
        .pick_folder()
        .await?;

    Some(folder.path().to_path_buf())
}

// ============================================================================
// Project File Operations
// ============================================================================

/// Load a project from a file
///
/// Opens a file dialog if no path is provided.
pub async fn open_project(path: Option<PathBuf>) -> EngineResult<(ProjectGraph, PathBuf)> {
    let file_path = match path {
        Some(p) => p,
        None => show_open_dialog()
            .await
            .ok_or_else(|| EngineError::Cancelled)?,
    };

    let project = load_project(&file_path)?;
    Ok((project, file_path))
}

/// Save a project to a file
///
/// If `path` is `Some`, saves directly to that path (no dialog shown).
/// If `path` is `None`, shows a save dialog. The dialog will start in
/// `hint_dir` if provided (e.g. the last saved directory).
///
/// The returned path always has the `.ieng` extension.
pub async fn save_project_to_file(
    project: &ProjectGraph,
    path: Option<PathBuf>,
    hint_dir: Option<PathBuf>,
) -> EngineResult<PathBuf> {
    let file_path = match path {
        Some(p) => ensure_extension(p),
        None => {
            let start_dir = hint_dir.as_deref();
            show_save_dialog(Some(&project.meta.name), start_dir)
                .await
                .ok_or_else(|| EngineError::Cancelled)?
        }
    };

    save_project(project, &file_path)?;
    Ok(file_path)
}

/// Save a project with a new name/location (Save As)
///
/// Always opens a save dialog. If the project was previously saved,
/// the dialog starts in the same directory.
pub async fn save_project_as(
    project: &ProjectGraph,
    current_path: Option<&std::path::Path>,
) -> EngineResult<PathBuf> {
    let file_path = show_save_dialog(Some(&project.meta.name), current_path)
        .await
        .ok_or_else(|| EngineError::Cancelled)?;

    save_project(project, &file_path)?;
    Ok(file_path)
}

// ============================================================================
// Recent Projects
// ============================================================================

/// Maximum number of recent projects to track
pub const MAX_RECENT_PROJECTS: usize = 10;

/// Recent project entry
#[derive(Debug, Clone)]
pub struct RecentProject {
    /// File path
    pub path: PathBuf,
    /// Project name (extracted from metadata or filename)
    pub name: String,
    /// Last opened timestamp
    pub last_opened: chrono::DateTime<chrono::Utc>,
}

impl RecentProject {
    /// Create a new recent project entry
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            last_opened: chrono::Utc::now(),
        }
    }

    /// Create from a path, extracting name from filename
    pub fn from_path(path: PathBuf) -> Self {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self::new(path, name)
    }

    /// Check if the file still exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

/// Manage recent projects list
#[derive(Debug, Clone, Default)]
pub struct RecentProjectsManager {
    projects: Vec<RecentProject>,
}

impl RecentProjectsManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a project to the recent list
    pub fn add(&mut self, path: PathBuf, name: String) {
        // Remove existing entry with same path
        self.projects.retain(|p| p.path != path);

        // Add to front
        self.projects.insert(0, RecentProject::new(path, name));

        // Limit size
        if self.projects.len() > MAX_RECENT_PROJECTS {
            self.projects.truncate(MAX_RECENT_PROJECTS);
        }
    }

    /// Get all recent projects
    pub fn list(&self) -> &[RecentProject] {
        &self.projects
    }

    /// Get recent projects that still exist
    pub fn list_existing(&self) -> Vec<&RecentProject> {
        self.projects.iter().filter(|p| p.exists()).collect()
    }

    /// Remove a project from the list
    pub fn remove(&mut self, path: &PathBuf) {
        self.projects.retain(|p| &p.path != path);
    }

    /// Clear all recent projects
    pub fn clear(&mut self) {
        self.projects.clear();
    }

    /// Remove projects that no longer exist
    pub fn cleanup(&mut self) {
        self.projects.retain(|p| p.exists());
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Ensure a path has the correct file extension
pub fn ensure_extension(path: PathBuf) -> PathBuf {
    if path
        .extension()
        .is_some_and(|ext| ext == FILE_EXTENSION.trim_start_matches('.'))
    {
        path
    } else {
        let mut new_path = path.clone();
        let new_name = format!(
            "{}.{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project"),
            FILE_EXTENSION
        );
        new_path.set_file_name(new_name);
        new_path
    }
}

/// Get a display-friendly name from a path
pub fn display_name(path: &PathBuf) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Project")
        .to_string()
}

/// Check if a path is a valid project file
pub fn is_project_file(path: &PathBuf) -> bool {
    path.extension()
        .is_some_and(|ext| ext == FILE_EXTENSION.trim_start_matches('.'))
}

// ============================================================================
// Confirmation Dialogs
// ============================================================================

/// Result of an unsaved changes prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChangesAction {
    /// Save changes before proceeding
    Save,
    /// Discard changes and proceed
    Discard,
    /// Cancel the operation
    Cancel,
}

/// Confirmation dialog result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResult {
    /// User confirmed the action
    Confirm,
    /// User cancelled the action
    Cancel,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_extension() {
        let path = PathBuf::from("my_project");
        let result = ensure_extension(path);
        // Should add .ieng extension
        assert_eq!(result.to_str().unwrap(), "my_project.ieng");

        let path_with_ext = PathBuf::from("my_project.ieng");
        let result = ensure_extension(path_with_ext.clone());
        // Should not modify already correct extension
        assert_eq!(result, path_with_ext);
    }

    #[test]
    fn test_display_name() {
        let path = PathBuf::from("/home/user/projects/my_project.ieng");
        assert_eq!(display_name(&path), "my_project");

        let path = PathBuf::from("another_project");
        assert_eq!(display_name(&path), "another_project");
    }

    #[test]
    fn test_is_project_file() {
        let valid = PathBuf::from("test.ieng");
        assert!(is_project_file(&valid));

        let invalid = PathBuf::from("test.txt");
        assert!(!is_project_file(&invalid));
    }

    #[test]
    fn test_recent_project() {
        let path = PathBuf::from("/tmp/test.ieng");
        let recent = RecentProject::new(path.clone(), "Test Project".to_string());

        assert_eq!(recent.path, path);
        assert_eq!(recent.name, "Test Project");
    }

    #[test]
    fn test_recent_project_from_path() {
        let path = PathBuf::from("/home/user/my_awesome_project.ieng");
        let recent = RecentProject::from_path(path);

        assert_eq!(recent.name, "my_awesome_project");
    }

    #[test]
    fn test_recent_projects_manager() {
        let mut manager = RecentProjectsManager::new();

        manager.add(PathBuf::from("/tmp/project1.ieng"), "Project 1".to_string());
        manager.add(PathBuf::from("/tmp/project2.ieng"), "Project 2".to_string());

        assert_eq!(manager.list().len(), 2);
        // Most recent should be first
        assert_eq!(manager.list()[0].name, "Project 2");

        // Adding same path should move to front, not duplicate
        manager.add(
            PathBuf::from("/tmp/project1.ieng"),
            "Project 1 Updated".to_string(),
        );
        assert_eq!(manager.list().len(), 2);
        assert_eq!(manager.list()[0].name, "Project 1 Updated");
    }

    #[test]
    fn test_recent_projects_max_size() {
        let mut manager = RecentProjectsManager::new();

        for i in 0..15 {
            manager.add(
                PathBuf::from(format!("/tmp/project{}.ieng", i)),
                format!("Project {}", i),
            );
        }

        assert_eq!(manager.list().len(), MAX_RECENT_PROJECTS);
    }

    #[test]
    fn test_recent_projects_remove() {
        let mut manager = RecentProjectsManager::new();
        let path = PathBuf::from("/tmp/project.ieng");

        manager.add(path.clone(), "Project".to_string());
        assert_eq!(manager.list().len(), 1);

        manager.remove(&path);
        assert_eq!(manager.list().len(), 0);
    }
}
