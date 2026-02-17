//! Basic CRUD operations with mdstore.
//!
//! Run with: cargo run --example basic_crud

use mdstore::config::IdStrategy;
use mdstore::{CreateOptions, Filters, Item, TypeConfig, TypeFeatures, UpdateOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn issue_config() -> TypeConfig {
    TypeConfig {
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

async fn create_items(
    type_dir: &Path,
    config: &TypeConfig,
) -> Result<(Item, Item), Box<dyn std::error::Error>> {
    let issue1 = mdstore::create(
        type_dir,
        config,
        CreateOptions {
            title: "Fix login timeout".to_string(),
            body: "Users experience timeouts after 30s of inactivity.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(1),
            custom_fields: HashMap::new(),
        },
    )
    .await?;
    println!(
        "Created issue #{}: {} (id: {})",
        issue1.frontmatter.display_number.unwrap_or(0),
        issue1.title,
        issue1.id
    );

    let issue2 = mdstore::create(
        type_dir,
        config,
        CreateOptions {
            title: "Add dark mode".to_string(),
            body: "Support system-level dark mode preference.".to_string(),
            id: None,
            status: Some("open".to_string()),
            priority: Some(3),
            custom_fields: HashMap::from([(
                "component".to_string(),
                serde_json::json!("frontend"),
            )]),
        },
    )
    .await?;
    println!(
        "Created issue #{}: {}",
        issue2.frontmatter.display_number.unwrap_or(0),
        issue2.title
    );

    Ok((issue1, issue2))
}

async fn demo_list_update_get(
    type_dir: &Path,
    config: &TypeConfig,
    issue1: &Item,
) -> Result<(), Box<dyn std::error::Error>> {
    // List all open issues
    let open_issues =
        mdstore::list(type_dir, Filters::new().with_status("open")).await?;
    println!("\nOpen issues: {}", open_issues.len());
    for item in &open_issues {
        println!(
            "  #{} [P{}] {}",
            item.frontmatter.display_number.unwrap_or(0),
            item.frontmatter.priority.unwrap_or(0),
            item.title
        );
    }

    // Update an issue
    let updated = mdstore::update(
        type_dir,
        config,
        &issue1.id,
        UpdateOptions {
            status: Some("in-progress".to_string()),
            ..Default::default()
        },
    )
    .await?;
    println!(
        "\nUpdated '{}' -> status: {}",
        updated.title,
        updated.frontmatter.status.as_deref().unwrap_or("none")
    );

    // Get a specific issue
    let fetched = mdstore::get(type_dir, &issue1.id).await?;
    println!(
        "\nFetched: {} (status: {}, priority: {})",
        fetched.title,
        fetched.frontmatter.status.as_deref().unwrap_or("none"),
        fetched.frontmatter.priority.unwrap_or(0)
    );

    Ok(())
}

async fn demo_delete_restore(
    type_dir: &Path,
    issue2: &Item,
) -> Result<(), Box<dyn std::error::Error>> {
    // Soft delete and restore
    mdstore::soft_delete(type_dir, &issue2.id).await?;
    println!("\nSoft-deleted '{}'", issue2.title);

    let visible = mdstore::list(type_dir, Filters::default()).await?;
    println!("Visible items: {}", visible.len());

    let all = mdstore::list(type_dir, Filters::new().include_deleted()).await?;
    println!("All items (including deleted): {}", all.len());

    mdstore::restore(type_dir, &issue2.id).await?;
    println!("Restored '{}'", issue2.title);

    // Hard delete
    mdstore::delete(type_dir, &issue2.id, true).await?;
    println!("Hard-deleted '{}'", issue2.title);

    let remaining = mdstore::list(type_dir, Filters::default()).await?;
    println!("\nRemaining items: {}", remaining.len());

    Ok(())
}

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/basic_crud")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = output_dir();
    if base.exists() {
        tokio::fs::remove_dir_all(&base).await?;
    }
    let type_dir = base.join("issues");
    let config = issue_config();

    let (issue1, issue2) = create_items(&type_dir, &config).await?;
    demo_list_update_get(&type_dir, &config, &issue1).await?;
    demo_delete_restore(&type_dir, &issue2).await?;

    Ok(())
}
