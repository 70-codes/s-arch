# Immortal Engine v2.0 (S-Arch-P) - Quick Start Guide & Implementation Checklist

> **Quick Reference for Developers**  
> Last Updated: January 29, 2026

---

## 📋 Pre-Implementation Checklist

### Environment Setup

- [ ] Rust 2024 Edition installed (`rustup default nightly` or stable with 2024 edition)
- [ ] Node.js 18+ (for Tailwind CSS compilation)
- [ ] VS Code with rust-analyzer extension
- [ ] Git configured

### System Dependencies (Linux)

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### System Dependencies (macOS)

```bash
# Xcode command line tools
xcode-select --install
```

### System Dependencies (Windows)

- Visual Studio Build Tools 2022
- WebView2 Runtime

---

## 🚀 Project Bootstrap

### Step 1: Initialize Workspace

```bash
cd devop_nd_sec/s-arch-p

# Update Cargo.toml to workspace configuration
cat > Cargo.toml << 'EOF'
[package]
name = "s-arch-p"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "immortal-engine"
path = "src/main.rs"

[dependencies]
imortal_core = { path = "crates/core" }
imortal_ir = { path = "crates/ir" }
imortal_codegen = { path = "crates/codegen" }
imortal_ui = { path = "crates/ui" }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[workspace]
resolver = "2"
members = [
    ".",
    "crates/core",
    "crates/ir",
    "crates/codegen",
    "crates/ui",
    "crates/cli",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["Stephen Kinuthia <kinuthiasteve098@gmail.com>"]
license = "MIT"

[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Unique identifiers
uuid = { version = "1.0", features = ["v4", "serde"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Code generation
quote = "1.0"
syn = { version = "2.0", features = ["full", "parsing"] }
proc-macro2 = "1.0"
heck = "0.5"

# Dioxus
dioxus = { version = "0.6", features = ["desktop"] }

# File dialogs
rfd = "0.15"

# Internal crates
imortal_core = { path = "crates/core" }
imortal_ir = { path = "crates/ir" }
imortal_codegen = { path = "crates/codegen" }
imortal_ui = { path = "crates/ui" }
EOF
```

### Step 2: Create Crate Structure

```bash
# Create all crate directories
mkdir -p crates/{core,ir,codegen,ui,cli}/src
mkdir -p assets/{styles,icons,fonts}
mkdir -p templates/{rest-api,fullstack}
mkdir -p tests

# Initialize each crate
for crate in core ir codegen ui cli; do
    touch crates/$crate/src/lib.rs
done
touch crates/cli/src/main.rs
```

### Step 3: Create Core Crate

```bash
cat > crates/core/Cargo.toml << 'EOF'
[package]
name = "imortal_core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
EOF
```

### Step 4: Create IR Crate

```bash
cat > crates/ir/Cargo.toml << 'EOF'
[package]
name = "imortal_ir"
version.workspace = true
edition.workspace = true

[dependencies]
imortal_core = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
EOF
```

### Step 5: Create Codegen Crate

```bash
cat > crates/codegen/Cargo.toml << 'EOF'
[package]
name = "imortal_codegen"
version.workspace = true
edition.workspace = true

[dependencies]
imortal_core = { workspace = true }
imortal_ir = { workspace = true }
quote = { workspace = true }
syn = { workspace = true }
proc-macro2 = { workspace = true }
heck = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true }
EOF
```

### Step 6: Create UI Crate

```bash
cat > crates/ui/Cargo.toml << 'EOF'
[package]
name = "imortal_ui"
version.workspace = true
edition.workspace = true

[dependencies]
imortal_core = { workspace = true }
imortal_ir = { workspace = true }
imortal_codegen = { workspace = true }
dioxus = { workspace = true }
rfd = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
EOF
```

### Step 7: Setup Tailwind CSS

```bash
# Install Tailwind CSS
npm init -y
npm install -D tailwindcss

# Create Tailwind config
cat > tailwind.config.js << 'EOF'
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./crates/ui/src/**/*.rs",
    "./src/**/*.rs",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eef2ff',
          100: '#e0e7ff',
          500: '#6366f1',
          600: '#4f46e5',
          700: '#4338ca',
        },
      },
    },
  },
  plugins: [],
}
EOF

# Create source CSS
cat > assets/styles/tailwind.css << 'EOF'
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Entity Card */
.entity-card {
  @apply bg-white dark:bg-slate-800 
         rounded-xl shadow-lg 
         border-2 border-slate-200 dark:border-slate-700
         min-w-[220px] 
         transition-all duration-200;
}

.entity-card:hover {
  @apply shadow-xl border-primary-300;
}

.entity-card.selected {
  @apply border-primary-500 ring-2 ring-primary-200;
}

/* Field Badges */
.field-type-badge {
  @apply text-xs px-2 py-0.5 rounded-full
         bg-blue-100 text-blue-700
         dark:bg-blue-900 dark:text-blue-300;
}

/* HTTP Method Badges */
.method-get { @apply bg-green-100 text-green-700 px-2 py-1 rounded text-xs font-bold; }
.method-post { @apply bg-blue-100 text-blue-700 px-2 py-1 rounded text-xs font-bold; }
.method-put { @apply bg-amber-100 text-amber-700 px-2 py-1 rounded text-xs font-bold; }
.method-delete { @apply bg-rose-100 text-rose-700 px-2 py-1 rounded text-xs font-bold; }
EOF

# Build CSS
npx tailwindcss -i ./assets/styles/tailwind.css -o ./assets/styles/main.css
```

---

## 📁 Implementation Order

### Phase 1: Core Types (Priority: HIGH)

```
crates/core/src/
├── lib.rs           # Module exports
├── types.rs         # DataType, Position, Size, Rect
├── error.rs         # EngineError, EngineResult
└── traits.rs        # Validatable, CodeGenerable
```

**Files to implement first:**
1. `types.rs` - All fundamental types
2. `error.rs` - Error handling
3. `traits.rs` - Core traits
4. `lib.rs` - Re-exports

### Phase 2: Intermediate Representation (Priority: HIGH)

```
crates/ir/src/
├── lib.rs           # Module exports
├── project.rs       # ProjectGraph, ProjectMeta, ProjectConfig
├── entity.rs        # Entity, EntityConfig
├── field.rs         # Field, FieldValidation, UiHints
├── relationship.rs  # Relationship, RelationType
├── endpoint.rs      # EndpointGroup, CrudOperation
├── validation.rs    # Schema validation
└── serialization.rs # Save/Load JSON
```

**Files to implement in order:**
1. `field.rs` - Fields depend only on core types
2. `entity.rs` - Entities contain fields
3. `relationship.rs` - Relationships connect entities
4. `endpoint.rs` - Endpoints reference entities
5. `project.rs` - Project contains everything
6. `validation.rs` - Validates the graph
7. `serialization.rs` - Persistence

### Phase 3: UI Foundation (Priority: HIGH)

```
crates/ui/src/
├── lib.rs
├── app.rs           # Main App component
├── state.rs         # AppState with Signals
├── theme.rs         # Dark/Light mode
├── components/
│   ├── mod.rs
│   ├── sidebar.rs
│   └── toolbar.rs
└── pages/
    ├── mod.rs
    └── welcome.rs
```

**Implement in order:**
1. `state.rs` - Application state structure
2. `app.rs` - Main app shell
3. `welcome.rs` - First visible page
4. `sidebar.rs` - Navigation
5. `toolbar.rs` - Top actions

### Phase 4: Canvas & Visual Editor (Priority: HIGH)

```
crates/ui/src/
├── components/
│   ├── canvas.rs        # Main canvas with pan/zoom
│   ├── entity_card.rs   # Entity visual component
│   ├── field_row.rs     # Field display
│   ├── port.rs          # Connection ports
│   └── connection.rs    # Relationship lines (SVG)
└── pages/
    └── entity_design.rs # Entity design page
```

### Phase 5: Code Generation (Priority: HIGH)

```
crates/codegen/src/
├── lib.rs
├── generator.rs     # Main orchestrator
├── context.rs       # Generation context
├── rust/
│   ├── mod.rs
│   ├── cargo.rs     # Cargo.toml generation
│   ├── models.rs    # SeaORM models
│   ├── handlers.rs  # Axum handlers
│   ├── routes.rs    # Router setup
│   ├── auth.rs      # JWT auth (if enabled)
│   └── main.rs      # main.rs generation
└── migrations/
    ├── mod.rs
    └── sql.rs       # SQL migrations
```

### Phase 6: Properties & Configuration (Priority: MEDIUM)

```
crates/ui/src/
├── components/
│   ├── properties.rs     # Properties panel
│   ├── dialogs/
│   │   ├── mod.rs
│   │   ├── new_project.rs
│   │   ├── field_editor.rs
│   │   └── export.rs
│   └── inputs/
│       ├── mod.rs
│       ├── text.rs
│       ├── select.rs
│       └── checkbox.rs
└── pages/
    ├── project_setup.rs
    ├── endpoints.rs
    └── generation.rs
```

### Phase 7: Fullstack Generation (Priority: MEDIUM)

```
crates/codegen/src/
└── frontend/
    ├── mod.rs
    ├── dioxus.rs     # Component generation
    ├── pages.rs      # Page generation
    └── api_client.rs # API client generation
```

---

## ✅ Implementation Checklist

### Core Crate (`imortal_core`)

- [ ] `DataType` enum with all variants
- [ ] `Position`, `Size`, `Rect` structs
- [ ] `EngineError` enum
- [ ] `Validatable` trait
- [ ] `CodeGenerable` trait
- [ ] Unit tests for all types

### IR Crate (`imortal_ir`)

- [ ] `Field` struct with validations
- [ ] `Entity` struct with CRUD methods
- [ ] `Relationship` struct
- [ ] `EndpointGroup` and `CrudOperation`
- [ ] `ProjectGraph` container
- [ ] `ProjectMeta` and `ProjectConfig`
- [ ] JSON serialization/deserialization
- [ ] Project validation rules
- [ ] Unit tests for all structures

### UI Crate (`imortal_ui`)

- [ ] `AppState` with Dioxus Signals
- [ ] Main `App` component
- [ ] `Sidebar` component
- [ ] `Toolbar` component
- [ ] `Canvas` component with pan/zoom
- [ ] `EntityCard` component
- [ ] `FieldRow` component
- [ ] `Port` component
- [ ] `ConnectionLine` SVG component
- [ ] `PropertiesPanel` component
- [ ] Welcome page
- [ ] Project setup page
- [ ] Entity design page
- [ ] Endpoint configuration page
- [ ] Code generation page
- [ ] File dialogs (open/save/export)
- [ ] Keyboard shortcuts

### Codegen Crate (`imortal_codegen`)

- [ ] `GenerationContext` struct
- [ ] `CodeGenerator` orchestrator
- [ ] Cargo.toml generation
- [ ] SeaORM model generation
- [ ] DTO generation (Create, Update, Response)
- [ ] Axum handler generation
- [ ] Router generation
- [ ] JWT auth generation
- [ ] SQL migration generation
- [ ] main.rs generation
- [ ] .env.example generation
- [ ] README.md generation
- [ ] Dioxus frontend generation (fullstack)
- [ ] Integration tests (generated code compiles)

---

## 🧪 Testing Commands

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p imortal_core
cargo test -p imortal_ir
cargo test -p imortal_codegen

# Run with verbose output
cargo test --workspace -- --nocapture

# Check code compiles
cargo check --workspace

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace
```

---

## 🏃 Running the Application

```bash
# Development mode
cargo run

# Release mode
cargo run --release

# With logging
RUST_LOG=debug cargo run
```

---

## 📊 Key Metrics to Track

| Metric | Target |
|--------|--------|
| Entity creation time | < 100ms |
| Canvas render (50 entities) | 60 FPS |
| Code generation (10 entities) | < 5 seconds |
| Project file save | < 500ms |
| Test coverage | > 80% |

---

## 🔗 Quick Links

- [Comprehensive Specification](./COMPREHENSIVE_SPECIFICATION.md)
- [Implementation Plan](./IMPLEMENTATION_PLAN.md)
- [Dioxus Documentation](https://dioxuslabs.com/docs/0.6/)
- [Axum Documentation](https://docs.rs/axum)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)

---

*Ready to build? Start with Phase 1: Core Types!*