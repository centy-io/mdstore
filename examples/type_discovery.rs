//! Discovering item types from a directory layout.
//!
//! Demonstrates scanning a base directory for type configs and then
//! performing operations on the discovered types.
//!
//! Run with: cargo run --example type_discovery

use mdstore::config::{write_type_config, IdStrategy};
use mdstore::{discover_types, CreateOptions, Filters, TypeConfig, TypeFeatures};
use std::collections::HashMap;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/type_discovery");
    if base_dir.exists() {
        tokio::fs::remove_dir_all(&base_dir).await?;
    }
    tokio::fs::create_dir_all(&base_dir).await?;

    // Set up multiple item types
    let issue_config = TypeConfig {
        name: "Issue".to_string(),
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
        statuses: vec!["open".to_string(), "closed".to_string()],
        default_status: Some("open".to_string()),
        priority_levels: Some(3),
        custom_fields: Vec::new(),
    };

    let epic_config = TypeConfig {
        name: "Epic".to_string(),
        identifier: IdStrategy::Slug,
        features: TypeFeatures {
            display_number: true,
            status: true,
            priority: false,
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
    };

    let doc_config = TypeConfig {
        name: "Doc".to_string(),
        identifier: IdStrategy::Slug,
        features: TypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    // Write configs to disk -- directory names are chosen by the caller
    write_type_config(&base_dir.join("issues"), &issue_config).await?;
    write_type_config(&base_dir.join("epics"), &epic_config).await?;
    write_type_config(&base_dir.join("docs"), &doc_config).await?;
    println!("Wrote 3 type configs to {}", base_dir.display());

    // Discover all types
    let types = discover_types(&base_dir).await?;
    println!("\nDiscovered {} item types:", types.len());
    for t in &types {
        println!(
            "  {} -- identifier: {}, statuses: {:?}",
            t.name, t.identifier, t.statuses
        );
    }

    // Create items in each discovered type
    // The directory name matches the folder that was scanned by discover_types
    let type_dirs = ["issues", "epics", "docs"];
    for (t, dir_name) in types.iter().zip(type_dirs.iter()) {
        let type_dir = base_dir.join(dir_name);
        let item = mdstore::create(
            &type_dir,
            t,
            CreateOptions {
                title: format!("Sample {}", t.name),
                body: format!("This is a sample {} item.", t.name.to_lowercase()),
                id: None,
                status: t.default_status.clone(),
                priority: if t.features.priority { Some(2) } else { None },
                tags: None,
                custom_fields: HashMap::new(),
                comment: None,
            },
        )
        .await?;
        println!("\nCreated {} '{}' (id: {})", t.name, item.title, item.id);
    }

    // List items across all types
    println!("\n--- All items ---");
    for (t, dir_name) in types.iter().zip(type_dirs.iter()) {
        let type_dir = base_dir.join(dir_name);
        let items = mdstore::list(&type_dir, Filters::default()).await?;
        for item in items {
            println!(
                "  [{}] {} ({})",
                t.name,
                item.title,
                item.frontmatter.status.as_deref().unwrap_or("no status")
            );
        }
    }

    // Show the on-disk layout
    println!("\nOn-disk layout:");
    print_tree(&base_dir, 0).await;

    Ok(())
}

async fn print_tree(dir: &Path, depth: usize) {
    let indent = "  ".repeat(depth);
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        let mut names = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            names.push((
                entry.file_name().to_string_lossy().to_string(),
                entry
                    .file_type()
                    .await
                    .map(|ft| ft.is_dir())
                    .unwrap_or(false),
            ));
        }
        names.sort();
        for (name, is_dir) in names {
            if is_dir {
                println!("{indent}{name}/");
                Box::pin(print_tree(&dir.join(&name), depth.saturating_add(1))).await;
            } else {
                println!("{indent}{name}");
            }
        }
    }
}
