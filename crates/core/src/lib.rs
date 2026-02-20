//! # Immortal Core
//!
//! Core types, traits, and error handling for Immortal Engine.
//!
//! This crate provides the foundational building blocks used throughout
//! the Immortal Engine ecosystem, including:
//!
//! - **Types**: Data types, geometric primitives (Position, Size, Rect)
//! - **Traits**: Common behaviors like `Validatable` and `CodeGenerable`
//! - **Errors**: Unified error handling with `EngineError` and `EngineResult`
//!

pub mod error;
pub mod traits;
pub mod types;

// Re-export commonly used items at crate root
pub use error::{EngineError, EngineResult};
pub use traits::{CodeGenContext, CodeGenerable, Persistable, Validatable};
pub use types::{
    ConfigValue, DataType, DatabaseType, EndpointId, EntityId, FieldId, IdType, Position, Rect,
    ReferentialAction, RelationType, RelationshipId, Size, Validation,
};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const NAME: &str = env!("CARGO_PKG_NAME");
