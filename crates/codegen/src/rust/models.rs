//! # Model Generator (SeaORM)
//!
//! Generates SeaORM entity definitions and associated DTOs for every entity
//! in the project.
//!
//! ## Generated Files
//!
//! For each entity (e.g. `User`), this generator produces:
//!
//! - `src/models/mod.rs` — module declarations and re-exports
//! - `src/models/user.rs` — SeaORM entity with:
//!   - `Model` struct (`DeriveEntityModel`)
//!   - `Relation` enum (`DeriveRelation`)
//!   - `Related<…>` implementations
//!   - `ActiveModelBehavior` implementation
//!   - `CreateUserDto` — fields for creation, with `validator` derives
//!   - `UpdateUserDto` — optional fields for partial update
//!   - `UserResponse` — safe output DTO (excludes secrets)
//!   - `impl From<Model> for UserResponse`
//!
//! ## Type Mapping
//!
//! The generator maps `DataType` variants to Rust types, SeaORM column
//! attributes, and `validator` annotations based on the field's configuration.

use imortal_core::{DataType, IdType, RelationType, Validation};
use imortal_ir::Entity;

use crate::context::{EntityInfo, GenerationContext};
use crate::rust::{doc_comment, file_header};
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate all model files (`src/models/mod.rs` + one file per entity).
pub fn generate_models(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    let mut files = Vec::new();

    // models/mod.rs
    files.push(generate_models_mod(ctx));

    // Per-entity files
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        files.push(generate_entity_model(&info, ctx));
    }

    files
}

// ============================================================================
// models/mod.rs
// ============================================================================

fn generate_models_mod(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(512);

    content.push_str(&file_header(
        "Model definitions (SeaORM entities and DTOs).",
    ));

    for entity in ctx.entities() {
        let module = GenerationContext::module_name(&entity.name);
        content.push_str(&format!("pub mod {};\n", module));
    }

    if !ctx.entities().is_empty() {
        content.push('\n');
        content.push_str("// Re-exports for convenience\n");

        for entity in ctx.entities() {
            let module = GenerationContext::module_name(&entity.name);
            let pascal = GenerationContext::pascal(&entity.name);
            let create_dto = GenerationContext::create_dto_name(&entity.name);
            let update_dto = GenerationContext::update_dto_name(&entity.name);
            let response_dto = GenerationContext::response_dto_name(&entity.name);

            content.push_str(&format!(
                "pub use {module}::{{Model as {pascal}Model, {create_dto}, {update_dto}, {response_dto}}};\n"
            ));
        }
    }

    GeneratedFile::new("src/models/mod.rs", content, FileType::Rust)
}

// ============================================================================
// Per-entity model file
// ============================================================================

fn generate_entity_model(info: &EntityInfo, ctx: &GenerationContext) -> GeneratedFile {
    let module = info.module_name();
    let path = format!("src/models/{}.rs", module);

    let mut content = String::with_capacity(4096);

    // File header
    content.push_str(&file_header(&format!(
        "{} model — SeaORM entity and DTOs.",
        info.pascal_name()
    )));

    // Imports
    content.push_str(&generate_imports(info, ctx));
    content.push('\n');

    // SeaORM Model
    content.push_str(&generate_model_struct(info, ctx));
    content.push('\n');

    // Relation enum
    content.push_str(&generate_relation_enum(info, ctx));
    content.push('\n');

    // Related implementations
    content.push_str(&generate_related_impls(info, ctx));

    // ActiveModelBehavior
    content.push_str("impl ActiveModelBehavior for ActiveModel {}\n\n");

    // Separator
    content.push_str(
        "// ============================================================================\n",
    );
    content.push_str("// DTOs (Data Transfer Objects)\n");
    content.push_str(
        "// ============================================================================\n\n",
    );

    // Create DTO
    content.push_str(&generate_create_dto(info, ctx));
    content.push('\n');

    // Update DTO
    content.push_str(&generate_update_dto(info, ctx));
    content.push('\n');

    // Response DTO
    content.push_str(&generate_response_dto(info, ctx));
    content.push('\n');

    // From<Model> for Response
    content.push_str(&generate_from_model(info, ctx));

    // Pagination params (only in first entity file to avoid duplication;
    // ideally this goes into a shared module, but we keep it simple here)
    // We skip it here and put it in the handlers or a shared module.

    GeneratedFile::new(path, content, FileType::Rust)
}

// ============================================================================
// Imports
// ============================================================================

fn generate_imports(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let mut out = String::with_capacity(512);

    out.push_str("use sea_orm::entity::prelude::*;\n");
    out.push_str("use serde::{Deserialize, Serialize};\n");
    out.push_str("use validator::Validate;\n");

    // Check if we need uuid
    let needs_uuid = info
        .entity
        .fields
        .iter()
        .any(|f| matches!(f.data_type, DataType::Uuid) || f.is_primary_key);
    if needs_uuid {
        out.push_str("use uuid::Uuid;\n");
    }

    // Check if we need chrono
    let needs_chrono = info.entity.fields.iter().any(|f| {
        matches!(
            f.data_type,
            DataType::DateTime | DataType::Date | DataType::Time
        )
    }) || info.has_timestamps();
    if needs_chrono {
        out.push_str("use chrono::{DateTime, Utc};\n");
    }

    out.push('\n');
    out
}

// ============================================================================
// SeaORM Model struct
// ============================================================================

fn generate_model_struct(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let table = info.table_name();
    let mut out = String::with_capacity(2048);

    // Doc comment
    out.push_str(&doc_comment(info.entity.description.as_deref(), ctx));

    // Derive block
    out.push_str("#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]\n");
    out.push_str(&format!("#[sea_orm(table_name = \"{}\")]\n", table));
    out.push_str("pub struct Model {\n");

    for field in &info.entity.fields {
        let col_name = if field.column_name.is_empty() {
            GenerationContext::snake(&field.name)
        } else {
            field.column_name.clone()
        };
        let rust_name = GenerationContext::snake(&field.name);
        let rust_type = field_rust_type(field, info);

        // SeaORM column attributes
        let mut attrs = Vec::new();

        if field.is_primary_key {
            match info.id_type() {
                IdType::Serial => attrs.push("primary_key".to_string()),
                _ => attrs.push("primary_key, auto_increment = false".to_string()),
            }
        }

        if col_name != rust_name {
            attrs.push(format!("column_name = \"{}\"", col_name));
        }

        if field.unique && !field.is_primary_key {
            attrs.push("unique".to_string());
        }

        // Field-level doc comment
        if let Some(desc) = &field.description {
            out.push_str(&format!("    /// {}\n", desc));
        }

        // SeaORM attribute
        if !attrs.is_empty() {
            out.push_str(&format!("    #[sea_orm({})]\n", attrs.join(", ")));
        }

        // Skip serializing secrets
        if field.secret {
            out.push_str("    #[serde(skip_serializing)]\n");
        }

        out.push_str(&format!("    pub {}: {},\n", rust_name, rust_type));

        // Blank line between fields for readability
        if field.is_primary_key {
            out.push('\n');
        }
    }

    // Add timestamp fields if enabled and not already present
    if info.has_timestamps() {
        if !info.entity.fields.iter().any(|f| f.name == "created_at") {
            out.push_str("    pub created_at: DateTimeUtc,\n");
        }
        if !info.entity.fields.iter().any(|f| f.name == "updated_at") {
            out.push_str("    pub updated_at: DateTimeUtc,\n");
        }
    }

    // Soft-delete field
    if info.has_soft_delete() {
        if !info.entity.fields.iter().any(|f| f.name == "deleted_at") {
            out.push_str("    pub deleted_at: Option<DateTimeUtc>,\n");
        }
    }

    out.push_str("}\n\n");
    out
}

// ============================================================================
// Relation enum
// ============================================================================

fn generate_relation_enum(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let mut out = String::with_capacity(1024);

    let outgoing = info.outgoing_relationships();
    let incoming = info.incoming_relationships();

    out.push_str("#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]\n");
    out.push_str("pub enum Relation {\n");

    // Outgoing relationships (has_many, has_one)
    for rel in &outgoing {
        let target_entity = ctx.entity_by_id(rel.to_entity_id);
        let target_module = target_entity
            .map(|e| GenerationContext::module_name(&e.name))
            .unwrap_or_else(|| "unknown".to_string());
        let target_pascal = target_entity
            .map(|e| GenerationContext::pascal(&e.name))
            .unwrap_or_else(|| "Unknown".to_string());

        let variant_name = GenerationContext::pascal(&rel.name);
        let variant_name = if variant_name.is_empty() {
            target_pascal.clone()
        } else {
            variant_name
        };

        let relation_attr = match &rel.relation_type {
            RelationType::OneToOne => {
                format!("#[sea_orm(has_one = \"super::{}::Entity\")]", target_module)
            }
            RelationType::OneToMany => {
                format!(
                    "#[sea_orm(has_many = \"super::{}::Entity\")]",
                    target_module
                )
            }
            _ => {
                // ManyToOne and ManyToMany are handled differently
                // ManyToOne is represented as a belongs_to on the other side
                format!(
                    "#[sea_orm(has_many = \"super::{}::Entity\")]",
                    target_module
                )
            }
        };

        out.push_str(&format!("    {}\n", relation_attr));
        out.push_str(&format!("    {},\n", variant_name));
    }

    // Incoming relationships (belongs_to)
    for rel in &incoming {
        let source_entity = ctx.entity_by_id(rel.from_entity_id);
        let source_module = source_entity
            .map(|e| GenerationContext::module_name(&e.name))
            .unwrap_or_else(|| "unknown".to_string());

        // Skip if already covered by outgoing
        let is_already_covered = outgoing
            .iter()
            .any(|r| r.from_entity_id == rel.from_entity_id);
        if is_already_covered {
            continue;
        }

        // For ManyToOne, the current entity has an FK field → belongs_to
        if matches!(rel.relation_type, RelationType::OneToMany) {
            // Find the FK field on this entity
            let fk_field = info.entity.fields.iter().find(|f| {
                f.is_foreign_key
                    && f.foreign_key_ref
                        .as_ref()
                        .map(|fk| fk.entity_id == rel.from_entity_id)
                        .unwrap_or(false)
            });

            let from_col = fk_field
                .map(|f| GenerationContext::pascal(&f.name))
                .unwrap_or_else(|| format!("{}Id", GenerationContext::pascal(&source_module)));

            let variant_name = source_entity
                .map(|e| GenerationContext::pascal(&e.name))
                .unwrap_or_else(|| "Unknown".to_string());

            out.push_str(&format!(
                "    #[sea_orm(\n        belongs_to = \"super::{}::Entity\",\n        from = \"Column::{}\",\n        to = \"super::{}::Column::Id\"\n    )]\n",
                source_module, from_col, source_module,
            ));
            out.push_str(&format!("    {},\n", variant_name));
        }
    }

    out.push_str("}\n\n");
    out
}

// ============================================================================
// Related<…> implementations
// ============================================================================

fn generate_related_impls(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let mut out = String::new();

    let outgoing = info.outgoing_relationships();

    for rel in &outgoing {
        let target_entity = ctx.entity_by_id(rel.to_entity_id);
        let target_module = target_entity
            .map(|e| GenerationContext::module_name(&e.name))
            .unwrap_or_else(|| "unknown".to_string());

        let variant_name = GenerationContext::pascal(&rel.name);
        let variant_name = if variant_name.is_empty() {
            target_entity
                .map(|e| GenerationContext::pascal(&e.name))
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            variant_name
        };

        out.push_str(&format!(
            "impl Related<super::{}::Entity> for Entity {{\n",
            target_module
        ));
        out.push_str(&format!(
            "    fn to() -> RelationDef {{\n        Relation::{}.def()\n    }}\n",
            variant_name
        ));
        out.push_str("}\n\n");
    }

    out
}

// ============================================================================
// Create DTO
// ============================================================================

fn generate_create_dto(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let name = GenerationContext::create_dto_name(&info.entity.name);
    let fields = info.create_fields();

    let mut out = String::with_capacity(1024);

    out.push_str(&doc_comment(
        Some(&format!(
            "Payload for creating a new {}.",
            info.pascal_name()
        )),
        ctx,
    ));

    out.push_str("#[derive(Debug, Clone, Deserialize, Validate)]\n");
    out.push_str(&format!("pub struct {} {{\n", name));

    for field in &fields {
        let rust_name = GenerationContext::snake(&field.name);
        let rust_type = field_rust_type_dto(field, false);

        // For password/secret fields: the DTO accepts plain text from the user
        // (e.g. "password" instead of "password_hash"). The handler is
        // responsible for hashing before storage.
        let is_password_field =
            field.secret || field.name.contains("password") || field.name.contains("secret");

        let dto_field_name = if is_password_field && rust_name.ends_with("_hash") {
            // password_hash → password (user sends plain text, handler hashes)
            rust_name.trim_end_matches("_hash").to_string()
        } else {
            rust_name.clone()
        };

        // Validation attributes
        let validators = generate_validator_attrs(field);
        for attr in &validators {
            out.push_str(&format!("    {}\n", attr));
        }

        if dto_field_name != rust_name {
            out.push_str(&format!(
                "    /// Plain-text value — will be hashed before storage.\n"
            ));
        }

        out.push_str(&format!("    pub {}: {},\n", dto_field_name, rust_type));
    }

    out.push_str("}\n");
    out
}

// ============================================================================
// Update DTO
// ============================================================================

fn generate_update_dto(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let name = GenerationContext::update_dto_name(&info.entity.name);
    let fields = info.update_fields();

    let mut out = String::with_capacity(1024);

    out.push_str(&doc_comment(
        Some(&format!(
            "Payload for updating an existing {}. All fields are optional.",
            info.pascal_name()
        )),
        ctx,
    ));

    out.push_str("#[derive(Debug, Clone, Deserialize, Validate)]\n");
    out.push_str(&format!("pub struct {} {{\n", name));

    for field in &fields {
        let rust_name = GenerationContext::snake(&field.name);
        let inner_type = field_rust_type_dto(field, false);

        // Wrap in Option for partial updates
        let rust_type = format!("Option<{}>", inner_type);

        // Validation (skip_if = None is implicit for Option fields in validator)
        let validators = generate_validator_attrs(field);
        for attr in &validators {
            out.push_str(&format!("    {}\n", attr));
        }

        out.push_str(&format!("    pub {}: {},\n", rust_name, rust_type));
    }

    out.push_str("}\n");
    out
}

// ============================================================================
// Response DTO
// ============================================================================

fn generate_response_dto(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let name = GenerationContext::response_dto_name(&info.entity.name);
    let fields = info.response_fields();

    let mut out = String::with_capacity(1024);

    out.push_str(&doc_comment(
        Some(&format!(
            "Response representation of a {}. Excludes sensitive fields.",
            info.pascal_name()
        )),
        ctx,
    ));

    out.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {} {{\n", name));

    for field in &fields {
        let rust_name = GenerationContext::snake(&field.name);
        let rust_type = field_rust_type(field, info);

        out.push_str(&format!("    pub {}: {},\n", rust_name, rust_type));
    }

    // Timestamps (if enabled and not already in fields)
    if info.has_timestamps() {
        if !fields.iter().any(|f| f.name == "created_at") {
            out.push_str("    pub created_at: DateTime<Utc>,\n");
        }
        if !fields.iter().any(|f| f.name == "updated_at") {
            out.push_str("    pub updated_at: DateTime<Utc>,\n");
        }
    }

    out.push_str("}\n");
    out
}

// ============================================================================
// From<Model> for Response
// ============================================================================

fn generate_from_model(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let response_name = GenerationContext::response_dto_name(&info.entity.name);
    let fields = info.response_fields();

    let mut out = String::with_capacity(512);

    out.push_str(&format!("impl From<Model> for {} {{\n", response_name));
    out.push_str("    fn from(model: Model) -> Self {\n");
    out.push_str("        Self {\n");

    for field in &fields {
        let rust_name = GenerationContext::snake(&field.name);
        out.push_str(&format!(
            "            {name}: model.{name},\n",
            name = rust_name
        ));
    }

    // Timestamps
    if info.has_timestamps() {
        if !fields.iter().any(|f| f.name == "created_at") {
            out.push_str("            created_at: model.created_at,\n");
        }
        if !fields.iter().any(|f| f.name == "updated_at") {
            out.push_str("            updated_at: model.updated_at,\n");
        }
    }

    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

// ============================================================================
// Validator attribute generation
// ============================================================================

/// Generate `#[validate(…)]` attributes for a field based on its validations.
fn generate_validator_attrs(field: &imortal_ir::Field) -> Vec<String> {
    let mut attrs = Vec::new();

    for validation in &field.validations {
        let attr = match validation {
            Validation::Required => None, // handled by type being non-Option
            Validation::MinLength(n) => Some(format!("#[validate(length(min = {}))]", n)),
            Validation::MaxLength(n) => Some(format!("#[validate(length(max = {}))]", n)),
            Validation::Min(n) => Some(format!("#[validate(range(min = {}))]", n)),
            Validation::Max(n) => Some(format!("#[validate(range(max = {}))]", n)),
            Validation::Pattern { regex: _, message } => {
                if message.is_empty() {
                    Some(format!(
                        "#[validate(regex(path = \"RE_{}\"))]",
                        GenerationContext::snake(&field.name).to_uppercase(),
                    ))
                } else {
                    Some(format!(
                        "#[validate(regex(path = \"RE_{}\", message = \"{}\"))]",
                        GenerationContext::snake(&field.name).to_uppercase(),
                        message,
                    ))
                }
            }
            Validation::Email => Some("#[validate(email)]".to_string()),
            Validation::Url => Some("#[validate(url)]".to_string()),
            Validation::Uuid => None, // UUID validation is handled by the type system
            Validation::Phone => Some("#[validate(phone)]".to_string()),
            Validation::OneOf(values) => {
                // validator doesn't have a built-in OneOf; use custom
                Some(format!(
                    "#[validate(custom(function = \"validate_one_of\"))] // allowed: {:?}",
                    values,
                ))
            }
            Validation::Custom { name, .. } => {
                Some(format!("#[validate(custom(function = \"{}\"))]", name))
            }
        };

        if let Some(a) = attr {
            attrs.push(a);
        }
    }

    // Combine multiple length validations into one if both min and max exist
    let has_min_length = attrs.iter().any(|a| a.contains("length(min"));
    let has_max_length = attrs.iter().any(|a| a.contains("length(max"));

    if has_min_length && has_max_length {
        let mut min_val: usize = 0;
        let mut max_val: usize = 0;

        // Extract values
        for v in &field.validations {
            if let Validation::MinLength(n) = v {
                min_val = *n;
            }
            if let Validation::MaxLength(n) = v {
                max_val = *n;
            }
        }

        // Remove individual length attrs and add combined
        attrs.retain(|a| !a.contains("length("));
        attrs.push(format!(
            "#[validate(length(min = {}, max = {}))]",
            min_val, max_val
        ));
    }

    attrs
}

// ============================================================================
// Rust type helpers
// ============================================================================

/// Get the Rust type for a field as it appears in the SeaORM `Model` struct.
fn field_rust_type(field: &imortal_ir::Field, info: &EntityInfo) -> String {
    if field.is_primary_key {
        return GenerationContext::pk_rust_type(info.id_type()).to_string();
    }

    data_type_to_rust(&field.data_type)
}

/// Get the Rust type for a DTO field.
/// - For create DTOs: required fields are non-Option, optional are Option
/// - For update DTOs: all fields are wrapped in Option by the caller
fn field_rust_type_dto(field: &imortal_ir::Field, _is_update: bool) -> String {
    if field.is_primary_key {
        // PKs are usually not in DTOs, but if they are, use the raw type
        return data_type_to_rust(&field.data_type);
    }

    let base = data_type_to_rust(&field.data_type);

    // If the field is optional (not required), wrap in Option
    if !field.required {
        format!("Option<{}>", base)
    } else {
        base
    }
}

/// Map a `DataType` to a Rust type string for generated code.
fn data_type_to_rust(dt: &DataType) -> String {
    match dt {
        DataType::String | DataType::Text => "String".to_string(),
        DataType::Int32 => "i32".to_string(),
        DataType::Int64 => "i64".to_string(),
        DataType::Float32 => "f32".to_string(),
        DataType::Float64 => "f64".to_string(),
        DataType::Bool => "bool".to_string(),
        DataType::Uuid => "Uuid".to_string(),
        DataType::DateTime => "DateTime<Utc>".to_string(),
        DataType::Date => "chrono::NaiveDate".to_string(),
        DataType::Time => "chrono::NaiveTime".to_string(),
        DataType::Bytes => "Vec<u8>".to_string(),
        DataType::Json => "serde_json::Value".to_string(),
        DataType::Optional(inner) => format!("Option<{}>", data_type_to_rust(inner)),
        DataType::Array(inner) => format!("Vec<{}>", data_type_to_rust(inner)),
        DataType::Reference { .. } => "Uuid".to_string(),
        DataType::Enum { name, .. } => name.clone(),
    }
}

// ============================================================================
// Pagination helper struct (generated once, used by handlers)
// ============================================================================

/// Generate a shared pagination types module.
/// This is called from the handlers module but defined here for proximity.
pub fn generate_pagination_types() -> String {
    r#"
/// Query parameters for paginated list endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-based). Defaults to 1.
    pub page: Option<u64>,
    /// Items per page. Defaults to 20, max 100.
    pub per_page: Option<u64>,
}

/// Paginated response wrapper.
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    /// The items in this page.
    pub items: Vec<T>,
    /// Total number of items across all pages.
    pub total: u64,
    /// Current page number (1-based).
    pub page: u64,
    /// Items per page.
    pub per_page: u64,
    /// Total number of pages.
    pub total_pages: u64,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Create a new paginated response.
    pub fn new(items: Vec<T>, total: u64, page: u64, per_page: u64) -> Self {
        let total_pages = if per_page > 0 {
            (total + per_page - 1) / per_page
        } else {
            0
        };
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}
"#
    .to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_core::DataType;
    use imortal_ir::{Entity, EntityConfig, Field, ProjectGraph};
    use uuid::Uuid;

    /// Create a simple User entity for testing.
    fn make_user_entity() -> Entity {
        let mut entity = Entity::new("User");
        entity.config.timestamps = true;

        let mut email = Field::new("email", DataType::String);
        email.required = true;
        email.unique = true;
        email.validations.push(Validation::Email);
        entity.fields.push(email);

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        name.validations.push(Validation::MinLength(1));
        name.validations.push(Validation::MaxLength(100));
        entity.fields.push(name);

        let mut password_hash = Field::new("password_hash", DataType::String);
        password_hash.required = true;
        password_hash.secret = true;
        entity.fields.push(password_hash);

        entity
    }

    fn make_post_entity() -> Entity {
        let mut entity = Entity::new("Post");
        entity.config.timestamps = true;

        let mut title = Field::new("title", DataType::String);
        title.required = true;
        entity.fields.push(title);

        let content = Field::new("content", DataType::Optional(Box::new(DataType::Text)));
        entity.fields.push(content);

        entity
    }

    #[test]
    fn test_generate_models_empty_project() {
        let project = ProjectGraph::new("test");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_models(&ctx);

        // Should still produce mod.rs
        assert_eq!(files.len(), 1);
        assert!(files[0].path.to_string_lossy().contains("mod.rs"));
    }

    #[test]
    fn test_generate_models_with_entities() {
        let mut project = ProjectGraph::new("blog");
        project.add_entity(make_user_entity());
        project.add_entity(make_post_entity());

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_models(&ctx);

        // mod.rs + user.rs + post.rs = 3
        assert_eq!(files.len(), 3);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();
        assert!(paths.iter().any(|p| p.contains("mod.rs")));
        assert!(paths.iter().any(|p| p.contains("user.rs")));
        assert!(paths.iter().any(|p| p.contains("post.rs")));
    }

    #[test]
    fn test_model_struct_contains_sea_orm_attrs() {
        let mut project = ProjectGraph::new("test");
        project.add_entity(make_user_entity());

        let ctx = GenerationContext::from_project_default(&project);
        let entity = ctx.entities().first().unwrap();
        let info = EntityInfo::new(entity, &ctx);

        let model = generate_model_struct(&info, &ctx);

        assert!(model.contains("DeriveEntityModel"));
        assert!(model.contains("table_name = \"users\""));
        assert!(model.contains("pub struct Model"));
        assert!(model.contains("primary_key"));
        assert!(model.contains("pub email: String"));
        assert!(model.contains("pub name: String"));
        assert!(model.contains("pub password_hash: String"));
        assert!(model.contains("serde(skip_serializing)"));
    }

    #[test]
    fn test_model_struct_timestamps() {
        let mut project = ProjectGraph::new("test");
        let mut entity = Entity::new("Item");
        entity.config.timestamps = true;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);
        project.add_entity(entity);

        let ctx = GenerationContext::from_project_default(&project);
        let e = ctx.entities().first().unwrap();
        let info = EntityInfo::new(e, &ctx);

        let model = generate_model_struct(&info, &ctx);
        assert!(model.contains("created_at"));
        assert!(model.contains("updated_at"));
    }

    #[test]
    fn test_model_struct_soft_delete() {
        let mut project = ProjectGraph::new("test");
        let mut entity = Entity::new("Post");
        entity.config.soft_delete = true;
        entity.config.timestamps = false;
        let mut title = Field::new("title", DataType::String);
        title.required = true;
        entity.fields.push(title);
        project.add_entity(entity);

        let ctx = GenerationContext::from_project_default(&project);
        let e = ctx.entities().first().unwrap();
        let info = EntityInfo::new(e, &ctx);

        let model = generate_model_struct(&info, &ctx);
        assert!(model.contains("deleted_at"));
        assert!(model.contains("Option<DateTimeUtc>"));
    }

    #[test]
    fn test_create_dto() {
        let mut project = ProjectGraph::new("test");
        project.add_entity(make_user_entity());

        let ctx = GenerationContext::from_project_default(&project);
        let entity = ctx.entities().first().unwrap();
        let info = EntityInfo::new(entity, &ctx);

        let dto = generate_create_dto(&info, &ctx);

        assert!(dto.contains("pub struct CreateUserDto"));
        assert!(dto.contains("Validate"));
        assert!(dto.contains("pub email: String"));
        assert!(dto.contains("pub name: String"));
        // password_hash → password in Create DTO (user sends plain text, handler hashes)
        assert!(
            dto.contains("pub password: String"),
            "password_hash should be renamed to password in Create DTO"
        );
        assert!(
            !dto.contains("pub password_hash:"),
            "Create DTO should NOT expose password_hash directly"
        );
        assert!(
            dto.contains("Plain-text value"),
            "Should have doc comment explaining hashing"
        );
        assert!(dto.contains("#[validate(email)]"));
        assert!(dto.contains("#[validate(length(min = 1, max = 100))]"));
    }

    #[test]
    fn test_update_dto() {
        let mut project = ProjectGraph::new("test");
        project.add_entity(make_user_entity());

        let ctx = GenerationContext::from_project_default(&project);
        let entity = ctx.entities().first().unwrap();
        let info = EntityInfo::new(entity, &ctx);

        let dto = generate_update_dto(&info, &ctx);

        assert!(dto.contains("pub struct UpdateUserDto"));
        assert!(dto.contains("Option<"));
        // password_hash is secret, should NOT be in update DTO
        assert!(!dto.contains("password_hash"));
    }

    #[test]
    fn test_response_dto() {
        let mut project = ProjectGraph::new("test");
        project.add_entity(make_user_entity());

        let ctx = GenerationContext::from_project_default(&project);
        let entity = ctx.entities().first().unwrap();
        let info = EntityInfo::new(entity, &ctx);

        let dto = generate_response_dto(&info, &ctx);

        assert!(dto.contains("pub struct UserResponse"));
        assert!(dto.contains("pub email: String"));
        assert!(dto.contains("pub name: String"));
        // password_hash is secret, should NOT be in response
        assert!(!dto.contains("password_hash"));
        // Should have timestamps
        assert!(dto.contains("created_at"));
        assert!(dto.contains("updated_at"));
    }

    #[test]
    fn test_from_model_impl() {
        let mut project = ProjectGraph::new("test");
        project.add_entity(make_user_entity());

        let ctx = GenerationContext::from_project_default(&project);
        let entity = ctx.entities().first().unwrap();
        let info = EntityInfo::new(entity, &ctx);

        let from_impl = generate_from_model(&info, &ctx);

        assert!(from_impl.contains("impl From<Model> for UserResponse"));
        assert!(from_impl.contains("model.email"));
        assert!(from_impl.contains("model.name"));
        assert!(!from_impl.contains("model.password_hash"));
    }

    #[test]
    fn test_models_mod_rs() {
        let mut project = ProjectGraph::new("test");
        project.add_entity(make_user_entity());
        project.add_entity(make_post_entity());

        let ctx = GenerationContext::from_project_default(&project);
        let file = generate_models_mod(&ctx);

        let content = &file.content;
        assert!(content.contains("pub mod user;"));
        assert!(content.contains("pub mod post;"));
        assert!(content.contains("CreateUserDto"));
        assert!(content.contains("UserResponse"));
        assert!(content.contains("CreatePostDto"));
        assert!(content.contains("PostResponse"));
    }

    #[test]
    fn test_data_type_to_rust() {
        assert_eq!(data_type_to_rust(&DataType::String), "String");
        assert_eq!(data_type_to_rust(&DataType::Int32), "i32");
        assert_eq!(data_type_to_rust(&DataType::Int64), "i64");
        assert_eq!(data_type_to_rust(&DataType::Float64), "f64");
        assert_eq!(data_type_to_rust(&DataType::Bool), "bool");
        assert_eq!(data_type_to_rust(&DataType::Uuid), "Uuid");
        assert_eq!(data_type_to_rust(&DataType::DateTime), "DateTime<Utc>");
        assert_eq!(data_type_to_rust(&DataType::Json), "serde_json::Value");
        assert_eq!(data_type_to_rust(&DataType::Bytes), "Vec<u8>");
        assert_eq!(
            data_type_to_rust(&DataType::Optional(Box::new(DataType::String))),
            "Option<String>"
        );
        assert_eq!(
            data_type_to_rust(&DataType::Array(Box::new(DataType::Int32))),
            "Vec<i32>"
        );
        assert_eq!(
            data_type_to_rust(&DataType::Reference {
                entity_name: "User".into(),
                field_name: "id".into()
            }),
            "Uuid"
        );
    }

    #[test]
    fn test_validator_attrs_email() {
        let mut field = Field::new("email", DataType::String);
        field.validations.push(Validation::Email);

        let attrs = generate_validator_attrs(&field);
        assert_eq!(attrs.len(), 1);
        assert!(attrs[0].contains("email"));
    }

    #[test]
    fn test_validator_attrs_length_combined() {
        let mut field = Field::new("name", DataType::String);
        field.validations.push(Validation::MinLength(1));
        field.validations.push(Validation::MaxLength(100));

        let attrs = generate_validator_attrs(&field);
        assert_eq!(attrs.len(), 1);
        assert!(attrs[0].contains("length(min = 1, max = 100)"));
    }

    #[test]
    fn test_validator_attrs_url() {
        let mut field = Field::new("website", DataType::String);
        field.validations.push(Validation::Url);

        let attrs = generate_validator_attrs(&field);
        assert_eq!(attrs.len(), 1);
        assert!(attrs[0].contains("url"));
    }

    #[test]
    fn test_validator_attrs_range() {
        let mut field = Field::new("age", DataType::Int32);
        field.validations.push(Validation::Min(0.0));
        field.validations.push(Validation::Max(150.0));

        let attrs = generate_validator_attrs(&field);
        assert_eq!(attrs.len(), 2);
        assert!(attrs.iter().any(|a| a.contains("range(min = 0)")));
        assert!(attrs.iter().any(|a| a.contains("range(max = 150)")));
    }

    #[test]
    fn test_pagination_types() {
        let content = generate_pagination_types();
        assert!(content.contains("PaginationParams"));
        assert!(content.contains("PaginatedResponse"));
        assert!(content.contains("total_pages"));
    }

    #[test]
    fn test_unique_field_attr() {
        let mut project = ProjectGraph::new("test");
        let mut entity = Entity::new("Tag");
        entity.config.timestamps = false;
        let mut slug = Field::new("slug", DataType::String);
        slug.required = true;
        slug.unique = true;
        entity.fields.push(slug);
        project.add_entity(entity);

        let ctx = GenerationContext::from_project_default(&project);
        let e = ctx.entities().first().unwrap();
        let info = EntityInfo::new(e, &ctx);

        let model = generate_model_struct(&info, &ctx);
        assert!(model.contains("unique"));
    }

    #[test]
    fn test_optional_field_in_dto() {
        let mut project = ProjectGraph::new("test");
        let mut entity = Entity::new("Profile");
        entity.config.timestamps = false;

        let bio = Field::new("bio", DataType::Text);
        // bio is not required
        entity.fields.push(bio);

        project.add_entity(entity);

        let ctx = GenerationContext::from_project_default(&project);
        let e = ctx.entities().first().unwrap();
        let info = EntityInfo::new(e, &ctx);

        let dto = generate_create_dto(&info, &ctx);
        assert!(dto.contains("Option<String>"));
    }
}
