//! Discover all item types in a directory.
//!
//! Demonstrates using `discover_types` to scan a base directory and
//! return the `TypeConfig` for every subdirectory that contains a
//! valid `config.yaml`.
//!
//! Run with: cargo run --example discover_types

use mdstore::config::{write_type_config, CustomFieldDef, IdStrategy};
use mdstore::{discover_types, TypeConfig, TypeFeatures};
use std::path::Path;

fn issue_config() -> TypeConfig {
    TypeConfig {
        name: "Issue".to_string(),
        identifier: IdStrategy::Uuid,
        features: TypeFeatures {
            display_number: true,
            status: true,
            priority: true,
            ..TypeFeatures::default()
        },
        statuses: vec![
            "open".to_string(),
            "in-progress".to_string(),
            "closed".to_string(),
        ],
        default_status: Some("open".to_string()),
        priority_levels: Some(3),
        custom_fields: Vec::new(),
    }
}

fn doc_config() -> TypeConfig {
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

fn epic_config() -> TypeConfig {
    TypeConfig {
        name: "Epic".to_string(),
        identifier: IdStrategy::Slug,
        features: TypeFeatures {
            display_number: true,
            status: true,
            ..TypeFeatures::default()
        },
        statuses: vec![
            "draft".to_string(),
            "active".to_string(),
            "completed".to_string(),
        ],
        default_status: Some("draft".to_string()),
        priority_levels: None,
        custom_fields: Vec::new(),
    }
}

fn task_config() -> TypeConfig {
    TypeConfig {
        name: "Task".to_string(),
        identifier: IdStrategy::Uuid,
        features: TypeFeatures {
            display_number: true,
            status: true,
            priority: true,
            move_item: true,
            duplicate: true,
            ..TypeFeatures::default()
        },
        statuses: vec![
            "todo".to_string(),
            "in-progress".to_string(),
            "review".to_string(),
            "done".to_string(),
        ],
        default_status: Some("todo".to_string()),
        priority_levels: Some(3),
        custom_fields: vec![CustomFieldDef {
            name: "assignee".to_string(),
            field_type: "string".to_string(),
            required: false,
            default_value: None,
            enum_values: Vec::new(),
        }],
    }
}

fn rfc_config() -> TypeConfig {
    TypeConfig {
        name: "RFC".to_string(),
        identifier: IdStrategy::Slug,
        features: TypeFeatures {
            display_number: true,
            status: true,
            ..TypeFeatures::default()
        },
        statuses: vec![
            "proposed".to_string(),
            "accepted".to_string(),
            "rejected".to_string(),
            "superseded".to_string(),
        ],
        default_status: Some("proposed".to_string()),
        priority_levels: None,
        custom_fields: Vec::new(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/discover_types");
    if base_dir.exists() {
        tokio::fs::remove_dir_all(&base_dir).await?;
    }

    // Seed type directories
    write_type_config(&base_dir.join("issues"), &issue_config()).await?;
    write_type_config(&base_dir.join("docs"), &doc_config()).await?;
    write_type_config(&base_dir.join("epics"), &epic_config()).await?;
    write_type_config(&base_dir.join("tasks"), &task_config()).await?;
    write_type_config(&base_dir.join("rfcs"), &rfc_config()).await?;

    // A directory without config.yaml -- will be skipped
    tokio::fs::create_dir_all(base_dir.join("assets")).await?;

    // Discover all types
    let types = discover_types(&base_dir).await?;

    println!(
        "Discovered {} type(s) in {}:",
        types.len(),
        base_dir.display()
    );
    for t in &types {
        println!(
            "  - {} (identifier: {}, statuses: {:?})",
            t.name, t.identifier, t.statuses
        );
    }

    Ok(())
}
