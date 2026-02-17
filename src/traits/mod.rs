//! Trait definitions for items, metadata, lifecycle, and operations.

pub mod item;
pub mod lifecycle;
pub mod metadata;
pub mod operations;

pub use item::{Item, ItemCrud, ItemWithProject};
pub use lifecycle::{Restorable, SoftDeletable};
pub use metadata::{CustomFielded, DisplayNumbered, ItemMetadata, Prioritized, Statusable};
pub use operations::{Duplicable, Movable};
