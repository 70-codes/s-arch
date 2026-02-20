# Immortal Engine v2.0 - Implementation Plan

## 🎯 Executive Summary

This document outlines the comprehensive implementation plan for **Immortal Engine v2.0**, a visual code generation platform built with **Dioxus** for the frontend and **Axum** as the recommended backend framework for generated projects. The application enables users to visually design, configure, and generate production-ready Rust backend and fullstack applications.

---

## 📋 Table of Contents

1. [Project Overview](#1-project-overview)
2. [Technology Stack](#2-technology-stack)
3. [Architecture Design](#3-architecture-design)
4. [UI/UX Design](#4-uiux-design)
5. [Core Features Implementation](#5-core-features-implementation)
6. [Code Generation Engine](#6-code-generation-engine)
7. [Data Flow & State Management](#7-data-flow--state-management)
8. [Security Implementation](#8-security-implementation)
9. [Database Support](#9-database-support)
10. [Export System](#10-export-system)
11. [Implementation Phases](#11-implementation-phases)
12. [Testing Strategy](#12-testing-strategy)
13. [Future Enhancements](#13-future-enhancements)

---

## 1. Project Overview

### 1.1 Vision

Immortal Engine v2.0 transforms the visual prototyping experience by migrating from egui to **Dioxus Desktop**, providing a modern, reactive UI with Tailwind CSS styling. Users can:

- Design data entities with fields and relationships
- Configure REST API endpoints (secured or open)
- Generate complete Rust backend projects
- Optionally generate fullstack applications with Dioxus frontend
- Export generated projects to any location on their filesystem

### 1.2 Core User Journey

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            USER JOURNEY FLOW                                │
└─────────────────────────────────────────────────────────────────────────────┘

  ┌──────────────┐     ┌───────────────┐     ┌───────────────┐     ┌──────────┐
  │   Project    │────▶│    Define     │────▶│   Configure   │────▶│ Generate │
  │   Setup      │     │   Entities    │     │   Endpoints   │     │   Code   │
  └──────────────┘     └───────────────┘     └───────────────┘     └──────────┘
        │                     │                     │                    │
        ▼                     ▼                     ▼                    ▼
   • Project name        • Field types         • CRUD operations    • REST API
   • Database choice     • Relationships       • Security config    • Fullstack
   • REST/Fullstack      • Constraints         • Rate limiting      • Export path
   • Auth enabled        • Validations         • Custom routes
```

### 1.3 Project Configuration Options

| Option | Choices | Description |
|--------|---------|-------------|
| **Project Type** | REST API, Fullstack | Determines if frontend code is generated |
| **Database** | PostgreSQL, MySQL, SQLite | Target database for migrations |
| **Authentication** | None, JWT, Session | Security mechanism for endpoints |
| **Endpoint Security** | Open, Secured, Mixed | Per-endpoint auth requirements |

---

## 2. Technology Stack

### 2.1 Application Stack (Immortal Engine)

| Layer | Technology | Justification |
|-------|------------|---------------|
| **Frontend Framework** | Dioxus Desktop | Native Rust, reactive, cross-platform |
| **Styling** | Tailwind CSS | Utility-first, rapid development |
| **State Management** | Dioxus Signals | Built-in reactive state |
| **File System** | Native (rfd crate) | File dialogs for export location |
| **Serialization** | Serde + JSON/TOML | Project file persistence |

### 2.2 Generated Project Stack

| Component | Technology | Justification |
|-----------|------------|---------------|
| **Backend Framework** | **Axum** | Modern, tower-based, excellent ecosystem |
| **ORM** | SeaORM | Async, type-safe, great migration support |
| **Authentication** | axum-extra + JWT | Industry standard, stateless |
| **Validation** | validator crate | Derive-based, comprehensive |
| **Frontend (Fullstack)** | Dioxus Web | Same language, shared types |
| **API Documentation** | utoipa (OpenAPI) | Auto-generated Swagger docs |

### 2.3 Why Axum for Generated Projects?

**Comparison Matrix:**

| Feature | Axum | Actix-Web | Rocket |
|---------|------|-----------|--------|
| Async Runtime | Tokio (native) | Actix (custom) | Tokio |
| Type Safety | Excellent | Good | Excellent |
| Middleware | Tower ecosystem | Custom | Custom |
| Learning Curve | Moderate | Steep | Easy |
| Performance | Excellent | Excellent | Good |
| Ecosystem | Growing rapidly | Mature | Moderate |
| Active Development | Very active | Active | Moderate |

**Recommendation: Axum** - Modern design, leverages Tower middleware ecosystem, excellent type extraction, and seamless Tokio integration.

---

## 3. Architecture Design

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         IMMORTAL ENGINE v2.0                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    DIOXUS DESKTOP APPLICATION                        │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────┐  │   │
│  │  │  Sidebar     │ │   Canvas     │ │  Properties  │ │  Toolbar   │  │   │
│  │  │  Navigation  │ │   Editor     │ │    Panel     │ │  Actions   │  │   │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         CORE ENGINE                                  │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────────┐  │   │
│  │  │  imortal_ir │ │ imortal_    │ │  imortal_   │ │   imortal_    │  │   │
│  │  │  (Graph)    │ │ components  │ │  codegen    │ │   core        │  │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └───────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     FILE SYSTEM / EXPORT                             │   │
│  │  ┌────────────────────────────────────────────────────────────────┐ │   │
│  │  │  User-specified directory with complete Rust project           │ │   │
│  │  └────────────────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Workspace Structure (Updated)

```
imortal_engine/
├── Cargo.toml                    # Workspace root
├── src/
│   └── main.rs                   # Dioxus Desktop entry point
│
├── crates/
│   ├── core/                     # Shared types and traits
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs          # DataType, Position, Size, etc.
│   │       ├── traits.rs         # Validatable, Serializable
│   │       └── error.rs          # EngineError, EngineResult
│   │
│   ├── ir/                       # Intermediate Representation
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── graph.rs          # ProjectGraph
│   │       ├── node.rs           # Node struct
│   │       ├── edge.rs           # Edge/Connection struct
│   │       ├── field.rs          # Field definitions
│   │       ├── port.rs           # Input/Output ports
│   │       ├── project.rs        # ProjectMeta, settings
│   │       ├── validation.rs     # Graph validation
│   │       └── serialization.rs  # Save/Load
│   │
│   ├── components/               # Component definitions
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── registry.rs       # ComponentRegistry
│   │       ├── definition.rs     # ComponentDefinition
│   │       └── definitions/
│   │           ├── data.rs       # Entity, Collection, Query
│   │           ├── api.rs        # REST, GraphQL, WebSocket
│   │           ├── auth.rs       # Login, Register, JWT
│   │           ├── storage.rs    # Database, Cache
│   │           └── logic.rs      # Validator, Transformer
│   │
│   ├── codegen/                  # Code Generation
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── generator.rs      # Main CodeGenerator
│   │       ├── rust/
│   │       │   ├── mod.rs
│   │       │   ├── models.rs     # Entity → Struct generation
│   │       │   ├── handlers.rs   # Endpoint → Handler generation
│   │       │   ├── auth.rs       # Auth middleware generation
│   │       │   ├── config.rs     # Config files generation
│   │       │   ├── migrations.rs # Database migrations
│   │       │   └── frontend.rs   # NEW: Dioxus frontend generation
│   │       └── templates/
│   │           ├── mod.rs
│   │           ├── axum.rs       # Axum-specific templates
│   │           └── dioxus.rs     # NEW: Dioxus UI templates
│   │
│   ├── ui/                       # NEW: Dioxus UI
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs            # Main App component
│   │       ├── state.rs          # Application state (Signals)
│   │       ├── components/
│   │       │   ├── mod.rs
│   │       │   ├── sidebar.rs    # Navigation sidebar
│   │       │   ├── canvas.rs     # Visual editor canvas
│   │       │   ├── properties.rs # Properties panel
│   │       │   ├── toolbar.rs    # Top toolbar
│   │       │   ├── dialogs.rs    # Modal dialogs
│   │       │   └── entity_card.rs # Entity node component
│   │       ├── pages/
│   │       │   ├── mod.rs
│   │       │   ├── welcome.rs    # Welcome/start page
│   │       │   ├── project.rs    # Project setup page
│   │       │   ├── entities.rs   # Entity design page
│   │       │   ├── endpoints.rs  # Endpoint configuration
│   │       │   ├── relations.rs  # Relationship manager
│   │       │   └── generate.rs   # Code generation page
│   │       └── hooks/
│   │           ├── mod.rs
│   │           ├── use_project.rs    # Project state hook
│   │           ├── use_canvas.rs     # Canvas interaction hook
│   │           └── use_codegen.rs    # Code generation hook
│   │
│   └── cli/                      # Command-line interface
│       └── src/
│           ├── lib.rs
│           └── main.rs
│
├── assets/                       # Static assets
│   ├── tailwind.css              # Tailwind CSS (compiled)
│   ├── icons/                    # UI icons
│   └── templates/                # Project templates
│
├── docs/                         # Documentation
│   ├── IMPLEMENTATION_PLAN.md    # This document
│   ├── architecture.md
│   └── ...
│
└── tests/                        # Integration tests
    └── ...
```

### 3.3 Module Dependency Graph

```
                    ┌─────────────────┐
                    │   imortal_core  │
                    │  (types, traits)│
                    └────────┬────────┘
                             │
           ┌─────────────────┼─────────────────┐
           │                 │                 │
           ▼                 ▼                 ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │  imortal_ir  │  │   imortal_   │  │   imortal_   │
    │   (graph)    │  │  components  │  │    codegen   │
    └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
           │                 │                 │
           └─────────────────┼─────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │   imortal_ui    │
                    │    (Dioxus)     │
                    └─────────────────┘
```

---

## 4. UI/UX Design

### 4.1 Application Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  ☰  Immortal Engine                              [Generate] [Settings] [?]  │
├────────────┬────────────────────────────────────────────────┬───────────────┤
│            │                                                │               │
│  SIDEBAR   │                  MAIN CANVAS                   │  PROPERTIES   │
│  ───────   │                  ───────────                   │  ──────────   │
│            │                                                │               │
│  📁 Project│   ┌────────────────┐    ┌────────────────┐    │  Entity: User │
│    Settings│   │   📊 User      │────│  🔌 /api/users │    │  ───────────  │
│            │   │   ──────────   │    │   ───────────  │    │               │
│  📊 Entities   │   id: UUID 🔑  │    │  GET  ✓ 🔒     │    │  Fields:      │
│    ├ User  │   │   email: Str   │    │  POST ✓ 🔒     │    │  + Add Field  │
│    ├ Post  │   │   name: Str    │    │  PUT  ✓ 🔒     │    │               │
│    └ Comment   │   created: DT   │    │  DEL  ✓ 🔒     │    │  • id         │
│            │   └────────────────┘    └────────────────┘    │    UUID       │
│  🔌 Endpoints  │         │                                  │    Primary    │
│    ├ /users│         ▼                                  │               │
│    └ /posts│   ┌────────────────┐                         │  • email      │
│            │   │   📊 Post      │                         │    String     │
│  🔐 Auth   │   │   ──────────   │                         │    Required   │
│    Config  │   │   id: UUID 🔑  │                         │    Unique     │
│            │   │   title: Str   │                         │               │
│  ⚙️ Settings   │   user_id: Ref │◀───────── User has many │  Relations:   │
│            │   └────────────────┘              Posts       │  → Post (1:N) │
│            │                                                │               │
├────────────┴────────────────────────────────────────────────┴───────────────┤
│  Status: Ready │ Entities: 3 │ Endpoints: 6 │ Validation: ✓ All checks pass │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Page Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   WELCOME    │────▶│   PROJECT    │────▶│   ENTITIES   │────▶│  ENDPOINTS   │
│    PAGE      │     │    SETUP     │     │    DESIGN    │     │   CONFIG     │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                                                                      │
┌──────────────┐     ┌──────────────┐     ┌──────────────┐           │
│   EXPORT     │◀────│   PREVIEW    │◀────│  RELATIONS   │◀──────────┘
│   SELECT     │     │   GENERATE   │     │    SETUP     │
└──────────────┘     └──────────────┘     └──────────────┘
```

### 4.3 Tailwind CSS Component Styles

```css
/* Entity Card */
.entity-card {
  @apply bg-white dark:bg-gray-800 rounded-xl shadow-lg border-2 border-gray-200 
         dark:border-gray-700 p-4 min-w-[220px] hover:shadow-xl transition-shadow;
}

.entity-card-header {
  @apply flex items-center gap-2 pb-2 border-b border-gray-200 dark:border-gray-600;
}

.entity-card-icon {
  @apply text-2xl;
}

.entity-card-title {
  @apply font-bold text-lg text-gray-800 dark:text-white;
}

/* Field Row */
.field-row {
  @apply flex items-center gap-2 py-1 text-sm;
}

.field-name {
  @apply font-medium text-gray-700 dark:text-gray-300;
}

.field-type {
  @apply text-xs px-2 py-0.5 rounded bg-blue-100 dark:bg-blue-900 
         text-blue-700 dark:text-blue-300;
}

/* Endpoint Badge */
.endpoint-method {
  @apply px-2 py-1 rounded text-xs font-bold uppercase;
}

.endpoint-method-get { @apply bg-green-100 text-green-700; }
.endpoint-method-post { @apply bg-blue-100 text-blue-700; }
.endpoint-method-put { @apply bg-yellow-100 text-yellow-700; }
.endpoint-method-delete { @apply bg-red-100 text-red-700; }

/* Connection Line */
.connection-line {
  @apply stroke-2 stroke-gray-400 fill-none;
}

.connection-line-selected {
  @apply stroke-blue-500;
}

/* Port Indicator */
.port-input {
  @apply w-3 h-3 rounded-full bg-blue-500 border-2 border-white shadow;
}

.port-output {
  @apply w-3 h-3 rounded-full bg-green-500 border-2 border-white shadow;
}
```

### 4.4 Key UI Components

#### 4.4.1 Welcome Page

```rust
// pages/welcome.rs
#[component]
fn WelcomePage() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gradient-to-br from-indigo-500 to-purple-600 
                      flex items-center justify-center",
            div { class: "bg-white rounded-2xl shadow-2xl p-8 max-w-md w-full",
                h1 { class: "text-3xl font-bold text-center mb-6",
                    "🔮 Immortal Engine"
                }
                p { class: "text-gray-600 text-center mb-8",
                    "Visual Code Generator for Rust"
                }
                
                button {
                    class: "w-full py-3 px-4 bg-indigo-600 hover:bg-indigo-700 
                            text-white rounded-lg font-medium transition",
                    onclick: move |_| { /* create new project */ },
                    "✨ Create New Project"
                }
                
                button {
                    class: "w-full py-3 px-4 mt-4 border-2 border-gray-300 
                            hover:border-indigo-500 rounded-lg font-medium transition",
                    onclick: move |_| { /* open existing */ },
                    "📂 Open Existing Project"
                }
                
                // Recent projects list
                div { class: "mt-8 pt-6 border-t border-gray-200",
                    h3 { class: "text-sm font-medium text-gray-500 mb-3",
                        "Recent Projects"
                    }
                    // ... recent project items
                }
            }
        }
    }
}
```

#### 4.4.2 Project Setup Page

```rust
// pages/project.rs
#[component]
fn ProjectSetupPage() -> Element {
    let mut project_name = use_signal(|| String::new());
    let mut project_type = use_signal(|| ProjectType::RestApi);
    let mut database = use_signal(|| DatabaseType::PostgreSQL);
    let mut auth_enabled = use_signal(|| true);
    
    rsx! {
        div { class: "p-8",
            h2 { class: "text-2xl font-bold mb-6", "Project Setup" }
            
            // Project Name
            div { class: "mb-6",
                label { class: "block text-sm font-medium mb-2", "Project Name" }
                input {
                    class: "w-full px-4 py-2 border rounded-lg focus:ring-2 
                            focus:ring-indigo-500 focus:border-transparent",
                    placeholder: "my_awesome_api",
                    value: "{project_name}",
                    oninput: move |e| project_name.set(e.value())
                }
            }
            
            // Project Type
            div { class: "mb-6",
                label { class: "block text-sm font-medium mb-2", "Project Type" }
                div { class: "grid grid-cols-2 gap-4",
                    ProjectTypeCard {
                        title: "REST API",
                        description: "Backend-only with Axum",
                        icon: "🔌",
                        selected: *project_type.read() == ProjectType::RestApi,
                        onclick: move |_| project_type.set(ProjectType::RestApi)
                    }
                    ProjectTypeCard {
                        title: "Fullstack",
                        description: "Axum backend + Dioxus frontend",
                        icon: "🖥️",
                        selected: *project_type.read() == ProjectType::Fullstack,
                        onclick: move |_| project_type.set(ProjectType::Fullstack)
                    }
                }
            }
            
            // Database Selection
            div { class: "mb-6",
                label { class: "block text-sm font-medium mb-2", "Database" }
                select {
                    class: "w-full px-4 py-2 border rounded-lg",
                    value: "{database:?}",
                    onchange: move |e| { /* update database */ },
                    option { value: "postgresql", "PostgreSQL" }
                    option { value: "mysql", "MySQL" }
                    option { value: "sqlite", "SQLite" }
                }
            }
            
            // Authentication Toggle
            div { class: "mb-6 flex items-center justify-between",
                div {
                    label { class: "font-medium", "Enable Authentication" }
                    p { class: "text-sm text-gray-500", 
                        "Add JWT-based authentication to your API" 
                    }
                }
                Toggle {
                    checked: *auth_enabled.read(),
                    onchange: move |v| auth_enabled.set(v)
                }
            }
            
            // Continue Button
            button {
                class: "w-full py-3 bg-indigo-600 hover:bg-indigo-700 
                        text-white rounded-lg font-medium",
                onclick: move |_| { /* proceed to entities */ },
                "Continue to Entity Design →"
            }
        }
    }
}
```

#### 4.4.3 Entity Design Canvas

```rust
// components/canvas.rs
#[component]
fn Canvas(
    entities: Signal<Vec<Entity>>,
    connections: Signal<Vec<Connection>>,
    selected: Signal<Option<EntityId>>,
) -> Element {
    let mut dragging = use_signal(|| None::<DragState>);
    let mut pan_offset = use_signal(|| (0.0, 0.0));
    let mut scale = use_signal(|| 1.0);
    
    rsx! {
        div {
            class: "relative w-full h-full bg-gray-50 dark:bg-gray-900 
                    overflow-hidden",
            // Grid background
            div {
                class: "absolute inset-0",
                style: "background-image: radial-gradient(circle, #ddd 1px, transparent 1px);
                        background-size: 20px 20px;"
            }
            
            // SVG Layer for connections
            svg {
                class: "absolute inset-0 pointer-events-none",
                for conn in connections.read().iter() {
                    ConnectionLine {
                        from: get_entity_port_position(entities, conn.from_entity, conn.from_port),
                        to: get_entity_port_position(entities, conn.to_entity, conn.to_port),
                        connection_type: conn.relation_type.clone()
                    }
                }
            }
            
            // Entity Cards Layer
            div {
                class: "absolute inset-0",
                style: "transform: translate({pan_offset.read().0}px, {pan_offset.read().1}px) 
                        scale({scale});",
                for entity in entities.read().iter() {
                    EntityCard {
                        entity: entity.clone(),
                        selected: *selected.read() == Some(entity.id),
                        on_select: move |_| selected.set(Some(entity.id)),
                        on_drag: move |delta| { /* handle drag */ },
                        on_port_click: move |port| { /* start connection */ }
                    }
                }
            }
            
            // Toolbar overlay
            div {
                class: "absolute bottom-4 left-1/2 transform -translate-x-1/2 
                        bg-white dark:bg-gray-800 rounded-full shadow-lg px-4 py-2 
                        flex items-center gap-2",
                button {
                    class: "p-2 hover:bg-gray-100 rounded-full",
                    onclick: move |_| scale.set((*scale.read() + 0.1).min(2.0)),
                    "➕"
                }
                span { class: "text-sm font-medium", "{(*scale.read() * 100.0) as i32}%" }
                button {
                    class: "p-2 hover:bg-gray-100 rounded-full",
                    onclick: move |_| scale.set((*scale.read() - 0.1).max(0.25)),
                    "➖"
                }
            }
        }
    }
}
```

#### 4.4.4 Entity Card Component

```rust
// components/entity_card.rs
#[component]
fn EntityCard(
    entity: Entity,
    selected: bool,
    on_select: EventHandler<()>,
    on_drag: EventHandler<(f32, f32)>,
    on_port_click: EventHandler<PortId>,
) -> Element {
    rsx! {
        div {
            class: "absolute bg-white dark:bg-gray-800 rounded-xl shadow-lg 
                    border-2 transition-all cursor-move min-w-[220px]
                    {if selected { \"border-indigo-500 ring-2 ring-indigo-200\" } 
                     else { \"border-gray-200 dark:border-gray-700\" }}",
            style: "left: {entity.position.x}px; top: {entity.position.y}px;",
            onclick: move |_| on_select.call(()),
            
            // Header
            div { class: "flex items-center gap-2 p-3 border-b border-gray-200 
                          dark:border-gray-600",
                span { class: "text-xl", "📊" }
                span { class: "font-bold text-gray-800 dark:text-white", 
                    "{entity.name}" 
                }
            }
            
            // Fields
            div { class: "p-3 space-y-1",
                for field in entity.fields.iter() {
                    div { class: "flex items-center gap-2 text-sm",
                        if field.is_primary {
                            span { class: "text-yellow-500", "🔑" }
                        }
                        span { class: "font-medium text-gray-700 dark:text-gray-300",
                            "{field.name}"
                        }
                        span { class: "text-xs px-2 py-0.5 rounded bg-blue-100 
                                      dark:bg-blue-900 text-blue-700 dark:text-blue-300",
                            "{field.data_type:?}"
                        }
                        if field.required {
                            span { class: "text-red-500 text-xs", "*" }
                        }
                    }
                }
            }
            
            // Output port (right side)
            div {
                class: "absolute right-0 top-1/2 transform translate-x-1/2 -translate-y-1/2",
                div {
                    class: "w-3 h-3 rounded-full bg-green-500 border-2 border-white 
                            shadow cursor-pointer hover:scale-125 transition",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_port_click.call(PortId::Output);
                    }
                }
            }
            
            // Input port (left side)
            div {
                class: "absolute left-0 top-1/2 transform -translate-x-1/2 -translate-y-1/2",
                div {
                    class: "w-3 h-3 rounded-full bg-blue-500 border-2 border-white 
                            shadow cursor-pointer hover:scale-125 transition",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_port_click.call(PortId::Input);
                    }
                }
            }
        }
    }
}
```

---

## 5. Core Features Implementation

### 5.1 Entity Definition

Entities are the core building blocks representing database tables/models.

#### 5.1.1 Entity Data Structure

```rust
// ir/src/entity.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub table_name: Option<String>,  // Custom table name override
    pub fields: Vec<Field>,
    pub position: Position,
    pub config: EntityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConfig {
    pub timestamps: bool,           // created_at, updated_at
    pub soft_delete: bool,          // deleted_at field
    pub id_type: IdType,            // UUID, Auto-increment, ULID
    pub auditable: bool,            // Track who created/modified
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: Uuid,
    pub name: String,
    pub data_type: DataType,
    pub required: bool,
    pub unique: bool,
    pub indexed: bool,
    pub default_value: Option<String>,
    pub validations: Vec<Validation>,
    pub is_primary: bool,
    pub is_foreign_key: bool,
    pub foreign_key_ref: Option<ForeignKeyRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    pub entity_id: Uuid,
    pub field_name: String,
    pub on_delete: ReferentialAction,
    pub on_update: ReferentialAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReferentialAction {
    Cascade,
    SetNull,
    Restrict,
    NoAction,
}
```

#### 5.1.2 Supported Data Types

| Type | Rust Type | SQL Type (PostgreSQL) | UI Widget |
|------|-----------|----------------------|-----------|
| `String` | `String` | `VARCHAR(255)` | Text input |
| `Text` | `String` | `TEXT` | Textarea |
| `Int32` | `i32` | `INTEGER` | Number input |
| `Int64` | `i64` | `BIGINT` | Number input |
| `Float32` | `f32` | `REAL` | Number input |
| `Float64` | `f64` | `DOUBLE PRECISION` | Number input |
| `Bool` | `bool` | `BOOLEAN` | Toggle |
| `Uuid` | `Uuid` | `UUID` | UUID display |
| `DateTime` | `DateTime<Utc>` | `TIMESTAMPTZ` | DateTime picker |
| `Date` | `NaiveDate` | `DATE` | Date picker |
| `Time` | `NaiveTime` | `TIME` | Time picker |
| `Json` | `serde_json::Value` | `JSONB` | JSON editor |
| `Bytes` | `Vec<u8>` | `BYTEA` | File upload |
| `Decimal` | `rust_decimal::Decimal` | `DECIMAL` | Number input |
| `Enum(variants)` | `enum` | `VARCHAR` / enum type | Dropdown |

### 5.2 Relationship Management

#### 5.2.1 Relationship Types

```rust
// ir/src/relationship.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub name: String,
    pub from_entity: Uuid,
    pub to_entity: Uuid,
    pub relation_type: RelationType,
    pub from_field: Option<String>,   // Foreign key field name
    pub to_field: String,             // Usually "id"
    pub inverse_name: Option<String>, // Name for inverse relation
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    /// One entity has exactly one of another (e.g., User has one Profile)
    OneToOne,
    /// One entity has many of another (e.g., User has many Posts)
    OneToMany,
    /// Many entities belong to one (e.g., Posts belong to User)
    ManyToOne,
    /// Many-to-many with junction table (e.g., Users and Roles)
    ManyToMany {
        junction_table: String,
    },
}
```

#### 5.2.2 Visual Relationship Lines

```
┌────────────────┐                    ┌────────────────┐
│     User       │                    │      Post      │
├────────────────┤                    ├────────────────┤
│ id: UUID 🔑    │                    │ id: UUID 🔑    │
│ email: String  │───── 1 ────< *────│ user_id: UUID  │
│ name: String   │    has many        │ title: String  │
└────────────────┘                    │ content: Text  │
                                      └────────────────┘
```

### 5.3 Endpoint Configuration

#### 5.3.1 Endpoint Data Structure

```rust
// ir/src/endpoint.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: Uuid,
    pub name: String,
    pub path: String,                 // e.g., "/api/users"
    pub entity_id: Option<Uuid>,      // Associated entity
    pub operations: Vec<CrudOperation>,
    pub security: EndpointSecurity,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrudOperation {
    pub operation: OperationType,
    pub enabled: bool,
    pub secured: bool,
    pub required_roles: Vec<String>,
    pub rate_limit: Option<RateLimit>,
    pub custom_path: Option<String>,  // Override default path
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Create,    // POST
    Read,      // GET (single)
    ReadAll,   // GET (list with pagination)
    Update,    // PUT/PATCH
    Delete,    // DELETE
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSecurity {
    pub auth_required: bool,
    pub roles: Vec<String>,
    pub rate_limit: Option<RateLimit>,
    pub cors_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests: u32,
    pub window_seconds: u32,
}
```

#### 5.3.2 Generated CRUD Operations

| Operation | HTTP Method | Path | Handler Function |
|-----------|-------------|------|------------------|
| Create | `POST` | `/api/{entity}` | `create_{entity}` |
| Read | `GET` | `/api/{entity}/:id` | `get_{entity}` |
| ReadAll | `GET` | `/api/{entity}` | `list_{entity}` |
| Update | `PUT` | `/api/{entity}/:id` | `update_{entity}` |
| Delete | `DELETE` | `/api/{entity}/:id` | `delete_{entity}` |

### 5.4 Integrated Endpoints (Relationships)

For relationships, additional endpoints are auto-generated:

```rust
// For User has many Posts relationship
GET    /api/users/:user_id/posts      // Get all posts by user
POST   /api/users/:user_id/posts      // Create post for user
GET    /api/posts/:id/user            // Get user who owns a post
```

---

## 6. Code Generation Engine

### 6.1 Generator Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CODE GENERATION PIPELINE                           │
└─────────────────────────────────────────────────────────────────────────────┘

  ProjectGraph          Analyzer           Templates          Output Files
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
  ┌─────────┐        ┌─────────┐        ┌─────────┐        ┌─────────────┐
  │ Entities │───────▶│ Extract │───────▶│ Models  │───────▶│ src/models/ │
  │  Nodes   │        │ Schema  │        │ Template│        │   *.rs      │
  └─────────┘        └─────────┘        └─────────┘        └─────────────┘
       │                   │                   │                   │
  ┌─────────┐        ┌─────────┐        ┌─────────┐        ┌─────────────┐
  │ Endpoint │───────▶│ Extract │───────▶│ Handler │───────▶│src/handlers/│
  │  Nodes   │        │ Routes  │        │ Template│        │   *.rs      │
  └─────────┘        └─────────┘        └─────────┘        └─────────────┘
       │                   │                   │                   │
  ┌─────────┐        ┌─────────┐        ┌─────────┐        ┌─────────────┐
  │  Auth   │───────▶│ Extract │───────▶│  Auth   │───────▶│ src/auth/   │
  │ Config  │        │ Config  │        │ Template│        │   *.rs      │
  └─────────┘        └─────────┘        └─────────┘        └─────────────┘
       │                   │                   │                   │
  ┌─────────┐        ┌─────────┐        ┌─────────┐        ┌─────────────┐
  │Relations│───────▶│ Build   │───────▶│Migration│───────▶│ migrations/ │
  │ Edges   │        │ Schema  │        │ Template│        │   *.sql     │
  └─────────┘        └─────────┘        └─────────┘        └─────────────┘
```

### 6.2 Generated Project Structure

#### 6.2.1 REST API Project

```
generated_project/
├── Cargo.toml
├── .env.example
├── .gitignore
├── README.md
│
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Module exports
│   ├── config.rs               # Configuration loading
│   ├── error.rs                # Error types
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs             # User entity struct + SeaORM
│   │   └── post.rs             # Post entity struct + SeaORM
│   │
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── user.rs             # User CRUD handlers
│   │   └── post.rs             # Post CRUD handlers
│   │
│   ├── auth/                   # If auth enabled
│   │   ├── mod.rs
│   │   ├── jwt.rs              # JWT utilities
│   │   ├── middleware.rs       # Auth middleware
│   │   └── handlers.rs         # Login/Register handlers
│   │
│   ├── routes.rs               # Route definitions
│   └── state.rs                # Application state
│
├── migrations/
│   ├── 001_create_users.sql
│   └── 002_create_posts.sql
│
└── tests/
    ├── mod.rs
    └── api_tests.rs
```

#### 6.2.2 Fullstack Project

```
generated_project/
├── Cargo.toml                  # Workspace root
├── .env.example
├── README.md
│
├── backend/                    # Axum backend
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── ... (same as REST API)
│
├── frontend/                   # Dioxus frontend
│   ├── Cargo.toml
│   ├── Dioxus.toml
│   ├── assets/
│   │   ├── tailwind.css
│   │   └── main.css
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── components/
│       │   ├── mod.rs
│       │   ├── navbar.rs
│       │   ├── sidebar.rs
│       │   ├── table.rs
│       │   └── form.rs
│       ├── pages/
│       │   ├── mod.rs
│       │   ├── home.rs
│       │   ├── users.rs        # User list page
│       │   ├── user_form.rs    # User create/edit
│       │   └── posts.rs        # Post list page
│       ├── api/
│       │   ├── mod.rs
│       │   └── client.rs       # API client
│       └── models/
│           └── mod.rs          # Shared types
│
└── shared/                     # Shared types crate
    ├── Cargo.toml
    └── src/
        └── lib.rs              # DTOs shared between frontend/backend
```

### 6.3 Generated Code Examples

#### 6.3.1 Model Generation (SeaORM)

```rust
// src/models/user.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, Validate)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    
    #[validate(email)]
    #[sea_orm(unique)]
    pub email: String,
    
    #[validate(length(min = 1, max = 100))]
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

// DTOs for API
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#### 6.3.2 Handler Generation (Axum)

```rust
// src/handlers/user.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};
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
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    
    let paginator = user::Entity::find()
        .paginate(&state.db, per_page);
    
    let total = paginator.num_items().await?;
    let total_pages = paginator.num_pages().await?;
    let users: Vec<UserResponse> = paginator
        .fetch_page(page - 1)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    
    Ok(Json(PaginatedResponse {
        data: users,
        meta: PaginationMeta {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

/// Get a single user by ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
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
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;
    
    // Check if email already exists
    let existing = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db)
        .await?;
    
    if existing.is_some() {
        return Err(AppError::Conflict("Email already exists".into()));
    }
    
    let password_hash = hash_password(&payload.password)?;
    
    let user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(payload.email),
        name: Set(payload.name),
        password_hash: Set(password_hash),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    
    let user = user.insert(&state.db).await?;
    
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

/// Update an existing user
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;
    
    let user = user::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    
    let mut user: user::ActiveModel = user.into();
    
    if let Some(name) = payload.name {
        user.name = Set(name);
    }
    user.updated_at = Set(Utc::now());
    
    let user = user.update(&state.db).await?;
    
    Ok(Json(UserResponse::from(user)))
}

/// Delete a user
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = user::Entity::delete_by_id(id)
        .exec(&state.db)
        .await?;
    
    if result.rows_affected == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(StatusCode::NO_CONTENT)
}
```

#### 6.3.3 Router Generation

```rust
// src/routes.rs
use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};

use crate::{
    handlers,
    auth::middleware::require_auth,
    state::AppState,
};

pub fn create_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }));
    
    let auth_routes = Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh_token));
    
    // User routes - secured
    let user_routes = Router::new()
        .route("/", get(handlers::user::list_users))
        .route("/", post(handlers::user::create_user))
        .route("/:id", get(handlers::user::get_user))
        .route("/:id", put(handlers::user::update_user))
        .route("/:id", delete(handlers::user::delete_user))
        // Nested routes for relationships
        .route("/:user_id/posts", get(handlers::post::list_user_posts))
        .route("/:user_id/posts", post(handlers::post::create_user_post))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));
    
    // Post routes - mixed security
    let post_routes = Router::new()
        .route("/", get(handlers::post::list_posts))  // Public
        .route("/:id", get(handlers::post::get_post)) // Public
        .merge(
            Router::new()
                .route("/", post(handlers::post::create_post))
                .route("/:id", put(handlers::post::update_post))
                .route("/:id", delete(handlers::post::delete_post))
                .layer(middleware::from_fn_with_state(state.clone(), require_auth))
        );
    
    Router::new()
        .merge(public_routes)
        .nest("/api/auth", auth_routes)
        .nest("/api/users", user_routes)
        .nest("/api/posts", post_routes)
        .with_state(state)
}
```

#### 6.3.4 Migration Generation

```sql
-- migrations/001_create_users.sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- migrations/002_create_posts.sql
CREATE TABLE IF NOT EXISTS posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_published ON posts(published) WHERE published = TRUE;
```

---

## 7. Data Flow & State Management

### 7.1 Application State

```rust
// ui/src/state.rs
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    /// Current project being edited
    pub project: Signal<Option<Project>>,
    /// All entities in the project
    pub entities: Signal<HashMap<Uuid, Entity>>,
    /// All relationships between entities
    pub relationships: Signal<Vec<Relationship>>,
    /// All endpoint configurations
    pub endpoints: Signal<HashMap<Uuid, Endpoint>>,
    /// Currently selected item
    pub selection: Signal<Selection>,
    /// Canvas view state
    pub canvas: Signal<CanvasState>,
    /// Undo/Redo history
    pub history: Signal<History>,
    /// UI state
    pub ui: Signal<UiState>,
}

#[derive(Clone, Default)]
pub struct Selection {
    pub selected_entities: HashSet<Uuid>,
    pub selected_relationships: HashSet<Uuid>,
    pub selected_endpoints: HashSet<Uuid>,
}

#[derive(Clone)]
pub struct CanvasState {
    pub pan: (f32, f32),
    pub zoom: f32,
    pub dragging: Option<DragState>,
    pub connecting: Option<ConnectionState>,
}

#[derive(Clone)]
pub struct UiState {
    pub sidebar_collapsed: bool,
    pub properties_collapsed: bool,
    pub active_page: Page,
    pub dialogs: DialogState,
    pub status_message: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum Page {
    Welcome,
    ProjectSetup,
    EntityDesign,
    EndpointConfig,
    RelationshipManager,
    CodeGeneration,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            project: Signal::new(None),
            entities: Signal::new(HashMap::new()),
            relationships: Signal::new(Vec::new()),
            endpoints: Signal::new(HashMap::new()),
            selection: Signal::new(Selection::default()),
            canvas: Signal::new(CanvasState::default()),
            history: Signal::new(History::new()),
            ui: Signal::new(UiState::default()),
        }
    }
    
    pub fn add_entity(&mut self, entity: Entity) {
        self.save_history();
        self.entities.write().insert(entity.id, entity);
    }
    
    pub fn remove_entity(&mut self, id: Uuid) {
        self.save_history();
        self.entities.write().remove(&id);
        // Also remove related relationships
        self.relationships.write().retain(|r| {
            r.from_entity != id && r.to_entity != id
        });
    }
    
    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.save_history();
        self.relationships.write().push(relationship);
    }
    
    fn save_history(&mut self) {
        let snapshot = ProjectSnapshot {
            entities: self.entities.read().clone(),
            relationships: self.relationships.read().clone(),
            endpoints: self.endpoints.read().clone(),
        };
        self.history.write().push(snapshot);
    }
    
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.history.write().undo() {
            *self.entities.write() = snapshot.entities;
            *self.relationships.write() = snapshot.relationships;
            *self.endpoints.write() = snapshot.endpoints;
        }
    }
    
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.history.write().redo() {
            *self.entities.write() = snapshot.entities;
            *self.relationships.write() = snapshot.relationships;
            *self.endpoints.write() = snapshot.endpoints;
        }
    }
}
```

### 7.2 Custom Hooks

```rust
// ui/src/hooks/use_project.rs
use dioxus::prelude::*;

/// Hook for managing project state and operations
pub fn use_project() -> UseProject {
    let state = use_context::<AppState>();
    
    UseProject { state }
}

pub struct UseProject {
    state: AppState,
}

impl UseProject {
    pub fn create_new(&self, config: ProjectConfig) {
        let project = Project::new(config);
        *self.state.project.write() = Some(project);
        self.state.entities.write().clear();
        self.state.relationships.write().clear();
        self.state.endpoints.write().clear();
    }
    
    pub async fn save(&self, path: &Path) -> Result<(), SaveError> {
        let project = self.state.project.read().clone()
            .ok_or(SaveError::NoProject)?;
        
        let data = ProjectData {
            meta: project,
            entities: self.state.entities.read().values().cloned().collect(),
            relationships: self.state.relationships.read().clone(),
            endpoints: self.state.endpoints.read().values().cloned().collect(),
        };
        
        let json = serde_json::to_string_pretty(&data)?;
        tokio::fs::write(path, json).await?;
        
        Ok(())
    }
    
    pub async fn load(&self, path: &Path) -> Result<(), LoadError> {
        let json = tokio::fs::read_to_string(path).await?;
        let data: ProjectData = serde_json::from_str(&json)?;
        
        *self.state.project.write() = Some(data.meta);
        *self.state.entities.write() = data.entities
            .into_iter()
            .map(|e| (e.id, e))
            .collect();
        *self.state.relationships.write() = data.relationships;
        *self.state.endpoints.write() = data.endpoints
            .into_iter()
            .map(|e| (e.id, e))
            .collect();
        
        Ok(())
    }
}

// ui/src/hooks/use_codegen.rs
pub fn use_codegen() -> UseCodegen {
    let state = use_context::<AppState>();
    UseCodegen { state }
}

pub struct UseCodegen {
    state: AppState,
}

impl UseCodegen {
    pub async fn generate(&self, output_path: &Path) -> Result<GenerationResult, CodegenError> {
        let project = self.state.project.read().clone()
            .ok_or(CodegenError::NoProject)?;
        
        // Build the ProjectGraph from state
        let graph = self.build_project_graph()?;
        
        // Create generator with appropriate config
        let config = GeneratorConfig {
            target_language: TargetLanguage::Rust,
            database_backend: project.database,
            auth_framework: if project.auth_enabled { 
                AuthFramework::Jwt 
            } else { 
                AuthFramework::None 
            },
            generate_frontend: project.project_type == ProjectType::Fullstack,
            output_dir: output_path.to_path_buf(),
            ..Default::default()
        };
        
        let generator = CodeGenerator::with_config(config);
        let generated = generator.generate(&graph)?;
        
        // Write files to disk
        generator.write_to_disk(&generated, output_path).await?;
        
        Ok(GenerationResult {
            files_generated: generated.file_count(),
            output_path: output_path.to_path_buf(),
            warnings: generated.warnings,
        })
    }
    
    fn build_project_graph(&self) -> Result<ProjectGraph, CodegenError> {
        // Convert application state to IR ProjectGraph
        // ... implementation
        Ok(ProjectGraph::default())
    }
}
```

---

## 8. Security Implementation

### 8.1 Authentication Options

The generated projects support multiple authentication strategies:

| Strategy | Use Case | Implementation |
|----------|----------|----------------|
| **None** | Public APIs, internal services | No auth middleware |
| **JWT** | Stateless APIs, microservices | `jsonwebtoken` crate |
| **Session** | Traditional web apps | `tower-sessions` crate |

### 8.2 JWT Authentication Generation

```rust
// Generated: src/auth/jwt.rs
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,           // User ID
    pub email: String,
    pub roles: Vec<String>,
    pub exp: usize,          // Expiration timestamp
    pub iat: usize,          // Issued at timestamp
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, roles: Vec<String>) -> Self {
        let now = chrono::Utc::now().timestamp() as usize;
        Self {
            sub: user_id,
            email,
            roles,
            exp: now + 3600 * 24, // 24 hours
            iat: now,
        }
    }
}

pub fn create_token(claims: &Claims, secret: &str) -> Result<String, AuthError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AuthError::TokenCreation)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AuthError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::InvalidToken)
}

// Extractor for authenticated user
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::MissingToken)?;

        let config = parts
            .extensions
            .get::<AppConfig>()
            .ok_or(AuthError::ConfigError)?;

        let claims = verify_token(bearer.token(), &config.jwt_secret)?;

        Ok(AuthUser {
            id: claims.sub,
            email: claims.email,
            roles: claims.roles,
        })
    }
}
```

### 8.3 Role-Based Access Control

```rust
// Generated: src/auth/middleware.rs
use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

/// Middleware that requires authentication
pub async fn require_auth<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    // AuthUser extractor will validate the token
    let _ = request.extensions().get::<AuthUser>()
        .ok_or(AuthError::Unauthorized)?;
    
    Ok(next.run(request).await)
}

/// Middleware that requires specific roles
pub fn require_roles(allowed_roles: &'static [&'static str]) -> impl Fn(
    AuthUser,
    Request<axum::body::Body>,
    Next<axum::body::Body>,
) -> impl Future<Output = Result<Response, AuthError>> + Clone {
    move |user: AuthUser, request, next| async move {
        let has_role = user.roles.iter()
            .any(|role| allowed_roles.contains(&role.as_str()));
        
        if !has_role {
            return Err(AuthError::Forbidden);
        }
        
        Ok(next.run(request).await)
    }
}

// Usage in routes:
// .route("/admin", get(admin_handler).layer(from_fn(require_roles(&["admin"]))))
```

### 8.4 Endpoint Security Configuration

In the UI, users configure per-endpoint security:

```
┌─────────────────────────────────────────────────────────────────┐
│  Endpoint: /api/users                                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  CRUD Operations                                                │
│  ┌────────────────────────────────────────────────────────────┐│
│  │  GET (List)     [✓ Enabled] [✓ Secured] Roles: [        ] ││
│  │  GET (Single)   [✓ Enabled] [✓ Secured] Roles: [        ] ││
│  │  POST (Create)  [✓ Enabled] [✓ Secured] Roles: [ admin  ] ││
│  │  PUT (Update)   [✓ Enabled] [✓ Secured] Roles: [ admin  ] ││
│  │  DELETE         [✓ Enabled] [✓ Secured] Roles: [ admin  ] ││
│  └────────────────────────────────────────────────────────────┘│
│                                                                 │
│  Rate Limiting                                                  │
│  [✓ Enabled]  Requests: [100] per [minute ▼]                   │
│                                                                 │
│  CORS                                                           │
│  [✓ Enabled]  Origins: [*                                    ] │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. Database Support

### 9.1 Supported Databases

| Database | ORM Support | Migration Format | Connection Pooling |
|----------|-------------|------------------|-------------------|
| **PostgreSQL** | SeaORM | SQL | deadpool-postgres |
| **MySQL** | SeaORM | SQL | deadpool |
| **SQLite** | SeaORM | SQL | Built-in |

### 9.2 Migration Generation

```rust
// codegen/src/rust/migrations.rs
pub fn generate_migration(entity: &Entity, index: usize, backend: DatabaseBackend) -> String {
    let table_name = entity.table_name.as_ref()
        .unwrap_or(&to_snake_case(&entity.name));
    
    let mut sql = format!(
        "-- Migration: Create {} table\n\
         -- Generated by Immortal Engine\n\n\
         CREATE TABLE IF NOT EXISTS {} (\n",
        entity.name, table_name
    );
    
    for (i, field) in entity.fields.iter().enumerate() {
        let column_def = generate_column_definition(field, backend);
        sql.push_str(&format!("    {}", column_def));
        
        if i < entity.fields.len() - 1 {
            sql.push(',');
        }
        sql.push('\n');
    }
    
    sql.push_str(");\n\n");
    
    // Add indexes
    for field in &entity.fields {
        if field.indexed && !field.is_primary {
            sql.push_str(&format!(
                "CREATE INDEX idx_{}_{} ON {}({});\n",
                table_name, field.name, table_name, field.name
            ));
        }
    }
    
    sql
}

fn generate_column_definition(field: &Field, backend: DatabaseBackend) -> String {
    let sql_type = data_type_to_sql(&field.data_type, backend);
    let mut def = format!("{} {}", field.name, sql_type);
    
    if field.is_primary {
        def.push_str(" PRIMARY KEY");
        if matches!(field.data_type, DataType::Uuid) && backend == DatabaseBackend::PostgreSQL {
            def.push_str(" DEFAULT gen_random_uuid()");
        }
    }
    
    if !field.required && !field.is_primary {
        // Nullable by default
    } else {
        def.push_str(" NOT NULL");
    }
    
    if field.unique {
        def.push_str(" UNIQUE");
    }
    
    if let Some(ref default) = field.default_value {
        def.push_str(&format!(" DEFAULT {}", default));
    }
    
    if let Some(ref fk) = field.foreign_key_ref {
        def.push_str(&format!(
            " REFERENCES {}({}) ON DELETE {}",
            fk.table_name, fk.column_name, 
            referential_action_to_sql(fk.on_delete)
        ));
    }
    
    def
}

fn data_type_to_sql(dt: &DataType, backend: DatabaseBackend) -> &'static str {
    match (dt, backend) {
        (DataType::Uuid, DatabaseBackend::PostgreSQL) => "UUID",
        (DataType::Uuid, _) => "VARCHAR(36)",
        (DataType::String, _) => "VARCHAR(255)",
        (DataType::Text, _) => "TEXT",
        (DataType::Int32, _) => "INTEGER",
        (DataType::Int64, DatabaseBackend::PostgreSQL) => "BIGINT",
        (DataType::Int64, _) => "BIGINT",
        (DataType::Float32, DatabaseBackend::PostgreSQL) => "REAL",
        (DataType::Float64, DatabaseBackend::PostgreSQL) => "DOUBLE PRECISION",
        (DataType::Float64, _) => "DOUBLE",
        (DataType::Bool, DatabaseBackend::PostgreSQL) => "BOOLEAN",
        (DataType::Bool, _) => "TINYINT(1)",
        (DataType::DateTime, DatabaseBackend::PostgreSQL) => "TIMESTAMPTZ",
        (DataType::DateTime, _) => "DATETIME",
        (DataType::Date, _) => "DATE",
        (DataType::Time, _) => "TIME",
        (DataType::Json, DatabaseBackend::PostgreSQL) => "JSONB",
        (DataType::Json, _) => "JSON",
        (DataType::Bytes, DatabaseBackend::PostgreSQL) => "BYTEA",
        (DataType::Bytes, _) => "BLOB",
        _ => "TEXT",
    }
}
```

### 9.3 Database Configuration Generation

```rust
// Generated: src/config.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}

// Generated: .env.example
DATABASE__URL=postgres://user:password@localhost:5432/myapp
DATABASE__MAX_CONNECTIONS=10
DATABASE__MIN_CONNECTIONS=2
DATABASE__CONNECT_TIMEOUT_SECONDS=30
DATABASE__IDLE_TIMEOUT_SECONDS=600
SERVER__HOST=0.0.0.0
SERVER__PORT=3000
AUTH__JWT_SECRET=your-super-secret-key-change-in-production
AUTH__TOKEN_EXPIRY_HOURS=24
```

---

## 10. Export System

### 10.1 Export Flow

```
┌──────────────────────────────────────────────────────────────────────────┐
│                           EXPORT DIALOG                                  │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  📁 Export Location                                                      │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ /home/user/projects/my_awesome_api                          [📂]  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Project Summary                                                         │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Name: my_awesome_api                                               │ │
│  │ Type: REST API                                                     │ │
│  │ Database: PostgreSQL                                               │ │
│  │ Entities: 3 (User, Post, Comment)                                  │ │
│  │ Endpoints: 15                                                      │ │
│  │ Auth: JWT enabled                                                  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  Files to Generate                                                       │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ ✓ Source files (.rs)                              ~25 files       │ │
│  │ ✓ Cargo.toml                                      1 file          │ │
│  │ ✓ Database migrations                             3 files         │ │
│  │ ✓ Configuration (.env.example)                    1 file          │ │
│  │ ✓ README.md                                       1 file          │ │
│  │ ○ Tests (optional)                                ~10 files       │ │
│  │ ○ Docker files (optional)                         2 files         │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────┐  ┌────────────────────────────────────────┐ │
│  │       Cancel           │  │      ✨ Generate Project               │ │
│  └────────────────────────┘  └────────────────────────────────────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### 10.2 File Dialog Integration

```rust
// ui/src/components/dialogs.rs
use rfd::FileDialog;
use std::path::PathBuf;

/// Opens a native file dialog for selecting export directory
pub async fn select_export_directory() -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select Export Location")
        .pick_folder()
}

/// Export dialog component
#[component]
fn ExportDialog(
    project: Project,
    entities: Vec<Entity>,
    endpoints: Vec<Endpoint>,
    on_export: EventHandler<PathBuf>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut export_path = use_signal(|| None::<PathBuf>);
    let mut include_tests = use_signal(|| false);
    let mut include_docker = use_signal(|| false);
    let mut is_generating = use_signal(|| false);
    
    let select_folder = move |_| {
        spawn(async move {
            if let Some(path) = select_export_directory().await {
                export_path.set(Some(path));
            }
        });
    };
    
    let start_export = move |_| {
        if let Some(path) = export_path.read().clone() {
            is_generating.set(true);
            on_export.call(path);
        }
    };
    
    rsx! {
        div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            div { class: "bg-white dark:bg-gray-800 rounded-xl shadow-2xl p-6 
                          max-w-lg w-full mx-4",
                
                h2 { class: "text-xl font-bold mb-4", "Export Project" }
                
                // Export location
                div { class: "mb-4",
                    label { class: "block text-sm font-medium mb-2", 
                        "📁 Export Location" 
                    }
                    div { class: "flex gap-2",
                        input {
                            class: "flex-1 px-3 py-2 border rounded-lg bg-gray-50",
                            readonly: true,
                            value: export_path.read().as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_default()
                        }
                        button {
                            class: "px-4 py-2 bg-gray-200 hover:bg-gray-300 rounded-lg",
                            onclick: select_folder,
                            "Browse..."
                        }
                    }
                }
                
                // Project summary
                div { class: "mb-4 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg",
                    h3 { class: "font-medium mb-2", "Project Summary" }
                    div { class: "text-sm space-y-1",
                        p { "Name: {project.name}" }
                        p { "Type: {project.project_type:?}" }
                        p { "Database: {project.database:?}" }
                        p { "Entities: {entities.len()}" }
                        p { "Endpoints: {endpoints.len()}" }
                    }
                }
                
                // Options
                div { class: "mb-6 space-y-2",
                    label { class: "flex items-center gap-2",
                        input {
                            r#type: "checkbox",
                            checked: *include_tests.read(),
                            onchange: move |e| include_tests.set(e.checked())
                        }
                        "Include test files"
                    }
                    label { class: "flex items-center gap-2",
                        input {
                            r#type: "checkbox",
                            checked: *include_docker.read(),
                            onchange: move |e| include_docker.set(e.checked())
                        }
                        "Include Docker configuration"
                    }
                }
                
                // Actions
                div { class: "flex gap-3",
                    button {
                        class: "flex-1 py-2 border rounded-lg hover:bg-gray-50",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "flex-1 py-2 bg-indigo-600 hover:bg-indigo-700 
                                text-white rounded-lg disabled:opacity-50",
                        disabled: export_path.read().is_none() || *is_generating.read(),
                        onclick: start_export,
                        if *is_generating.read() {
                            "Generating..."
                        } else {
                            "✨ Generate Project"
                        }
                    }
                }
            }
        }
    }
}
```

### 10.3 Post-Generation Actions

After successful generation:

1. **Show Success Dialog** with generated file summary
2. **Option to Open** the generated project folder
3. **Option to Open** in VS Code / preferred editor
4. **Display Next Steps** (run migrations, start server, etc.)

```rust
#[component]
fn GenerationSuccessDialog(
    result: GenerationResult,
    on_close: EventHandler<()>,
    on_open_folder: EventHandler<PathBuf>,
    on_open_editor: EventHandler<PathBuf>,
) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-xl shadow-2xl p-6 max-w-md w-full",
                
                div { class: "text-center mb-6",
                    div { class: "text-5xl mb-3", "🎉" }
                    h2 { class: "text-2xl font-bold", "Project Generated!" }
                }
                
                div { class: "bg-green-50 border border-green-200 rounded-lg p-4 mb-6",
                    p { class: "text-green-800",
                        "Successfully generated {result.files_generated} files"
                    }
                    p { class: "text-sm text-green-600 mt-1",
                        "Location: {result.output_path.display()}"
                    }
                }
                
                // Next steps
                div { class: "mb-6",
                    h3 { class: "font-medium mb-2", "Next Steps:" }
                    ol { class: "text-sm space-y-2 list-decimal list-inside",
                        li { "cd {result.output_path.display()}" }
                        li { "Copy .env.example to .env and configure" }
                        li { "Run database migrations" }
                        li { "cargo run" }
                    }
                }
                
                // Actions
                div { class: "space-y-2",
                    button {
                        class: "w-full py-2 bg-indigo-600 text-white rounded-lg",
                        onclick: move |_| on_open_folder.call(result.output_path.clone()),
                        "📂 Open Folder"
                    }
                    button {
                        class: "w-full py-2 border rounded-lg",
                        onclick: move |_| on_open_editor.call(result.output_path.clone()),
                        "💻 Open in VS Code"
                    }
                    button {
                        class: "w-full py-2 text-gray-600",
                        onclick: move |_| on_close.call(()),
                        "Close"
                    }
                }
            }
        }
    }
}
```

---

## 11. Implementation Phases

### Phase 1: Foundation (Weeks 1-2)

**Goal:** Set up Dioxus desktop application with basic navigation

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Set up Dioxus desktop project | High | 2d | None |
| Configure Tailwind CSS | High | 1d | Dioxus setup |
| Implement application shell (sidebar, main area) | High | 2d | Tailwind |
| Create welcome page | Medium | 1d | App shell |
| Create project setup page | High | 2d | App shell |
| Implement project state management | High | 2d | None |
| Set up file save/load with rfd | Medium | 1d | State mgmt |

**Deliverable:** Basic app that can create/save/load empty projects

### Phase 2: Entity Design (Weeks 3-4)

**Goal:** Visual entity design with fields and basic validation

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Implement canvas component | High | 3d | Phase 1 |
| Create entity card component | High | 2d | Canvas |
| Implement drag-and-drop for entities | High | 2d | Entity card |
| Build properties panel for entities | High | 2d | Entity card |
| Implement field CRUD | High | 2d | Properties |
| Add data type selection UI | Medium | 1d | Field CRUD |
| Implement field validation config | Medium | 2d | Field CRUD |

**Deliverable:** Can create entities with fields visually

### Phase 3: Relationships (Weeks 5-6)

**Goal:** Visual relationship drawing and configuration

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Implement port components (in/out) | High | 1d | Phase 2 |
| Create connection drawing logic | High | 3d | Ports |
| Implement relationship line rendering (SVG) | High | 2d | Connection |
| Build relationship properties panel | Medium | 2d | Connections |
| Add relationship type selection | Medium | 1d | Properties |
| Implement foreign key auto-generation | High | 2d | Relations |

**Deliverable:** Can create and configure entity relationships

### Phase 4: Endpoint Configuration (Weeks 7-8)

**Goal:** Configure API endpoints with security options

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Create endpoint configuration page | High | 2d | Phase 3 |
| Implement CRUD operation toggles | High | 2d | Endpoint page |
| Add security configuration per endpoint | High | 2d | CRUD toggles |
| Implement role-based access config | Medium | 2d | Security |
| Add rate limiting configuration | Low | 1d | Security |
| Create integrated endpoints for relationships | High | 2d | Relations |

**Deliverable:** Full endpoint configuration UI

### Phase 5: Code Generation (Weeks 9-11)

**Goal:** Generate complete, runnable Rust projects

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Refactor existing codegen for new IR | High | 3d | Phase 4 |
| Implement model generation (SeaORM) | High | 3d | Refactor |
| Implement handler generation (Axum) | High | 3d | Models |
| Implement auth generation (JWT) | High | 2d | Handlers |
| Implement migration generation | High | 2d | Models |
| Implement router generation | High | 2d | Handlers |
| Add Cargo.toml generation | Medium | 1d | All above |
| Add README generation | Low | 1d | All above |

**Deliverable:** Generate working REST API projects

### Phase 6: Fullstack Generation (Weeks 12-13)

**Goal:** Generate Dioxus frontend for fullstack projects

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Design frontend templates | High | 2d | Phase 5 |
| Generate Dioxus components | High | 3d | Templates |
| Generate API client code | High | 2d | Components |
| Generate CRUD pages | High | 3d | API client |
| Implement shared types crate | Medium | 1d | All above |
| Add navigation generation | Medium | 2d | Pages |

**Deliverable:** Generate fullstack projects with working UI

### Phase 7: Polish & Testing (Weeks 14-16)

**Goal:** Production-ready application

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Implement undo/redo | High | 2d | Phase 6 |
| Add keyboard shortcuts | Medium | 1d | All phases |
| Implement canvas zoom/pan | Medium | 2d | Canvas |
| Write integration tests | High | 3d | All phases |
| Write E2E tests for generated code | High | 3d | Codegen |
| Performance optimization | Medium | 2d | All |
| Documentation | High | 3d | All |
| Bug fixes and polish | High | 5d | All |

**Deliverable:** Production-ready v1.0 release

---

## 12. Testing Strategy

### 12.1 Test Categories

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            TESTING PYRAMID                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                            ┌───────────┐                                    │
│                            │    E2E    │  Generated project tests           │
│                            │   Tests   │  (compile & run)                   │
│                            └─────┬─────┘                                    │
│                         ┌────────┴────────┐                                 │
│                         │   Integration   │  Code generation tests          │
│                         │     Tests       │  (output verification)          │
│                         └────────┬────────┘                                 │
│                    ┌─────────────┴─────────────┐                            │
│                    │        Unit Tests         │  Component logic tests     │
│                    │        (Fast, Many)       │  (IR, codegen, UI state)   │
│                    └───────────────────────────┘                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 12.2 Unit Tests

```imortal_engine/tests/ir_tests.rs#L1-50
// tests/ir_tests.rs
#[cfg(test)]
mod tests {
    use imortal_ir::*;
    use imortal_core::DataType;
    
    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("User");
        assert_eq!(entity.name, "User");
        assert!(entity.fields.is_empty());
    }
    
    #[test]
    fn test_add_field_to_entity() {
        let mut entity = Entity::new("User");
        entity.add_field(Field {
            name: "email".to_string(),
            data_type: DataType::String,
            required: true,
            unique: true,
            ..Default::default()
        });
        
        assert_eq!(entity.fields.len(), 1);
        assert_eq!(entity.fields[0].name, "email");
    }
    
    #[test]
    fn test_relationship_creation() {
        let user = Entity::new("User");
        let post = Entity::new("Post");
        
        let relationship = Relationship::one_to_many(
            user.id,
            post.id,
            "posts",
        );
        
        assert_eq!(relationship.relation_type, RelationType::OneToMany);
    }
}
```

### 12.3 Integration Tests

```imortal_engine/tests/codegen_tests.rs#L1-60
// tests/codegen_tests.rs
#[cfg(test)]
mod codegen_integration_tests {
    use imortal_codegen::*;
    use imortal_ir::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_generate_simple_project() {
        // Create a simple project graph
        let mut graph = ProjectGraph::new("test_project");
        
        // Add User entity
        let user = Entity::new("User")
            .with_field(Field::primary_key("id", DataType::Uuid))
            .with_field(Field::new("email", DataType::String).required().unique())
            .with_field(Field::new("name", DataType::String).required());
        
        graph.add_entity(user);
        
        // Generate code
        let config = GeneratorConfig::default();
        let generator = CodeGenerator::with_config(config);
        let result = generator.generate(&graph).unwrap();
        
        // Verify files were generated
        assert!(result.files.contains_key("Cargo.toml"));
        assert!(result.files.contains_key("src/main.rs"));
        assert!(result.files.contains_key("src/models/user.rs"));
        assert!(result.files.contains_key("src/handlers/user.rs"));
    }
    
    #[tokio::test]
    async fn test_write_to_disk() {
        let graph = create_test_graph();
        let generator = CodeGenerator::default();
        let result = generator.generate(&graph).unwrap();
        
        let temp_dir = TempDir::new().unwrap();
        generator.write_to_disk(&result, temp_dir.path()).await.unwrap();
        
        // Verify files exist
        assert!(temp_dir.path().join("Cargo.toml").exists());
        assert!(temp_dir.path().join("src/main.rs").exists());
    }
}
```

### 12.4 E2E Tests (Generated Code)

```imortal_engine/tests/e2e_tests.rs#L1-45
// tests/e2e_tests.rs
#[cfg(test)]
mod e2e_tests {
    use std::process::Command;
    use tempfile::TempDir;
    
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_generated_project_compiles() {
        // Generate a project
        let temp_dir = TempDir::new().unwrap();
        let graph = create_full_test_graph();
        let generator = CodeGenerator::default();
        let result = generator.generate(&graph).unwrap();
        generator.write_to_disk_sync(&result, temp_dir.path()).unwrap();
        
        // Try to compile it
        let output = Command::new("cargo")
            .arg("check")
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run cargo check");
        
        assert!(
            output.status.success(),
            "Generated project failed to compile: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    #[test]
    #[ignore]
    fn test_generated_tests_pass() {
        let temp_dir = generate_test_project();
        
        let output = Command::new("cargo")
            .arg("test")
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to run cargo test");
        
        assert!(output.status.success());
    }
}
```

### 12.5 UI Testing

For Dioxus desktop UI testing, we use:

1. **Component Unit Tests** - Test individual components in isolation
2. **State Tests** - Verify state management logic
3. **Snapshot Tests** - Compare rendered output

```imortal_engine/tests/ui_tests.rs#L1-30
// tests/ui_tests.rs
#[cfg(test)]
mod ui_tests {
    use dioxus::prelude::*;
    use imortal_ui::components::*;
    use imortal_ui::state::*;
    
    #[test]
    fn test_app_state_add_entity() {
        let mut state = AppState::new();
        let entity = Entity::new("TestEntity");
        
        state.add_entity(entity.clone());
        
        assert!(state.entities.read().contains_key(&entity.id));
    }
    
    #[test]
    fn test_app_state_undo_redo() {
        let mut state = AppState::new();
        
        state.add_entity(Entity::new("Entity1"));
        state.add_entity(Entity::new("Entity2"));
        
        assert_eq!(state.entities.read().len(), 2);
        
        state.undo();
        assert_eq!(state.entities.read().len(), 1);
        
        state.redo();
        assert_eq!(state.entities.read().len(), 2);
    }
}
```

---

## 13. Future Enhancements

### 13.1 Short-term Roadmap (v1.1 - v1.3)

| Version | Feature | Description |
|---------|---------|-------------|
| **v1.1** | Template Library | Pre-built project templates (Blog, E-commerce, SaaS) |
| **v1.1** | Custom Validators | User-defined validation rules |
| **v1.2** | GraphQL Support | Generate GraphQL APIs alongside REST |
| **v1.2** | Multi-database | Support multiple databases in one project |
| **v1.3** | Plugin System | Community component plugins |
| **v1.3** | Import OpenAPI | Import existing OpenAPI specs |

### 13.2 Medium-term Roadmap (v2.0+)

| Feature | Description | Complexity |
|---------|-------------|------------|
| **Real-time Collaboration** | Multiple users editing same project | High |
| **Cloud Sync** | Sync projects across devices | Medium |
| **AI Assistant** | Natural language to entity/endpoint | High |
| **Web Version** | Browser-based editor (Dioxus Web) | Medium |
| **Template Marketplace** | Share/sell custom templates | Medium |
| **CI/CD Integration** | Generate GitHub Actions, Dockerfiles | Low |

### 13.3 Long-term Vision

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      IMMORTAL ENGINE ECOSYSTEM                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐    │
│  │  Desktop    │   │    Web      │   │    CLI      │   │    VSCode   │    │
│  │    App      │   │   Editor    │   │   Tools     │   │  Extension  │    │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘    │
│         │                 │                 │                 │            │
│         └─────────────────┴─────────────────┴─────────────────┘            │
│                                    │                                        │
│                           ┌────────▼────────┐                              │
│                           │   Core Engine   │                              │
│                           │  (Rust Library) │                              │
│                           └────────┬────────┘                              │
│                                    │                                        │
│         ┌──────────────────────────┼──────────────────────────┐            │
│         │                          │                          │            │
│  ┌──────▼──────┐            ┌──────▼──────┐           ┌──────▼──────┐     │
│  │   Backend   │            │  Fullstack  │           │  Embedded   │     │
│  │ Generation  │            │ Generation  │           │ Generation  │     │
│  │  (Axum,     │            │  (+ Dioxus, │           │  (ESP32,    │     │
│  │   Actix)    │            │   Leptos)   │           │   RPi)      │     │
│  └─────────────┘            └─────────────┘           └─────────────┘     │
│                                                                             │
│                           ┌─────────────────┐                              │
│                           │    Template     │                              │
│                           │   Marketplace   │                              │
│                           └─────────────────┘                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 13.4 Potential Integrations

| Integration | Purpose | Priority |
|-------------|---------|----------|
| **GitHub** | Project hosting, CI/CD | High |
| **Supabase** | Backend-as-a-service option | Medium |
| **Vercel/Netlify** | Frontend deployment | Medium |
| **AWS/GCP/Azure** | Cloud deployment templates | Low |
| **Stripe** | Payment integration templates | Medium |
| **Auth0/Clerk** | External auth providers | Medium |

---

## 14. Conclusion

This implementation plan provides a comprehensive roadmap for building Immortal Engine v2.0, a visual code generator for Rust applications. Key highlights:

### Technical Decisions

1. **Dioxus Desktop** - Modern, reactive UI with native performance
2. **Axum** - Best-in-class Rust backend framework for generated code
3. **SeaORM** - Type-safe, async ORM for database operations
4. **Tailwind CSS** - Rapid UI development with utility-first CSS

### Core Differentiators

1. **Visual-First Design** - Drag-and-drop entity and endpoint design
2. **Relationship Visualization** - See data model connections at a glance
3. **Security Built-In** - Per-endpoint authentication configuration
4. **Production-Ready Output** - Generated code follows Rust best practices

### Success Metrics

| Metric | Target |
|--------|--------|
| Time to generate basic API | < 5 minutes |
| Generated code compilation | 100% success rate |
| Lines of boilerplate saved | ~80% reduction |
| User satisfaction | > 4.5/5 rating |

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Entity** | A data model representing a database table/struct |
| **Relationship** | A connection between entities (1:1, 1:N, N:M) |
| **Endpoint** | An API route with CRUD operations |
| **IR** | Intermediate Representation - the graph data structure |
| **Codegen** | Code Generation - transforming IR to source code |

## Appendix B: File Templates

See `crates/codegen/src/templates/` for all code generation templates.

## Appendix C: Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New Project |
| `Ctrl+O` | Open Project |
| `Ctrl+S` | Save Project |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+E` | Add Entity |
| `Del` | Delete Selected |
| `Ctrl+G` | Generate Code |
| `Esc` | Cancel Action |

---

*Document Version: 1.0*
*Last Updated: 2025*
*Author: Immortal Engine Team*