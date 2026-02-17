//! Using the frontmatter parser with custom types.
//!
//! The frontmatter module works with any Serialize/Deserialize type,
//! independent of the rest of mdstore.
//!
//! Run with: cargo run --example custom_frontmatter

use mdstore::{generate_frontmatter, parse_frontmatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlogPost {
    author: String,
    tags: Vec<String>,
    published: bool,
    read_time_minutes: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse existing markdown with custom frontmatter
    let markdown = r"---
author: Alice
tags:
  - rust
  - storage
  - tutorial
published: true
readTimeMinutes: 5
---

# Building a File-Based Database in Rust

In this post, we explore how to use Markdown files as a structured data store...

## Why Markdown?

Markdown files are human-readable, version-control friendly, and work with
existing tooling like editors, linters, and static site generators.";

    let (meta, title, body): (BlogPost, String, String) = parse_frontmatter(markdown)?;

    println!("Title: {title}");
    println!("Author: {}", meta.author);
    println!("Tags: {:?}", meta.tags);
    println!("Published: {}", meta.published);
    println!("Read time: {} min", meta.read_time_minutes);
    println!("Body length: {} chars", body.len());

    // Generate markdown from structured data
    let new_post = BlogPost {
        author: "Bob".to_string(),
        tags: vec!["rust".to_string(), "async".to_string()],
        published: false,
        read_time_minutes: 3,
    };

    let generated = generate_frontmatter(
        &new_post,
        "Async I/O Patterns",
        "A deep dive into async file operations in Rust.",
    );

    println!("\n--- Generated markdown ---\n{generated}");

    // Round-trip: parse what we just generated
    let (parsed, parsed_title, parsed_body): (BlogPost, String, String) =
        parse_frontmatter(&generated)?;
    println!("Round-trip title: {parsed_title}");
    println!("Round-trip author: {}", parsed.author);
    println!("Round-trip body: {parsed_body}");

    Ok(())
}
