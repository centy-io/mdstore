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

    let comment = "# This file was auto-generated. Do not edit manually.";

    // --- File 1: comment is created and preserved ---

    let item1 = mdstore::create(
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

    println!("Created '{}' (id: {})", item1.title, item1.id);
    println!("Comment: {:?}", item1.comment);

    // Update body — comment is preserved because UpdateOptions::comment is None
    let updated1 = mdstore::update(
        &type_dir,
        &config,
        &item1.id,
        UpdateOptions {
            body: Some("Updated body content.".to_string()),
            ..Default::default()
        },
    )
    .await?;
    println!("After update (comment preserved): {:?}", updated1.comment);

    let raw1 = tokio::fs::read_to_string(type_dir.join(format!("{}.md", item1.id))).await?;
    println!("\n--- File 1 (comment kept) ---\n{raw1}--- end ---\n");

    // --- File 2: comment is created then cleared ---

    let item2 = mdstore::create(
        &type_dir,
        &config,
        CreateOptions {
            title: "Reference".to_string(),
            body: "API reference docs.".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
            comment: Some(comment.to_string()),
        },
    )
    .await?;

    println!("Created '{}' (id: {})", item2.title, item2.id);
    println!("Comment: {:?}", item2.comment);

    // Clear the comment by passing Some("")
    let cleared2 = mdstore::update(
        &type_dir,
        &config,
        &item2.id,
        UpdateOptions {
            comment: Some(String::new()),
            ..Default::default()
        },
    )
    .await?;
    println!("After comment clear: {:?}", cleared2.comment);

    let raw2 = tokio::fs::read_to_string(type_dir.join(format!("{}.md", item2.id))).await?;
    println!("\n--- File 2 (comment cleared) ---\n{raw2}--- end ---\n");

    Ok(())
}
