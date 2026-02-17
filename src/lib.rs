//! mdstore - A file-based storage engine that stores structured data as Markdown
//! files with YAML frontmatter.

// Allow panics/unwraps in test code
#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result
)]
mod test_cfg {}

pub mod config;
pub mod error;
pub mod filters;
pub mod frontmatter;
pub mod id;
pub mod metadata;
pub mod reconcile;
pub mod storage;
pub mod traits;
pub mod types;
mod util;
pub mod validation;

// Re-export primary types for convenience
pub use config::{ConfigError, CustomFieldDef, IdStrategy, TypeConfig, TypeFeatures};
pub use error::StoreError;
pub use filters::Filters;
pub use frontmatter::{
    generate_frontmatter, generate_frontmatter_raw, parse_frontmatter, parse_frontmatter_raw,
    FrontmatterError,
};
pub use id::{Identifiable, ItemId};
pub use metadata::CommonMetadata;
pub use reconcile::{get_next_display_number, reconcile_display_numbers};
pub use storage::{create, delete, duplicate, get, list, move_item, restore, soft_delete, update};
pub use types::{
    CreateOptions, DuplicateOptions, DuplicateResult, Frontmatter, Item, MoveOptions, MoveResult,
    UpdateOptions,
};
pub use validation::{validate_priority, PriorityError, StatusValidator, ValidationMode};
