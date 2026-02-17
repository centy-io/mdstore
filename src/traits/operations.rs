//! Cross-project and cross-item operations.

use async_trait::async_trait;

use crate::error::StoreError;

use super::item::ItemCrud;

/// Trait for items that can be moved between projects.
#[async_trait]
pub trait Movable: ItemCrud {
    /// Options for moving an item
    type MoveOptions;
    /// Result of moving an item
    type MoveResult;

    /// Move an item from one project to another.
    ///
    /// This operation:
    /// 1. Creates the item in the target project
    /// 2. Deletes the item from the source project
    /// 3. Updates any cross-references
    async fn move_item(options: Self::MoveOptions) -> Result<Self::MoveResult, StoreError>;
}

/// Trait for items that can be duplicated.
#[async_trait]
pub trait Duplicable: ItemCrud {
    /// Options for duplicating an item
    type DuplicateOptions;
    /// Result of duplicating an item
    type DuplicateResult;

    /// Create a copy of an item, optionally in a different project.
    ///
    /// The duplicated item gets a new ID and display number but preserves
    /// the content and metadata from the original.
    async fn duplicate(
        options: Self::DuplicateOptions,
    ) -> Result<Self::DuplicateResult, StoreError>;
}
