//! Slug-based document storage example.
//!
//! Demonstrates using mdstore for documentation with slug identifiers
//! (human-readable IDs derived from titles).
//!
//! Run with: cargo run --example slug_docs

use mdstore::config::IdStrategy;
use mdstore::{CreateOptions, Filters, TypeConfig, TypeFeatures};
use std::collections::HashMap;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output/slug_docs");
    if base.exists() {
        tokio::fs::remove_dir_all(&base).await?;
    }
    let type_dir = base.join("docs");

    // Minimal config -- no status, priority, or display numbers
    let config = TypeConfig {
        name: "Doc".to_string(),

        identifier: IdStrategy::Slug,
        features: TypeFeatures::default(),
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    };

    // Create docs -- IDs are auto-generated from titles
    let doc1 = mdstore::create(
        &type_dir,
        &config,
        CreateOptions {
            title: "Getting Started".to_string(),
            body: "Welcome to the project!\n\n## Installation\n\nRun `cargo add mdstore`."
                .to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await?;
    println!("Created: {} (id: {})", doc1.title, doc1.id);

    let doc2 = mdstore::create(
        &type_dir,
        &config,
        CreateOptions {
            title: "API Reference".to_string(),
            body: "Full API documentation for mdstore.".to_string(),
            id: None,
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await?;
    println!("Created: {} (id: {})", doc2.title, doc2.id);

    // You can also provide an explicit slug
    let doc3 = mdstore::create(
        &type_dir,
        &config,
        CreateOptions {
            title: "FAQ".to_string(),
            body: "Frequently asked questions.".to_string(),
            id: Some("faq".to_string()),
            status: None,
            priority: None,
            custom_fields: HashMap::new(),
        },
    )
    .await?;
    println!("Created: {} (id: {})", doc3.title, doc3.id);

    // List all docs
    let all_docs = mdstore::list(&type_dir, Filters::default()).await?;
    println!("\nAll documents:");
    for doc in &all_docs {
        println!("  /{} - {}", doc.id, doc.title);
    }

    // Fetch by slug
    let fetched = mdstore::get(&type_dir, "getting-started").await?;
    println!(
        "\nFetched doc:\n  Title: {}\n  Body: {}",
        fetched.title, fetched.body
    );

    // On-disk, the file is simply: docs/getting-started.md
    let file_path = type_dir.join("getting-started.md");
    let raw = tokio::fs::read_to_string(&file_path).await?;
    println!("\nRaw file content:\n{raw}");

    Ok(())
}
