# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
