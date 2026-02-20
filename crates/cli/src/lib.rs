//! # Immortal CLI
//!
//! Command-line interface for Immortal Engine.
//!
//! This crate provides CLI tools for creating, validating, and generating
//! projects from the command line without using the GUI.
//!
//! ## Commands
//!
//! - `new` - Create a new Immortal Engine project
//! - `generate` - Generate code from a project file
//! - `validate` - Validate a project file
//! - `info` - Display information about a project
//!

// Re-export dependencies for use in main.rs
pub use imortal_codegen;
pub use imortal_core;
pub use imortal_ir;

/// CLI version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// CLI name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Verify the crate compiles correctly.
pub fn placeholder() -> &'static str {
    "imortal_cli placeholder - implementation pending"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert!(!placeholder().is_empty());
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
