//! # Confirm Delete Dialog Component
//!
//! Dialog for confirming destructive delete operations in the Immortal Engine.
//!
//! ## Features
//!
//! - Confirm deletion of entities, fields, relationships, and endpoints
//! - Shows what will be affected by the deletion
//! - Requires explicit confirmation before proceeding
//! - Warning for cascading deletions
//!

use dioxus::prelude::*;
use imortal_core::types::{EndpointId, EntityId, FieldId, RelationshipId};

use crate::state::{APP_STATE, DeleteTarget, StatusLevel};

// ============================================================================
// Component Props
// ============================================================================

#[derive(Props, Clone, PartialEq)]
pub struct ConfirmDeleteDialogProps {
    /// The target to delete
    pub target: DeleteTarget,

    /// Optional callback when deletion is confirmed
    #[props(default)]
    pub on_confirm: EventHandler<()>,

    /// Optional callback when dialog is cancelled
    #[props(default)]
    pub on_cancel: EventHandler<()>,
}

// ============================================================================
// Main Component
// ============================================================================

/// Confirmation dialog for delete operations
#[component]
pub fn ConfirmDeleteDialog(props: ConfirmDeleteDialogProps) -> Element {
    let mut is_deleting = use_signal(|| false);
    let mut confirm_text = use_signal(|| String::new());

    // Get information about what's being deleted
    let (title, message, item_name, requires_confirm_text, cascading_info) =
        get_delete_info(&props.target);

    // Check if confirmation text matches (for dangerous operations)
    let can_delete = use_memo(move || {
        if requires_confirm_text {
            confirm_text.read().to_lowercase() == "delete"
        } else {
            true
        }
    });

    // Handle deletion
    let handle_delete = move |_| {
        if !*can_delete.read() {
            return;
        }

        is_deleting.set(true);

        // Perform the deletion based on target type
        match &props.target {
            DeleteTarget::Entity(entity_id) => {
                delete_entity(*entity_id);
            }
            DeleteTarget::Entities(entity_ids) => {
                for entity_id in entity_ids {
                    delete_entity(*entity_id);
                }
            }
            DeleteTarget::Field(entity_id, field_id) => {
                delete_field(*entity_id, *field_id);
            }
            DeleteTarget::Relationship(relationship_id) => {
                delete_relationship(*relationship_id);
            }
            DeleteTarget::Endpoint(endpoint_id) => {
                delete_endpoint(*endpoint_id);
            }
        }

        is_deleting.set(false);
        props.on_confirm.call(());
    };

    // Handle cancel
    let handle_cancel = move |_| {
        APP_STATE.write().ui.close_dialog();
        props.on_cancel.call(());
    };

    let deleting = *is_deleting.read();

    rsx! {
        div {
            class: "confirm-delete-dialog p-6",

            // Header with warning icon
            div {
                class: "flex items-start gap-4 mb-6",

                // Warning icon
                div {
                    class: "flex-shrink-0 w-12 h-12 rounded-full bg-red-500/20 flex items-center justify-center",
                    span { class: "text-2xl", "⚠️" }
                }

                // Title and message
                div {
                    class: "flex-1",
                    h2 {
                        class: "text-xl font-bold text-red-400 mb-2",
                        "{title}"
                    }
                    p {
                        class: "text-slate-300",
                        "{message}"
                    }
                }
            }

            // Item being deleted
            if !item_name.is_empty() {
                div {
                    class: "mb-4 p-3 bg-slate-700/50 rounded-lg border border-slate-600",
                    div {
                        class: "flex items-center gap-2",
                        span { class: "text-slate-400", "Item:" }
                        span { class: "font-medium text-white", "{item_name}" }
                    }
                }
            }

            // Cascading deletion warning
            if !cascading_info.is_empty() {
                div {
                    class: "mb-4 p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg",
                    div {
                        class: "flex items-start gap-2",
                        span { class: "text-amber-400", "⚠" }
                        div {
                            class: "text-sm text-amber-300",
                            p { class: "font-medium mb-1", "This will also delete:" }
                            ul {
                                class: "list-disc list-inside text-amber-200/80",
                                for info in cascading_info.iter() {
                                    li { "{info}" }
                                }
                            }
                        }
                    }
                }
            }

            // Confirmation text input (for dangerous operations)
            if requires_confirm_text {
                div {
                    class: "mb-6",
                    label {
                        class: "block text-sm font-medium text-slate-400 mb-2",
                        "Type \"delete\" to confirm:"
                    }
                    input {
                        class: "w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg focus:outline-none focus:border-red-500 text-white",
                        r#type: "text",
                        placeholder: "delete",
                        value: "{confirm_text}",
                        disabled: deleting,
                        oninput: move |e| confirm_text.set(e.value()),
                    }
                }
            }

            // Actions
            div {
                class: "flex justify-end gap-3",

                button {
                    r#type: "button",
                    class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg transition-colors",
                    disabled: deleting,
                    onclick: handle_cancel,
                    "Cancel"
                }

                button {
                    r#type: "button",
                    class: "px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-red-600/50 disabled:cursor-not-allowed rounded-lg transition-colors flex items-center gap-2",
                    disabled: deleting || !*can_delete.read(),
                    onclick: handle_delete,

                    if deleting {
                        span { class: "animate-spin", "⏳" }
                        "Deleting..."
                    } else {
                        span { "🗑️" }
                        "Delete"
                    }
                }
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get information about the deletion target
fn get_delete_info(target: &DeleteTarget) -> (&'static str, String, String, bool, Vec<String>) {
    let state = APP_STATE.read();

    match target {
        DeleteTarget::Entity(entity_id) => {
            let entity_name = state
                .project
                .as_ref()
                .and_then(|p| p.entities.get(entity_id))
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            // Check for related items
            let mut cascading = Vec::new();
            if let Some(project) = &state.project {
                // Count relationships that will be deleted
                let relationship_count = project
                    .relationships
                    .values()
                    .filter(|r| r.from_entity_id == *entity_id || r.to_entity_id == *entity_id)
                    .count();

                if relationship_count > 0 {
                    cascading.push(format!(
                        "{} relationship{}",
                        relationship_count,
                        if relationship_count == 1 { "" } else { "s" }
                    ));
                }

                // Count endpoints that will be deleted
                let endpoint_count = project
                    .endpoints
                    .values()
                    .filter(|e| e.entity_id == *entity_id)
                    .count();

                if endpoint_count > 0 {
                    cascading.push(format!(
                        "{} endpoint group{}",
                        endpoint_count,
                        if endpoint_count == 1 { "" } else { "s" }
                    ));
                }
            }

            (
                "Delete Entity",
                format!(
                    "Are you sure you want to delete the entity \"{}\"? This action cannot be undone.",
                    entity_name
                ),
                entity_name,
                !cascading.is_empty(), // Require confirmation text if there are cascading deletions
                cascading,
            )
        }

        DeleteTarget::Entities(entity_ids) => {
            let count = entity_ids.len();

            // Check for related items
            let mut cascading = Vec::new();
            if let Some(project) = &state.project {
                // Count relationships that will be deleted
                let relationship_count = project
                    .relationships
                    .values()
                    .filter(|r| {
                        entity_ids.contains(&r.from_entity_id)
                            || entity_ids.contains(&r.to_entity_id)
                    })
                    .count();

                if relationship_count > 0 {
                    cascading.push(format!(
                        "{} relationship{}",
                        relationship_count,
                        if relationship_count == 1 { "" } else { "s" }
                    ));
                }

                // Count endpoints that will be deleted
                let endpoint_count = project
                    .endpoints
                    .values()
                    .filter(|e| entity_ids.contains(&e.entity_id))
                    .count();

                if endpoint_count > 0 {
                    cascading.push(format!(
                        "{} endpoint group{}",
                        endpoint_count,
                        if endpoint_count == 1 { "" } else { "s" }
                    ));
                }
            }

            (
                "Delete Multiple Entities",
                format!(
                    "Are you sure you want to delete {} entities? This action cannot be undone.",
                    count
                ),
                format!("{} entities", count),
                true, // Always require confirmation for bulk delete
                cascading,
            )
        }

        DeleteTarget::Field(entity_id, field_id) => {
            let (entity_name, field_name) = state
                .project
                .as_ref()
                .and_then(|p| {
                    p.entities.get(entity_id).and_then(|e| {
                        e.get_field(*field_id)
                            .map(|f| (e.name.clone(), f.name.clone()))
                    })
                })
                .unwrap_or_else(|| ("Unknown".to_string(), "Unknown".to_string()));

            // Check if this field is referenced by relationships
            let mut cascading = Vec::new();
            if let Some(project) = &state.project {
                let relationship_count = project
                    .relationships
                    .values()
                    .filter(|r| {
                        (r.from_entity_id == *entity_id && r.from_field == field_name)
                            || (r.to_entity_id == *entity_id && r.to_field == field_name)
                    })
                    .count();

                if relationship_count > 0 {
                    cascading.push(format!(
                        "{} relationship{} using this field",
                        relationship_count,
                        if relationship_count == 1 { "" } else { "s" }
                    ));
                }
            }

            (
                "Delete Field",
                format!(
                    "Are you sure you want to delete the field \"{}\" from \"{}\"?",
                    field_name, entity_name
                ),
                format!("{}.{}", entity_name, field_name),
                false,
                cascading,
            )
        }

        DeleteTarget::Relationship(relationship_id) => {
            let relationship_name = state
                .project
                .as_ref()
                .and_then(|p| p.relationships.get(relationship_id))
                .map(|r| {
                    if r.name.is_empty() {
                        format!("{} → {}", r.from_entity_id, r.to_entity_id)
                    } else {
                        r.name.clone()
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());

            (
                "Delete Relationship",
                "Are you sure you want to delete this relationship?".to_string(),
                relationship_name,
                false,
                Vec::new(),
            )
        }

        DeleteTarget::Endpoint(endpoint_id) => {
            let endpoint_name = state
                .project
                .as_ref()
                .and_then(|p| p.endpoints.get(endpoint_id))
                .map(|e| e.base_path.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            (
                "Delete Endpoint Group",
                "Are you sure you want to delete this endpoint group?".to_string(),
                endpoint_name,
                false,
                Vec::new(),
            )
        }
    }
}

/// Delete an entity and its related items
fn delete_entity(entity_id: EntityId) {
    let mut state = APP_STATE.write();

    if let Some(project) = &mut state.project {
        // Get entity name for status message
        let entity_name = project
            .entities
            .get(&entity_id)
            .map(|e| e.name.clone())
            .unwrap_or_default();

        // Remove the entity
        project.entities.remove(&entity_id);

        // Remove related relationships
        let relationship_ids: Vec<_> = project
            .relationships
            .iter()
            .filter(|(_, r)| r.from_entity_id == entity_id || r.to_entity_id == entity_id)
            .map(|(id, _)| *id)
            .collect();

        for rel_id in relationship_ids {
            project.relationships.remove(&rel_id);
        }

        // Remove related endpoints
        let endpoint_ids: Vec<_> = project
            .endpoints
            .iter()
            .filter(|(_, e)| e.entity_id == entity_id)
            .map(|(id, _)| *id)
            .collect();

        for ep_id in endpoint_ids {
            project.endpoints.remove(&ep_id);
        }

        // Update selection
        state.selection.entities.remove(&entity_id);

        // Mark dirty and set status
        state.is_dirty = true;
        state.ui.close_dialog();
        state.ui.set_status(
            &format!("Deleted entity '{}'", entity_name),
            StatusLevel::Success,
        );

        // Save to history
        drop(state);
        APP_STATE.write().save_to_history("Delete entity");
    }
}

/// Delete a field from an entity
fn delete_field(entity_id: EntityId, field_id: FieldId) {
    let mut state = APP_STATE.write();

    if let Some(project) = &mut state.project {
        if let Some(entity) = project.entities.get_mut(&entity_id) {
            // Get field name for status message
            let field_name = entity
                .get_field(field_id)
                .map(|f| f.name.clone())
                .unwrap_or_default();

            // Remove the field
            entity.remove_field(field_id);
            entity.touch();

            // Update selection
            if state.selection.field == Some((entity_id, field_id)) {
                state.selection.field = None;
            }

            // Mark dirty and set status
            state.is_dirty = true;
            state.ui.close_dialog();
            state.ui.set_status(
                &format!("Deleted field '{}'", field_name),
                StatusLevel::Success,
            );

            // Save to history
            drop(state);
            APP_STATE.write().save_to_history("Delete field");
        }
    }
}

/// Delete a relationship
fn delete_relationship(relationship_id: RelationshipId) {
    let mut state = APP_STATE.write();

    if let Some(project) = &mut state.project {
        // Remove the relationship
        project.relationships.remove(&relationship_id);

        // Update selection
        state.selection.relationships.remove(&relationship_id);

        // Mark dirty and set status
        state.is_dirty = true;
        state.ui.close_dialog();
        state
            .ui
            .set_status("Deleted relationship", StatusLevel::Success);

        // Save to history
        drop(state);
        APP_STATE.write().save_to_history("Delete relationship");
    }
}

/// Delete an endpoint group
fn delete_endpoint(endpoint_id: EndpointId) {
    let mut state = APP_STATE.write();

    if let Some(project) = &mut state.project {
        // Get endpoint path for status message
        let path = project
            .endpoints
            .get(&endpoint_id)
            .map(|e| e.base_path.clone())
            .unwrap_or_default();

        // Remove the endpoint
        project.endpoints.remove(&endpoint_id);

        // Update selection
        state.selection.endpoints.remove(&endpoint_id);

        // Mark dirty and set status
        state.is_dirty = true;
        state.ui.close_dialog();
        state.ui.set_status(
            &format!("Deleted endpoint group '{}'", path),
            StatusLevel::Success,
        );

        // Save to history
        drop(state);
        APP_STATE.write().save_to_history("Delete endpoint");
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_target_variants() {
        // Test that DeleteTarget variants can be created
        use uuid::Uuid;

        let entity_id = Uuid::new_v4();
        let field_id = Uuid::new_v4();
        let rel_id = Uuid::new_v4();
        let ep_id = Uuid::new_v4();

        // Verify variants are constructible
        let _ = DeleteTarget::Entity(entity_id);
        let _ = DeleteTarget::Entities(vec![entity_id]);
        let _ = DeleteTarget::Field(entity_id, field_id);
        let _ = DeleteTarget::Relationship(rel_id);
        let _ = DeleteTarget::Endpoint(ep_id);

        assert!(true);
    }

    #[test]
    fn test_delete_target_clone() {
        use uuid::Uuid;

        let target = DeleteTarget::Entity(Uuid::new_v4());
        let cloned = target.clone();
        assert_eq!(target, cloned);
    }

    #[test]
    fn test_delete_target_eq() {
        use uuid::Uuid;

        let id = Uuid::new_v4();
        let target1 = DeleteTarget::Entity(id);
        let target2 = DeleteTarget::Entity(id);
        assert_eq!(target1, target2);

        let different_id = Uuid::new_v4();
        let target3 = DeleteTarget::Entity(different_id);
        assert_ne!(target1, target3);
    }
}
