# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Stricter Clippy lint baseline to catch latent bugs and tighten code quality: enabled `clippy::clone_on_ref_ptr`, `clippy::print_stdout`, `clippy::dbg_macro`, `clippy::string_slice`, `clippy::todo`, and `clippy::redundant_clone` (#26, #28, #30, #32, #34, #36)
- Enabled `clippy::filetype_is_file` and switched directory-scan guards in `list`, `get_next_display_number`, and `reconcile_display_numbers` from `!is_file()` to `is_dir()`, so symlinked `.md` item files are no longer silently skipped (#50)

## [1.2.0] - 2026-04-04

### Added

- `projects` field on `Frontmatter`, `CreateOptions`, and `UpdateOptions` so items can record the projects they belong to

### Fixed

- Add the missing `projects` field to `CreateOptions` usages in the examples

## [1.1.1] - 2026-03-31

### Fixed

- Skip the empty H1 heading when an item's title is blank instead of emitting a bare `# `
- Refresh the `updatedAt` timestamp when an issue's status moves to in-progress

## [1.1.0] - 2026-03-24

### Added

- First-class tags support for items

## [1.0.0] - 2026-03-05

### Added

- `comment` field on `Item`, `CreateOptions`, and `UpdateOptions`; frontmatter comments are preserved through all CRU operations, soft delete, restore, duplicate, and move
- Custom fields are now flattened directly to top-level frontmatter instead of being nested under a `customFields:` key — **breaking change for existing stored files**

### Fixed

- Prevent race condition in display number assignment by holding a per-directory mutex from number selection through file write (#9)
- Sort items by `DateTime<Utc>` in reconciliation to ensure correct chronological ordering when timestamps carry non-UTC offsets
- Check target type's `move` feature flag in `move_item` before proceeding
- Check `duplicate` feature flag in `duplicate()` before proceeding
- Use `u32::try_from` instead of unchecked `as`-cast when counting reconciliation reassignments

## [0.4.0] - 2026-02-22

### Added

- Multi-status filter: pass multiple statuses to `Filters` to match items in any of the given statuses
- Priority range filter: filter items by minimum and maximum priority bounds

## [0.3.0] - 2026-02-17

### Added

- `discover_types_map` function returning a `HashMap<String, TypeConfig>` keyed by folder name

## [0.2.0] - 2026-02-17

### Added

- Re-export `discover_types` from crate root
- Dedicated `discover_types` example

### Fixed

- Emit `tracing::warn` when `list()` skips malformed `.md` files instead of silently ignoring them
- Store `now_iso()` once in `soft_delete` to ensure identical timestamps on all fields (#4)

## [0.1.0] - 2026-02-17

### Added

- Config-driven item types via `config.yaml` with toggleable features
- YAML frontmatter + Markdown body storage format
- UUID and slug-based identifiers
- Auto-incrementing display numbers with offline conflict reconciliation
- Status and priority validation with configurable allowed values
- Custom fields as extensible key-value metadata
- Soft delete and restore
- Move and duplicate operations with field preservation
- Filtering and pagination (status, priority, deleted state, limit, offset)
- Async I/O via Tokio
- Standalone frontmatter parser for any `Serialize`/`Deserialize` type
