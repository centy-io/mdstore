//! Generic item types that work with any config-driven item type.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Generic frontmatter that adapts to the item type's features.
///
/// Optional fields are gated by `skip_serializing_if` so they don't appear
/// in the YAML output when the feature is disabled. This works with the
/// existing `parse_frontmatter<T>` / `generate_frontmatter<T>` generics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Frontmatter {
    /// Human-readable display number (only if features.displayNumber)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_number: Option<u32>,
    /// Item status (only if features.status)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Priority level, 1 = highest (only if features.priority)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    /// ISO timestamp when the item was created (always present)
    pub created_at: String,
    /// ISO timestamp when the item was last updated (always present)
    pub updated_at: String,
    /// ISO timestamp when soft-deleted (empty if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    /// Custom fields for extensibility
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// A generic item parsed from a `<type>/<id>.md` file.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// The item's identifier (UUID or slug)
    pub id: String,
    /// The item's title (from the H1 heading)
    pub title: String,
    /// The item's body content (markdown after the title)
    pub body: String,
    /// Parsed frontmatter metadata
    pub frontmatter: Frontmatter,
}

/// Options for creating a new item.
#[derive(Debug, Clone)]
pub struct CreateOptions {
    /// Title for the new item
    pub title: String,
    /// Body content (markdown)
    pub body: String,
    /// Optional explicit ID (if not provided, one is generated based on id_strategy)
    pub id: Option<String>,
    /// Initial status (validated if features.status is enabled)
    pub status: Option<String>,
    /// Initial priority (validated if features.priority is enabled)
    pub priority: Option<u32>,
    /// Custom fields
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Options for duplicating an item.
#[derive(Debug, Clone)]
pub struct DuplicateOptions {
    /// Path to the directory containing the source item type
    pub source_dir: PathBuf,
    /// Path to the target directory (can be same as source)
    pub target_dir: PathBuf,
    /// ID of the item to duplicate
    pub item_id: String,
    /// Override for the new item's ID (for slug-identified types)
    pub new_id: Option<String>,
    /// Override for the new item's title (default: "Copy of {original}")
    pub new_title: Option<String>,
}

/// Result of duplicating an item.
#[derive(Debug, Clone)]
pub struct DuplicateResult {
    /// The newly created duplicate item
    pub item: Item,
    /// ID of the original item that was duplicated
    pub original_id: String,
}

/// Options for updating an existing item.
#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    /// New title (None = keep current)
    pub title: Option<String>,
    /// New body (None = keep current)
    pub body: Option<String>,
    /// New status (None = keep current, validated if features.status is enabled)
    pub status: Option<String>,
    /// New priority (None = keep current, validated if features.priority is enabled)
    pub priority: Option<u32>,
    /// Custom fields to merge (existing keys are overwritten, new keys are added)
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Options for moving an item to another location.
#[derive(Debug, Clone)]
pub struct MoveOptions {
    pub source_dir: PathBuf,
    pub target_dir: PathBuf,
    pub item_id: String,
    /// For slug-based items, optionally rename the item on move.
    pub new_id: Option<String>,
}

/// Result of moving an item.
#[derive(Debug, Clone)]
pub struct MoveResult {
    pub item: Item,
    pub old_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter_minimal_serialization() {
        let fm = Frontmatter {
            display_number: None,
            status: None,
            priority: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            deleted_at: None,
            custom_fields: HashMap::new(),
        };
        let yaml = serde_yaml::to_string(&fm).unwrap();
        // Should NOT contain optional fields that are None
        assert!(!yaml.contains("displayNumber"));
        assert!(!yaml.contains("status"));
        assert!(!yaml.contains("priority"));
        assert!(!yaml.contains("deletedAt"));
        assert!(!yaml.contains("customFields"));
        // Should contain required fields
        assert!(yaml.contains("createdAt"));
        assert!(yaml.contains("updatedAt"));
    }

    #[test]
    fn test_frontmatter_full_serialization() {
        let fm = Frontmatter {
            display_number: Some(42),
            status: Some("open".to_string()),
            priority: Some(2),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
            deleted_at: Some("2024-01-03T00:00:00Z".to_string()),
            custom_fields: HashMap::from([("env".to_string(), serde_json::json!("prod"))]),
        };
        let yaml = serde_yaml::to_string(&fm).unwrap();
        assert!(yaml.contains("displayNumber: 42"));
        assert!(yaml.contains("status: open"));
        assert!(yaml.contains("priority: 2"));
        assert!(yaml.contains("deletedAt"));
        assert!(yaml.contains("customFields"));
    }

    #[test]
    fn test_frontmatter_roundtrip() {
        let fm = Frontmatter {
            display_number: Some(1),
            status: Some("closed".to_string()),
            priority: Some(3),
            created_at: "2024-06-15T12:00:00Z".to_string(),
            updated_at: "2024-06-15T13:00:00Z".to_string(),
            deleted_at: None,
            custom_fields: HashMap::new(),
        };
        let yaml = serde_yaml::to_string(&fm).unwrap();
        let parsed: Frontmatter = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(fm, parsed);
    }

    #[test]
    fn test_create_options() {
        let opts = CreateOptions {
            title: "Test".to_string(),
            body: "Body".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(2),
            custom_fields: HashMap::new(),
        };
        assert_eq!(opts.title, "Test");
        assert!(opts.id.is_none());
    }

    #[test]
    fn test_update_options_default() {
        let opts = UpdateOptions::default();
        assert!(opts.title.is_none());
        assert!(opts.body.is_none());
        assert!(opts.status.is_none());
        assert!(opts.priority.is_none());
        assert!(opts.custom_fields.is_empty());
    }
}
