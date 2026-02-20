<div align="center">

# 🔮 Immortal Engine v2.0 (S-Arch-P)

**A Visual Code Generator for Production-Ready Rust Applications**

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Dioxus](https://img.shields.io/badge/UI-Dioxus%200.7-purple.svg)](https://dioxuslabs.com/)
[![Axum](https://img.shields.io/badge/Generated-Axum%200.8-green.svg)](https://github.com/tokio-rs/axum)
[![SeaORM](https://img.shields.io/badge/ORM-SeaORM%201.1-blue.svg)](https://www.sea-ql.org/SeaORM/)

[Features](#-features) •
[Installation](#-installation) •
[Quick Start](#-quick-start) •
[Architecture](#-architecture) •
[Code Generation](#-code-generation) •
[Documentation](#-documentation)

---

*Design entities, define relationships, configure endpoints, and generate complete production-ready Rust projects — all from a visual interface.*

</div>

## 🎯 What is Immortal Engine?

**Immortal Engine v2.0** (codenamed **S-Arch-P** — Schema Architecture Platform) is a **desktop application** built entirely in Rust that lets you visually design and generate production-ready Rust backend and fullstack applications.

Instead of writing repetitive boilerplate, you:

1. **🎨 Design** data entities visually on an interactive canvas
2. **🔗 Connect** them with relationships (1:1, 1:N, N:M) — auto-detected from FK fields
3. **🔌 Configure** REST API endpoints with CRUD operations, security, and rate limiting
4. **🔐 Set up** JWT authentication with role-based access control
5. **⚡ Generate** a complete Rust project ready to `cargo run`

The generated project includes **Axum** web framework, **SeaORM** for database access, **SQL migrations**, **JWT authentication**, **integration tests**, and optionally a **Dioxus Web frontend**.

## ✨ Features

### Visual Entity Designer
- **Interactive canvas** with pan, zoom, grid snap, and drag-and-drop
- **Entity cards** showing fields, types, constraints, and relationships
- **Field templates** — pre-configured presets for common patterns:
  - 🔒 Password (hashed with bcrypt, secret, min 8 chars)
  - 📧 Email (unique, indexed, validated)
  - 👤 Username (unique, alphanumeric pattern)
  - 📱 Phone, 🔗 URL, 🏷️ Status, 🔤 Slug, ✅ Boolean, 🔢 Counter, 💰 Price, 📝 Rich Text, { } JSON, 🔑 Foreign Key
- **15+ data types** — String, Text, Int32, Int64, Float32, Float64, Bool, UUID, DateTime, Date, Time, JSON, Bytes, Arrays, Enums
- **Field validations** — Required, MinLength, MaxLength, Min, Max, Email, URL, Phone, Regex patterns, Custom
- **Entity configuration** — timestamps (created_at/updated_at), soft delete, auditable

### Relationship Management
- **Auto-detection** of relationships from foreign key fields
- **Visual connection lines** (bezier curves) drawn between entity cards
- **Relationship types** — One-to-One, One-to-Many, Many-to-One, Many-to-Many
- **Referential actions** — CASCADE, SET NULL, RESTRICT, NO ACTION, SET DEFAULT
- **Canvas and list views** with search and filtering

### Endpoint Configuration
- **Per-entity CRUD endpoints** — Create, Read, ReadAll, Update, Delete
- **Toggle individual operations** directly on endpoint cards
- **3 view modes** — Grid, List, Compact
- **Security configuration** — global and per-operation auth overrides
- **Rate limiting** — per-operation with presets (Permissive, Moderate, Strict)
- **Auto-generate endpoints** for all entities with one click
- **Authentication endpoints** (auto-generated when auth enabled):
  - `POST /api/auth/register` — User registration
  - `POST /api/auth/login` — Login with JWT token response
  - `GET /api/auth/me` — Current user profile
  - `POST /api/auth/refresh` — Token refresh
  - `PUT /api/auth/me/password` — Change password
  - `POST /api/auth/forgot-password` — Password reset request
  - `POST /api/auth/reset-password` — Password reset
- **Relationship-based nested endpoints** (dynamically generated):
  - `GET /api/users/:user_id/posts` — List children of parent
  - `POST /api/users/:user_id/posts` — Create child under parent
  - `GET /api/users/:user_id/posts/:post_id` — Get specific child
  - `DELETE /api/users/:user_id/posts/:post_id` — Delete child
  - `GET /api/users/:user_id/posts/count` — Count children
- **Toggleable** — enable/disable individual relationship endpoints
- **Dynamic descriptions** — each endpoint has an auto-generated explanation

### Database Configuration
- **3 databases supported** — PostgreSQL, MySQL, SQLite
- **Connection configuration** — host, port, username, password, database name
- **Connection pool settings** — max/min connections, SSL toggle
- **🔌 Test Connection** — verifies credentials by actually authenticating (not just TCP)
- **🗄️ Create Database** — creates the database on the server from the UI
- **Connection URL preview** with password masking

### JWT Authentication
- **Configurable** — enable/disable from Project Setup
- **JWT strategy** — Claims with sub (user ID), email, roles, iat, exp
- **Password hashing** — bcrypt with automatic detection of password fields
- **Token expiry** — configurable hours
- **Per-endpoint security** — open, authenticated, or role-based per operation
- **Generated auth code** includes:
  - Claims struct with role checking helpers
  - `create_token` / `verify_token` functions
  - `hash_password` / `verify_password` with bcrypt
  - `require_auth` middleware for Axum
  - `check_roles` for per-handler authorization

### Code Generation Engine
- **Complete project generation** — every file needed to `cargo build && cargo run`
- **Generated REST API project structure**:
  ```
  my_app/
  ├── Cargo.toml              # Dependencies based on your config
  ├── .env.example            # Environment variables with your DB connection
  ├── .gitignore
  ├── README.md               # Setup instructions
  ├── src/
  │   ├── main.rs             # Tokio entry point
  │   ├── lib.rs              # Module declarations
  │   ├── config.rs           # Config from environment
  │   ├── error.rs            # AppError with JSON responses
  │   ├── state.rs            # AppState (DB pool + config)
  │   ├── middleware.rs        # Request logging, request ID
  │   ├── models/             # SeaORM entities + DTOs
  │   │   ├── mod.rs
  │   │   └── {entity}.rs     # Model, Relations, Create/Update/Response DTOs
  │   ├── handlers/           # Axum CRUD handlers
  │   │   ├── mod.rs          # Pagination types
  │   │   └── {entity}.rs     # list, get, create, update, delete
  │   ├── routes/             # Router configuration
  │   │   ├── mod.rs          # create_router with middleware
  │   │   └── api.rs          # Per-entity routes with auth layers
  │   └── auth/               # JWT authentication (if enabled)
  │       ├── mod.rs
  │       ├── jwt.rs          # Claims, tokens, password hashing
  │       └── middleware.rs   # require_auth, check_roles
  ├── migrations/             # SQL migrations per entity
  │   └── {date}_create_{table}.sql
  └── tests/
      └── api_tests.rs        # Integration tests with TestServer
  ```
- **Fullstack project** (when configured) adds:
  - `frontend/` — Dioxus Web app with components, pages, router
  - `shared/` — DTOs shared between frontend and backend
  - Workspace `Cargo.toml` tying everything together
- **Smart code generation**:
  - Password fields automatically hashed with bcrypt
  - Create DTO renames `password_hash` → `password` (plain text from user)
  - Response DTO excludes secret fields
  - Soft-delete generates `SET deleted_at` instead of `DELETE`
  - Timestamps auto-set on create/update
  - Validation attributes from field configuration
  - Conditional dependencies (auth, DB driver, OpenAPI, CORS)

### SQL Migration Generation
- **Database-specific type mapping** — UUID/CHAR(36)/TEXT, JSONB/JSON/TEXT, BOOLEAN/TINYINT(1)/INTEGER
- **Foreign key constraints** with referential actions
- **Indexes** for indexed and FK fields
- **Soft-delete** column with index
- **Default values** — NOW(), gen_random_uuid(), literals, expressions
- **Dependency-ordered** — referenced tables created first
- **PostgreSQL comments** from entity/field descriptions
- **Proper quoting** — double-quotes for PostgreSQL/SQLite, backticks for MySQL

### Project Management
- **Save/Open** project files (`.ieng` format)
- **Recent projects** — shown on Welcome page, persisted across sessions
- **Project location** — visible and configurable in Project Setup
- **Setup validation** — warns before proceeding with incomplete database or missing config
- **Auto-save path** — first Save picks location, subsequent saves go to same path

### Frontend Generation (Fullstack Mode)
- **Dioxus Web** application with Tailwind CSS
- **Components** — Navbar, Sidebar, DataTable, Pagination, Forms, Alerts, Modals
- **Per-entity pages** — List (with pagination + delete) and Create/Edit forms
- **Smart form inputs** — type inferred from field (email→email, password→password, url→url)
- **API client** — type-safe reqwest wrapper with per-entity CRUD methods
- **Shared crate** — DTOs used by both frontend and backend
- **Router** — Dioxus Routable with per-entity List/New/Edit routes + 404

## 🏗️ Architecture

### Workspace Structure

```
s-arch-p/
├── Cargo.toml                    # Workspace root
├── src/main.rs                   # Desktop app entry point
│
├── crates/
│   ├── core/                     # imortal_core — shared types, errors, traits
│   ├── ir/                       # imortal_ir — intermediate representation
│   ├── codegen/                  # imortal_codegen — code generation engine
│   ├── ui/                       # imortal_ui — Dioxus desktop UI
│   └── cli/                      # imortal_cli — command line interface
│
├── assets/styles/                # Tailwind CSS source and compiled output
├── docs/                         # Specification and design documents
└── tests/                        # Integration tests
```

### Crate Dependencies

```text
                  imortal_core
                  Types, Errors
                       |
          +------------+------------+
          |            |            |
     imortal_ir   imortal_    imortal_
      Entities     codegen       cli
      Relations   Generator   Commands
      Endpoints   Templates
          |            ^
          +------------+
                |
                v
           imortal_ui
          Dioxus Desktop
          Pages, Canvas
```

### Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **UI Framework** | Dioxus 0.7 Desktop | Native cross-platform GUI |
| **Styling** | Tailwind CSS 4.1 | Utility-first CSS |
| **State** | Dioxus GlobalSignal | Reactive state management |
| **File Dialogs** | rfd 0.17 | Native OS file dialogs |
| **Code Generation** | String templates | Generates Rust, SQL, TOML, Markdown |
| **Case Conversion** | heck 0.5 | snake_case, PascalCase, camelCase |
| **Serialization** | Serde + JSON | Project file persistence |
| **Generated Backend** | Axum 0.8 | Web framework |
| **Generated ORM** | SeaORM 1.1 | Async database access |
| **Generated Auth** | jsonwebtoken + bcrypt | JWT tokens + password hashing |
| **Generated Frontend** | Dioxus Web | SPA with reqwest API client |

## 📦 Installation

### Prerequisites

- **Rust** (2024 Edition) — `rustup default stable`
- **Node.js 18+** — for Tailwind CSS compilation
- **System dependencies** (Linux):

**Fedora:**
```bash
sudo dnf install -y \
    webkit2gtk4.1-devel \
    libsoup3-devel \
    javascriptcoregtk4.1-devel \
    openssl-devel \
    gtk3-devel \
    libxdo-devel
```

**Ubuntu/Debian:**
```bash
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/70-codes/immortal_engine.git
cd immortal_engine/s-arch-p

# Install Node dependencies and build CSS
npm install
npm run build

# Build and run the application
cargo run --release
```

## 🚀 Quick Start

### 1. Create a New Project

Launch the application and click **"New Project"** on the Welcome page. Enter a project name and press Enter.

### 2. Configure Project Settings

Navigate to **Project Setup** to configure:
- **Project type** — REST API or Fullstack
- **Database** — PostgreSQL, MySQL, or SQLite (with connection details)
- **Authentication** — JWT tokens with configurable expiry
- **Test Connection** — verify your database is reachable
- **Create Database** — create the database directly from the UI

### 3. Design Entities

Go to **Entity Design** and:
- Click **"+ Add Entity"** or double-click the canvas
- Add fields using **Quick Templates** (Password, Email, Username, etc.) or custom
- Configure constraints, validations, and foreign keys per field
- Drag entities to arrange them on the canvas

### 4. Configure Endpoints

Navigate to **Endpoints** and:
- Click **"Auto-Generate"** to create CRUD endpoints for all entities
- Toggle individual operations (Create, Read, List, Update, Delete)
- Configure security (open, authenticated, role-based) per operation
- Set rate limits with presets (Permissive, Moderate, Strict)
- View auto-generated auth and relationship endpoints

### 5. Generate Code

Go to **Code Generation** and:
- Review the project summary (entities, relationships, endpoints, auth)
- Select generation options (tests, docs, migrations, overwrite)
- Choose an output directory
- Click **🚀 Generate Project**
- View generated files organized by category
- Follow the Quick Start commands to build and run

### 6. Run Your Generated Project

```bash
cd /path/to/generated/project
cp .env.example .env
# Edit .env with your database credentials
cargo build
cargo run
```

Your API server starts at `http://0.0.0.0:8080` with all configured endpoints ready to use.

## ⚡ Code Generation

### What Gets Generated

| Category | Files | Description |
|----------|-------|-------------|
| **Scaffold** | `Cargo.toml`, `.env.example`, `.gitignore`, `README.md` | Project setup with correct dependencies |
| **Models** | `src/models/{entity}.rs` | SeaORM Model, Relations, Create/Update/Response DTOs |
| **Handlers** | `src/handlers/{entity}.rs` | Axum CRUD with pagination, validation, error handling |
| **Routes** | `src/routes/api.rs` | Router with public/secured route splitting |
| **Auth** | `src/auth/jwt.rs`, `middleware.rs` | JWT Claims, tokens, bcrypt, require_auth middleware |
| **Config** | `src/config.rs` | Environment-based configuration |
| **Error** | `src/error.rs` | AppError → JSON response with proper status codes |
| **Middleware** | `src/middleware.rs` | Request logging, request ID, body size limit |
| **Migrations** | `migrations/*.sql` | CREATE TABLE with FK, indexes, multi-DB support |
| **Tests** | `tests/api_tests.rs` | TestServer, per-entity CRUD lifecycle tests |
| **Frontend** | `frontend/src/**` | Dioxus Web with pages, components, API client (fullstack only) |
| **Shared** | `shared/src/lib.rs` | DTOs shared between frontend & backend (fullstack only) |

### Database Support

| Feature | PostgreSQL | MySQL | SQLite |
|---------|-----------|-------|--------|
| UUID primary keys | `UUID` | `CHAR(36)` | `TEXT` |
| JSON fields | `JSONB` | `JSON` | `TEXT` |
| Boolean | `BOOLEAN` | `TINYINT(1)` | `INTEGER` |
| Timestamps | `TIMESTAMP WITH TIME ZONE` | `DATETIME` | `TEXT` |
| Auto-increment | `SERIAL` | `INT AUTO_INCREMENT` | `INTEGER` |
| Arrays | `TYPE[]` | `JSON` | `JSON` |
| Identifier quoting | `"double_quotes"` | `` `backticks` `` | `"double_quotes"` |

## 🧪 Testing

```bash
# Run all workspace tests
cargo test --workspace

# Run specific crate tests
cargo test -p imortal_core      # 43 tests — types, errors, traits
cargo test -p imortal_ir        # 97 tests — entities, fields, relationships, endpoints
cargo test -p imortal_codegen   # 362 tests — all code generators

# Run with output
cargo test --workspace -- --nocapture

# Check code
cargo check --workspace
cargo clippy --workspace
cargo fmt --all
```

**Total: 502+ tests** across all crates.

## 📁 Documentation

Detailed documentation is available in the `docs/` directory:

| Document | Description |
|----------|-------------|
| [Comprehensive Specification](docs/COMPREHENSIVE_SPECIFICATION.md) | Full technical spec — data models, UI design, code generation, security |
| [Implementation Plan](docs/IMPLEMENTATION_PLAN.md) | Detailed implementation plan with architecture and examples |
| [Quick Start Guide](docs/QUICK_START_GUIDE.md) | Developer onboarding — environment setup, crate structure, checklists |

## 🔮 Planned Features

### Business Logic System
A visual system for defining custom logic beyond CRUD:

- **Entity Lifecycle Hooks** — before/after create, update, delete (e.g., send email after registration, validate business rules)
- **Custom Endpoints** — non-CRUD routes (e.g., `POST /api/payments/process`)
- **Service Integrations** — third-party API calls (Stripe, SendGrid, Firebase)

### Additional Planned
- CLI tool for headless code generation
- Undo/redo with full history
- Dark/light theme toggle
- Real-time collaboration (v3.0)
- GraphQL generation
- OpenAPI/Swagger UI generation

## 🛠️ Development

### Build CSS
```bash
npm run build          # One-time build
npm run watch          # Watch mode for development
```

### Project Structure
```
crates/core/       — DataType, Position, EngineError, Validatable trait
crates/ir/         — Entity, Field, Relationship, EndpointGroup, ProjectGraph
crates/codegen/    — Generator, context, rust/*, migrations/*, frontend/*
crates/ui/         — App, pages/*, components/*, hooks/*, state
crates/cli/        — Command-line interface (placeholder)
```

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## 👤 Author

**Stephen Kinuthia**
- GitHub: [@70-codes](https://github.com/70-codes)
- Email: kinuthiasteve098@gmail.com

---

<div align="center">

**Immortal Engine v2.0** — Design Visually, Generate Professionally.

*Stephen Kinuthia*

</div>