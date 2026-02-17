# mdstore

A file-based storage engine that stores structured data as Markdown files with YAML frontmatter.

Each item is a single `.md` file:

```markdown
---
displayNumber: 1
status: open
priority: 2
createdAt: "2024-01-15T10:30:00+00:00"
updatedAt: "2024-01-15T10:30:00+00:00"
---

# Fix login timeout

Users are experiencing timeouts after 30 seconds of inactivity.
```

## Features

- **Config-driven item types** -- define any item type (issues, docs, epics, tasks) via a `config.yaml` with toggleable features
- **YAML frontmatter + Markdown body** -- metadata lives in frontmatter, content lives in the document
- **UUID or slug identifiers** -- UUID for conflict-free distributed creation, slugs for human-readable URLs
- **Auto-incrementing display numbers** -- human-friendly sequential IDs with offline conflict reconciliation
- **Status and priority validation** -- configurable allowed statuses and priority levels
- **Custom fields** -- extensible key-value metadata per item
- **Soft delete / restore** -- mark items as deleted without losing data
- **Move and duplicate** -- transfer or copy items between directories with field preservation
- **Filtering and pagination** -- filter by status, priority, deleted state; limit and offset results
- **Async (Tokio)** -- all I/O operations are async

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
mdstore = { path = "../mdstore" }
```

### Define a type config

```rust
use mdstore::{TypeConfig, TypeFeatures};

let config = TypeConfig {
    name: "Issue".to_string(),
    plural: "issues".to_string(),
    identifier: "uuid".to_string(),
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
};
```

### CRUD operations

```rust
use mdstore::{CreateOptions, Filters, UpdateOptions};
use std::collections::HashMap;
use std::path::Path;

let type_dir = Path::new("/data/issues");

// Create
let item = mdstore::create(&type_dir, &config, CreateOptions {
    title: "Fix login bug".to_string(),
    body: "Users can't log in after password reset.".to_string(),
    id: None,                              // auto-generates UUID
    status: Some("open".to_string()),
    priority: Some(1),
    custom_fields: HashMap::new(),
}).await?;

// Read
let item = mdstore::get(&type_dir, &config, &item.id).await?;

// List with filters
let open_items = mdstore::list(&type_dir, &config,
    Filters::new().with_status("open").with_limit(10),
).await?;

// Update
let updated = mdstore::update(&type_dir, &config, &item.id, UpdateOptions {
    status: Some("in-progress".to_string()),
    ..Default::default()
}).await?;

// Soft delete and restore
mdstore::soft_delete(&type_dir, &item.id).await?;
mdstore::restore(&type_dir, &item.id).await?;

// Hard delete
mdstore::delete(&type_dir, &item.id, true).await?;
```

### Slug-based items

```rust
let doc_config = TypeConfig {
    name: "Doc".to_string(),
    plural: "docs".to_string(),
    identifier: "slug".to_string(),   // generates ID from title
    features: TypeFeatures::default(), // no status, priority, etc.
    statuses: Vec::new(),
    default_status: None,
    priority_levels: None,
    custom_fields: Vec::new(),
};

let doc = mdstore::create(&type_dir, &doc_config, CreateOptions {
    title: "Getting Started".to_string(),
    body: "Welcome to the project!".to_string(),
    id: None,  // auto-generates slug: "getting-started"
    status: None,
    priority: None,
    custom_fields: HashMap::new(),
}).await?;

assert_eq!(doc.id, "getting-started");
```

### Config from YAML files

```rust
use mdstore::config::{read_type_config, write_type_config, discover_types};

// Write a config.yaml
write_type_config(Path::new("/data/issues"), &config).await?;

// Read it back
let config = read_type_config(Path::new("/data/issues")).await?;

// Discover all types in a base directory
// Scans /data/*/config.yaml
let all_types = discover_types(Path::new("/data")).await?;
```

### Frontmatter parsing

The frontmatter module can be used standalone for any Markdown+YAML workflow:

```rust
use mdstore::{parse_frontmatter, generate_frontmatter};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct BlogMeta {
    author: String,
    tags: Vec<String>,
}

let content = r#"---
author: Alice
tags:
  - rust
  - tutorial
---

# My Blog Post

Content here..."#;

let (meta, title, body): (BlogMeta, String, String) =
    parse_frontmatter(content)?;

let regenerated = generate_frontmatter(&meta, &title, &body);
```

## File layout

```
<base_dir>/
  issues/
    config.yaml           # TypeConfig for this item type
    <uuid>.md             # individual items
    <uuid>.md
  docs/
    config.yaml
    getting-started.md    # slug-based IDs
    api-reference.md
```

## Modules

| Module | Description |
|---|---|
| `storage` | Core CRUD: `create`, `get`, `list`, `update`, `delete`, `soft_delete`, `restore`, `duplicate`, `move_item` |
| `config` | `TypeConfig`, `TypeFeatures`, `CustomFieldDef`; read/write/discover config.yaml files |
| `types` | `Item`, `Frontmatter`, `CreateOptions`, `UpdateOptions`, `DuplicateOptions`, `MoveOptions` |
| `filters` | `Filters` with builder pattern for status, priority, pagination |
| `frontmatter` | Generic parse/generate for any `Serialize`/`Deserialize` type |
| `error` | `StoreError` enum covering all failure modes |
| `id` | `ItemId` enum (UUID or Slug) with parsing and serialization |
| `metadata` | `CommonMetadata` with priority migration support |
| `validation` | Priority and status validation utilities |
| `reconcile` | Display number conflict detection and resolution |
| `traits` | `Item`, `ItemCrud`, `ItemMetadata`, `SoftDeletable`, `Restorable`, `Movable`, `Duplicable` |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, Git hooks, and commit conventions.

## License

PolyForm Noncommercial 1.0.0
