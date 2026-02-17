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

## Examples

- [**Basic CRUD**](examples/basic_crud.rs) -- create, read, update, list, soft-delete, restore, and hard-delete items
- [**Slug-based docs**](examples/slug_docs.rs) -- human-readable slug identifiers derived from titles
- [**Type discovery**](examples/type_discovery.rs) -- write, read, and discover type configs from YAML files
- [**Custom frontmatter**](examples/custom_frontmatter.rs) -- use the frontmatter parser standalone with any serde type

Run any example with:

```sh
cargo run --example basic_crud
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
