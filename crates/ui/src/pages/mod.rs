//! Page Components for Immortal Engine
//!
//! This module contains all the page/view components for the application.
//! Each page represents a distinct screen or workflow in the application.
//!
//! ## Available Pages
//!
//! - **WelcomePage**: Initial landing page with quick actions
//! - **ProjectSetupPage**: Project configuration form
//! - **EntityDesignPage**: Visual entity designer with canvas
//! - **RelationshipsPage**: Relationship management between entities
//! - **EndpointsPage**: API endpoint configuration with CRUD toggles and security
//!

pub mod code_generation;
pub mod endpoints;
pub mod entity_design;
pub mod project_setup;
pub mod relationships;
pub mod welcome;

// Re-export page components for convenience
pub use code_generation::CodeGenerationPage;
pub use endpoints::EndpointsPage;
pub use entity_design::EntityDesignPage;
pub use project_setup::ProjectSetupPage;
pub use relationships::RelationshipsPage;
pub use welcome::WelcomePage;
