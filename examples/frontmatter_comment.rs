//! Demonstrates optional comment support at the top of frontmatter blocks.
//!
//! Run with: cargo run --example frontmatter_comment

use mdstore::config::IdStrategy;
use mdstore::{CreateOptions, TypeConfig, TypeFeatures, UpdateOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn config() -> TypeConfig {
    TypeConfig {
        name: "Doc".to_string(),
        identifier: IdStrategy::Uuid,
        features: TypeFeatures {
            display_number: false,
            status: false,
            priority: false,
            assets: false,
            org_sync: false,
            move_item: false,
            duplicate: false,
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    }
}

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/frontmatter_comment")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = output_dir();
    if base.exists() {
        tokio::fs::remove_dir_all(&base).await?;
    }
    let type_dir = base.join("docs");
    let config = config();

    // 1. Create an item with a frontmatter comment
    let comment = "# This file was auto-generated. Do not edit manually.";
    let item = mdstore::create(
        &type_dir,
        &config,
        CreateOptions {
            title: "Getting Started".to_string(),
            body: "Welcome to the docs.".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
            comment: Some(comment.to_string()),
        },
    )
    .await?;

    println!("Created '{}' (id: {})", item.title, item.id);
    println!("Comment: {:?}\n", item.comment);

    // 2. Read it back — comment is returned as part of the item
    let fetched = mdstore::get(&type_dir, &item.id).await?;
    println!("Fetched comment: {:?}", fetched.comment);

    // Show the raw file content so the comment placement is visible
    let raw = tokio::fs::read_to_string(type_dir.join(format!("{}.md", item.id))).await?;
    println!("\n--- Raw file ---\n{raw}--- end ---\n");

    // 3. Update without specifying comment — existing comment is preserved
    let updated = mdstore::update(
        &type_dir,
        &config,
        &item.id,
        UpdateOptions {
            body: Some("Updated body content.".to_string()),
            ..Default::default()
        },
    )
    .await?;
    println!("After update (comment preserved): {:?}", updated.comment);

    // 4. Replace the comment on update
    let replaced = mdstore::update(
        &type_dir,
        &config,
        &item.id,
        UpdateOptions {
            comment: Some("# Updated comment.".to_string()),
            ..Default::default()
        },
    )
    .await?;
    println!("After comment replace: {:?}", replaced.comment);

    // 5. Clear the comment by passing Some("")
    let cleared = mdstore::update(
        &type_dir,
        &config,
        &item.id,
        UpdateOptions {
            comment: Some(String::new()),
            ..Default::default()
        },
    )
    .await?;
    println!("After comment clear: {:?}", cleared.comment);

    Ok(())
}
