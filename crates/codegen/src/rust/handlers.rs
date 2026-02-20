//! # Handler Generator (Axum)
//!
//! Generates Axum request handler functions for every entity that has
//! configured endpoints. Each enabled CRUD operation produces a dedicated
//! async handler function.
//!
//! ## Generated Files
//!
//! - `src/handlers/mod.rs` — module declarations, pagination types
//! - `src/handlers/{entity}.rs` — CRUD handlers per entity
//!
//! ## Handler Signatures
//!
//! | Operation | Signature |
//! |-----------|-----------|
//! | List      | `async fn list_{entities}(State, Query<PaginationParams>) -> Result<Json<PaginatedResponse<…>>, AppError>` |
//! | Get       | `async fn get_{entity}(State, Path<PK>) -> Result<Json<Response>, AppError>` |
//! | Create    | `async fn create_{entity}(State, Json<CreateDto>) -> Result<(StatusCode, Json<Response>), AppError>` |
//! | Update    | `async fn update_{entity}(State, Path<PK>, Json<UpdateDto>) -> Result<Json<Response>, AppError>` |
//! | Delete    | `async fn delete_{entity}(State, Path<PK>) -> Result<StatusCode, AppError>` |

use imortal_ir::OperationType;

use crate::context::{EntityInfo, GenerationContext};
use crate::rust::models::generate_pagination_types;
use crate::rust::{doc_comment, file_header};
use crate::{FileType, GeneratedFile};

// ============================================================================
// Public API
// ============================================================================

/// Generate all handler files (`src/handlers/mod.rs` + one file per entity
/// that has at least one enabled endpoint operation).
pub fn generate_handlers(ctx: &GenerationContext) -> Vec<GeneratedFile> {
    let mut files = Vec::new();

    // handlers/mod.rs
    files.push(generate_handlers_mod(ctx));

    // Per-entity handler files
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);

        // Only generate handlers if the entity has an endpoint group with
        // at least one enabled operation.
        let endpoint = match info.endpoint() {
            Some(ep) if ep.enabled && !ep.enabled_operations().is_empty() => ep,
            _ => continue,
        };

        files.push(generate_entity_handlers(&info, ctx));
    }

    files
}

// ============================================================================
// handlers/mod.rs
// ============================================================================

fn generate_handlers_mod(ctx: &GenerationContext) -> GeneratedFile {
    let mut content = String::with_capacity(2048);

    content.push_str(&file_header("Request handlers for all API endpoints."));

    content.push_str("use serde::{Deserialize, Serialize};\n\n");

    // Module declarations
    for entity in ctx.entities() {
        let info = EntityInfo::new(entity, ctx);
        let has_handlers = info
            .endpoint()
            .map(|ep| ep.enabled && !ep.enabled_operations().is_empty())
            .unwrap_or(false);
        if has_handlers {
            let module = GenerationContext::module_name(&entity.name);
            content.push_str(&format!("pub mod {};\n", module));
        }
    }

    content.push('\n');

    // Pagination types (shared across all handlers)
    content.push_str(
        "// ============================================================================\n",
    );
    content.push_str("// Shared Types\n");
    content.push_str(
        "// ============================================================================\n",
    );
    content.push_str(&generate_pagination_types());

    GeneratedFile::new("src/handlers/mod.rs", content, FileType::Rust)
}

// ============================================================================
// Per-entity handler file
// ============================================================================

fn generate_entity_handlers(info: &EntityInfo, ctx: &GenerationContext) -> GeneratedFile {
    let module = info.module_name();
    let path = format!("src/handlers/{}.rs", module);

    let endpoint = info.endpoint().unwrap(); // caller guarantees this exists
    let enabled_ops: Vec<OperationType> = endpoint
        .enabled_operations()
        .iter()
        .map(|op| op.operation_type)
        .collect();

    let mut content = String::with_capacity(4096);

    // File header
    content.push_str(&file_header(&format!(
        "Request handlers for {} endpoints.",
        info.pascal_name()
    )));

    // Imports
    content.push_str(&generate_handler_imports(info, ctx, &enabled_ops));
    content.push('\n');

    // Individual handlers
    if enabled_ops.contains(&OperationType::ReadAll) {
        content.push_str(&generate_list_handler(info, ctx));
        content.push('\n');
    }

    if enabled_ops.contains(&OperationType::Read) {
        content.push_str(&generate_get_handler(info, ctx));
        content.push('\n');
    }

    if enabled_ops.contains(&OperationType::Create) {
        content.push_str(&generate_create_handler(info, ctx));
        content.push('\n');
    }

    if enabled_ops.contains(&OperationType::Update) {
        content.push_str(&generate_update_handler(info, ctx));
        content.push('\n');
    }

    if enabled_ops.contains(&OperationType::Delete) {
        content.push_str(&generate_delete_handler(info, ctx));
        content.push('\n');
    }

    GeneratedFile::new(path, content, FileType::Rust)
}

// ============================================================================
// Imports
// ============================================================================

fn generate_handler_imports(
    info: &EntityInfo,
    ctx: &GenerationContext,
    ops: &[OperationType],
) -> String {
    let module = info.module_name();
    let create_dto = GenerationContext::create_dto_name(&info.entity.name);
    let update_dto = GenerationContext::update_dto_name(&info.entity.name);
    let response_dto = GenerationContext::response_dto_name(&info.entity.name);

    // Check if this entity has any password/secret fields that need hashing
    let has_secret_fields = info
        .entity
        .fields
        .iter()
        .any(|f| f.secret || f.name.contains("password"));

    let mut out = String::with_capacity(1024);

    // axum imports
    let mut axum_extracts = vec!["State"];
    if ops.contains(&OperationType::Read)
        || ops.contains(&OperationType::Update)
        || ops.contains(&OperationType::Delete)
    {
        axum_extracts.push("Path");
    }
    if ops.contains(&OperationType::ReadAll) {
        axum_extracts.push("Query");
    }

    out.push_str(&format!(
        "use axum::extract::{{{}}};\n",
        axum_extracts.join(", ")
    ));

    if ops.contains(&OperationType::Create) {
        out.push_str("use axum::http::StatusCode;\n");
    } else if ops.contains(&OperationType::Delete) {
        out.push_str("use axum::http::StatusCode;\n");
    }

    out.push_str("use axum::Json;\n");

    // SeaORM imports
    let mut sea_imports = vec!["EntityTrait"];
    if ops.contains(&OperationType::Create) || ops.contains(&OperationType::Update) {
        sea_imports.push("ActiveModelTrait");
        sea_imports.push("Set");
    }
    if ops.contains(&OperationType::ReadAll) {
        sea_imports.push("PaginatorTrait");
    }
    if ops.contains(&OperationType::Update) {
        sea_imports.push("IntoActiveModel");
    }

    out.push_str(&format!("use sea_orm::{{{}}};\n", sea_imports.join(", ")));

    // uuid (for path parameters)
    let pk_type = info.pk_rust_type();
    if pk_type == "Uuid" {
        out.push_str("use uuid::Uuid;\n");
    }

    // Validator
    if ops.contains(&OperationType::Create) || ops.contains(&OperationType::Update) {
        out.push_str("use validator::Validate;\n");
    }

    // Password hashing (when entity has secret/password fields and auth is enabled)
    if has_secret_fields
        && ctx.auth_enabled()
        && (ops.contains(&OperationType::Create) || ops.contains(&OperationType::Update))
    {
        out.push_str("use crate::auth::jwt::hash_password;\n");
    }

    out.push('\n');

    // Local imports
    out.push_str("use crate::error::AppError;\n");
    out.push_str("use crate::state::AppState;\n");

    // Model imports
    let mut model_imports = vec![format!("self")];
    if ops.contains(&OperationType::Create) {
        model_imports.push(create_dto.clone());
    }
    if ops.contains(&OperationType::Update) {
        model_imports.push(update_dto.clone());
    }
    model_imports.push(response_dto.clone());

    out.push_str(&format!(
        "use crate::models::{}::{{{}}};\n",
        module,
        model_imports.join(", "),
    ));

    // Pagination types (for list handler)
    if ops.contains(&OperationType::ReadAll) {
        out.push_str("use crate::handlers::{PaginationParams, PaginatedResponse};\n");
    }

    out.push('\n');
    out
}

// ============================================================================
// List handler (ReadAll)
// ============================================================================

fn generate_list_handler(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let fn_name = format!("list_{}", info.plural_name());
    let response_dto = GenerationContext::response_dto_name(&info.entity.name);
    let pascal = info.pascal_name();

    let mut out = String::with_capacity(1024);

    out.push_str(&doc_comment(
        Some(&format!(
            "List all {}s with pagination.\n\nGET {}",
            info.snake_name(),
            info.base_path()
        )),
        ctx,
    ));

    out.push_str(&format!(
        r#"pub async fn {fn_name}(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<{response_dto}>>, AppError> {{
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100).max(1);

    let paginator = {module}::Entity::find()
        .paginate(&state.db, per_page);

    let total = paginator.num_items().await.map_err(AppError::from)?;

    let items: Vec<{response_dto}> = paginator
        .fetch_page(page - 1)
        .await
        .map_err(AppError::from)?
        .into_iter()
        .map({response_dto}::from)
        .collect();

    Ok(Json(PaginatedResponse::new(items, total, page, per_page)))
}}
"#,
        module = info.module_name(),
    ));

    out
}

// ============================================================================
// Get handler (Read)
// ============================================================================

fn generate_get_handler(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let fn_name = format!("get_{}", info.snake_name());
    let response_dto = GenerationContext::response_dto_name(&info.entity.name);
    let pk_type = info.pk_rust_type();

    let mut out = String::with_capacity(512);

    out.push_str(&doc_comment(
        Some(&format!(
            "Get a single {} by ID.\n\nGET {}/:id",
            info.snake_name(),
            info.base_path()
        )),
        ctx,
    ));

    out.push_str(&format!(
        r#"pub async fn {fn_name}(
    State(state): State<AppState>,
    Path(id): Path<{pk_type}>,
) -> Result<Json<{response_dto}>, AppError> {{
    let item = {module}::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound)?;

    Ok(Json({response_dto}::from(item)))
}}
"#,
        module = info.module_name(),
    ));

    out
}

// ============================================================================
// Create handler
// ============================================================================

fn generate_create_handler(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let fn_name = format!("create_{}", info.snake_name());
    let create_dto = GenerationContext::create_dto_name(&info.entity.name);
    let response_dto = GenerationContext::response_dto_name(&info.entity.name);
    let pk_type = info.pk_rust_type();

    let create_fields = info.create_fields();

    let mut out = String::with_capacity(2048);

    out.push_str(&doc_comment(
        Some(&format!(
            "Create a new {}.\n\nPOST {}",
            info.snake_name(),
            info.base_path()
        )),
        ctx,
    ));

    out.push_str(&format!(
        r#"pub async fn {fn_name}(
    State(state): State<AppState>,
    Json(payload): Json<{create_dto}>,
) -> Result<(StatusCode, Json<{response_dto}>), AppError> {{
    payload.validate().map_err(AppError::from)?;

    let model = {module}::ActiveModel {{
"#,
        module = info.module_name(),
    ));

    // Primary key assignment
    if pk_type.contains("Uuid") {
        out.push_str("        id: Set(Uuid::new_v4()),\n");
    }
    // else Serial — the DB auto-increments

    // Set fields from payload — hash password/secret fields
    for field in &create_fields {
        let name = GenerationContext::snake(&field.name);
        let is_password_field =
            field.secret || field.name.contains("password") || field.name.contains("secret");

        if is_password_field && ctx.auth_enabled() {
            // Hash the password before storing
            // The DTO field is named without the _hash suffix (e.g. "password"),
            // but the model field may be "password_hash". We accept the DTO field
            // name as-is and hash it.
            let dto_field = if name.ends_with("_hash") {
                // If the model field is "password_hash", the DTO likely has "password"
                name.trim_end_matches("_hash").to_string()
            } else {
                name.clone()
            };
            out.push_str(&format!(
                "        {name}: Set(hash_password(&payload.{dto_field}).map_err(|e| AppError::internal(format!(\"Password hashing failed: {{}}\", e)))?),\n"
            ));
        } else {
            out.push_str(&format!("        {name}: Set(payload.{name}),\n"));
        }
    }

    // Timestamps
    if info.has_timestamps() {
        if !create_fields.iter().any(|f| f.name == "created_at") {
            out.push_str("        created_at: Set(chrono::Utc::now()),\n");
        }
        if !create_fields.iter().any(|f| f.name == "updated_at") {
            out.push_str("        updated_at: Set(chrono::Utc::now()),\n");
        }
    }

    // Soft-delete default
    if info.has_soft_delete() {
        out.push_str("        deleted_at: Set(None),\n");
    }

    // Close ActiveModel and insert
    out.push_str(&format!(
        r#"        ..Default::default()
    }}
    .insert(&state.db)
    .await
    .map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json({response_dto}::from(model))))
}}
"#,
    ));

    out
}

// ============================================================================
// Update handler
// ============================================================================

fn generate_update_handler(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let fn_name = format!("update_{}", info.snake_name());
    let update_dto = GenerationContext::update_dto_name(&info.entity.name);
    let response_dto = GenerationContext::response_dto_name(&info.entity.name);
    let pk_type = info.pk_rust_type();

    let update_fields = info.update_fields();

    let mut out = String::with_capacity(2048);

    out.push_str(&doc_comment(
        Some(&format!(
            "Update an existing {} by ID.\n\nPUT {}/:id",
            info.snake_name(),
            info.base_path()
        )),
        ctx,
    ));

    out.push_str(&format!(
        r#"pub async fn {fn_name}(
    State(state): State<AppState>,
    Path(id): Path<{pk_type}>,
    Json(payload): Json<{update_dto}>,
) -> Result<Json<{response_dto}>, AppError> {{
    payload.validate().map_err(AppError::from)?;

    // Find existing record
    let existing = {module}::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound)?;

    let mut active: {module}::ActiveModel = existing.into_active_model();

"#,
        module = info.module_name(),
    ));

    // Apply optional field updates — hash password/secret fields
    for field in &update_fields {
        let name = GenerationContext::snake(&field.name);
        let is_password_field =
            field.secret || field.name.contains("password") || field.name.contains("secret");

        if is_password_field && ctx.auth_enabled() {
            out.push_str(&format!(
                "    if let Some(val) = payload.{name} {{\n        active.{name} = Set(hash_password(&val).map_err(|e| AppError::internal(format!(\"Password hashing failed: {{}}\", e)))?);\n    }}\n",
            ));
        } else {
            out.push_str(&format!(
                "    if let Some(val) = payload.{name} {{\n        active.{name} = Set(val);\n    }}\n",
            ));
        }
    }

    // Update the updated_at timestamp
    if info.has_timestamps() {
        out.push_str("\n    active.updated_at = Set(chrono::Utc::now());\n");
    }

    // Save and return
    out.push_str(&format!(
        r#"
    let updated = active.update(&state.db).await.map_err(AppError::from)?;

    Ok(Json({response_dto}::from(updated)))
}}
"#,
    ));

    out
}

// ============================================================================
// Delete handler
// ============================================================================

fn generate_delete_handler(info: &EntityInfo, ctx: &GenerationContext) -> String {
    let fn_name = format!("delete_{}", info.snake_name());
    let pk_type = info.pk_rust_type();

    let mut out = String::with_capacity(1024);

    out.push_str(&doc_comment(
        Some(&format!(
            "Delete a {} by ID.\n\nDELETE {}/:id",
            info.snake_name(),
            info.base_path()
        )),
        ctx,
    ));

    if info.has_soft_delete() {
        // Soft delete: set deleted_at instead of actual deletion
        out.push_str(&format!(
            r#"pub async fn {fn_name}(
    State(state): State<AppState>,
    Path(id): Path<{pk_type}>,
) -> Result<StatusCode, AppError> {{
    let existing = {module}::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound)?;

    let mut active: {module}::ActiveModel = existing.into_active_model();
    active.deleted_at = Set(Some(chrono::Utc::now()));
    active.update(&state.db).await.map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}}
"#,
            module = info.module_name(),
        ));
    } else {
        // Hard delete
        out.push_str(&format!(
            r#"pub async fn {fn_name}(
    State(state): State<AppState>,
    Path(id): Path<{pk_type}>,
) -> Result<StatusCode, AppError> {{
    let result = {module}::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(AppError::from)?;

    if result.rows_affected == 0 {{
        return Err(AppError::NotFound);
    }}

    Ok(StatusCode::NO_CONTENT)
}}
"#,
            module = info.module_name(),
        ));
    }

    out
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use imortal_core::DataType;
    use imortal_ir::{CrudOperation, EndpointGroup, Entity, Field, OperationType, ProjectGraph};
    use uuid::Uuid;

    /// Create a User entity with an endpoint group.
    fn setup_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("test_api");

        // User entity
        let mut user = Entity::new("User");
        user.config.timestamps = true;
        let user_id = user.id;

        let mut email = Field::new("email", DataType::String);
        email.required = true;
        email.unique = true;
        user.fields.push(email);

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        user.fields.push(name);

        project.add_entity(user);

        // Endpoint for User (all CRUD enabled)
        let endpoint = EndpointGroup::new(user_id, "User");
        project.add_endpoint(endpoint);

        project
    }

    /// Create a project where only Read + ReadAll are enabled.
    fn setup_read_only_project() -> ProjectGraph {
        let mut project = ProjectGraph::new("read_api");

        let mut item = Entity::new("Item");
        item.config.timestamps = false;
        let item_id = item.id;

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        item.fields.push(name);

        project.add_entity(item);

        let endpoint = EndpointGroup::new(item_id, "Item")
            .with_operations(&[OperationType::Read, OperationType::ReadAll]);
        project.add_endpoint(endpoint);

        project
    }

    #[test]
    fn test_generate_handlers_empty_project() {
        let project = ProjectGraph::new("empty");
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        // Should have mod.rs but no entity handlers
        assert_eq!(files.len(), 1);
        assert!(files[0].path.to_string_lossy().contains("mod.rs"));
    }

    #[test]
    fn test_generate_handlers_with_entity() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        // mod.rs + user.rs
        assert_eq!(files.len(), 2);

        let paths: Vec<String> = files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();
        assert!(paths.iter().any(|p| p.contains("mod.rs")));
        assert!(paths.iter().any(|p| p.contains("user.rs")));
    }

    #[test]
    fn test_handlers_mod_has_pagination() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let mod_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("mod.rs"))
            .unwrap();

        assert!(mod_file.content.contains("PaginationParams"));
        assert!(mod_file.content.contains("PaginatedResponse"));
        assert!(mod_file.content.contains("pub mod user;"));
    }

    #[test]
    fn test_user_handlers_all_crud() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let user_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user.rs"))
            .unwrap();

        let content = &user_file.content;

        // All 5 CRUD handlers should be present
        assert!(
            content.contains("pub async fn list_users("),
            "Missing list handler"
        );
        assert!(
            content.contains("pub async fn get_user("),
            "Missing get handler"
        );
        assert!(
            content.contains("pub async fn create_user("),
            "Missing create handler"
        );
        assert!(
            content.contains("pub async fn update_user("),
            "Missing update handler"
        );
        assert!(
            content.contains("pub async fn delete_user("),
            "Missing delete handler"
        );

        // Check imports
        assert!(content.contains("use axum::extract::{"));
        assert!(content.contains("use sea_orm::"));
        assert!(content.contains("use crate::error::AppError;"));
        assert!(content.contains("use crate::state::AppState;"));
        assert!(content.contains("use crate::models::user::"));
    }

    #[test]
    fn test_list_handler_uses_pagination() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let user_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user.rs"))
            .unwrap();

        let content = &user_file.content;
        assert!(content.contains("PaginationParams"));
        assert!(content.contains("PaginatedResponse"));
        assert!(content.contains("paginate"));
        assert!(content.contains("num_items"));
        assert!(content.contains("fetch_page"));
    }

    #[test]
    fn test_get_handler_returns_response_dto() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let user_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user.rs"))
            .unwrap();

        let content = &user_file.content;
        assert!(content.contains("UserResponse"));
        assert!(content.contains("find_by_id"));
        assert!(content.contains("AppError::NotFound"));
    }

    #[test]
    fn test_create_handler_validates_and_inserts() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let user_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user.rs"))
            .unwrap();

        let content = &user_file.content;
        assert!(content.contains("CreateUserDto"));
        assert!(content.contains("payload.validate()"));
        assert!(
            content.contains("Uuid::new_v4()"),
            "Create handler should generate UUID PK assignment. Content:\n{}",
            content
        );
        assert!(content.contains("Set(payload.email)"));
        assert!(content.contains("Set(payload.name)"));
        assert!(content.contains(".insert(&state.db)"));
        assert!(content.contains("StatusCode::CREATED"));
    }

    #[test]
    fn test_create_handler_with_timestamps() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let user_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user.rs"))
            .unwrap();

        let content = &user_file.content;
        assert!(content.contains("chrono::Utc::now()"));
    }

    #[test]
    fn test_update_handler_partial_update() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let user_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user.rs"))
            .unwrap();

        let content = &user_file.content;
        assert!(content.contains("UpdateUserDto"));
        assert!(content.contains("into_active_model()"));
        assert!(content.contains("if let Some(val)"));
        assert!(content.contains(".update(&state.db)"));
    }

    #[test]
    fn test_delete_handler_hard_delete() {
        let project = setup_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let user_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("user.rs"))
            .unwrap();

        let content = &user_file.content;
        assert!(content.contains("delete_by_id"));
        assert!(content.contains("rows_affected == 0"));
        assert!(content.contains("StatusCode::NO_CONTENT"));
    }

    #[test]
    fn test_delete_handler_soft_delete() {
        let mut project = ProjectGraph::new("soft_api");

        let mut article = Entity::new("Article");
        article.config.timestamps = true;
        article.config.soft_delete = true;
        let article_id = article.id;

        let mut title = Field::new("title", DataType::String);
        title.required = true;
        article.fields.push(title);

        project.add_entity(article);

        let endpoint = EndpointGroup::new(article_id, "Article");
        project.add_endpoint(endpoint);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let article_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("article.rs"))
            .unwrap();

        let content = &article_file.content;
        // Soft delete should NOT use delete_by_id
        assert!(!content.contains("delete_by_id"));
        // Instead it should set deleted_at
        assert!(content.contains("deleted_at"));
        assert!(content.contains("Set(Some(chrono::Utc::now()))"));
    }

    #[test]
    fn test_read_only_handlers() {
        let project = setup_read_only_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let item_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("item.rs"))
            .unwrap();

        let content = &item_file.content;

        // Should have Read and ReadAll
        assert!(content.contains("pub async fn list_items("));
        assert!(content.contains("pub async fn get_item("));

        // Should NOT have Create, Update, Delete
        assert!(!content.contains("pub async fn create_item("));
        assert!(!content.contains("pub async fn update_item("));
        assert!(!content.contains("pub async fn delete_item("));
    }

    #[test]
    fn test_no_handlers_without_endpoint() {
        let mut project = ProjectGraph::new("no_ep");

        let mut entity = Entity::new("Secret");
        entity.config.timestamps = false;
        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);
        project.add_entity(entity);

        // No endpoint group added!
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        // Only mod.rs, no entity handlers
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_handler_imports_vary_by_ops() {
        let project = setup_read_only_project();
        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        let item_file = files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("item.rs"))
            .unwrap();

        let content = &item_file.content;

        // Read-only should not import Validate, Set, etc.
        assert!(!content.contains("Validate"));
        assert!(!content.contains("ActiveModelTrait"));

        // Should import PaginatorTrait for ReadAll
        assert!(content.contains("PaginatorTrait"));
    }

    #[test]
    fn test_handlers_with_disabled_endpoint() {
        let mut project = ProjectGraph::new("disabled_api");

        let mut entity = Entity::new("Widget");
        entity.config.timestamps = false;
        let widget_id = entity.id;

        let mut name = Field::new("name", DataType::String);
        name.required = true;
        entity.fields.push(name);
        project.add_entity(entity);

        // Disabled endpoint
        let endpoint = EndpointGroup::new(widget_id, "Widget").disabled();
        project.add_endpoint(endpoint);

        let ctx = GenerationContext::from_project_default(&project);
        let files = generate_handlers(&ctx);

        // Only mod.rs (disabled endpoint should not produce handlers)
        assert_eq!(files.len(), 1);
    }
}
