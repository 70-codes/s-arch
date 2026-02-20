# Immortal Engine v2.0 (S-Arch-P) - Comprehensive Implementation Specification

> **Document Version:** 1.0  
> **Last Updated:** January 29, 2026  
> **Rust Edition:** 2024  
> **Author:** Stephen Kinuthia

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Project Vision & Goals](#2-project-vision--goals)
3. [Technology Stack](#3-technology-stack)
4. [System Architecture](#4-system-architecture)
5. [Data Models & Core Types](#5-data-models--core-types)
6. [User Interface Design](#6-user-interface-design)
7. [Core Features Specification](#7-core-features-specification)
8. [Code Generation Engine](#8-code-generation-engine)
9. [Security Implementation](#9-security-implementation)
10. [Database Support](#10-database-support)
11. [Export System](#11-export-system)
12. [API Reference](#12-api-reference)
13. [Implementation Roadmap](#13-implementation-roadmap)
14. [Testing Strategy](#14-testing-strategy)
15. [Deployment & Distribution](#15-deployment--distribution)
16. [Appendices](#16-appendices)

---

## 1. Executive Summary

### 1.1 What is Immortal Engine v2.0?

Immortal Engine v2.0 (codenamed **S-Arch-P** - Schema Architecture Platform) is a **desktop application** built entirely in Rust that enables developers to visually design and generate production-ready Rust backend and fullstack applications. It replaces the traditional approach of writing boilerplate code by allowing users to:

- **Visually define data entities** with fields, constraints, and validations
- **Draw relationships** between entities (one-to-one, one-to-many, many-to-many)
- **Configure REST API endpoints** with CRUD operations, security settings, and rate limiting
- **Choose project type**: REST API only or Fullstack (with Dioxus frontend)
- **Generate complete Rust projects** ready to compile and deploy
- **Export to any filesystem location** on the user's machine

### 1.2 Key Differentiators from v1.0

| Aspect | v1.0 (imortal_engine) | v2.0 (S-Arch-P) |
|--------|----------------------|-----------------|
| **UI Framework** | egui (immediate mode) | Dioxus Desktop (reactive) |
| **Styling** | Custom egui theming | Tailwind CSS |
| **Generated Backend** | Generic Rust | Axum (opinionated, production-ready) |
| **Generated ORM** | Basic structs | SeaORM with migrations |
| **Fullstack Option** | Not available | Dioxus Web frontend |
| **Authentication** | Basic concept | JWT with RBAC, configurable |
| **Target Audience** | Prototyping | Production deployments |

### 1.3 Success Criteria

1. **Functional**: Generate a complete, compiling Rust project from visual design
2. **Usable**: 90% of tasks achievable via mouse-driven visual interface
3. **Performant**: Handle projects with 50+ entities without lag
4. **Extensible**: Plugin architecture for custom components
5. **Reliable**: 95%+ test coverage on code generation

---

## 2. Project Vision & Goals

### 2.1 Problem Statement

Developing backend applications in Rust involves significant boilerplate:
- Entity/model definitions
- Database migrations
- CRUD handlers
- Route configurations
- Authentication/authorization middleware
- Input validation
- Error handling

This repetitive work slows development and introduces inconsistencies across projects.

### 2.2 Solution

Immortal Engine provides a visual interface where developers:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            USER WORKFLOW                                     │
└─────────────────────────────────────────────────────────────────────────────┘

     DESIGN                    CONFIGURE                  GENERATE
        │                          │                          │
        ▼                          ▼                          ▼
┌───────────────┐          ┌───────────────┐          ┌───────────────┐
│ Define        │          │ Set up        │          │ Export        │
│ Entities      │   ───▶   │ Endpoints     │   ───▶   │ Complete      │
│ & Relations   │          │ & Security    │          │ Rust Project  │
└───────────────┘          └───────────────┘          └───────────────┘
        │                          │                          │
        ▼                          ▼                          ▼
   Visual cards              CRUD toggles              src/models/*.rs
   with fields               Auth config               src/handlers/*.rs
   Drag-to-connect           Rate limits               migrations/*.sql
```

### 2.3 Target Users

| User Type | Use Case | Key Needs |
|-----------|----------|-----------|
| **Solo Developers** | Rapid prototyping | Speed, simplicity |
| **Startups** | MVP development | Full project generation |
| **Enterprise Teams** | Standardized backends | Consistency, security |
| **Educators** | Teaching Rust backends | Visual understanding |
| **API Designers** | Contract-first development | Schema visualization |

### 2.4 Non-Goals (Out of Scope for v2.0)

- Real-time collaboration (planned for v3.0)
- Cloud deployment automation
- GraphQL generation (REST only for now)
- Custom code injection (templates only)
- Mobile app generation

---

## 3. Technology Stack

### 3.1 Immortal Engine Application Stack

| Layer | Technology | Version | Justification |
|-------|------------|---------|---------------|
| **Language** | Rust | 2024 Edition | Memory safety, performance, ecosystem |
| **UI Framework** | Dioxus Desktop | 0.6.x | Native Rust, reactive, cross-platform |
| **Styling** | Tailwind CSS | 3.x | Utility-first, rapid UI development |
| **State Management** | Dioxus Signals | Built-in | Reactive, fine-grained updates |
| **File Dialogs** | rfd crate | Latest | Native OS file dialogs |
| **Serialization** | Serde + JSON/TOML | 1.x | Project persistence |
| **Code Generation** | quote + syn | 1.x / 2.x | Rust token manipulation |
| **Async Runtime** | Tokio | 1.x | Async file I/O |
| **Error Handling** | thiserror + anyhow | 1.x | Ergonomic errors |
| **Logging** | tracing | 0.1.x | Structured logging |

### 3.2 Generated Project Stack (REST API)

| Component | Technology | Justification |
|-----------|------------|---------------|
| **Web Framework** | Axum 0.7+ | Modern, Tower-based, excellent types |
| **ORM** | SeaORM | Async, type-safe, migration support |
| **Validation** | validator | Derive macros, comprehensive rules |
| **Authentication** | axum-extra + JWT | Industry standard, stateless |
| **Database Driver** | sqlx | Compile-time query checking |
| **Configuration** | config crate | Multi-source config loading |
| **API Documentation** | utoipa | OpenAPI spec generation |
| **Testing** | tokio-test | Async test utilities |

### 3.3 Generated Project Stack (Fullstack)

In addition to REST API stack:

| Component | Technology | Justification |
|-----------|------------|---------------|
| **Frontend Framework** | Dioxus Web | Same language, shared types |
| **HTTP Client** | reqwest | Async, widely used |
| **Shared Types** | Separate crate | Type safety between frontend/backend |
| **CSS Framework** | Tailwind CSS | Consistent styling |

### 3.4 Backend Framework Comparison

We evaluated three major Rust web frameworks:

| Criteria | Axum | Actix-Web | Rocket |
|----------|------|-----------|--------|
| **Performance** | ★★★★★ | ★★★★★ | ★★★★☆ |
| **Type Safety** | ★★★★★ | ★★★★☆ | ★★★★★ |
| **Learning Curve** | ★★★★☆ | ★★★☆☆ | ★★★★★ |
| **Ecosystem** | ★★★★★ | ★★★★★ | ★★★☆☆ |
| **Active Development** | ★★★★★ | ★★★★☆ | ★★★☆☆ |
| **Tower Compatibility** | ★★★★★ | ★★☆☆☆ | ★★☆☆☆ |
| **Async Model** | Tokio Native | Custom | Tokio |

**Decision: Axum** - Best balance of modern design, type safety, and ecosystem integration.

---

## 4. System Architecture

### 4.1 High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         IMMORTAL ENGINE v2.0                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                    PRESENTATION LAYER (Dioxus Desktop)                  │ │
│  │  ┌────────────┐ ┌────────────────────────────┐ ┌──────────────────┐   │ │
│  │  │  Sidebar   │ │      Canvas Editor         │ │  Properties      │   │ │
│  │  │  ────────  │ │      ──────────────        │ │  Panel           │   │ │
│  │  │  • Pages   │ │  ┌─────┐    ┌─────┐       │ │  ──────────────  │   │ │
│  │  │  • Tree    │ │  │User │───▶│Post │       │ │  • Entity props  │   │ │
│  │  │  • Search  │ │  └─────┘    └─────┘       │ │  • Field editor  │   │ │
│  │  └────────────┘ └────────────────────────────┘ └──────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      APPLICATION STATE (Signals)                        │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │ │
│  │  │ ProjectState │  │ CanvasState  │  │ SelectionSt  │  │ UIState    │ │ │
│  │  │ • entities   │  │ • pan/zoom   │  │ • selected   │  │ • page     │ │ │
│  │  │ • relations  │  │ • dragging   │  │ • multi      │  │ • dialogs  │ │ │
│  │  │ • endpoints  │  │ • connecting │  │              │  │ • alerts   │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         CORE ENGINE LAYER                               │ │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌────────────┐ │ │
│  │  │  imortal_core │ │   imortal_ir  │ │   imortal_    │ │  imortal_  │ │ │
│  │  │  ───────────  │ │   ──────────  │ │   components  │ │  codegen   │ │ │
│  │  │  Types        │ │  ProjectGraph │ │  ComponentReg │ │  Generator │ │ │
│  │  │  Traits       │ │  Node/Edge    │ │  Definitions  │ │  Templates │ │ │
│  │  │  Errors       │ │  Validation   │ │  Instantiate  │ │  Output    │ │ │
│  │  └───────────────┘ └───────────────┘ └───────────────┘ └────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│                                      ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        PERSISTENCE LAYER                                │ │
│  │  ┌─────────────────────────┐    ┌─────────────────────────────────┐   │ │
│  │  │  Project Files (.ieng)  │    │  Generated Output (Rust Project) │   │ │
│  │  │  • JSON serialization   │    │  • src/ directory                │   │ │
│  │  │  • Version migration    │    │  • migrations/ directory         │   │ │
│  │  └─────────────────────────┘    └─────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Workspace Structure

```
s-arch-p/
├── Cargo.toml                      # Workspace root
├── rust-toolchain.toml             # Rust 2024 edition
├── .gitignore
├── README.md
├── LICENSE
│
├── src/
│   └── main.rs                     # Dioxus Desktop entry point
│
├── crates/
│   │
│   ├── core/                       # imortal_core - Shared types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs            # DataType, Position, Size, etc.
│   │       ├── traits.rs           # Validatable, Codegen traits
│   │       ├── error.rs            # EngineError, Result types
│   │       └── config.rs           # Global configuration
│   │
│   ├── ir/                         # imortal_ir - Intermediate Representation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── project.rs          # ProjectMeta, ProjectSettings
│   │       ├── entity.rs           # Entity definition (renamed from node)
│   │       ├── field.rs            # Field definitions with validations
│   │       ├── relationship.rs     # Relationship/Edge definitions
│   │       ├── endpoint.rs         # API Endpoint definitions
│   │       ├── graph.rs            # ProjectGraph container
│   │       ├── validation.rs       # Schema validation rules
│   │       └── serialization.rs    # Save/Load operations
│   │
│   ├── codegen/                    # imortal_codegen - Code Generation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── generator.rs        # Main CodeGenerator orchestrator
│   │       ├── context.rs          # Generation context/state
│   │       ├── rust/
│   │       │   ├── mod.rs
│   │       │   ├── cargo.rs        # Cargo.toml generation
│   │       │   ├── models.rs       # SeaORM entity generation
│   │       │   ├── handlers.rs     # Axum handler generation
│   │       │   ├── routes.rs       # Router setup generation
│   │       │   ├── auth.rs         # JWT auth generation
│   │       │   ├── middleware.rs   # Middleware generation
│   │       │   ├── config.rs       # Config module generation
│   │       │   ├── error.rs        # Error types generation
│   │       │   ├── main.rs         # main.rs generation
│   │       │   └── tests.rs        # Test file generation
│   │       ├── migrations/
│   │       │   ├── mod.rs
│   │       │   └── sql.rs          # SQL migration generation
│   │       ├── frontend/
│   │       │   ├── mod.rs
│   │       │   ├── dioxus.rs       # Dioxus component generation
│   │       │   ├── pages.rs        # Page generation
│   │       │   └── api_client.rs   # API client generation
│   │       └── templates/
│   │           ├── mod.rs
│   │           └── snippets.rs     # Code snippets/templates
│   │
│   ├── ui/                         # imortal_ui - Dioxus UI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs              # Main App component
│   │       ├── state.rs            # Application state (Signals)
│   │       ├── theme.rs            # Theming/dark mode
│   │       ├── icons.rs            # Icon components
│   │       │
│   │       ├── components/         # Reusable UI components
│   │       │   ├── mod.rs
│   │       │   ├── sidebar.rs      # Navigation sidebar
│   │       │   ├── toolbar.rs      # Top toolbar
│   │       │   ├── canvas.rs       # Visual editor canvas
│   │       │   ├── properties.rs   # Properties panel
│   │       │   ├── entity_card.rs  # Entity node card
│   │       │   ├── endpoint_card.rs# Endpoint configuration card
│   │       │   ├── connection.rs   # Relationship line component
│   │       │   ├── port.rs         # Connection port component
│   │       │   ├── field_row.rs    # Field row in entity
│   │       │   ├── dialogs/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── new_project.rs
│   │       │   │   ├── export.rs
│   │       │   │   ├── field_editor.rs
│   │       │   │   └── confirm.rs
│   │       │   └── inputs/
│   │       │       ├── mod.rs
│   │       │       ├── text.rs
│   │       │       ├── select.rs
│   │       │       ├── checkbox.rs
│   │       │       └── number.rs
│   │       │
│   │       ├── pages/              # Application pages
│   │       │   ├── mod.rs
│   │       │   ├── welcome.rs      # Welcome/landing page
│   │       │   ├── project_setup.rs# Project configuration
│   │       │   ├── entity_design.rs# Entity canvas design
│   │       │   ├── endpoints.rs    # Endpoint configuration
│   │       │   ├── relationships.rs# Relationship manager
│   │       │   ├── generation.rs   # Code generation page
│   │       │   └── settings.rs     # App settings
│   │       │
│   │       └── hooks/              # Custom Dioxus hooks
│   │           ├── mod.rs
│   │           ├── use_project.rs  # Project state hook
│   │           ├── use_canvas.rs   # Canvas interaction hook
│   │           ├── use_selection.rs# Selection management
│   │           ├── use_history.rs  # Undo/Redo hook
│   │           └── use_codegen.rs  # Code generation hook
│   │
│   └── cli/                        # imortal_cli - Command Line Interface
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── main.rs
│           └── commands/
│               ├── mod.rs
│               ├── new.rs          # Create new project
│               ├── generate.rs     # Generate code from file
│               ├── validate.rs     # Validate project file
│               └── info.rs         # Project info
│
├── assets/                         # Static assets
│   ├── styles/
│   │   ├── tailwind.css            # Tailwind source
│   │   └── main.css                # Additional styles
│   ├── icons/                      # SVG icons
│   └── fonts/                      # Custom fonts
│
├── templates/                      # Project templates
│   ├── rest-api/                   # REST API project template
│   └── fullstack/                  # Fullstack project template
│
├── docs/                           # Documentation
│   ├── COMPREHENSIVE_SPECIFICATION.md  # This document
│   ├── IMPLEMENTATION_PLAN.md
│   ├── architecture.md
│   └── user-guide.md
│
└── tests/                          # Integration tests
    ├── codegen_tests.rs
    ├── ir_tests.rs
    └── e2e_tests.rs
```

### 4.3 Module Dependency Graph

```
                         ┌──────────────────┐
                         │   imortal_core   │
                         │   ────────────   │
                         │  • DataType      │
                         │  • Position/Size │
                         │  • Errors        │
                         │  • Traits        │
                         └────────┬─────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
              ▼                   ▼                   ▼
       ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
       │  imortal_ir  │   │   imortal_   │   │   imortal_   │
       │  ──────────  │   │   codegen    │   │     cli      │
       │  Entity      │──▶│  ──────────  │   │  ──────────  │
       │  Relationship│   │  Generator   │◀──│  Commands    │
       │  Endpoint    │   │  Templates   │   │              │
       │  ProjectGraph│   │              │   │              │
       └──────┬───────┘   └──────────────┘   └──────────────┘
              │                   ▲                   ▲
              │                   │                   │
              └───────────────────┴───────────────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │    imortal_ui    │
                         │   ────────────   │
                         │  • App           │
                         │  • Pages         │
                         │  • Components    │
                         │  • State/Hooks   │
                         └──────────────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │     main.rs      │
                         │   (Entry Point)  │
                         └──────────────────┘
```

### 4.4 Crate Dependencies

#### `imortal_core/Cargo.toml`
```toml
[package]
name = "imortal_core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

#### `imortal_ir/Cargo.toml`
```toml
[package]
name = "imortal_ir"
version = "0.1.0"
edition = "2024"

[dependencies]
imortal_core = { path = "../core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
```

#### `imortal_codegen/Cargo.toml`
```toml
[package]
name = "imortal_codegen"
version = "0.1.0"
edition = "2024"

[dependencies]
imortal_core = { path = "../core" }
imortal_ir = { path = "../ir" }
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
proc-macro2 = "1.0"
heck = "0.5"  # Case conversion (snake_case, PascalCase, etc.)
thiserror = "1.0"
```

#### `imortal_ui/Cargo.toml`
```toml
[package]
name = "imortal_ui"
version = "0.1.0"
edition = "2024"

[dependencies]
imortal_core = { path = "../core" }
imortal_ir = { path = "../ir" }
imortal_codegen = { path = "../codegen" }
dioxus = { version = "0.6", features = ["desktop"] }
rfd = "0.15"  # File dialogs
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
```

---

## 5. Data Models & Core Types

### 5.1 Core Types (`imortal_core`)

#### 5.1.1 Data Types

```rust
/// Supported data types for entity fields
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "params")]
pub enum DataType {
    // Primitive Types
    String,
    Text,           // Long-form text (CLOB/TEXT)
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    Uuid,
    DateTime,
    Date,
    Time,
    Bytes,          // Binary data
    Json,           // JSON/JSONB
    
    // Complex Types
    Optional(Box<DataType>),
    Array(Box<DataType>),
    
    // Reference Types
    Reference {
        entity_name: String,
        field_name: String,  // Usually "id"
    },
    
    // Enum Type
    Enum {
        name: String,
        variants: Vec<String>,
    },
}

impl DataType {
    /// Convert to Rust type string
    pub fn to_rust_type(&self) -> String { ... }
    
    /// Convert to SQL type string for given database
    pub fn to_sql_type(&self, db: DatabaseType) -> String { ... }
    
    /// Convert to SeaORM column type
    pub fn to_sea_orm_type(&self) -> String { ... }
    
    /// Check if this type is nullable
    pub fn is_nullable(&self) -> bool { ... }
}
```

#### 5.1.2 Position and Geometry

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub position: Position,
    pub size: Size,
}

impl Rect {
    pub fn contains(&self, point: Position) -> bool { ... }
    pub fn intersects(&self, other: &Rect) -> bool { ... }
    pub fn center(&self) -> Position { ... }
}
```

#### 5.1.3 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    
    #[error("Duplicate entity name: {0}")]
    DuplicateName(String),
    
    #[error("Invalid relationship: {0}")]
    InvalidRelationship(String),
    
    #[error("Code generation failed: {0}")]
    CodegenError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type EngineResult<T> = Result<T, EngineError>;
```

### 5.2 Intermediate Representation (`imortal_ir`)

#### 5.2.1 Project Structure

```rust
/// Root container for an Immortal Engine project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGraph {
    /// Project metadata
    pub meta: ProjectMeta,
    
    /// Project configuration
    pub config: ProjectConfig,
    
    /// All entities in the project
    pub entities: HashMap<Uuid, Entity>,
    
    /// All relationships between entities
    pub relationships: HashMap<Uuid, Relationship>,
    
    /// API endpoint configurations
    pub endpoints: HashMap<Uuid, EndpointGroup>,
    
    /// UI state (for saving canvas position, etc.)
    pub canvas_state: CanvasState,
    
    /// Version for migration
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project type (REST API only or Fullstack)
    pub project_type: ProjectType,
    
    /// Target database
    pub database: DatabaseType,
    
    /// Authentication configuration
    pub auth: AuthConfig,
    
    /// Generated project name (for Cargo.toml)
    pub package_name: String,
    
    /// Rust edition for generated code
    pub rust_edition: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    RestApi,
    Fullstack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub strategy: AuthStrategy,
    pub jwt_secret_env_var: String,
    pub token_expiry_hours: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthStrategy {
    None,
    Jwt,
    Session,
}
```

#### 5.2.2 Entity Definition

```rust
/// Represents a data entity (maps to a database table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub table_name: String,         // Snake_case database table name
    pub description: Option<String>,
    pub fields: Vec<Field>,
    pub position: Position,         // Canvas position
    pub config: EntityConfig,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConfig {
    /// Auto-generate created_at/updated_at fields
    pub timestamps: bool,
    
    /// Use soft delete (deleted_at) instead of hard delete
    pub soft_delete: bool,
    
    /// Primary key type
    pub id_type: IdType,
    
    /// Enable audit logging
    pub auditable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdType {
    Uuid,
    Serial,      // Auto-incrementing integer
    Cuid,        // Collision-resistant unique ID
}

impl Entity {
    pub fn new(name: impl Into<String>) -> Self { ... }
    
    pub fn add_field(&mut self, field: Field) { ... }
    
    pub fn remove_field(&mut self, field_id: Uuid) { ... }
    
    pub fn get_primary_key(&self) -> Option<&Field> { ... }
    
    pub fn get_foreign_keys(&self) -> Vec<&Field> { ... }
    
    pub fn validate(&self) -> EngineResult<()> { ... }
}
```

#### 5.2.3 Field Definition

```rust
/// Represents a field within an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: Uuid,
    pub name: String,
    pub column_name: String,        // Snake_case database column name
    pub data_type: DataType,
    pub required: bool,
    pub unique: bool,
    pub indexed: bool,
    pub default_value: Option<DefaultValue>,
    pub validations: Vec<FieldValidation>,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub foreign_key_ref: Option<ForeignKeyRef>,
    pub description: Option<String>,
    pub ui_hints: UiHints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    pub entity_id: Uuid,
    pub entity_name: String,
    pub field_name: String,
    pub on_delete: ReferentialAction,
    pub on_update: ReferentialAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferentialAction {
    Cascade,
    SetNull,
    Restrict,
    NoAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefaultValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Now,            // CURRENT_TIMESTAMP
    Uuid,           // Generate UUID
    Expression(String),  // Custom SQL expression
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldValidation {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Min(f64),
    Max(f64),
    Pattern { regex: String, message: String },
    Email,
    Url,
    Uuid,
    Custom { name: String, expression: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiHints {
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub widget: Option<WidgetType>,
    pub display_order: i32,
    pub hidden: bool,
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Text,
    TextArea,
    Number,
    Email,
    Password,
    Select,
    Checkbox,
    Radio,
    Date,
    DateTime,
    File,
    RichText,
}
```

#### 5.2.4 Relationship Definition

```rust
/// Represents a relationship between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub name: String,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub relation_type: RelationType,
    
    /// Field on the "from" side (usually the FK field)
    pub from_field: String,
    
    /// Field on the "to" side (usually "id")
    pub to_field: String,
    
    /// Name for the inverse relation (e.g., "posts" for User -> Post)
    pub inverse_name: Option<String>,
    
    /// Visual connection points
    pub from_port: PortPosition,
    pub to_port: PortPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    /// One record relates to exactly one other record
    OneToOne,
    
    /// One record relates to many others (e.g., User has many Posts)
    OneToMany,
    
    /// Many records relate to one (inverse of OneToMany)
    ManyToOne,
    
    /// Many-to-many through junction table
    ManyToMany {
        junction_table: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PortPosition {
    Top,
    Right,
    Bottom,
    Left,
}

impl Relationship {
    pub fn one_to_one(from: Uuid, to: Uuid) -> Self { ... }
    pub fn one_to_many(from: Uuid, to: Uuid) -> Self { ... }
    pub fn many_to_many(from: Uuid, to: Uuid, junction: &str) -> Self { ... }
}
```

#### 5.2.5 Endpoint Configuration

```rust
/// Group of endpoints for an entity (CRUD operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointGroup {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub base_path: String,          // e.g., "/api/users"
    pub operations: Vec<CrudOperation>,
    pub global_security: EndpointSecurity,
    pub position: Position,         // Canvas position for visual
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrudOperation {
    pub operation_type: OperationType,
    pub enabled: bool,
    pub path_suffix: String,        // e.g., "" for list, "/:id" for single
    pub security: Option<EndpointSecurity>,  // Override global
    pub rate_limit: Option<RateLimit>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Create,     // POST /
    Read,       // GET /:id
    ReadAll,    // GET /
    Update,     // PUT /:id
    Delete,     // DELETE /:id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSecurity {
    pub auth_required: bool,
    pub roles: Vec<String>,         // Required roles (e.g., ["admin", "editor"])
    pub scopes: Vec<String>,        // OAuth scopes if applicable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests: u32,
    pub window_seconds: u32,
}

impl EndpointGroup {
    /// Create default CRUD endpoints for an entity
    pub fn default_crud(entity_id: Uuid, entity_name: &str) -> Self { ... }
    
    /// Enable/disable specific operations
    pub fn with_operations(mut self, ops: &[OperationType]) -> Self { ... }
    
    /// Set global security for all endpoints
    pub fn secured(mut self, roles: Vec<String>) -> Self { ... }
}
```

---

## 6. User Interface Design

### 6.1 Application Layout

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│  ☰  Immortal Engine v2.0                    [💾 Save] [▶ Generate] [⚙ Settings] │
├──────────────┬──────────────────────────────────────────────────┬───────────────┤
│              │                                                   │               │
│   SIDEBAR    │                 MAIN CANVAS                       │  PROPERTIES   │
│   ────────   │                 ───────────                       │  ──────────   │
│              │                                                   │               │
│  📁 Project  │    ┌───────────────┐        ┌───────────────┐    │  📊 Entity    │
│     my-app   │    │   📊 User     │───────▶│   📊 Post     │    │  ──────────   │
│              │    │   ──────────  │        │   ──────────  │    │               │
│  📊 Entities │    │  🔑 id: Uuid  │        │  🔑 id: Uuid  │    │  Name: User   │
│   ├─ User    │    │  📧 email     │        │  📝 title     │    │               │
│   ├─ Post    │    │  👤 name      │        │  📝 content   │    │  Fields:      │
│   └─ Comment │    │  🔒 password  │        │  🔗 user_id   │    │  ┌──────────┐ │
│              │    │  📅 created   │        │  📅 created   │    │  │ + Add    │ │
│  🔌 Endpoints│    └───────────────┘        └───────────────┘    │  └──────────┘ │
│   ├─ /users  │              │                      │            │               │
│   └─ /posts  │              │      ┌───────────────┘            │  • id         │
│              │              │      │                            │    Uuid, PK   │
│  🔐 Auth     │              ▼      ▼                            │               │
│   JWT Config │    ┌───────────────────────┐                     │  • email      │
│              │    │   📊 Comment          │                     │    String     │
│  ⚙️ Settings │    │   ──────────────────  │                     │    Required   │
│              │    │  🔑 id: Uuid          │                     │    Unique     │
│              │    │  📝 content           │                     │               │
│              │    │  🔗 user_id           │                     │  • name       │
│              │    │  🔗 post_id           │                     │    String     │
│              │    └───────────────────────┘                     │               │
│              │                                                   │               │
├──────────────┴──────────────────────────────────────────────────┴───────────────┤
│  Status: Ready  │  Entities: 3  │  Endpoints: 6  │  ✅ All validations pass      │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Page Descriptions

| Page | Purpose | Key Components |
|------|---------|----------------|
| **Welcome** | Landing page, create/open projects | Hero section, recent projects list |
| **Project Setup** | Configure project settings | Form for name, database, project type, auth |
| **Entity Design** | Visual canvas for designing entities | Drag-drop entity cards, connection drawing |
| **Endpoints** | Configure API endpoints per entity | CRUD toggles, security settings |
| **Relationships** | Manage entity relationships | Visual connections, relationship type selector |
| **Generation** | Preview and generate code | Directory picker, generation options, progress |
| **Settings** | Application preferences | Theme, shortcuts, defaults |

### 6.3 Tailwind CSS Theme

```css
/* tailwind.config.js color scheme */
:root {
  /* Primary - Indigo */
  --color-primary-50: #eef2ff;
  --color-primary-500: #6366f1;
  --color-primary-600: #4f46e5;
  --color-primary-700: #4338ca;
  
  /* Success - Emerald */
  --color-success-500: #10b981;
  
  /* Warning - Amber */
  --color-warning-500: #f59e0b;
  
  /* Danger - Rose */
  --color-danger-500: #f43f5e;
  
  /* Neutral - Slate */
  --color-neutral-50: #f8fafc;
  --color-neutral-100: #f1f5f9;
  --color-neutral-800: #1e293b;
  --color-neutral-900: #0f172a;
}

/* Entity Card Component */
.entity-card {
  @apply bg-white dark:bg-slate-800 
         rounded-xl shadow-lg 
         border-2 border-slate-200 dark:border-slate-700
         min-w-[220px] 
         hover:shadow-xl hover:border-primary-300
         transition-all duration-200;
}

.entity-card.selected {
  @apply border-primary-500 ring-2 ring-primary-200;
}

.entity-card-header {
  @apply flex items-center gap-2 px-4 py-3 
         border-b border-slate-100 dark:border-slate-700
         bg-slate-50 dark:bg-slate-900 
         rounded-t-xl;
}

/* Field Row */
.field-row {
  @apply flex items-center justify-between 
         px-4 py-2 
         hover:bg-slate-50 dark:hover:bg-slate-700
         cursor-pointer;
}

.field-type-badge {
  @apply text-xs px-2 py-0.5 rounded-full
         bg-blue-100 text-blue-700
         dark:bg-blue-900 dark:text-blue-300;
}

/* Connection Line */
.connection-line {
  stroke: theme('colors.slate.400');
  stroke-width: 2;
  fill: none;
}

.connection-line.selected {
  stroke: theme('colors.primary.500');
  stroke-width: 3;
}

/* HTTP Method Badges */
.method-get { @apply bg-green-100 text-green-700; }
.method-post { @apply bg-blue-100 text-blue-700; }
.method-put { @apply bg-amber-100 text-amber-700; }
.method-delete { @apply bg-rose-100 text-rose-700; }
```

### 6.4 Key UI Components (Dioxus RSX)

#### 6.4.1 Entity Card Component

```rust
#[component]
fn EntityCard(
    entity: Entity,
    selected: bool,
    on_select: EventHandler<Uuid>,
    on_drag: EventHandler<(Uuid, Position)>,
) -> Element {
    let class = if selected {
        "entity-card selected"
    } else {
        "entity-card"
    };
    
    rsx! {
        div {
            class: "{class}",
            style: "position: absolute; left: {entity.position.x}px; top: {entity.position.y}px;",
            onclick: move |_| on_select.call(entity.id),
            
            // Header
            div { class: "entity-card-header",
                span { class: "text-2xl", "📊" }
                span { class: "font-bold text-lg", "{entity.name}" }
            }
            
            // Fields
            div { class: "py-2",
                for field in entity.fields.iter() {
                    FieldRow { field: field.clone() }
                }
            }
            
            // Connection Ports
            Port { direction: PortDirection::Input, position: PortPosition::Left }
            Port { direction: PortDirection::Output, position: PortPosition::Right }
        }
    }
}
```

#### 6.4.2 Canvas Component

```rust
#[component]
fn Canvas() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut canvas_state = use_signal(|| CanvasState::default());
    
    let on_mouse_move = move |e: MouseEvent| {
        if canvas_state.read().dragging {
            // Handle panning or entity dragging
        }
    };
    
    let on_wheel = move |e: WheelEvent| {
        // Handle zoom
        let delta = e.delta_y();
        canvas_state.write().zoom *= if delta > 0.0 { 0.9 } else { 1.1 };
    };
    
    rsx! {
        div {
            class: "relative w-full h-full overflow-hidden bg-slate-100 dark:bg-slate-900",
            onmousemove: on_mouse_move,
            onwheel: on_wheel,
            
            // Grid background
            svg { class: "absolute inset-0 w-full h-full",
                defs {
                    pattern {
                        id: "grid",
                        width: "20",
                        height: "20",
                        pattern_units: "userSpaceOnUse",
                        path {
                            d: "M 20 0 L 0 0 0 20",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_opacity: "0.1",
                        }
                    }
                }
                rect { width: "100%", height: "100%", fill: "url(#grid)" }
            }
            
            // Transform container for zoom/pan
            div {
                class: "absolute",
                style: "transform: translate({canvas_state.read().pan.x}px, {canvas_state.read().pan.y}px) scale({canvas_state.read().zoom});",
                
                // Render relationships first (behind entities)
                for rel in state.read().project.relationships.values() {
                    ConnectionLine { relationship: rel.clone() }
                }
                
                // Render entities
                for entity in state.read().project.entities.values() {
                    EntityCard {
                        entity: entity.clone(),
                        selected: state.read().selection.contains(&entity.id),
                        on_select: move |id| { /* handle selection */ },
                        on_drag: move |(id, pos)| { /* handle drag */ },
                    }
                }
            }
        }
    }
}
```

---

## 7. Core Features Specification

### 7.1 Entity Management

#### 7.1.1 Create Entity
- **Trigger**: Click "Add Entity" button or double-click canvas
- **Default values**: 
  - Name: "NewEntity"
  - Fields: `id` (UUID, primary key), `created_at`, `updated_at` (if timestamps enabled)
- **Validation**: Name must be unique, PascalCase

#### 7.1.2 Edit Entity
- **Rename**: Click entity name → inline edit
- **Move**: Drag entity card on canvas
- **Delete**: Select + Delete key, or context menu

#### 7.1.3 Field Management
- **Add Field**: Click "+" button in entity card or properties panel
- **Edit Field**: Click field row → properties panel updates
- **Reorder Fields**: Drag-and-drop in properties panel
- **Delete Field**: Hover → delete icon, or properties panel

### 7.2 Relationship Management

#### 7.2.1 Create Relationship
1. Click output port on source entity
2. Drag to input port on target entity
3. Relationship dialog appears:
   - Select relationship type (1:1, 1:N, M:N)
   - Configure field names
   - Set referential actions

#### 7.2.2 Relationship Visualization

```
User (1) ────────────────► (N) Post
         "user_id" FK

User (1) ◄───────────────► (1) Profile  
         "profile_id" FK

Tag (N) ◄────────────────► (N) Post
         "post_tags" junction table
```

### 7.3 Endpoint Configuration

#### 7.3.1 Auto-Generated Endpoints
When an entity is created, default endpoints are suggested:

| Operation | Method | Path | Default Security |
|-----------|--------|------|------------------|
| Create | POST | `/api/{entity}` | Auth required |
| Read | GET | `/api/{entity}/:id` | Open |
| ReadAll | GET | `/api/{entity}` | Open |
| Update | PUT | `/api/{entity}/:id` | Auth required |
| Delete | DELETE | `/api/{entity}/:id` | Auth + Role |

#### 7.3.2 Security Configuration
Per-endpoint or global:
- **Open**: No authentication required
- **Auth Required**: Valid JWT token needed
- **Role-Based**: Specific roles required (e.g., "admin", "editor")

#### 7.3.3 Integrated Endpoints
For relationships, auto-generate nested endpoints:
- `GET /api/users/:id/posts` - Get user's posts
- `GET /api/posts/:id/comments` - Get post's comments

### 7.4 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New Project |
| `Ctrl+O` | Open Project |
| `Ctrl+S` | Save Project |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+E` | Add Entity |
| `Delete` | Delete Selected |
| `Escape` | Deselect All |
| `Ctrl+A` | Select All |
| `Ctrl+G` | Generate Code |
| `Space` (hold) | Pan Canvas |
| `Scroll` | Zoom |

---

## 8. Code Generation Engine

### 8.1 Generation Pipeline

```
┌────────────────┐     ┌────────────────┐     ┌────────────────┐     ┌────────────────┐
│  ProjectGraph  │────▶│    Analyzer    │────▶│   Generator    │────▶│    Writer      │
│                │     │                │     │                │     │                │
│  • Entities    │     │  • Validate    │     │  • Models      │     │  • File I/O    │
│  • Relations   │     │  • Extract     │     │  • Handlers    │     │  • Format      │
│  • Endpoints   │     │  • Dependencies│     │  • Routes      │     │  • Structure   │
└────────────────┘     └────────────────┘     └────────────────┘     └────────────────┘
```

### 8.2 Generated File Structure

#### REST API Project
```
{project_name}/
├── Cargo.toml
├── .env.example
├── .gitignore
├── README.md
│
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── state.rs
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── post.rs
│   │   └── ...
│   │
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── post.rs
│   │   └── ...
│   │
│   ├── routes/
│   │   ├── mod.rs
│   │   └── api.rs
│   │
│   └── auth/                    # If auth enabled
│       ├── mod.rs
│       ├── jwt.rs
│       └── middleware.rs
│
├── migrations/
│   ├── 20260129000001_create_users.sql
│   └── 20260129000002_create_posts.sql
│
└── tests/
    └── api_tests.rs
```

#### Fullstack Project
```
{project_name}/
├── Cargo.toml                   # Workspace
│
├── backend/
│   └── (same as REST API)
│
├── frontend/
│   ├── Cargo.toml
│   ├── Dioxus.toml
│   ├── assets/
│   │   └── tailwind.css
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── router.rs
│       ├── components/
│       │   ├── mod.rs
│       │   ├── navbar.rs
│       │   ├── sidebar.rs
│       │   ├── table.rs
│       │   └── form.rs
│       ├── pages/
│       │   ├── mod.rs
│       │   ├── home.rs
│       │   ├── users/
│       │   │   ├── mod.rs
│       │   │   ├── list.rs
│       │   │   └── form.rs
│       │   └── ...
│       └── api/
│           ├── mod.rs
│           └── client.rs
│
└── shared/
    ├── Cargo.toml
    └── src/
        └── lib.rs               # Shared DTOs
```

### 8.3 Code Generation Examples

#### 8.3.1 Model Generation (SeaORM)

```rust
// Generated: src/models/user.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub email: String,

    pub name: String,

    #[serde(skip_serializing)]
    pub password_hash: String,

    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post::Entity")]
    Posts,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Posts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// DTOs
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTimeUtc,
}

impl From<Model> for UserResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            email: model.email,
            name: model.name,
            created_at: model.created_at,
        }
    }
}
```

#### 8.3.2 Handler Generation (Axum)

```rust
// Generated: src/handlers/user.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::user::{self, CreateUserDto, UpdateUserDto, UserResponse},
    state::AppState,
};

/// List all users with pagination
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<UserResponse>>, AppError> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    
    let paginator = user::Entity::find()
        .paginate(&state.db, per_page);
    
    let total = paginator.num_items().await?;
    let items: Vec<UserResponse> = paginator
        .fetch_page(page - 1)
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect();
    
    Ok(Json(PaginatedResponse {
        items,
        total,
        page,
        per_page,
    }))
}

/// Get a single user by ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let user = user::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    
    Ok(Json(UserResponse::from(user)))
}

/// Create a new user
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    payload.validate()?;
    
    let password_hash = hash_password(&payload.password)?;
    
    let user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(payload.email),
        name: Set(payload.name),
        password_hash: Set(password_hash),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(&state.db)
    .await?;
    
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}
```

#### 8.3.3 Migration Generation

```sql
-- Generated: migrations/20260129000001_create_users.sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);

-- Generated: migrations/20260129000002_create_posts.sql
CREATE TABLE IF NOT EXISTS posts (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_posts_user_id ON posts(user_id);
```

---

## 9. Security Implementation

### 9.1 Authentication Strategies

| Strategy | Use Case | Token Storage |
|----------|----------|---------------|
| **JWT** | Stateless APIs, microservices | Client (localStorage/cookie) |
| **Session** | Traditional web apps | Server-side (Redis/DB) |
| **None** | Public APIs, prototyping | N/A |

### 9.2 JWT Implementation (Generated Code)

```rust
// Generated: src/auth/jwt.rs
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User ID
    pub email: String,
    pub roles: Vec<String>,
    pub exp: u64,           // Expiration time
    pub iat: u64,           // Issued at
}

impl Claims {
    pub fn new(user_id: &str, email: &str, roles: Vec<String>, expiry_hours: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            sub: user_id.to_string(),
            email: email.to_string(),
            roles,
            iat: now,
            exp: now + (expiry_hours * 3600),
        }
    }
}

pub fn create_token(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
```

### 9.3 Auth Middleware (Generated Code)

```rust
// Generated: src/auth/middleware.rs
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{auth::jwt::verify_token, state::AppState};

pub async fn require_auth(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = auth.token();
    
    let claims = verify_token(token, &state.config.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Insert claims into request extensions for handlers to use
    request.extensions_mut().insert(claims);
    
    Ok(next.run(request).await)
}

pub fn require_roles(required_roles: &[&str]) -> impl Fn(Claims) -> bool + '_ {
    move |claims: Claims| {
        required_roles.iter().any(|role| claims.roles.contains(&role.to_string()))
    }
}
```

### 9.4 Per-Endpoint Security Configuration

Users can configure security at three levels:

1. **Global** (Project level): Default for all endpoints
2. **Entity** (EndpointGroup level): Override for all entity endpoints
3. **Operation** (Individual endpoint): Fine-grained control

```rust
// Example: Different security per operation
EndpointGroup {
    entity_id: user_id,
    base_path: "/api/users",
    global_security: EndpointSecurity { auth_required: true, roles: vec![] },
    operations: vec![
        CrudOperation {
            operation_type: OperationType::ReadAll,
            security: Some(EndpointSecurity { auth_required: false, .. }),  // Public
            ..
        },
        CrudOperation {
            operation_type: OperationType::Delete,
            security: Some(EndpointSecurity { 
                auth_required: true, 
                roles: vec!["admin".into()] 
            }),  // Admin only
            ..
        },
    ],
}
```

---

## 10. Database Support

### 10.1 Supported Databases

| Database | Connection String Example | Features |
|----------|--------------------------|----------|
| **PostgreSQL** | `postgres://user:pass@localhost/db` | Full feature support, recommended |
| **MySQL** | `mysql://user:pass@localhost/db` | Full feature support |
| **SQLite** | `sqlite://./data.db` | Development/embedded use |

### 10.2 Type Mapping

| DataType | PostgreSQL | MySQL | SQLite |
|----------|------------|-------|--------|
| `String` | VARCHAR(255) | VARCHAR(255) | TEXT |
| `Text` | TEXT | LONGTEXT | TEXT |
| `Int32` | INTEGER | INT | INTEGER |
| `Int64` | BIGINT | BIGINT | INTEGER |
| `Float32` | REAL | FLOAT | REAL |
| `Float64` | DOUBLE PRECISION | DOUBLE | REAL |
| `Bool` | BOOLEAN | TINYINT(1) | INTEGER |
| `Uuid` | UUID | CHAR(36) | TEXT |
| `DateTime` | TIMESTAMP WITH TIME ZONE | DATETIME | TEXT |
| `Date` | DATE | DATE | TEXT |
| `Json` | JSONB | JSON | TEXT |
| `Bytes` | BYTEA | BLOB | BLOB |

### 10.3 Migration Generation Strategy

1. **Ordering**: Migrations are ordered by dependency (referenced tables first)
2. **Naming**: `{timestamp}_{operation}_{table}.sql`
3. **Idempotency**: All migrations use `IF NOT EXISTS` / `IF EXISTS`
4. **Rollback**: Down migrations generated alongside up migrations

---

## 11. Export System

### 11.1 Export Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Validate  │────▶│   Select    │────▶│   Generate  │────▶│   Write     │
│   Project   │     │  Directory  │     │    Code     │     │   Files     │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                          │
                          ▼
                    Native File
                    Dialog (rfd)
```

### 11.2 File Dialog Integration

```rust
use rfd::AsyncFileDialog;

pub async fn select_export_directory() -> Option<PathBuf> {
    AsyncFileDialog::new()
        .set_title("Select Export Location")
        .pick_folder()
        .await
        .map(|handle| handle.path().to_path_buf())
}
```

### 11.3 Post-Generation Actions

After successful generation, offer:
1. **Open in File Manager**: Open the generated directory
2. **Open in VS Code**: Launch VS Code with the project
3. **Run Project**: Execute `cargo run` in terminal
4. **Copy Commands**: Copy setup commands to clipboard

---

## 12. API Reference

### 12.1 Core Traits

```rust
/// Trait for types that can be validated
pub trait Validatable {
    fn validate(&self) -> EngineResult<()>;
}

/// Trait for types that can generate code
pub trait CodeGenerable {
    fn generate(&self, ctx: &mut GenerationContext) -> EngineResult<String>;
}

/// Trait for types that can be serialized to project files
pub trait Persistable: Serialize + DeserializeOwned {
    fn file_extension() -> &'static str;
}
```

### 12.2 State Management Hooks

```rust
/// Hook for accessing project state
pub fn use_project() -> UseProject {
    let state = use_context::<Signal<AppState>>();
    UseProject { state }
}

impl UseProject {
    pub fn entities(&self) -> Vec<Entity> { ... }
    pub fn add_entity(&self, entity: Entity) { ... }
    pub fn remove_entity(&self, id: Uuid) { ... }
    pub fn update_entity(&self, id: Uuid, f: impl FnOnce(&mut Entity)) { ... }
}

/// Hook for undo/redo functionality
pub fn use_history() -> UseHistory {
    let history = use_context::<Signal<History>>();
    UseHistory { history }
}

impl UseHistory {
    pub fn undo(&self) { ... }
    pub fn redo(&self) { ... }
    pub fn can_undo(&self) -> bool { ... }
    pub fn can_redo(&self) -> bool { ... }
    pub fn push(&self, snapshot: ProjectSnapshot) { ... }
}
```

---

## 13. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Set up workspace structure with all crates
- [ ] Implement `imortal_core` types
- [ ] Set up Dioxus desktop application shell
- [ ] Configure Tailwind CSS
- [ ] Implement welcome page and project setup page

### Phase 2: Entity Design (Weeks 3-4)
- [ ] Implement canvas component with pan/zoom
- [ ] Create entity card component
- [ ] Implement drag-and-drop for entities
- [ ] Build properties panel
- [ ] Implement field CRUD operations

### Phase 3: Relationships (Weeks 5-6)
- [ ] Implement port components
- [ ] Create connection drawing logic
- [ ] Implement relationship visualization (SVG lines)
- [ ] Build relationship configuration dialog

### Phase 4: Endpoints (Weeks 7-8)
- [ ] Create endpoint configuration page
- [ ] Implement CRUD operation toggles
- [ ] Add security configuration UI
- [ ] Implement integrated endpoints for relationships

### Phase 5: Code Generation (Weeks 9-11)
- [ ] Implement SeaORM model generation
- [ ] Implement Axum handler generation
- [ ] Implement router generation
- [ ] Implement migration generation
- [ ] Add auth code generation

### Phase 6: Fullstack Generation (Weeks 12-13)
- [ ] Implement Dioxus frontend generation
- [ ] Generate API client code
- [ ] Generate CRUD pages

### Phase 7: Polish (Weeks 14-16)
- [ ] Implement undo/redo
- [ ] Add keyboard shortcuts
- [ ] Performance optimization
- [ ] Testing and bug fixes
- [ ] Documentation

---

## 14. Testing Strategy

### 14.1 Unit Tests
- All core types and IR structures
- Code generation for individual components
- Validation rules

### 14.2 Integration Tests
- Full project generation pipeline
- File I/O operations
- State management

### 14.3 E2E Tests (Generated Code)
- Generated projects must compile
- Generated migrations must run
- Generated API endpoints must respond correctly

### 14.4 Test Commands
```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p imortal_codegen

# Run with coverage
cargo tarpaulin --workspace
```

---

## 15. Deployment & Distribution

### 15.1 Build Targets
- **Linux**: AppImage, .deb, .rpm
- **macOS**: .dmg, .app bundle
- **Windows**: .msi, portable .exe

### 15.2 Build Commands
```bash
# Development
cargo run

# Release build
cargo build --release

# Create distributable (using cargo-bundle or similar)
cargo bundle --release
```

---

## 16. Appendices

### Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Entity** | A data model that maps to a database table |
| **Field** | A property of an entity (database column) |
| **Relationship** | A connection between two entities |
| **Endpoint** | An API route for CRUD operations |
| **IR** | Intermediate Representation - the in-memory project model |
| **Canvas** | The visual editor area where entities are arranged |

### Appendix B: Configuration Files

#### `.ieng` Project File Format
```json
{
  "schema_version": 1,
  "meta": {
    "id": "uuid",
    "name": "My Project",
    "created_at": "2026-01-29T00:00:00Z"
  },
  "config": {
    "project_type": "RestApi",
    "database": "PostgreSQL",
    "auth": { "enabled": true, "strategy": "Jwt" }
  },
  "entities": { ... },
  "relationships": { ... },
  "endpoints": { ... }
}
```

### Appendix C: Environment Variables (Generated Project)

```env
# Database
DATABASE_URL=postgres://user:password@localhost:5432/myapp

# Server
HOST=0.0.0.0
PORT=8080

# Authentication (if enabled)
JWT_SECRET=your-super-secret-key-change-in-production
JWT_EXPIRY_HOURS=24

# Logging
RUST_LOG=info
```

---

## Conclusion

This comprehensive specification provides the complete blueprint for implementing Immortal Engine v2.0 (S-Arch-P). The key technical decisions are:

1. **Dioxus Desktop** for a native, reactive UI
2. **Tailwind CSS** for rapid, consistent styling
3. **Axum** for generated backend code (modern, type-safe, Tower-based)
4. **SeaORM** for database abstraction with migrations
5. **JWT** as the default authentication strategy

The modular workspace architecture allows independent development and testing of each component while maintaining clear boundaries and dependencies.

---

*Document maintained by Stephen Kinuthia - kinuthiasteve098@gmail.com*