//! Type configuration for item types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use thiserror::Error;
use tokio::fs;
use tracing::error;

/// Strategy for generating item identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdStrategy {
    /// UUID-based identifiers for conflict-free distributed creation.
    Uuid,
    /// Slug-based identifiers derived from the item title.
    Slug,
}

impl fmt::Display for IdStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdStrategy::Uuid => write!(f, "uuid"),
            IdStrategy::Slug => write!(f, "slug"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

/// Custom field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeFeatures {
    pub display_number: bool,
    pub status: bool,
    pub priority: bool,
    pub assets: bool,
    pub org_sync: bool,
    #[serde(rename = "move")]
    pub move_item: bool,
    pub duplicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeConfig {
    pub name: String,
    pub identifier: IdStrategy,
    pub features: TypeFeatures,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority_levels: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_fields: Vec<CustomFieldDef>,
}

/// Read a type config from `{type_dir}/config.yaml`.
pub async fn read_type_config(type_dir: &Path) -> Result<Option<TypeConfig>, ConfigError> {
    let config_path = type_dir.join("config.yaml");

    if !config_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&config_path).await?;
    let config: TypeConfig = serde_yaml::from_str(&content)?;
    Ok(Some(config))
}

/// Write a type config to `{type_dir}/config.yaml`.
pub async fn write_type_config(type_dir: &Path, config: &TypeConfig) -> Result<(), ConfigError> {
    fs::create_dir_all(type_dir).await?;

    let config_path = type_dir.join("config.yaml");
    let content = serde_yaml::to_string(config)?;
    fs::write(&config_path, content).await?;
    Ok(())
}

/// Discover all item types, returning a map of folder name to config.
///
/// Scans `{base_dir}/*/config.yaml` and returns a `HashMap` keyed by
/// the subdirectory name. Malformed configs are logged and skipped.
pub async fn discover_types_map(
    base_dir: &Path,
) -> Result<HashMap<String, TypeConfig>, ConfigError> {
    if !base_dir.exists() {
        return Ok(HashMap::new());
    }

    let mut configs = HashMap::new();
    let mut entries = fs::read_dir(base_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }

        let config_path = entry.path().join("config.yaml");
        if !config_path.exists() {
            continue;
        }

        let folder_name = entry.file_name().to_string_lossy().to_string();

        let content = match fs::read_to_string(&config_path).await {
            Ok(c) => c,
            Err(e) => {
                error!(folder = %folder_name, error = %e, "Failed to read config.yaml, skipping type");
                continue;
            }
        };

        match serde_yaml::from_str::<TypeConfig>(&content) {
            Ok(config) => {
                configs.insert(folder_name, config);
            }
            Err(e) => {
                error!(folder = %folder_name, error = %e, "Malformed config.yaml, skipping type");
            }
        }
    }

    Ok(configs)
}

/// Discover all item types by scanning `{base_dir}/*/config.yaml`.
///
/// Returns a list of `TypeConfig` for each subdirectory that contains
/// a valid `config.yaml`. Malformed configs are logged and skipped.
pub async fn discover_types(base_dir: &Path) -> Result<Vec<TypeConfig>, ConfigError> {
    Ok(discover_types_map(base_dir)
        .await?
        .into_values()
        .collect())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_config() -> TypeConfig {
        TypeConfig {
            name: "Issue".to_string(),

            identifier: IdStrategy::Uuid,
            features: TypeFeatures {
                display_number: true,
                status: true,
                priority: true,
                assets: true,
                org_sync: true,
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
            name: "Doc".to_string(),

            identifier: IdStrategy::Slug,
            features: TypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
        }
    }

    #[test]
    fn test_issue_config_yaml_serialization() {
        let config = sample_config();
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");

        assert!(yaml.contains("name: Issue"));
        assert!(!yaml.contains("plural"));
        assert!(yaml.contains("identifier: uuid"));
        assert!(yaml.contains("displayNumber: true"));
        assert!(yaml.contains("move: true"));
        assert!(yaml.contains("defaultStatus: open"));
    }

    #[test]
    fn test_doc_config_yaml_serialization() {
        let config = minimal_config();
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");

        assert!(yaml.contains("name: Doc"));
        assert!(!yaml.contains("plural"));
        assert!(yaml.contains("identifier: slug"));
        assert!(yaml.contains("displayNumber: false"));
        // Should NOT have optional fields
        assert!(!yaml.contains("statuses"));
        assert!(!yaml.contains("defaultStatus"));
        assert!(!yaml.contains("priorityLevels"));
    }

    #[test]
    fn test_type_config_yaml_roundtrip() {
        let config = sample_config();
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");
        let deserialized: TypeConfig = serde_yaml::from_str(&yaml).expect("Should deserialize");

        assert_eq!(deserialized.name, "Issue");
        assert_eq!(deserialized.statuses.len(), config.statuses.len());
        assert_eq!(deserialized.default_status, config.default_status);
        assert_eq!(deserialized.priority_levels, config.priority_levels);
    }

    #[tokio::test]
    async fn test_read_type_config_nonexistent() {
        let temp = tempdir().expect("Should create temp dir");
        let type_dir = temp.path().join("issues");
        let result = read_type_config(&type_dir).await.expect("Should not error");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_write_and_read_type_config() {
        let temp = tempdir().expect("Should create temp dir");
        let type_dir = temp.path().join("issues");

        let config = sample_config();
        write_type_config(&type_dir, &config)
            .await
            .expect("Should write");

        let read = read_type_config(&type_dir)
            .await
            .expect("Should read")
            .expect("Should exist");

        assert_eq!(read.name, "Issue");
        assert_eq!(read.statuses, config.statuses);
    }

    #[tokio::test]
    async fn test_discover_types_empty() {
        let temp = tempdir().expect("Should create temp dir");
        let configs = discover_types(temp.path()).await.expect("Should discover");
        assert!(configs.is_empty());
    }

    #[tokio::test]
    async fn test_discover_types_nonexistent() {
        let temp = tempdir().expect("Should create temp dir");
        let nonexistent = temp.path().join("does-not-exist");
        let configs = discover_types(&nonexistent).await.expect("Should discover");
        assert!(configs.is_empty());
    }

    #[tokio::test]
    async fn test_discover_types_with_configs() {
        let temp = tempdir().expect("Should create temp dir");

        // Create issues config
        let issues_dir = temp.path().join("issues");
        write_type_config(&issues_dir, &sample_config())
            .await
            .expect("Should write");

        // Create docs config
        let docs_dir = temp.path().join("docs");
        write_type_config(&docs_dir, &minimal_config())
            .await
            .expect("Should write");

        let configs = discover_types(temp.path()).await.expect("Should discover");
        assert_eq!(configs.len(), 2);
    }

    #[tokio::test]
    async fn test_discover_types_skips_dirs_without_config() {
        let temp = tempdir().expect("Should create temp dir");

        // Create one valid config
        let issues_dir = temp.path().join("issues");
        write_type_config(&issues_dir, &sample_config())
            .await
            .expect("Should write");

        // Create directories without config.yaml
        fs::create_dir_all(temp.path().join("assets"))
            .await
            .unwrap();
        fs::create_dir_all(temp.path().join("templates"))
            .await
            .unwrap();

        let configs = discover_types(temp.path()).await.expect("Should discover");
        assert_eq!(configs.len(), 1);
    }

    #[tokio::test]
    async fn test_discover_types_skips_malformed_yaml() {
        let temp = tempdir().expect("Should create temp dir");

        // Create a valid config
        let issues_dir = temp.path().join("issues");
        write_type_config(&issues_dir, &sample_config())
            .await
            .expect("Should write");

        // Create a malformed config.yaml
        let bad_dir = temp.path().join("broken");
        fs::create_dir_all(&bad_dir).await.unwrap();
        fs::write(
            bad_dir.join("config.yaml"),
            "this is: [not: valid: yaml: {{",
        )
        .await
        .unwrap();

        let configs = discover_types(temp.path()).await.expect("Should discover");

        // Should have only the valid config
        assert_eq!(configs.len(), 1);
    }

    #[test]
    fn test_custom_field_def_serialization() {
        let field = CustomFieldDef {
            name: "environment".to_string(),
            field_type: "enum".to_string(),
            required: true,
            default_value: Some("dev".to_string()),
            enum_values: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
        };

        let json = serde_json::to_string(&field).expect("Should serialize");
        let deserialized: CustomFieldDef = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.name, "environment");
        assert_eq!(deserialized.field_type, "enum");
        assert!(deserialized.required);
        assert_eq!(deserialized.default_value, Some("dev".to_string()));
        assert_eq!(deserialized.enum_values.len(), 3);
    }

    #[test]
    fn test_custom_field_def_json_uses_camel_case() {
        let field = CustomFieldDef {
            name: "test".to_string(),
            field_type: "string".to_string(),
            required: false,
            default_value: None,
            enum_values: vec![],
        };

        let json = serde_json::to_string(&field).expect("Should serialize");

        assert!(json.contains("\"type\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"required\""));
        assert!(!json.contains("field_type"));
    }

    #[test]
    fn test_yaml_with_legacy_plural_field_deserializes() {
        let yaml = "name: Task\nplural: tasks\nidentifier: uuid\nfeatures:\n  displayNumber: false\n  status: false\n  priority: false\n  assets: false\n  orgSync: false\n  move: false\n  duplicate: false\n";
        let config: TypeConfig = serde_yaml::from_str(yaml).expect("Should deserialize");
        assert_eq!(config.name, "Task");
    }

    // ─── discover_types_map tests ────────────────────────────────────────────

    #[tokio::test]
    async fn test_discover_types_map_with_configs() {
        let temp = tempdir().expect("Should create temp dir");

        // Create issues config
        let issues_dir = temp.path().join("issues");
        write_type_config(&issues_dir, &sample_config())
            .await
            .expect("Should write");

        // Create docs config
        let docs_dir = temp.path().join("docs");
        write_type_config(&docs_dir, &minimal_config())
            .await
            .expect("Should write");

        let map = discover_types_map(temp.path())
            .await
            .expect("Should discover");
        assert_eq!(map.len(), 2);

        let issues = map.get("issues").expect("Should have issues key");
        assert_eq!(issues.name, "Issue");

        let docs = map.get("docs").expect("Should have docs key");
        assert_eq!(docs.name, "Doc");
    }

    #[tokio::test]
    async fn test_discover_types_map_empty() {
        let temp = tempdir().expect("Should create temp dir");
        let map = discover_types_map(temp.path())
            .await
            .expect("Should discover");
        assert!(map.is_empty());
    }

    #[tokio::test]
    async fn test_discover_types_map_skips_malformed() {
        let temp = tempdir().expect("Should create temp dir");

        // Create a valid config
        let issues_dir = temp.path().join("issues");
        write_type_config(&issues_dir, &sample_config())
            .await
            .expect("Should write");

        // Create a malformed config.yaml
        let bad_dir = temp.path().join("broken");
        fs::create_dir_all(&bad_dir).await.unwrap();
        fs::write(
            bad_dir.join("config.yaml"),
            "this is: [not: valid: yaml: {{",
        )
        .await
        .unwrap();

        let map = discover_types_map(temp.path())
            .await
            .expect("Should discover");

        assert_eq!(map.len(), 1);
        assert!(map.contains_key("issues"));
        assert!(!map.contains_key("broken"));
    }
}
