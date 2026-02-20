//! # UI Hooks
//!
//! Custom Dioxus hooks for the Immortal Engine UI.
//!
//! This module provides reusable hooks for managing:
//! - Canvas interactions (pan, zoom, drag)
//! - Connection drawing (drag-to-connect for relationships)
//! - Selection state
//! - History (undo/redo)
//! - Project state
//! - Code generation

// ============================================================================
// Module Declarations
// ============================================================================

pub mod use_canvas;
pub mod use_connection;

// ============================================================================
// Re-exports
// ============================================================================

pub use use_canvas::{CanvasInteractions, DragState, PanState, use_canvas_interactions};
pub use use_connection::{
    ConnectionDrawingState, ConnectionResult, UseConnectionDrawing, use_connection_drawing,
};
