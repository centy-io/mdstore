//! Core CRUD operations for config-driven item types.
//!
//! All functions take a `type_dir: &Path` pointing to the directory where
//! items of a given type are stored, working generically across any item type.

use crate::config::{IdStrategy, TypeConfig};
use crate::error::StoreError;
use crate::filters::Filters;
use crate::frontmatter::{
    generate_frontmatter, generate_frontmatter_raw, parse_frontmatter, parse_frontmatter_raw,
};
use crate::reconcile::get_next_display_number;
use crate::types::{
    CreateOptions, DuplicateOptions, DuplicateResult, Frontmatter, Item, MoveResult, UpdateOptions,
};
use crate::util::now_iso;
use std::path::Path;
use tokio::fs;

/// Check if a filename is a valid item file (`.md` but not `config.yaml`).
fn is_item_file(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

/// Validate status against the config's allowed statuses.
fn validate_status(config: &TypeConfig, status: &str) -> Result<(), StoreError> {
    if !config.features.status {
        return Ok(());
    }
    if config.statuses.is_empty() {
        return Ok(());
    }
    if config
        .statuses
        .iter()
        .any(|s| s.eq_ignore_ascii_case(status))
    {
        Ok(())
    } else {
        Err(StoreError::InvalidStatus {
            status: status.to_string(),
            allowed: config.statuses.clone(),
        })
    }
}

/// Validate priority against the config's priority levels.
fn validate_priority(config: &TypeConfig, priority: u32) -> Result<(), StoreError> {
    if !config.features.priority {
        return Ok(());
    }
    let max = config.priority_levels.unwrap_or(3);
    if priority < 1 || priority > max {
        return Err(StoreError::InvalidPriority { priority, max });
    }
    Ok(())
}

/// Create a new item.
///
/// Generates an ID based on the config's `identifier`, assigns a display number
/// if enabled, validates status/priority, and writes the item file.
pub async fn create(
    type_dir: &Path,
    config: &TypeConfig,
    options: CreateOptions,
) -> Result<Item, StoreError> {
    fs::create_dir_all(type_dir).await?;

    // Generate ID
    let id = match &options.id {
        Some(explicit_id) => explicit_id.clone(),
        None => {
            if config.identifier == IdStrategy::Slug {
                let slug = slug::slugify(&options.title);
                if slug.is_empty() {
                    return Err(StoreError::ValidationError(
                        "Cannot generate slug from empty title".to_string(),
                    ));
                }
                slug
            } else {
                // Default to UUID
                uuid::Uuid::new_v4().to_string()
            }
        }
    };

    // Check for existing item
    let file_path = type_dir.join(format!("{id}.md"));
    if file_path.exists() {
        return Err(StoreError::AlreadyExists(id));
    }

    // Assign display number if enabled
    let display_number = if config.features.display_number {
        Some(get_next_display_number(type_dir).await?)
    } else {
        None
    };

    // Resolve and validate status
    let status = if config.features.status {
        let s = options
            .status
            .or_else(|| config.default_status.clone())
            .unwrap_or_default();
        validate_status(config, &s)?;
        Some(s)
    } else {
        None
    };

    // Resolve and validate priority
    let priority = if config.features.priority {
        let max = config.priority_levels.unwrap_or(3);
        let p = options
            .priority
            .unwrap_or_else(|| crate::validation::priority::default_priority(max));
        validate_priority(config, p)?;
        Some(p)
    } else {
        None
    };

    let now = now_iso();
    let frontmatter = Frontmatter {
        display_number,
        status,
        priority,
        created_at: now.clone(),
        updated_at: now,
        deleted_at: None,
        custom_fields: options.custom_fields,
    };

    // Write the item file
    let content = generate_frontmatter(&frontmatter, &options.title, &options.body);
    fs::write(&file_path, &content).await?;

    Ok(Item {
        id,
        item_type: config.plural.clone(),
        title: options.title,
        body: options.body,
        frontmatter,
    })
}

/// Get a single item by ID.
pub async fn get(type_dir: &Path, config: &TypeConfig, id: &str) -> Result<Item, StoreError> {
    let file_path = type_dir.join(format!("{id}.md"));

    if !file_path.exists() {
        return Err(StoreError::NotFound(id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    let (frontmatter, title, body) = parse_frontmatter::<Frontmatter>(&content)
        .map_err(|e| StoreError::Custom(format!("Frontmatter error: {e}")))?;

    Ok(Item {
        id: id.to_string(),
        item_type: config.plural.clone(),
        title,
        body,
        frontmatter,
    })
}

/// List items with optional filters.
pub async fn list(
    type_dir: &Path,
    config: &TypeConfig,
    filters: Filters,
) -> Result<Vec<Item>, StoreError> {
    if !type_dir.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    let mut entries = fs::read_dir(type_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        if !is_item_file(&name) {
            continue;
        }

        let id = name.trim_end_matches(".md").to_string();
        let content = match fs::read_to_string(entry.path()).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (frontmatter, title, body) = match parse_frontmatter::<Frontmatter>(&content) {
            Ok(result) => result,
            Err(_) => continue, // Skip malformed files
        };

        items.push(Item {
            id,
            item_type: config.plural.clone(),
            title,
            body,
            frontmatter,
        });
    }

    // Apply filters
    let items = apply_filters(items, &filters);

    Ok(items)
}

/// Apply filters to a list of items.
fn apply_filters(mut items: Vec<Item>, filters: &Filters) -> Vec<Item> {
    // Filter out soft-deleted unless include_deleted
    if !filters.include_deleted {
        items.retain(|item| item.frontmatter.deleted_at.is_none());
    }

    // Filter by status
    if let Some(ref status_filter) = filters.status {
        items.retain(|item| {
            item.frontmatter
                .status
                .as_ref()
                .is_some_and(|s| s.eq_ignore_ascii_case(status_filter))
        });
    }

    // Filter by priority
    if let Some(priority_filter) = filters.priority {
        items.retain(|item| item.frontmatter.priority == Some(priority_filter));
    }

    // Sort by display_number (if present), then by created_at
    items.sort_by(
        |a, b| match (a.frontmatter.display_number, b.frontmatter.display_number) {
            (Some(an), Some(bn)) => an.cmp(&bn),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.frontmatter.created_at.cmp(&b.frontmatter.created_at),
        },
    );

    // Apply offset
    if let Some(offset) = filters.offset {
        if offset < items.len() {
            items = items.split_off(offset);
        } else {
            items.clear();
        }
    }

    // Apply limit
    if let Some(limit) = filters.limit {
        items.truncate(limit);
    }

    items
}

/// Update an existing item.
pub async fn update(
    type_dir: &Path,
    config: &TypeConfig,
    id: &str,
    options: UpdateOptions,
) -> Result<Item, StoreError> {
    let file_path = type_dir.join(format!("{id}.md"));

    if !file_path.exists() {
        return Err(StoreError::NotFound(id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    let (mut frontmatter, current_title, current_body) =
        parse_frontmatter::<Frontmatter>(&content)
            .map_err(|e| StoreError::Custom(format!("Frontmatter error: {e}")))?;

    // Check if item is soft-deleted
    if frontmatter.deleted_at.is_some() {
        return Err(StoreError::IsDeleted(id.to_string()));
    }

    // Update status if provided
    if let Some(ref new_status) = options.status {
        validate_status(config, new_status)?;
        frontmatter.status = Some(new_status.clone());
    }

    // Update priority if provided
    if let Some(new_priority) = options.priority {
        validate_priority(config, new_priority)?;
        frontmatter.priority = Some(new_priority);
    }

    // Merge custom fields
    for (key, value) in &options.custom_fields {
        frontmatter.custom_fields.insert(key.clone(), value.clone());
    }

    frontmatter.updated_at = now_iso();

    let title = options.title.unwrap_or(current_title);
    let body = options.body.unwrap_or(current_body);

    // Write updated file
    let new_content = generate_frontmatter(&frontmatter, &title, &body);
    fs::write(&file_path, &new_content).await?;

    Ok(Item {
        id: id.to_string(),
        item_type: config.plural.clone(),
        title,
        body,
        frontmatter,
    })
}

/// Delete an item (hard delete).
///
/// If the item is not yet soft-deleted and `force` is false, this performs a
/// soft delete instead. If the item is already soft-deleted (or `force` is
/// true), this removes the file permanently.
pub async fn delete(type_dir: &Path, id: &str, force: bool) -> Result<(), StoreError> {
    let file_path = type_dir.join(format!("{id}.md"));

    if !file_path.exists() {
        return Err(StoreError::NotFound(id.to_string()));
    }

    // If not forcing, soft-delete instead
    if !force {
        // Check if already soft-deleted; if so, hard delete
        let content = fs::read_to_string(&file_path).await?;
        let (frontmatter, _, _) = parse_frontmatter::<Frontmatter>(&content)
            .map_err(|e| StoreError::Custom(format!("Frontmatter error: {e}")))?;

        if frontmatter.deleted_at.is_none() {
            // Soft delete instead
            return soft_delete(type_dir, id).await;
        }
    }

    // Hard delete: remove the file
    fs::remove_file(&file_path).await?;

    Ok(())
}

/// Soft-delete an item by setting the `deleted_at` timestamp.
pub async fn soft_delete(type_dir: &Path, id: &str) -> Result<(), StoreError> {
    let file_path = type_dir.join(format!("{id}.md"));

    if !file_path.exists() {
        return Err(StoreError::NotFound(id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    let (mut frontmatter, title, body) = parse_frontmatter::<Frontmatter>(&content)
        .map_err(|e| StoreError::Custom(format!("Frontmatter error: {e}")))?;

    if frontmatter.deleted_at.is_some() {
        return Err(StoreError::IsDeleted(id.to_string()));
    }

    frontmatter.deleted_at = Some(now_iso());
    frontmatter.updated_at = now_iso();

    let new_content = generate_frontmatter(&frontmatter, &title, &body);
    fs::write(&file_path, &new_content).await?;

    Ok(())
}

/// Restore a soft-deleted item by clearing the `deleted_at` timestamp.
pub async fn restore(type_dir: &Path, id: &str) -> Result<(), StoreError> {
    let file_path = type_dir.join(format!("{id}.md"));

    if !file_path.exists() {
        return Err(StoreError::NotFound(id.to_string()));
    }

    let content = fs::read_to_string(&file_path).await?;
    let (mut frontmatter, title, body) = parse_frontmatter::<Frontmatter>(&content)
        .map_err(|e| StoreError::Custom(format!("Frontmatter error: {e}")))?;

    if frontmatter.deleted_at.is_none() {
        return Err(StoreError::Custom(format!("Item '{id}' is not deleted")));
    }

    frontmatter.deleted_at = None;
    frontmatter.updated_at = now_iso();

    let new_content = generate_frontmatter(&frontmatter, &title, &body);
    fs::write(&file_path, &new_content).await?;

    Ok(())
}

/// Duplicate an item to the same or different directory.
///
/// Creates a copy of the item with a new ID and fresh timestamps.
/// For UUID-identified types, a new UUID is generated.
/// For slug-identified types, the default new ID is `"{id}-copy"`.
pub async fn duplicate(
    config: &TypeConfig,
    options: DuplicateOptions,
) -> Result<DuplicateResult, StoreError> {
    // Read source item
    let source_item = get(&options.source_dir, config, &options.item_id).await?;

    // Generate new ID
    let new_id = match options.new_id {
        Some(ref id) if !id.is_empty() => {
            if config.identifier == IdStrategy::Slug {
                slug::slugify(id)
            } else {
                id.clone()
            }
        }
        _ => {
            if config.identifier == IdStrategy::Slug {
                format!("{}-copy", options.item_id)
            } else {
                uuid::Uuid::new_v4().to_string()
            }
        }
    };

    // Check for existing item in target
    fs::create_dir_all(&options.target_dir).await?;
    let target_file = options.target_dir.join(format!("{new_id}.md"));
    if target_file.exists() {
        return Err(StoreError::AlreadyExists(new_id));
    }

    // Validate status/priority against config
    if let Some(ref status) = source_item.frontmatter.status {
        validate_status(config, status)?;
    }
    if let Some(priority) = source_item.frontmatter.priority {
        validate_priority(config, priority)?;
    }

    // Assign display number if enabled
    let display_number = if config.features.display_number {
        Some(get_next_display_number(&options.target_dir).await?)
    } else {
        None
    };

    // Prepare title
    let new_title = options
        .new_title
        .unwrap_or_else(|| format!("Copy of {}", source_item.title));

    // Create new frontmatter with fresh timestamps
    let now = now_iso();
    let frontmatter = Frontmatter {
        display_number,
        status: source_item.frontmatter.status.clone(),
        priority: source_item.frontmatter.priority,
        created_at: now.clone(),
        updated_at: now,
        deleted_at: None,
        custom_fields: source_item.frontmatter.custom_fields.clone(),
    };

    // Write new item file
    let content = generate_frontmatter(&frontmatter, &new_title, &source_item.body);
    fs::write(&target_file, &content).await?;

    Ok(DuplicateResult {
        item: Item {
            id: new_id,
            item_type: config.plural.clone(),
            title: new_title,
            body: source_item.body,
            frontmatter,
        },
        original_id: options.item_id,
    })
}

/// Move an item from one directory to another.
///
/// Uses raw YAML preservation so that type-specific fields (e.g., `draft`,
/// `isOrgIssue`, `orgSlug`) are kept intact without the generic layer needing
/// to know about them.
pub async fn move_item(
    source_dir: &Path,
    target_dir: &Path,
    source_config: &TypeConfig,
    target_config: &TypeConfig,
    item_id: &str,
    new_id: Option<&str>,
) -> Result<MoveResult, StoreError> {
    // 1. Check move feature is enabled
    if !source_config.features.move_item {
        return Err(StoreError::FeatureNotEnabled(format!(
            "move is not enabled for {}",
            source_config.plural
        )));
    }

    // 2. Same-location check
    let source_canonical =
        std::fs::canonicalize(source_dir).unwrap_or_else(|_| source_dir.to_path_buf());
    let target_canonical =
        std::fs::canonicalize(target_dir).unwrap_or_else(|_| target_dir.to_path_buf());
    if source_canonical == target_canonical {
        return Err(StoreError::SameLocation);
    }

    // 3. Read source file
    let source_file = source_dir.join(format!("{item_id}.md"));
    if !source_file.exists() {
        return Err(StoreError::NotFound(item_id.to_string()));
    }
    let content = fs::read_to_string(&source_file).await?;

    // Parse as Frontmatter for validation
    let (frontmatter, _title, _body) = parse_frontmatter::<Frontmatter>(&content)
        .map_err(|e| StoreError::FrontmatterError(e.to_string()))?;

    // 4. Validate status against target config
    if target_config.features.status {
        if let Some(ref status) = frontmatter.status {
            validate_status(target_config, status)?;
        }
    }

    // 5. Validate priority against target config
    if target_config.features.priority {
        if let Some(priority) = frontmatter.priority {
            validate_priority(target_config, priority)?;
        }
    }

    // 6. Determine target ID
    let target_id = if source_config.identifier == IdStrategy::Slug {
        new_id.unwrap_or(item_id).to_string()
    } else {
        // UUID-based: keep the same ID
        item_id.to_string()
    };

    // 7. Check target doesn't already exist
    let target_file = target_dir.join(format!("{target_id}.md"));
    if target_file.exists() {
        return Err(StoreError::AlreadyExists(target_id));
    }

    // 8. Parse raw YAML for field-preserving copy
    let (mut yaml_value, title, body) =
        parse_frontmatter_raw(&content).map_err(|e| StoreError::FrontmatterError(e.to_string()))?;

    // 9. If display_number feature: get next display number and update YAML
    if target_config.features.display_number {
        fs::create_dir_all(target_dir).await?;
        let next_dn = get_next_display_number(target_dir).await?;
        if let serde_yaml::Value::Mapping(ref mut map) = yaml_value {
            map.insert(
                serde_yaml::Value::String("displayNumber".to_string()),
                serde_yaml::Value::Number(next_dn.into()),
            );
        }
    }

    // 10. Update updatedAt in YAML
    if let serde_yaml::Value::Mapping(ref mut map) = yaml_value {
        map.insert(
            serde_yaml::Value::String("updatedAt".to_string()),
            serde_yaml::Value::String(now_iso()),
        );
    }

    // 11. Write to target
    fs::create_dir_all(target_dir).await?;
    let new_content = generate_frontmatter_raw(&yaml_value, &title, &body);
    fs::write(&target_file, &new_content).await?;

    // 12. Delete source file
    fs::remove_file(&source_file).await?;

    // 13. Read and return the moved item
    let moved_item = get(target_dir, target_config, &target_id).await?;

    Ok(MoveResult {
        item: moved_item,
        old_id: item_id.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TypeFeatures;
    use std::collections::HashMap;

    fn issue_config() -> TypeConfig {
        TypeConfig {
            name: "Issue".to_string(),
            plural: "issues".to_string(),
            identifier: IdStrategy::Uuid,
            features: TypeFeatures {
                display_number: true,
                status: true,
                priority: true,
                assets: false,
                org_sync: false,
                move_item: true,
                duplicate: true,
            },
            statuses: vec![
                "open".to_string(),
                "planning".to_string(),
                "in-progress".to_string(),
                "closed".to_string(),
            ],
            default_status: Some("open".to_string()),
            priority_levels: Some(3),
            custom_fields: Vec::new(),
        }
    }

    fn minimal_config() -> TypeConfig {
        TypeConfig {
            name: "Note".to_string(),
            plural: "notes".to_string(),
            identifier: IdStrategy::Uuid,
            features: TypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_create_and_get() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let options = CreateOptions {
            title: "Test Issue".to_string(),
            body: "This is a test.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = create(&type_dir, &config, options).await.unwrap();
        assert_eq!(created.title, "Test Issue");
        assert_eq!(created.body, "This is a test.");
        assert_eq!(created.frontmatter.display_number, Some(1));
        assert_eq!(created.frontmatter.status, Some("open".to_string()));
        assert_eq!(created.frontmatter.priority, Some(2));

        // Get it back
        let fetched = get(&type_dir, &config, &created.id).await.unwrap();
        assert_eq!(fetched.title, "Test Issue");
        assert_eq!(fetched.frontmatter.display_number, Some(1));
    }

    #[tokio::test]
    async fn test_create_minimal_features() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("notes");

        let config = minimal_config();
        let options = CreateOptions {
            title: "Simple Note".to_string(),
            body: "Just a note.".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        let created = create(&type_dir, &config, options).await.unwrap();
        assert!(created.frontmatter.display_number.is_none());
        assert!(created.frontmatter.status.is_none());
        assert!(created.frontmatter.priority.is_none());
    }

    #[tokio::test]
    async fn test_create_slug_id_strategy() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("docs");

        let mut config = minimal_config();
        config.identifier = IdStrategy::Slug;
        config.plural = "docs".to_string();

        let options = CreateOptions {
            title: "Getting Started Guide".to_string(),
            body: "Welcome!".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        let created = create(&type_dir, &config, options).await.unwrap();
        assert_eq!(created.id, "getting-started-guide");
    }

    #[tokio::test]
    async fn test_create_invalid_status() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let options = CreateOptions {
            title: "Bad Status".to_string(),
            body: String::new(),
            id: None,
            status: Some("nonexistent".to_string()),
            priority: None,
            custom_fields: HashMap::new(),
        };

        let result = create(&type_dir, &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(StoreError::InvalidStatus { .. })));
    }

    #[tokio::test]
    async fn test_create_invalid_priority() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let options = CreateOptions {
            title: "Bad Priority".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(99),
            custom_fields: HashMap::new(),
        };

        let result = create(&type_dir, &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(StoreError::InvalidPriority { .. })));
    }

    #[tokio::test]
    async fn test_list_with_filters() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();

        // Create multiple items
        for (title, status) in [
            ("Open 1", "open"),
            ("Open 2", "open"),
            ("Closed 1", "closed"),
        ] {
            let options = CreateOptions {
                title: title.to_string(),
                body: String::new(),
                id: None,
                status: Some(status.to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            };
            create(&type_dir, &config, options).await.unwrap();
        }

        // List all
        let all = list(&type_dir, &config, Filters::default()).await.unwrap();
        assert_eq!(all.len(), 3);

        // List open only
        let open = list(&type_dir, &config, Filters::new().with_status("open"))
            .await
            .unwrap();
        assert_eq!(open.len(), 2);

        // List with limit
        let limited = list(&type_dir, &config, Filters::new().with_limit(1))
            .await
            .unwrap();
        assert_eq!(limited.len(), 1);

        // List with offset
        let offset = list(&type_dir, &config, Filters::new().with_offset(2))
            .await
            .unwrap();
        assert_eq!(offset.len(), 1);
    }

    #[tokio::test]
    async fn test_update() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let options = CreateOptions {
            title: "Original Title".to_string(),
            body: "Original body.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = create(&type_dir, &config, options).await.unwrap();

        let update_options = UpdateOptions {
            title: Some("Updated Title".to_string()),
            body: Some("Updated body.".to_string()),
            status: Some("closed".to_string()),
            priority: Some(1),
            custom_fields: HashMap::from([("env".to_string(), serde_json::json!("prod"))]),
        };

        let updated = update(&type_dir, &config, &created.id, update_options)
            .await
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.body, "Updated body.");
        assert_eq!(updated.frontmatter.status, Some("closed".to_string()));
        assert_eq!(updated.frontmatter.priority, Some(1));
        assert_eq!(
            updated.frontmatter.custom_fields.get("env"),
            Some(&serde_json::json!("prod"))
        );
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");
        fs::create_dir_all(&type_dir).await.unwrap();

        let config = issue_config();
        let result = update(
            &type_dir,
            &config,
            "nonexistent",
            UpdateOptions::default(),
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result, Err(StoreError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_soft_delete_and_restore() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let options = CreateOptions {
            title: "To Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = create(&type_dir, &config, options).await.unwrap();

        // Soft delete
        soft_delete(&type_dir, &created.id).await.unwrap();

        // Should not appear in default list
        let items = list(&type_dir, &config, Filters::default()).await.unwrap();
        assert!(items.is_empty());

        // Should appear with include_deleted
        let items = list(&type_dir, &config, Filters::new().include_deleted())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.first().unwrap().frontmatter.deleted_at.is_some());

        // Restore
        restore(&type_dir, &created.id).await.unwrap();

        // Should appear again
        let items = list(&type_dir, &config, Filters::default()).await.unwrap();
        assert_eq!(items.len(), 1);
        assert!(items.first().unwrap().frontmatter.deleted_at.is_none());
    }

    #[tokio::test]
    async fn test_hard_delete() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let options = CreateOptions {
            title: "To Hard Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = create(&type_dir, &config, options).await.unwrap();

        // Force hard delete
        delete(&type_dir, &created.id, true).await.unwrap();

        // Should not exist at all
        let result = get(&type_dir, &config, &created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_display_number_auto_increment() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();

        for i in 1..=3u32 {
            let options = CreateOptions {
                title: format!("Issue {i}"),
                body: String::new(),
                id: None,
                status: Some("open".to_string()),
                priority: Some(2),
                custom_fields: HashMap::new(),
            };

            let created = create(&type_dir, &config, options).await.unwrap();
            assert_eq!(created.frontmatter.display_number, Some(i));
        }
    }

    #[tokio::test]
    async fn test_update_preserves_fields() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let options = CreateOptions {
            title: "Keep Fields".to_string(),
            body: "Original body.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(1),
            custom_fields: HashMap::from([("key".to_string(), serde_json::json!("value"))]),
        };

        let created = create(&type_dir, &config, options).await.unwrap();

        // Update only the title
        let updated = update(
            &type_dir,
            &config,
            &created.id,
            UpdateOptions {
                title: Some("New Title".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.body, "Original body.");
        assert_eq!(updated.frontmatter.status, Some("open".to_string()));
        assert_eq!(updated.frontmatter.priority, Some(1));
        assert_eq!(
            updated.frontmatter.custom_fields.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[tokio::test]
    async fn test_cannot_update_deleted_item() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let options = CreateOptions {
            title: "Will Delete".to_string(),
            body: String::new(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };

        let created = create(&type_dir, &config, options).await.unwrap();
        soft_delete(&type_dir, &created.id).await.unwrap();

        let result = update(
            &type_dir,
            &config,
            &created.id,
            UpdateOptions {
                title: Some("Fail".to_string()),
                ..Default::default()
            },
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result, Err(StoreError::IsDeleted(_))));
    }

    #[tokio::test]
    async fn test_already_exists() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("notes");

        let mut config = minimal_config();
        config.identifier = IdStrategy::Slug;

        let options = CreateOptions {
            title: "Same Title".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        };

        create(&type_dir, &config, options.clone()).await.unwrap();

        let result = create(&type_dir, &config, options).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(StoreError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");
        fs::create_dir_all(&type_dir).await.unwrap();

        let config = issue_config();
        let result = get(&type_dir, &config, "nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(StoreError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_list_empty() {
        let temp = tempfile::tempdir().unwrap();
        let type_dir = temp.path().join("issues");

        let config = issue_config();
        let items = list(&type_dir, &config, Filters::default()).await.unwrap();
        assert!(items.is_empty());
    }
}
