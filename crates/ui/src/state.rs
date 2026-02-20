//! Application State Management for Immortal Engine
//!
//! This module provides centralized state management using Dioxus 0.7 Signals.
//! It handles all application state including project data, UI state, selection,
//! canvas state, and history for undo/redo operations.

use dioxus::prelude::*;
use imortal_core::{EngineError, EngineResult, Position};
use imortal_ir::ProjectGraph;
use std::collections::HashSet;
use uuid::Uuid;

// ============================================================================
// Page Navigation
// ============================================================================

/// Application pages/views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    /// Welcome/landing page
    #[default]
    Welcome,
    /// Project setup and configuration
    ProjectSetup,
    /// Entity design canvas
    EntityDesign,
    /// Relationship manager
    Relationships,
    /// Endpoint configuration
    Endpoints,
    /// Code generation preview
    CodeGeneration,
    /// Application settings
    Settings,
}

impl Page {
    /// Get the display name for this page
    pub fn display_name(&self) -> &'static str {
        match self {
            Page::Welcome => "Welcome",
            Page::ProjectSetup => "Project Setup",
            Page::EntityDesign => "Entity Design",
            Page::Relationships => "Relationships",
            Page::Endpoints => "Endpoints",
            Page::CodeGeneration => "Code Generation",
            Page::Settings => "Settings",
        }
    }

    /// Get the icon emoji for this page (for UI display)
    pub fn icon(&self) -> &'static str {
        match self {
            Page::Welcome => "🏠",
            Page::ProjectSetup => "⚙️",
            Page::EntityDesign => "🗃️",
            Page::Relationships => "🔗",
            Page::Endpoints => "🔌",
            Page::CodeGeneration => "⚡",
            Page::Settings => "🔧",
        }
    }

    /// Check if this page requires a project to be loaded
    pub fn requires_project(&self) -> bool {
        !matches!(self, Page::Welcome | Page::Settings)
    }
}

// ============================================================================
// Selection State
// ============================================================================

/// Tracks what items are currently selected in the UI
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selection {
    /// Selected entity IDs
    pub entities: HashSet<Uuid>,
    /// Selected relationship IDs
    pub relationships: HashSet<Uuid>,
    /// Selected endpoint group IDs
    pub endpoints: HashSet<Uuid>,
    /// Selected field ID (entity_id, field_id)
    pub field: Option<(Uuid, Uuid)>,
}

impl Selection {
    /// Create a new empty selection
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.entities.clear();
        self.relationships.clear();
        self.endpoints.clear();
        self.field = None;
    }

    /// Check if anything is selected
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
            && self.relationships.is_empty()
            && self.endpoints.is_empty()
            && self.field.is_none()
    }

    /// Check if a specific entity is selected
    pub fn is_entity_selected(&self, id: &Uuid) -> bool {
        self.entities.contains(id)
    }

    /// Check if a specific relationship is selected
    pub fn is_relationship_selected(&self, id: &Uuid) -> bool {
        self.relationships.contains(id)
    }

    /// Select a single entity (clears other selections)
    pub fn select_entity(&mut self, id: Uuid) {
        self.clear();
        self.entities.insert(id);
    }

    /// Select a single relationship (clears other selections)
    pub fn select_relationship(&mut self, id: Uuid) {
        self.clear();
        self.relationships.insert(id);
    }

    /// Toggle entity selection (for multi-select with Ctrl/Cmd)
    pub fn toggle_entity(&mut self, id: Uuid) {
        if self.entities.contains(&id) {
            self.entities.remove(&id);
        } else {
            self.entities.insert(id);
        }
    }

    /// Add entity to selection (for multi-select)
    pub fn add_entity(&mut self, id: Uuid) {
        self.entities.insert(id);
    }

    /// Get the single selected entity (if exactly one is selected)
    pub fn single_entity(&self) -> Option<Uuid> {
        if self.entities.len() == 1 {
            self.entities.iter().next().copied()
        } else {
            None
        }
    }

    /// Get the single selected relationship (if exactly one is selected)
    pub fn single_relationship(&self) -> Option<Uuid> {
        if self.relationships.len() == 1 {
            self.relationships.iter().next().copied()
        } else {
            None
        }
    }

    /// Get count of selected items
    pub fn count(&self) -> usize {
        self.entities.len()
            + self.relationships.len()
            + self.endpoints.len()
            + if self.field.is_some() { 1 } else { 0 }
    }
}

// ============================================================================
// Canvas State
// ============================================================================

/// State for the visual canvas (pan, zoom, interactions)
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasState {
    /// Current pan offset (x, y)
    pub pan: Position,
    /// Current zoom level (1.0 = 100%)
    pub zoom: f32,
    /// Whether the canvas is being panned (middle mouse or space+drag)
    pub is_panning: bool,
    /// Whether a connection is being drawn
    pub is_connecting: bool,
    /// Start entity for connection being drawn
    pub connection_start: Option<(Uuid, ConnectionPort)>,
    /// Current mouse position (for connection line preview)
    pub mouse_position: Position,
    /// Entity being dragged
    pub dragging_entity: Option<Uuid>,
    /// Drag start position
    pub drag_start: Option<Position>,
    /// Whether grid snapping is enabled
    pub snap_to_grid: bool,
    /// Grid size in pixels
    pub grid_size: f32,
    /// Whether to show grid
    pub show_grid: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan: Position::new(0.0, 0.0),
            zoom: 1.0,
            is_panning: false,
            is_connecting: false,
            connection_start: None,
            mouse_position: Position::new(0.0, 0.0),
            dragging_entity: None,
            drag_start: None,
            snap_to_grid: true,
            grid_size: 20.0,
            show_grid: true,
        }
    }
}

impl CanvasState {
    /// Create a new canvas state
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset to default view
    pub fn reset_view(&mut self) {
        self.pan = Position::new(0.0, 0.0);
        self.zoom = 1.0;
    }

    /// Zoom in by a step
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.2).min(3.0);
    }

    /// Zoom out by a step
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.2).max(0.25);
    }

    /// Set zoom to fit (called with calculated value)
    pub fn zoom_to_fit(&mut self, zoom: f32, center: Position) {
        self.zoom = zoom.clamp(0.25, 3.0);
        self.pan = center;
    }

    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen_pos: Position) -> Position {
        Position::new(
            (screen_pos.x - self.pan.x) / self.zoom,
            (screen_pos.y - self.pan.y) / self.zoom,
        )
    }

    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas_pos: Position) -> Position {
        Position::new(
            canvas_pos.x * self.zoom + self.pan.x,
            canvas_pos.y * self.zoom + self.pan.y,
        )
    }

    /// Snap position to grid if enabled
    pub fn snap_position(&self, pos: Position) -> Position {
        if self.snap_to_grid {
            Position::new(
                (pos.x / self.grid_size).round() * self.grid_size,
                (pos.y / self.grid_size).round() * self.grid_size,
            )
        } else {
            pos
        }
    }

    /// Start drawing a connection
    pub fn start_connection(&mut self, entity_id: Uuid, port: ConnectionPort) {
        self.is_connecting = true;
        self.connection_start = Some((entity_id, port));
    }

    /// Cancel connection drawing
    pub fn cancel_connection(&mut self) {
        self.is_connecting = false;
        self.connection_start = None;
    }

    /// Start dragging an entity
    pub fn start_drag(&mut self, entity_id: Uuid, position: Position) {
        self.dragging_entity = Some(entity_id);
        self.drag_start = Some(position);
    }

    /// Stop dragging
    pub fn stop_drag(&mut self) {
        self.dragging_entity = None;
        self.drag_start = None;
    }
}

/// Connection port on an entity card
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionPort {
    /// Input port (left side, for incoming relationships)
    Input,
    /// Output port (right side, for outgoing relationships)
    Output,
}

// ============================================================================
// UI State
// ============================================================================

/// General UI state (dialogs, panels, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct UiState {
    /// Whether the sidebar is collapsed
    pub sidebar_collapsed: bool,
    /// Whether the properties panel is collapsed
    pub properties_collapsed: bool,
    /// Currently active page
    pub active_page: Page,
    /// Active dialog (if any)
    pub active_dialog: Option<Dialog>,
    /// Status bar message
    pub status_message: Option<StatusMessage>,
    /// Whether dark mode is enabled
    pub dark_mode: bool,
    /// Whether the app is in fullscreen
    pub fullscreen: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            sidebar_collapsed: false,
            properties_collapsed: false,
            active_page: Page::Welcome,
            active_dialog: None,
            status_message: None,
            dark_mode: true, // Default to dark mode
            fullscreen: false,
        }
    }
}

impl UiState {
    /// Create new UI state
    pub fn new() -> Self {
        Self::default()
    }

    /// Navigate to a page
    pub fn navigate(&mut self, page: Page) {
        self.active_page = page;
    }

    /// Show a dialog
    pub fn show_dialog(&mut self, dialog: Dialog) {
        self.active_dialog = Some(dialog);
    }

    /// Close the current dialog
    pub fn close_dialog(&mut self) {
        self.active_dialog = None;
    }

    /// Set status message
    pub fn set_status(&mut self, message: impl Into<String>, level: StatusLevel) {
        self.status_message = Some(StatusMessage {
            text: message.into(),
            level,
        });
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Toggle sidebar
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
    }

    /// Toggle properties panel
    pub fn toggle_properties(&mut self) {
        self.properties_collapsed = !self.properties_collapsed;
    }

    /// Toggle dark mode
    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
    }
}

/// Dialog types
#[derive(Debug, Clone, PartialEq)]
pub enum Dialog {
    /// New project dialog
    NewProject,
    /// Open project dialog
    OpenProject,
    /// Save project as dialog
    SaveProjectAs,
    /// New entity dialog
    NewEntity,
    /// Edit entity dialog
    EditEntity(Uuid),
    /// New field dialog
    NewField(Uuid), // entity_id
    /// Edit field dialog
    EditField(Uuid, Uuid), // entity_id, field_id
    /// New relationship dialog (with optional pre-selected entities)
    NewRelationship(Option<Uuid>, Option<Uuid>), // from_entity_id, to_entity_id
    /// Edit relationship dialog
    EditRelationship(Uuid), // relationship_id
    /// New endpoint dialog (with optional pre-selected entity)
    NewEndpoint(Option<Uuid>), // entity_id
    /// Edit endpoint dialog
    EditEndpoint(Uuid), // endpoint_id
    /// Delete confirmation dialog
    ConfirmDelete(DeleteTarget),
    /// Export/generate code dialog
    Export,
    /// Project settings dialog
    ProjectSettings,
    /// About dialog
    About,
    /// Error dialog
    Error(String),
}

/// Target for delete confirmation
#[derive(Debug, Clone, PartialEq)]
pub enum DeleteTarget {
    Entity(Uuid),
    Entities(Vec<Uuid>),
    Field(Uuid, Uuid), // entity_id, field_id
    Relationship(Uuid),
    Endpoint(Uuid),
}

/// Status message for the status bar
#[derive(Debug, Clone, PartialEq)]
pub struct StatusMessage {
    pub text: String,
    pub level: StatusLevel,
}

/// Status message severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

// ============================================================================
// History (Undo/Redo)
// ============================================================================

/// History state for undo/redo operations
#[derive(Debug, Clone)]
pub struct History {
    /// Past states (for undo)
    past: Vec<HistorySnapshot>,
    /// Future states (for redo)
    future: Vec<HistorySnapshot>,
    /// Maximum history size
    max_size: usize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            max_size: 50,
        }
    }
}

impl History {
    /// Create new history
    pub fn new() -> Self {
        Self::default()
    }

    /// Create history with custom max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            max_size,
            ..Default::default()
        }
    }

    /// Push a new snapshot (clears redo stack)
    pub fn push(&mut self, snapshot: HistorySnapshot) {
        self.past.push(snapshot);
        self.future.clear();

        // Limit history size
        if self.past.len() > self.max_size {
            self.past.remove(0);
        }
    }

    /// Undo: pop from past, push current to future
    pub fn undo(&mut self, current: HistorySnapshot) -> Option<HistorySnapshot> {
        if let Some(previous) = self.past.pop() {
            self.future.push(current);
            Some(previous)
        } else {
            None
        }
    }

    /// Redo: pop from future, push current to past
    pub fn redo(&mut self, current: HistorySnapshot) -> Option<HistorySnapshot> {
        if let Some(next) = self.future.pop() {
            self.past.push(current);
            Some(next)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }

    /// Get undo stack size
    pub fn undo_count(&self) -> usize {
        self.past.len()
    }

    /// Get redo stack size
    pub fn redo_count(&self) -> usize {
        self.future.len()
    }
}

/// A snapshot of project state for history
#[derive(Debug, Clone)]
pub struct HistorySnapshot {
    /// Description of the action
    pub action: String,
    /// Serialized project state
    pub project_json: String,
}

impl HistorySnapshot {
    /// Create a new snapshot
    pub fn new(action: impl Into<String>, project: &ProjectGraph) -> EngineResult<Self> {
        let project_json = serde_json::to_string(project)
            .map_err(|e| EngineError::Internal(format!("Failed to serialize project: {}", e)))?;

        Ok(Self {
            action: action.into(),
            project_json,
        })
    }

    /// Restore project from snapshot
    pub fn restore(&self) -> EngineResult<ProjectGraph> {
        serde_json::from_str(&self.project_json)
            .map_err(|e| EngineError::Internal(format!("Failed to deserialize project: {}", e)))
    }
}

// ============================================================================
// Application State
// ============================================================================

/// Main application state container
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current project (None if no project loaded)
    pub project: Option<ProjectGraph>,
    /// Path to current project file (None if new/unsaved)
    pub project_path: Option<std::path::PathBuf>,
    /// Whether the project has unsaved changes
    pub is_dirty: bool,
    /// Selection state
    pub selection: Selection,
    /// Canvas state
    pub canvas: CanvasState,
    /// UI state
    pub ui: UiState,
    /// History for undo/redo
    pub history: History,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            project: None,
            project_path: None,
            is_dirty: false,
            selection: Selection::new(),
            canvas: CanvasState::new(),
            ui: UiState::new(),
            history: History::new(),
        }
    }
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a project is loaded
    pub fn has_project(&self) -> bool {
        self.project.is_some()
    }

    /// Get the project name (or "Untitled" if none)
    pub fn project_name(&self) -> &str {
        self.project
            .as_ref()
            .map(|p| p.meta.name.as_str())
            .unwrap_or("Untitled")
    }

    /// Create a new project
    pub fn new_project(&mut self, name: impl Into<String>) {
        self.project = Some(ProjectGraph::new(name));
        self.project_path = None;
        self.is_dirty = true;
        self.selection.clear();
        self.canvas.reset_view();
        self.history.clear();
        self.ui.navigate(Page::ProjectSetup);
    }

    /// Load a project
    pub fn load_project(&mut self, project: ProjectGraph, path: std::path::PathBuf) {
        self.project = Some(project);
        self.project_path = Some(path);
        self.is_dirty = false;
        self.selection.clear();
        self.canvas.reset_view();
        self.history.clear();
        self.ui.navigate(Page::EntityDesign);
    }

    /// Close current project
    pub fn close_project(&mut self) {
        self.project = None;
        self.project_path = None;
        self.is_dirty = false;
        self.selection.clear();
        self.canvas.reset_view();
        self.history.clear();
        self.ui.navigate(Page::Welcome);
    }

    /// Mark project as saved
    pub fn mark_saved(&mut self, path: Option<std::path::PathBuf>) {
        self.is_dirty = false;
        if let Some(p) = path {
            self.project_path = Some(p);
        }
    }

    /// Mark project as dirty (has unsaved changes)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Save current state to history before making changes
    pub fn save_to_history(&mut self, action: impl Into<String>) {
        if let Some(project) = &self.project {
            if let Ok(snapshot) = HistorySnapshot::new(action, project) {
                self.history.push(snapshot);
            }
        }
    }

    /// Undo last action
    pub fn undo(&mut self) -> bool {
        if let Some(project) = &self.project {
            if let Ok(current) = HistorySnapshot::new("current", project) {
                if let Some(previous) = self.history.undo(current) {
                    if let Ok(restored) = previous.restore() {
                        self.project = Some(restored);
                        self.is_dirty = true;
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Redo last undone action
    pub fn redo(&mut self) -> bool {
        if let Some(project) = &self.project {
            if let Ok(current) = HistorySnapshot::new("current", project) {
                if let Some(next) = self.history.redo(current) {
                    if let Ok(restored) = next.restore() {
                        self.project = Some(restored);
                        self.is_dirty = true;
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get window title
    pub fn window_title(&self) -> String {
        let name = self.project_name();
        let dirty = if self.is_dirty { " •" } else { "" };
        format!("{}{} - Immortal Engine", name, dirty)
    }
}

// ============================================================================
// Global State Context
// ============================================================================

/// Global application state signal
/// Use this in components to access and modify app state
pub static APP_STATE: GlobalSignal<AppState> = Signal::global(AppState::new);

/// Initialize the global app state
/// Call this once at app startup
pub fn init_app_state() {
    // State is initialized with defaults via Signal::global
    // Add any additional initialization here if needed
}

// ============================================================================
// State Hooks (for component use)
// ============================================================================

/// Hook to access the current page
pub fn use_current_page() -> Page {
    let state = APP_STATE.read();
    state.ui.active_page
}

/// Hook to check if project is loaded
pub fn use_has_project() -> bool {
    let state = APP_STATE.read();
    state.has_project()
}

/// Hook to get project name
pub fn use_project_name() -> String {
    let state = APP_STATE.read();
    state.project_name().to_string()
}

/// Hook to check if there are unsaved changes
pub fn use_is_dirty() -> bool {
    let state = APP_STATE.read();
    state.is_dirty
}

/// Hook to get selection state
pub fn use_selection() -> Selection {
    let state = APP_STATE.read();
    state.selection.clone()
}

/// Hook to get canvas state
pub fn use_canvas_state() -> CanvasState {
    let state = APP_STATE.read();
    state.canvas.clone()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection() {
        let mut selection = Selection::new();
        assert!(selection.is_empty());

        let id = Uuid::new_v4();
        selection.select_entity(id);
        assert!(!selection.is_empty());
        assert!(selection.is_entity_selected(&id));
        assert_eq!(selection.single_entity(), Some(id));

        selection.clear();
        assert!(selection.is_empty());
    }

    #[test]
    fn test_selection_toggle() {
        let mut selection = Selection::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        selection.toggle_entity(id1);
        assert!(selection.is_entity_selected(&id1));

        selection.toggle_entity(id2);
        assert!(selection.is_entity_selected(&id1));
        assert!(selection.is_entity_selected(&id2));
        assert_eq!(selection.count(), 2);

        selection.toggle_entity(id1);
        assert!(!selection.is_entity_selected(&id1));
        assert!(selection.is_entity_selected(&id2));
    }

    #[test]
    fn test_canvas_state() {
        let mut canvas = CanvasState::new();
        assert_eq!(canvas.zoom, 1.0);

        canvas.zoom_in();
        assert!(canvas.zoom > 1.0);

        canvas.zoom_out();
        canvas.zoom_out();
        assert!(canvas.zoom < 1.0);

        canvas.reset_view();
        assert_eq!(canvas.zoom, 1.0);
    }

    #[test]
    fn test_canvas_coordinate_conversion() {
        let mut canvas = CanvasState::new();
        canvas.pan = Position::new(100.0, 50.0);
        canvas.zoom = 2.0;

        let screen = Position::new(200.0, 150.0);
        let canvas_pos = canvas.screen_to_canvas(screen);
        assert_eq!(canvas_pos.x, 50.0);
        assert_eq!(canvas_pos.y, 50.0);

        let back = canvas.canvas_to_screen(canvas_pos);
        assert_eq!(back.x, screen.x);
        assert_eq!(back.y, screen.y);
    }

    #[test]
    fn test_ui_state() {
        let mut ui = UiState::new();
        assert_eq!(ui.active_page, Page::Welcome);

        ui.navigate(Page::EntityDesign);
        assert_eq!(ui.active_page, Page::EntityDesign);

        ui.show_dialog(Dialog::NewProject);
        assert!(ui.active_dialog.is_some());

        ui.close_dialog();
        assert!(ui.active_dialog.is_none());
    }

    #[test]
    fn test_history() {
        let mut history = History::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        let project = ProjectGraph::new("test");
        let snapshot1 = HistorySnapshot::new("action1", &project).unwrap();
        history.push(snapshot1);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let snapshot2 = HistorySnapshot::new("action2", &project).unwrap();
        if let Some(_) = history.undo(snapshot2) {
            assert!(!history.can_undo());
            assert!(history.can_redo());
        }
    }

    #[test]
    fn test_app_state() {
        let mut state = AppState::new();
        assert!(!state.has_project());
        assert_eq!(state.project_name(), "Untitled");

        state.new_project("My Project");
        assert!(state.has_project());
        assert_eq!(state.project_name(), "My Project");
        assert!(state.is_dirty);

        state.mark_saved(None);
        assert!(!state.is_dirty);

        state.close_project();
        assert!(!state.has_project());
    }

    #[test]
    fn test_page_properties() {
        assert!(!Page::Welcome.requires_project());
        assert!(!Page::Settings.requires_project());
        assert!(Page::EntityDesign.requires_project());
        assert!(Page::Endpoints.requires_project());
    }
}
