//! Discover all item types as a folder-keyed map.
//!
//! Demonstrates using `discover_types_map` to scan a base directory and
//! return a `HashMap<String, TypeConfig>` keyed by subdirectory name.
//!
//! Run with: cargo run --example discover_types_map

use mdstore::config::write_type_config;
use mdstore::{discover_types_map, IdStrategy, TypeConfig, TypeFeatures};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/discover_types_map");
    if base_dir.exists() {
        tokio::fs::remove_dir_all(&base_dir).await?;
    }

    // Seed type directories
    write_type_config(&base_dir.join("issues"), &issue_config()).await?;
    write_type_config(&base_dir.join("docs"), &doc_config()).await?;
    write_type_config(&base_dir.join("epics"), &epic_config()).await?;

    // A directory without config.yaml -- will be skipped
    tokio::fs::create_dir_all(base_dir.join("assets")).await?;

    // Discover all types as a folder-keyed map
    let type_map = discover_types_map(&base_dir).await?;

    println!(
        "Discovered {} type(s) in {}:\n",
        type_map.len(),
        base_dir.display()
    );
    for (folder, config) in &type_map {
        println!(
            "  folder: {:<10} name: {:<10} identifier: {}",
            folder, config.name, config.identifier
        );
    }

    // Look up a specific folder
    if let Some(config) = type_map.get("issues") {
        println!("\nLooked up 'issues' folder -> type name: {}", config.name);
    }

    Ok(())
}
